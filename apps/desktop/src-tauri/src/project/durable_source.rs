//! M55's local-only durable source admission controller.  It intentionally has
//! no provider, retrieval, context-manifest, connector, or project-path API.

use std::{
    collections::HashMap,
    fs,
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::{
    storage::{DurableSourceInsert, ProjectRepository},
    types::{
        DurableSourceClass, DurableSourceDiagnosticCode, DurableSourcePreparation,
        DurableSourceSummary,
    },
};

pub(crate) const MAX_BYTES: usize = 128 * 1024;
const MAX_LINES: u32 = 2_000;
const MAX_CODEPOINTS: usize = 32_768;
const PREVIEW_BYTES: usize = 4 * 1024;
const PREPARATION_TTL: Duration = Duration::from_secs(5 * 60);

struct Pending {
    nonce: String,
    expires: Instant,
    project_id: String,
    task_id: Option<String>,
    class: DurableSourceClass,
    title: String,
    origin_display: Option<String>,
    sha256: String,
    bytes: usize,
    lines: u32,
    staged: PathBuf,
}

struct Intake {
    project_id: String,
    task_id: Option<String>,
    class: DurableSourceClass,
    title: String,
    origin_display: Option<String>,
    bytes: Vec<u8>,
}

pub(crate) struct DurableSourceController {
    root: PathBuf,
    pending: HashMap<String, Pending>,
    deletions: HashMap<String, (String, String, Instant)>,
}

impl DurableSourceController {
    pub(crate) fn open(database_path: &Path) -> Result<Self, DurableSourceDiagnosticCode> {
        let parent = database_path
            .parent()
            .ok_or(DurableSourceDiagnosticCode::PrivateStorageFailure)?;
        let root = parent.join("durable-sources");
        fs::create_dir_all(&root)
            .map_err(|_| DurableSourceDiagnosticCode::PrivateStorageFailure)?;
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
            .map_err(|_| DurableSourceDiagnosticCode::PrivateStorageFailure)?;
        let controller = Self {
            root,
            pending: HashMap::new(),
            deletions: HashMap::new(),
        };
        controller.cleanup_staged()?;
        Ok(controller)
    }

    #[cfg(test)]
    pub(crate) fn temporary() -> Self {
        let root = std::env::temp_dir().join(format!("quireforge-m55-{}", Uuid::now_v7()));
        fs::create_dir_all(&root).expect("test durable source directory");
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
            .expect("test source permissions");
        Self {
            root,
            pending: HashMap::new(),
            deletions: HashMap::new(),
        }
    }

    pub(crate) fn prepare_manual(
        &mut self,
        repository: &ProjectRepository,
        project_id: String,
        task_id: Option<String>,
        title: String,
        text: String,
    ) -> DurableSourcePreparation {
        self.prepare_bytes(
            repository,
            Intake {
                project_id,
                task_id,
                class: DurableSourceClass::ManualText,
                title,
                origin_display: None,
                bytes: text.into_bytes(),
            },
        )
    }

    pub(crate) fn prepare_file(
        &mut self,
        repository: &ProjectRepository,
        project_id: String,
        task_id: Option<String>,
        title: String,
        path: PathBuf,
    ) -> DurableSourcePreparation {
        let metadata = match fs::symlink_metadata(&path) {
            Ok(value) => value,
            Err(_) => return unavailable(DurableSourceDiagnosticCode::SourceUnavailable),
        };
        if metadata.file_type().is_symlink() {
            return unavailable(DurableSourceDiagnosticCode::SymlinkRejected);
        }
        if !metadata.file_type().is_file() {
            return unavailable(DurableSourceDiagnosticCode::FileNotRegular);
        }
        let identity = (
            metadata.dev(),
            metadata.ino(),
            metadata.len(),
            metadata.mtime(),
            metadata.mtime_nsec(),
        );
        if metadata.len() > MAX_BYTES as u64 {
            return unavailable(DurableSourceDiagnosticCode::SourceOversized);
        }
        let mut file = match fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&path)
        {
            Ok(file) => file,
            Err(_) => return unavailable(DurableSourceDiagnosticCode::SymlinkRejected),
        };
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        if file.read_to_end(&mut bytes).is_err() {
            return unavailable(DurableSourceDiagnosticCode::SourceUnavailable);
        }
        let after = match fs::symlink_metadata(&path) {
            Ok(value) => value,
            Err(_) => return unavailable(DurableSourceDiagnosticCode::FileChangedDuringIntake),
        };
        if after.file_type().is_symlink()
            || !after.file_type().is_file()
            || identity
                != (
                    after.dev(),
                    after.ino(),
                    after.len(),
                    after.mtime(),
                    after.mtime_nsec(),
                )
        {
            return unavailable(DurableSourceDiagnosticCode::FileChangedDuringIntake);
        }
        let origin = path
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .map(str::to_owned);
        self.prepare_bytes(
            repository,
            Intake {
                project_id,
                task_id,
                class: DurableSourceClass::LocalTextFile,
                title,
                origin_display: origin,
                bytes,
            },
        )
    }

    pub(crate) fn prepare_artifact(
        &mut self,
        repository: &ProjectRepository,
        project_id: String,
        task_id: Option<String>,
        title: String,
        artifact_id: String,
        bytes: Vec<u8>,
    ) -> DurableSourcePreparation {
        self.prepare_bytes(
            repository,
            Intake {
                project_id,
                task_id,
                class: DurableSourceClass::ReviewedArtifactText,
                title,
                origin_display: Some(artifact_id),
                bytes,
            },
        )
    }

    fn prepare_bytes(
        &mut self,
        repository: &ProjectRepository,
        intake: Intake,
    ) -> DurableSourcePreparation {
        if let Err(code) = binding(repository, &intake.project_id, intake.task_id.as_deref()) {
            return unavailable(code);
        }
        if intake.title.trim().is_empty()
            || intake.title.chars().count() > 240
            || intake.title.chars().any(char::is_control)
        {
            return unavailable(DurableSourceDiagnosticCode::SourceUnavailable);
        }
        if intake.bytes.len() > MAX_BYTES {
            return unavailable(DurableSourceDiagnosticCode::SourceOversized);
        }
        let text = match std::str::from_utf8(&intake.bytes) {
            Ok(value) => value,
            Err(_) => return unavailable(DurableSourceDiagnosticCode::InvalidUtf8),
        };
        if intake.class == DurableSourceClass::ManualText && text.chars().count() > MAX_CODEPOINTS {
            return unavailable(DurableSourceDiagnosticCode::SourceOversized);
        }
        let lines = line_count(text);
        if lines > MAX_LINES {
            return unavailable(DurableSourceDiagnosticCode::TooManyLines);
        }
        self.expire();
        let id = Uuid::now_v7().to_string();
        let nonce = Uuid::now_v7().to_string();
        let staged = self.root.join(format!(".{id}.stage"));
        if write_private(&staged, &intake.bytes).is_err() {
            return unavailable(DurableSourceDiagnosticCode::PrivateStorageFailure);
        }
        let sha256 = digest(&intake.bytes);
        let preparation = DurableSourcePreparation {
            schema_version: 1,
            preparation_id: id.clone(),
            nonce: nonce.clone(),
            expires_at_ms: now_ms() + PREPARATION_TTL.as_millis() as i64,
            project_id: intake.project_id.clone(),
            task_id: intake.task_id.clone(),
            source_class: intake.class,
            title: intake.title.clone(),
            origin_display: intake.origin_display.clone(),
            sha256: sha256.clone(),
            byte_size: intake.bytes.len() as u64,
            line_count: lines,
            preview: preview(text),
            diagnostic_code: None,
        };
        self.pending.insert(
            id.clone(),
            Pending {
                nonce,
                expires: Instant::now() + PREPARATION_TTL,
                project_id: intake.project_id,
                task_id: intake.task_id,
                class: intake.class,
                title: intake.title,
                origin_display: intake.origin_display,
                sha256,
                bytes: intake.bytes.len(),
                lines,
                staged,
            },
        );
        preparation
    }

    pub(crate) fn confirm(
        &mut self,
        repository: &mut ProjectRepository,
        preparation_id: &str,
        nonce: &str,
        sha256: &str,
    ) -> Result<DurableSourceSummary, DurableSourceDiagnosticCode> {
        self.expire();
        let pending = self
            .pending
            .remove(preparation_id)
            .ok_or(DurableSourceDiagnosticCode::PreparationMissing)?;
        if pending.nonce != nonce || pending.sha256 != sha256 {
            let _ = fs::remove_file(&pending.staged);
            return Err(DurableSourceDiagnosticCode::ConfirmationMismatch);
        }
        let bytes = fs::read(&pending.staged)
            .map_err(|_| DurableSourceDiagnosticCode::PrivateStorageFailure)?;
        if bytes.len() != pending.bytes
            || digest(&bytes) != pending.sha256
            || std::str::from_utf8(&bytes).is_err()
        {
            let _ = fs::remove_file(&pending.staged);
            return Err(DurableSourceDiagnosticCode::ConfirmationMismatch);
        }
        if let Err(code) = binding(repository, &pending.project_id, pending.task_id.as_deref()) {
            let _ = fs::remove_file(&pending.staged);
            return Err(code);
        }
        let source_id = Uuid::now_v7().to_string();
        let final_path = self.root.join(&source_id);
        fs::rename(&pending.staged, &final_path)
            .map_err(|_| DurableSourceDiagnosticCode::PrivateStorageFailure)?;
        let result = repository.durable_source_insert(DurableSourceInsert {
            id: &source_id,
            project_id: &pending.project_id,
            task_id: pending.task_id.as_deref(),
            source_class: pending.class,
            title: &pending.title,
            origin_display: pending.origin_display.as_deref(),
            byte_size: pending.bytes as u64,
            line_count: pending.lines,
            sha256: &pending.sha256,
        });
        match result {
            Ok(summary) => Ok(summary),
            Err(_) => {
                let _ = fs::remove_file(final_path);
                Err(DurableSourceDiagnosticCode::AdmissionAmbiguous)
            }
        }
    }

    pub(crate) fn cancel(
        &mut self,
        preparation_id: &str,
        nonce: &str,
    ) -> Result<(), DurableSourceDiagnosticCode> {
        self.expire();
        let pending = self
            .pending
            .remove(preparation_id)
            .ok_or(DurableSourceDiagnosticCode::PreparationMissing)?;
        if pending.nonce != nonce {
            let _ = fs::remove_file(&pending.staged);
            return Err(DurableSourceDiagnosticCode::ConfirmationMismatch);
        }
        fs::remove_file(&pending.staged)
            .map_err(|_| DurableSourceDiagnosticCode::PrivateStorageFailure)
    }

    pub(crate) fn delete(
        &mut self,
        repository: &mut ProjectRepository,
        source_id: &str,
    ) -> Result<(), DurableSourceDiagnosticCode> {
        let source = repository
            .durable_source(source_id)
            .map_err(|_| DurableSourceDiagnosticCode::SourceUnavailable)?
            .ok_or(DurableSourceDiagnosticCode::SourceUnavailable)?;
        if source.state != super::types::DurableSourceLifecycleState::Active {
            return Err(DurableSourceDiagnosticCode::SourceAlreadyDeleted);
        }
        fs::remove_file(self.root.join(source_id))
            .map_err(|_| DurableSourceDiagnosticCode::PrivateStorageFailure)?;
        repository
            .durable_source_delete(source_id)
            .map_err(|_| DurableSourceDiagnosticCode::DeletionAmbiguous)?;
        Ok(())
    }

    pub(crate) fn prepare_delete(
        &mut self,
        repository: &ProjectRepository,
        source_id: &str,
    ) -> DurableSourcePreparation {
        self.expire();
        let source = match repository.durable_source(source_id) {
            Ok(Some(value)) if value.state == super::types::DurableSourceLifecycleState::Active => {
                value
            }
            Ok(Some(_)) => return unavailable(DurableSourceDiagnosticCode::SourceAlreadyDeleted),
            _ => return unavailable(DurableSourceDiagnosticCode::SourceUnavailable),
        };
        let preparation_id = Uuid::now_v7().to_string();
        let nonce = Uuid::now_v7().to_string();
        self.deletions.insert(
            preparation_id.clone(),
            (
                source_id.to_owned(),
                nonce.clone(),
                Instant::now() + PREPARATION_TTL,
            ),
        );
        DurableSourcePreparation {
            schema_version: 1,
            preparation_id,
            nonce,
            expires_at_ms: now_ms() + PREPARATION_TTL.as_millis() as i64,
            project_id: source.project_id,
            task_id: source.task_id,
            source_class: source.source_class,
            title: source.title,
            origin_display: source.origin_display,
            sha256: source.sha256,
            byte_size: source.byte_size,
            line_count: source.line_count,
            preview: String::new(),
            diagnostic_code: None,
        }
    }

    pub(crate) fn confirm_delete(
        &mut self,
        repository: &mut ProjectRepository,
        preparation_id: &str,
        nonce: &str,
        source_id: &str,
    ) -> Result<(), DurableSourceDiagnosticCode> {
        self.expire();
        let (expected_source, expected_nonce, _) = self
            .deletions
            .remove(preparation_id)
            .ok_or(DurableSourceDiagnosticCode::PreparationMissing)?;
        if expected_source != source_id || expected_nonce != nonce {
            return Err(DurableSourceDiagnosticCode::ConfirmationMismatch);
        }
        self.delete(repository, source_id)
    }

    fn expire(&mut self) {
        let expired = self
            .pending
            .keys()
            .filter(|id| {
                self.pending
                    .get(*id)
                    .is_some_and(|pending| pending.expires <= Instant::now())
            })
            .cloned()
            .collect::<Vec<_>>();
        for id in expired {
            if let Some(pending) = self.pending.remove(&id) {
                let _ = fs::remove_file(pending.staged);
            }
        }
        self.deletions
            .retain(|_, (_, _, expires)| *expires > Instant::now());
    }
    fn cleanup_staged(&self) -> Result<(), DurableSourceDiagnosticCode> {
        for entry in fs::read_dir(&self.root)
            .map_err(|_| DurableSourceDiagnosticCode::RecoveryCleanupFailure)?
        {
            let path = entry
                .map_err(|_| DurableSourceDiagnosticCode::RecoveryCleanupFailure)?
                .path();
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with('.') && name.ends_with(".stage"))
            {
                fs::remove_file(path)
                    .map_err(|_| DurableSourceDiagnosticCode::RecoveryCleanupFailure)?;
            }
        }
        Ok(())
    }
}

impl Drop for DurableSourceController {
    fn drop(&mut self) {
        for pending in self.pending.values() {
            let _ = fs::remove_file(&pending.staged);
        }
    }
}

fn binding(
    repository: &ProjectRepository,
    project_id: &str,
    task_id: Option<&str>,
) -> Result<(), DurableSourceDiagnosticCode> {
    if repository
        .list_projects()
        .map_err(|_| DurableSourceDiagnosticCode::ProjectUnavailable)?
        .iter()
        .all(|project| project.id != project_id || project.archived)
    {
        return Err(DurableSourceDiagnosticCode::ProjectUnavailable);
    }
    if let Some(task_id) = task_id {
        match repository.task_project_binding(task_id) {
            Ok(Some(bound)) if bound == project_id => {}
            Ok(Some(_)) => return Err(DurableSourceDiagnosticCode::ProjectTaskMismatch),
            Ok(None) | Err(_) => return Err(DurableSourceDiagnosticCode::TaskUnavailable),
        }
    }
    Ok(())
}
fn write_private(path: &Path, bytes: &[u8]) -> Result<(), ()> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|_| ())?;
    file.write_all(bytes).map_err(|_| ())?;
    file.sync_all().map_err(|_| ())
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn line_count(value: &str) -> u32 {
    if value.is_empty() {
        0
    } else {
        value.bytes().filter(|byte| *byte == b'\n').count() as u32 + 1
    }
}
fn preview(value: &str) -> String {
    value
        .char_indices()
        .take_while(|(index, _)| *index < PREVIEW_BYTES)
        .map(|(_, character)| character)
        .collect()
}
fn unavailable(code: DurableSourceDiagnosticCode) -> DurableSourcePreparation {
    DurableSourcePreparation {
        schema_version: 1,
        preparation_id: String::new(),
        nonce: String::new(),
        expires_at_ms: 0,
        project_id: String::new(),
        task_id: None,
        source_class: DurableSourceClass::ManualText,
        title: String::new(),
        origin_display: None,
        sha256: String::new(),
        byte_size: 0,
        line_count: 0,
        preview: String::new(),
        diagnostic_code: Some(code),
    }
}
fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |value| value.as_millis() as i64)
}
