use std::{
    collections::VecDeque,
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::fs::OpenOptionsExt,
    path::Path,
    sync::Mutex,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const MAX_ARTIFACTS: usize = 5;
pub const MAX_ARTIFACT_BYTES: usize = 512 * 1024;
pub const MAX_TOTAL_ARTIFACT_BYTES: usize = 2 * 1024 * 1024;
const ARTIFACT_TTL: Duration = Duration::from_secs(15 * 60);

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GeneratedArtifactClass {
    Text,
    Markdown,
    Json,
    Csv,
    Python,
}
impl GeneratedArtifactClass {
    pub fn suffix(self) -> &'static str {
        match self {
            Self::Text => ".txt",
            Self::Markdown => ".md",
            Self::Json => ".json",
            Self::Csv => ".csv",
            Self::Python => ".py",
        }
    }
    pub fn mime_type(self) -> &'static str {
        match self {
            Self::Text => "text/plain; charset=utf-8",
            Self::Markdown => "text/markdown; charset=utf-8",
            Self::Json => "application/json",
            Self::Csv => "text/csv; charset=utf-8",
            Self::Python => "text/x-python; charset=utf-8",
        }
    }
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GeneratedArtifactSourceKind {
    VisibleCompletedReply,
    VisibleFencedBlock,
    ExplicitReviewPromotion,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GeneratedArtifactState {
    Ready,
    Saving,
    Expired,
    Saved,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GeneratedArtifactDisposal {
    TransientMemoryOneSuccessfulSave,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GeneratedArtifactDiagnosticCode {
    InvalidRequest,
    InvalidContent,
    InvalidJson,
    InvalidCsv,
    UnsafeName,
    ArtifactNotFound,
    ManifestMismatch,
    ArtifactExpired,
    AlreadySaving,
    CapacityExceeded,
    AggregateExceeded,
    SaveCancelled,
    SaveFailed,
    FileExists,
    CleanupFailed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeneratedArtifactManifestV1 {
    pub schema_version: u16,
    pub artifact_id: String,
    pub class: GeneratedArtifactClass,
    pub mime_type: String,
    pub source_kind: GeneratedArtifactSourceKind,
    pub display_label: String,
    pub suggested_filename: String,
    pub byte_size: u64,
    pub sha256: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub state: GeneratedArtifactState,
    pub disposal: GeneratedArtifactDisposal,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeneratedArtifactSnapshotV1 {
    pub schema_version: u16,
    pub artifacts: Vec<GeneratedArtifactManifestV1>,
    pub diagnostic_code: Option<GeneratedArtifactDiagnosticCode>,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeneratedArtifactCreateRequest {
    pub class: GeneratedArtifactClass,
    pub source_kind: GeneratedArtifactSourceKind,
    pub display_label: String,
    pub suggested_filename: String,
    pub content: String,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeneratedArtifactClaimRequest {
    pub artifact_id: String,
    pub manifest_sha256: String,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeneratedArtifactPreviewV1 {
    pub schema_version: u16,
    pub artifact_id: String,
    pub sha256: String,
    pub text: String,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GeneratedArtifactSaveReceiptV1 {
    pub schema_version: u16,
    pub artifact_id: String,
    pub class: GeneratedArtifactClass,
    pub filename: String,
    pub byte_size: u64,
    pub sha256: String,
    pub saved_at: u64,
}

struct Entry {
    manifest: GeneratedArtifactManifestV1,
    bytes: Vec<u8>,
    created: Instant,
    saving: bool,
}
pub struct SaveReservation {
    artifact_id: String,
    sha256: String,
    class: GeneratedArtifactClass,
    suggested_filename: String,
    bytes: Vec<u8>,
}
/// A verified, in-memory M48 artifact copy source. This is deliberately not
/// serializable: Local Review receives only native-owned canonical bytes.
pub(crate) struct LocalReviewArtifactCopySource {
    pub artifact_id: String,
    pub class: GeneratedArtifactClass,
    pub display_label: String,
    pub sha256: String,
    pub bytes: Vec<u8>,
}
/// A verified, metadata-only M48 source for Local Review evidence. It omits
/// content, filenames, paths, and save information by construction.
pub(crate) struct LocalReviewArtifactMetadataSource {
    pub class: GeneratedArtifactClass,
    pub display_label: String,
    pub byte_size: u64,
    pub sha256: String,
    pub state: GeneratedArtifactState,
}
pub struct AdvisorGeneratedArtifactService {
    state: Mutex<VecDeque<Entry>>,
    epoch: Instant,
}
impl Default for AdvisorGeneratedArtifactService {
    fn default() -> Self {
        Self {
            state: Mutex::new(VecDeque::new()),
            epoch: Instant::now(),
        }
    }
}

impl AdvisorGeneratedArtifactService {
    pub fn snapshot(&self) -> GeneratedArtifactSnapshotV1 {
        self.with_entries(|entries| GeneratedArtifactSnapshotV1 {
            schema_version: 1,
            artifacts: entries.iter().map(|item| item.manifest.clone()).collect(),
            diagnostic_code: None,
        })
    }
    pub fn create(
        &self,
        request: GeneratedArtifactCreateRequest,
    ) -> Result<GeneratedArtifactManifestV1, GeneratedArtifactDiagnosticCode> {
        validate_label(&request.display_label)?;
        validate_filename(&request.suggested_filename, request.class)?;
        if request.source_kind == GeneratedArtifactSourceKind::VisibleCompletedReply
            && request.class != GeneratedArtifactClass::Text
        {
            return Err(GeneratedArtifactDiagnosticCode::InvalidRequest);
        }
        let text = request.content.replace("\r\n", "\n").replace('\r', "\n");
        if text.is_empty() || text.len() > MAX_ARTIFACT_BYTES {
            return Err(GeneratedArtifactDiagnosticCode::InvalidContent);
        }
        validate_content(request.class, &text)?;
        let bytes = text.into_bytes();
        let hash = digest(&bytes);
        let mut state = self
            .state
            .lock()
            .map_err(|_| GeneratedArtifactDiagnosticCode::SaveFailed)?;
        self.expire_locked(&mut state);
        if state.len() >= MAX_ARTIFACTS {
            return Err(GeneratedArtifactDiagnosticCode::CapacityExceeded);
        }
        if state.iter().map(|item| item.bytes.len()).sum::<usize>() + bytes.len()
            > MAX_TOTAL_ARTIFACT_BYTES
        {
            return Err(GeneratedArtifactDiagnosticCode::AggregateExceeded);
        }
        let now = self.epoch.elapsed().as_millis() as u64;
        let manifest = GeneratedArtifactManifestV1 {
            schema_version: 1,
            artifact_id: Uuid::now_v7().to_string(),
            class: request.class,
            mime_type: request.class.mime_type().to_owned(),
            source_kind: request.source_kind,
            display_label: request.display_label,
            suggested_filename: request.suggested_filename,
            byte_size: bytes.len() as u64,
            sha256: hash,
            created_at: now,
            expires_at: now + ARTIFACT_TTL.as_millis() as u64,
            state: GeneratedArtifactState::Ready,
            disposal: GeneratedArtifactDisposal::TransientMemoryOneSuccessfulSave,
        };
        state.push_back(Entry {
            manifest: manifest.clone(),
            bytes,
            created: Instant::now(),
            saving: false,
        });
        Ok(manifest)
    }
    pub fn preview(
        &self,
        claim: &GeneratedArtifactClaimRequest,
    ) -> Result<GeneratedArtifactPreviewV1, GeneratedArtifactDiagnosticCode> {
        self.validate_claim(claim, false)
            .map(|entry| GeneratedArtifactPreviewV1 {
                schema_version: 1,
                artifact_id: entry.artifact_id.clone(),
                sha256: entry.sha256.clone(),
                text: String::from_utf8_lossy(&entry.bytes).into_owned(),
            })
    }
    pub(crate) fn local_review_copy_source(
        &self,
        claim: &GeneratedArtifactClaimRequest,
    ) -> Result<LocalReviewArtifactCopySource, GeneratedArtifactDiagnosticCode> {
        if !valid_uuid_v7(&claim.artifact_id) || !valid_hash(&claim.manifest_sha256) {
            return Err(GeneratedArtifactDiagnosticCode::InvalidRequest);
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| GeneratedArtifactDiagnosticCode::SaveFailed)?;
        self.expire_locked(&mut state);
        let entry = state
            .iter()
            .find(|item| item.manifest.artifact_id == claim.artifact_id)
            .ok_or(GeneratedArtifactDiagnosticCode::ArtifactNotFound)?;
        if entry.manifest.sha256 != claim.manifest_sha256
            || entry.manifest.state != GeneratedArtifactState::Ready
            || entry.saving
            || entry.manifest.byte_size != entry.bytes.len() as u64
            || digest(&entry.bytes) != entry.manifest.sha256
        {
            return Err(GeneratedArtifactDiagnosticCode::ManifestMismatch);
        }
        validate_content(
            entry.manifest.class,
            std::str::from_utf8(&entry.bytes)
                .map_err(|_| GeneratedArtifactDiagnosticCode::InvalidContent)?,
        )?;
        Ok(LocalReviewArtifactCopySource {
            artifact_id: entry.manifest.artifact_id.clone(),
            class: entry.manifest.class,
            display_label: entry.manifest.display_label.clone(),
            sha256: entry.manifest.sha256.clone(),
            bytes: entry.bytes.clone(),
        })
    }
    pub(crate) fn local_review_metadata_source(
        &self,
        claim: &GeneratedArtifactClaimRequest,
    ) -> Result<LocalReviewArtifactMetadataSource, GeneratedArtifactDiagnosticCode> {
        if !valid_uuid_v7(&claim.artifact_id) || !valid_hash(&claim.manifest_sha256) {
            return Err(GeneratedArtifactDiagnosticCode::InvalidRequest);
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| GeneratedArtifactDiagnosticCode::SaveFailed)?;
        self.expire_locked(&mut state);
        let entry = state
            .iter()
            .find(|item| item.manifest.artifact_id == claim.artifact_id)
            .ok_or(GeneratedArtifactDiagnosticCode::ArtifactNotFound)?;
        if entry.manifest.sha256 != claim.manifest_sha256
            || entry.manifest.state != GeneratedArtifactState::Ready
            || entry.saving
            || entry.manifest.byte_size != entry.bytes.len() as u64
            || digest(&entry.bytes) != entry.manifest.sha256
        {
            return Err(GeneratedArtifactDiagnosticCode::ManifestMismatch);
        }
        Ok(LocalReviewArtifactMetadataSource {
            class: entry.manifest.class,
            display_label: entry.manifest.display_label.clone(),
            byte_size: entry.manifest.byte_size,
            sha256: entry.manifest.sha256.clone(),
            state: entry.manifest.state,
        })
    }
    pub fn discard(
        &self,
        claim: GeneratedArtifactClaimRequest,
    ) -> Result<(), GeneratedArtifactDiagnosticCode> {
        self.validate_claim(&claim, false)?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| GeneratedArtifactDiagnosticCode::SaveFailed)?;
        self.expire_locked(&mut state);
        let index = state
            .iter()
            .position(|item| item.manifest.artifact_id == claim.artifact_id)
            .ok_or(GeneratedArtifactDiagnosticCode::ArtifactNotFound)?;
        if state[index].saving {
            return Err(GeneratedArtifactDiagnosticCode::AlreadySaving);
        }
        state.remove(index);
        Ok(())
    }
    pub fn reserve(
        &self,
        claim: &GeneratedArtifactClaimRequest,
    ) -> Result<SaveReservation, GeneratedArtifactDiagnosticCode> {
        self.validate_claim(claim, true)
    }
    pub fn release(&self, reservation: &SaveReservation) {
        if let Ok(mut state) = self.state.lock() {
            self.expire_locked(&mut state);
            if let Some(entry) = state.iter_mut().find(|item| {
                item.manifest.artifact_id == reservation.artifact_id
                    && item.manifest.sha256 == reservation.sha256
            }) {
                entry.saving = false;
                entry.manifest.state = GeneratedArtifactState::Ready;
            }
        }
    }
    pub fn consume(
        &self,
        reservation: &SaveReservation,
        filename: String,
    ) -> Result<GeneratedArtifactSaveReceiptV1, GeneratedArtifactDiagnosticCode> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| GeneratedArtifactDiagnosticCode::SaveFailed)?;
        self.expire_locked(&mut state);
        let index = state
            .iter()
            .position(|item| {
                item.manifest.artifact_id == reservation.artifact_id
                    && item.manifest.sha256 == reservation.sha256
                    && item.saving
            })
            .ok_or(GeneratedArtifactDiagnosticCode::ArtifactNotFound)?;
        let entry = state.remove(index).expect("entry index exists");
        Ok(GeneratedArtifactSaveReceiptV1 {
            schema_version: 1,
            artifact_id: entry.manifest.artifact_id,
            class: entry.manifest.class,
            filename,
            byte_size: entry.manifest.byte_size,
            sha256: entry.manifest.sha256,
            saved_at: self.epoch.elapsed().as_millis() as u64,
        })
    }
    fn validate_claim(
        &self,
        claim: &GeneratedArtifactClaimRequest,
        reserve: bool,
    ) -> Result<SaveReservation, GeneratedArtifactDiagnosticCode> {
        if !valid_uuid_v7(&claim.artifact_id) || !valid_hash(&claim.manifest_sha256) {
            return Err(GeneratedArtifactDiagnosticCode::InvalidRequest);
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| GeneratedArtifactDiagnosticCode::SaveFailed)?;
        self.expire_locked(&mut state);
        let entry = state
            .iter_mut()
            .find(|item| item.manifest.artifact_id == claim.artifact_id)
            .ok_or(GeneratedArtifactDiagnosticCode::ArtifactNotFound)?;
        if entry.manifest.sha256 != claim.manifest_sha256 {
            return Err(GeneratedArtifactDiagnosticCode::ManifestMismatch);
        }
        if entry.saving {
            return Err(GeneratedArtifactDiagnosticCode::AlreadySaving);
        }
        if reserve {
            entry.saving = true;
            entry.manifest.state = GeneratedArtifactState::Saving;
        }
        Ok(SaveReservation {
            artifact_id: entry.manifest.artifact_id.clone(),
            sha256: entry.manifest.sha256.clone(),
            class: entry.manifest.class,
            suggested_filename: entry.manifest.suggested_filename.clone(),
            bytes: entry.bytes.clone(),
        })
    }
    fn with_entries<T>(&self, f: impl FnOnce(&VecDeque<Entry>) -> T) -> T {
        let mut state = self.state.lock().expect("artifact registry lock");
        self.expire_locked(&mut state);
        f(&state)
    }
    fn expire_locked(&self, state: &mut VecDeque<Entry>) {
        state.retain(|entry| entry.created.elapsed() <= ARTIFACT_TTL);
    }
}
impl SaveReservation {
    pub fn class(&self) -> GeneratedArtifactClass {
        self.class
    }
    pub fn suggested_filename(&self) -> &str {
        &self.suggested_filename
    }
}

pub fn save_reserved(
    reservation: &SaveReservation,
    target: &Path,
) -> Result<String, GeneratedArtifactDiagnosticCode> {
    let filename = target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(GeneratedArtifactDiagnosticCode::UnsafeName)?
        .to_owned();
    validate_filename(&filename, reservation.class)?;
    if !target.is_absolute() {
        return Err(GeneratedArtifactDiagnosticCode::InvalidRequest);
    }
    let parent = target
        .parent()
        .ok_or(GeneratedArtifactDiagnosticCode::InvalidRequest)?;
    let parent_metadata =
        fs::metadata(parent).map_err(|_| GeneratedArtifactDiagnosticCode::SaveFailed)?;
    if !parent_metadata.is_dir() {
        return Err(GeneratedArtifactDiagnosticCode::SaveFailed);
    }
    if fs::symlink_metadata(target).is_ok() {
        return Err(GeneratedArtifactDiagnosticCode::FileExists);
    }
    let temporary = parent.join(format!(".quireforge-artifact-{}.tmp", Uuid::now_v7()));
    let outcome = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
            .open(&temporary)
            .map_err(map_io)?;
        file.write_all(&reservation.bytes)
            .map_err(|_| GeneratedArtifactDiagnosticCode::SaveFailed)?;
        file.sync_all()
            .map_err(|_| GeneratedArtifactDiagnosticCode::SaveFailed)?;
        atomic_no_replace(&temporary, target)?;
        let directory =
            File::open(parent).map_err(|_| GeneratedArtifactDiagnosticCode::SaveFailed)?;
        directory
            .sync_all()
            .map_err(|_| GeneratedArtifactDiagnosticCode::SaveFailed)?;
        Ok(())
    })();
    if outcome.is_err() && temporary.exists() && fs::remove_file(&temporary).is_err() {
        return Err(GeneratedArtifactDiagnosticCode::CleanupFailed);
    }
    outcome.map(|_| filename)
}
fn atomic_no_replace(from: &Path, to: &Path) -> Result<(), GeneratedArtifactDiagnosticCode> {
    let source = std::ffi::CString::new(from.as_os_str().as_encoded_bytes())
        .map_err(|_| GeneratedArtifactDiagnosticCode::SaveFailed)?;
    let destination = std::ffi::CString::new(to.as_os_str().as_encoded_bytes())
        .map_err(|_| GeneratedArtifactDiagnosticCode::SaveFailed)?;
    let result = unsafe {
        libc::syscall(
            libc::SYS_renameat2,
            libc::AT_FDCWD,
            source.as_ptr(),
            libc::AT_FDCWD,
            destination.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if result == 0 {
        Ok(())
    } else if std::io::Error::last_os_error().raw_os_error() == Some(libc::EEXIST) {
        Err(GeneratedArtifactDiagnosticCode::FileExists)
    } else {
        Err(GeneratedArtifactDiagnosticCode::SaveFailed)
    }
}
fn map_io(error: std::io::Error) -> GeneratedArtifactDiagnosticCode {
    if error.kind() == std::io::ErrorKind::AlreadyExists {
        GeneratedArtifactDiagnosticCode::FileExists
    } else {
        GeneratedArtifactDiagnosticCode::SaveFailed
    }
}
fn validate_content(
    class: GeneratedArtifactClass,
    text: &str,
) -> Result<(), GeneratedArtifactDiagnosticCode> {
    match class {
        GeneratedArtifactClass::Json => serde_json::from_str::<serde_json::Value>(text)
            .map(|_| ())
            .map_err(|_| GeneratedArtifactDiagnosticCode::InvalidJson),
        GeneratedArtifactClass::Csv => validate_csv(text),
        _ => Ok(()),
    }
}
fn validate_csv(text: &str) -> Result<(), GeneratedArtifactDiagnosticCode> {
    let mut expected = None;
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let mut fields = 1usize;
        let mut quoted = false;
        let mut chars = line.chars().peekable();
        while let Some(character) = chars.next() {
            match character {
                '"' if quoted && chars.peek() == Some(&'"') => {
                    chars.next();
                }
                '"' => quoted = !quoted,
                ',' if !quoted => fields += 1,
                _ => {}
            }
        }
        if quoted {
            return Err(GeneratedArtifactDiagnosticCode::InvalidCsv);
        }
        if expected
            .replace(fields)
            .is_some_and(|count| count != fields)
        {
            return Err(GeneratedArtifactDiagnosticCode::InvalidCsv);
        }
    }
    Ok(())
}
fn validate_label(value: &str) -> Result<(), GeneratedArtifactDiagnosticCode> {
    if value.is_empty() || value.chars().count() > 120 || unsafe_name(value) {
        Err(GeneratedArtifactDiagnosticCode::UnsafeName)
    } else {
        Ok(())
    }
}
fn validate_filename(
    value: &str,
    class: GeneratedArtifactClass,
) -> Result<(), GeneratedArtifactDiagnosticCode> {
    if value.is_empty()
        || value.chars().count() > 120
        || unsafe_name(value)
        || value
            != Path::new(value)
                .file_name()
                .and_then(|item| item.to_str())
                .unwrap_or_default()
        || !value.ends_with(class.suffix())
    {
        Err(GeneratedArtifactDiagnosticCode::UnsafeName)
    } else {
        Ok(())
    }
}
fn unsafe_name(value: &str) -> bool {
    value.contains('/')
        || value.contains('\\')
        || value.chars().any(|character| {
            character.is_control()
                || matches!(character, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
fn valid_uuid_v7(value: &str) -> bool {
    Uuid::parse_str(value).is_ok_and(|id| id.get_version_num() == 7)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn creates_closed_classes_and_rejects_bad_content() {
        for (class, text, name) in [
            (GeneratedArtifactClass::Text, "x", "x.txt"),
            (GeneratedArtifactClass::Markdown, "x", "x.md"),
            (GeneratedArtifactClass::Json, "{}", "x.json"),
            (GeneratedArtifactClass::Csv, "a,b\n1,2", "x.csv"),
            (GeneratedArtifactClass::Python, "print(1)", "x.py"),
        ] {
            let service = AdvisorGeneratedArtifactService::default();
            assert!(service
                .create(GeneratedArtifactCreateRequest {
                    class,
                    source_kind: GeneratedArtifactSourceKind::VisibleFencedBlock,
                    display_label: "Output".into(),
                    suggested_filename: name.into(),
                    content: text.into()
                })
                .is_ok());
        }
        let service = AdvisorGeneratedArtifactService::default();
        assert!(service
            .create(GeneratedArtifactCreateRequest {
                class: GeneratedArtifactClass::Json,
                source_kind: GeneratedArtifactSourceKind::VisibleFencedBlock,
                display_label: "Output".into(),
                suggested_filename: "x.json".into(),
                content: "{} trailing".into()
            })
            .is_err());
        assert!(service
            .create(GeneratedArtifactCreateRequest {
                class: GeneratedArtifactClass::Csv,
                source_kind: GeneratedArtifactSourceKind::VisibleFencedBlock,
                display_label: "Output".into(),
                suggested_filename: "x.csv".into(),
                content: "a,b\n1".into()
            })
            .is_err());
    }
    #[test]
    fn capacity_claim_and_discard_are_closed() {
        let service = AdvisorGeneratedArtifactService::default();
        let mut ids = Vec::new();
        for n in 0..MAX_ARTIFACTS {
            let manifest = service
                .create(GeneratedArtifactCreateRequest {
                    class: GeneratedArtifactClass::Text,
                    source_kind: GeneratedArtifactSourceKind::VisibleCompletedReply,
                    display_label: format!("Output {n}"),
                    suggested_filename: "advisor-response.txt".into(),
                    content: "x".into(),
                })
                .unwrap();
            ids.push(manifest);
        }
        assert_eq!(service.snapshot().artifacts.len(), MAX_ARTIFACTS);
        assert!(service
            .create(GeneratedArtifactCreateRequest {
                class: GeneratedArtifactClass::Text,
                source_kind: GeneratedArtifactSourceKind::VisibleCompletedReply,
                display_label: "extra".into(),
                suggested_filename: "advisor-response.txt".into(),
                content: "x".into()
            })
            .is_err());
        let claim = GeneratedArtifactClaimRequest {
            artifact_id: ids[0].artifact_id.clone(),
            manifest_sha256: ids[0].sha256.clone(),
        };
        let reservation = service.reserve(&claim).unwrap();
        assert!(service.reserve(&claim).is_err());
        service.release(&reservation);
        service.discard(claim).unwrap();
        assert_eq!(service.snapshot().artifacts.len(), 4);
    }
    #[test]
    fn atomically_saves_exact_bytes_once_without_overwrite() {
        let service = AdvisorGeneratedArtifactService::default();
        let manifest = service
            .create(GeneratedArtifactCreateRequest {
                class: GeneratedArtifactClass::Text,
                source_kind: GeneratedArtifactSourceKind::VisibleCompletedReply,
                display_label: "Output".into(),
                suggested_filename: "advisor-response.txt".into(),
                content: "saved\ntext".into(),
            })
            .unwrap();
        let claim = GeneratedArtifactClaimRequest {
            artifact_id: manifest.artifact_id.clone(),
            manifest_sha256: manifest.sha256.clone(),
        };
        let reservation = service.reserve(&claim).unwrap();
        let root =
            std::env::temp_dir().join(format!("quireforge-artifact-test-{}", Uuid::now_v7()));
        fs::create_dir(&root).unwrap();
        let target = root.join("advisor-response.txt");
        let filename = save_reserved(&reservation, &target).unwrap();
        let receipt = service.consume(&reservation, filename).unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"saved\ntext");
        assert_eq!(digest(&fs::read(&target).unwrap()), receipt.sha256);
        assert_eq!(
            save_reserved(&reservation, &target),
            Err(GeneratedArtifactDiagnosticCode::FileExists)
        );
        fs::remove_dir_all(root).unwrap();
    }
}
