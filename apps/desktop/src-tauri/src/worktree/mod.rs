pub mod types;

use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Mutex,
    time::{Duration, Instant},
};

use tokio::{io::AsyncReadExt, process::Command, time::timeout};
use uuid::Uuid;

use crate::project::{
    ProjectExecutionError, ProjectReviewRoot, ProjectService, ProjectWorktreeCandidate,
    ProjectWorktreeContext, WorktreeRegistrationError,
};
use types::{
    WorktreeCancelRequest, WorktreeConfirmRequest, WorktreeCreatePreviewRequest,
    WorktreeDiagnosticCode, WorktreeEntry, WorktreeEntryState, WorktreeOperation,
    WorktreeOwnership, WorktreePreviewSnapshot, WorktreePreviewState,
    WorktreeRecoverPreviewRequest, WorktreeRemovePreviewRequest, WorktreeResultSnapshot,
    WorktreeResultState, WorktreeWorkspaceSnapshot, WorktreeWorkspaceState,
    WORKTREE_SCHEMA_VERSION,
};

const GIT_TIMEOUT: Duration = Duration::from_secs(20);
const CONFIRMATION_TTL: Duration = Duration::from_secs(5 * 60);
const MAX_OUTPUT_BYTES: usize = 256 * 1024;
const MAX_STDERR_BYTES: usize = 8 * 1024;
const MAX_WORKTREES: usize = 256;
const MAX_DISCOVERED_WORKTREES: usize = 512;
const MAX_BRANCH_BYTES: usize = 96;

#[derive(Clone, Debug)]
enum PendingOperation {
    Create {
        destination: PathBuf,
    },
    Attach {
        candidate: ProjectWorktreeCandidate,
    },
    Recover {
        candidate: ProjectWorktreeCandidate,
    },
    Remove {
        worktree_project_id: String,
        requesting_project_id: String,
        selected_path: PathBuf,
        candidate: Option<ProjectWorktreeCandidate>,
    },
}

#[derive(Clone, Debug)]
struct PendingWorktree {
    confirmation_id: String,
    expires_at: Instant,
    source_project_id: String,
    source_root: PathBuf,
    common_dir: PathBuf,
    branch_name: Option<String>,
    base_commit: Option<String>,
    operation: PendingOperation,
}

#[derive(Clone, Debug)]
struct RecoverableWorktree {
    expires_at: Instant,
    source_project_id: String,
    candidate: ProjectWorktreeCandidate,
    branch_name: Option<String>,
}

#[derive(Clone, Debug)]
struct DiscoveredWorktree {
    path: PathBuf,
    branch_name: Option<String>,
    detached: bool,
    locked: bool,
    prunable: bool,
}

#[derive(Debug, Eq, PartialEq)]
enum GitRunError {
    Unavailable,
    Failed,
    TooLarge,
    TimedOut,
}

struct GitOutput {
    stdout: Vec<u8>,
    success: bool,
    code: Option<i32>,
}

pub struct WorktreeService {
    storage_root: Option<PathBuf>,
    pending: Mutex<Option<PendingWorktree>>,
    recoverable: Mutex<HashMap<String, RecoverableWorktree>>,
}

impl WorktreeService {
    pub fn unavailable() -> Self {
        Self {
            storage_root: None,
            pending: Mutex::new(None),
            recoverable: Mutex::new(HashMap::new()),
        }
    }

    pub fn open(storage_root: &Path) -> Self {
        let root = prepare_storage_root(storage_root).ok();
        Self {
            storage_root: root,
            pending: Mutex::new(None),
            recoverable: Mutex::new(HashMap::new()),
        }
    }

    #[cfg(test)]
    fn for_test(storage_root: &Path) -> Self {
        Self::open(storage_root)
    }

    pub async fn status(
        &self,
        project_id: String,
        projects: &ProjectService,
    ) -> WorktreeWorkspaceSnapshot {
        let context = match projects.worktree_context(&project_id) {
            Ok(context) => context,
            Err(error) => {
                return WorktreeWorkspaceSnapshot::unavailable(None, map_project_error(error));
            }
        };
        let source_root = match projects.review_root(&context.source_project_id) {
            Ok(root) => root,
            Err(error) => {
                return WorktreeWorkspaceSnapshot::unavailable(
                    Some(context.source_project_id),
                    map_project_error(error),
                );
            }
        };
        let discovered = match list_worktrees(&source_root).await {
            Ok(worktrees) => worktrees,
            Err(error) => {
                return WorktreeWorkspaceSnapshot::unavailable(
                    Some(context.source_project_id),
                    map_git_error(error),
                );
            }
        };
        let recovery_ids =
            match self.refresh_recovery_candidates(&context, &source_root, &discovered, projects) {
                Ok(recovery_ids) => recovery_ids,
                Err(diagnostic_code) => {
                    return WorktreeWorkspaceSnapshot::unavailable(
                        Some(context.source_project_id),
                        diagnostic_code,
                    );
                }
            };
        build_workspace(
            project_id,
            context,
            source_root.worktree_root.clone(),
            discovered,
            &recovery_ids,
        )
    }

    pub async fn preview_create(
        &self,
        request: WorktreeCreatePreviewRequest,
        projects: &ProjectService,
    ) -> WorktreePreviewSnapshot {
        let source_project_id = match projects.worktree_context(&request.project_id) {
            Ok(context) => context.source_project_id,
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    request.project_id,
                    WorktreeOperation::Create,
                    map_project_error(error),
                );
            }
        };
        if !valid_branch_name(&request.branch_name) {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Create,
                WorktreeDiagnosticCode::InvalidBranch,
            );
        }
        let Some(storage_root) = self.storage_root.as_ref() else {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Create,
                WorktreeDiagnosticCode::MetadataUnavailable,
            );
        };
        let source_root = match projects.review_root(&source_project_id) {
            Ok(root) if root.writable => root,
            Ok(_) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Create,
                    WorktreeDiagnosticCode::DirectoryUnavailable,
                );
            }
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Create,
                    map_project_error(error),
                );
            }
        };
        match check_branch(&source_root, &request.branch_name).await {
            Ok(false) => {}
            Ok(true) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Create,
                    WorktreeDiagnosticCode::BranchExists,
                );
            }
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Create,
                    map_git_error(error),
                );
            }
        }
        let base_commit = match read_head(&source_root).await {
            Ok(base_commit) => base_commit,
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Create,
                    map_git_error(error),
                );
            }
        };

        let destination =
            generated_destination(storage_root, &source_project_id, &request.branch_name);
        let confirmation_id = Uuid::now_v7().to_string();
        let pending = PendingWorktree {
            confirmation_id: confirmation_id.clone(),
            expires_at: Instant::now() + CONFIRMATION_TTL,
            source_project_id: source_project_id.clone(),
            source_root: source_root.worktree_root,
            common_dir: source_root.common_dir,
            branch_name: Some(request.branch_name.clone()),
            base_commit: Some(base_commit),
            operation: PendingOperation::Create {
                destination: destination.clone(),
            },
        };
        if !self.replace_pending(pending) {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Create,
                WorktreeDiagnosticCode::MetadataUnavailable,
            );
        }
        WorktreePreviewSnapshot {
            schema_version: WORKTREE_SCHEMA_VERSION,
            state: WorktreePreviewState::Ready,
            source_project_id,
            operation: WorktreeOperation::Create,
            branch_name: Some(request.branch_name),
            display_path: Some(display_path(&destination)),
            ownership: Some(WorktreeOwnership::Managed),
            destructive: false,
            confirmation_id: Some(confirmation_id),
            diagnostic_code: None,
        }
    }

    pub async fn preview_attach(
        &self,
        project_id: String,
        selected_path: PathBuf,
        projects: &ProjectService,
    ) -> WorktreePreviewSnapshot {
        let source_project_id = match projects.worktree_context(&project_id) {
            Ok(context) => context.source_project_id,
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    project_id,
                    WorktreeOperation::Attach,
                    map_project_error(error),
                );
            }
        };
        let source_root = match projects.review_root(&source_project_id) {
            Ok(root) => root,
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Attach,
                    map_project_error(error),
                );
            }
        };
        let candidate = match projects.inspect_worktree_candidate(&selected_path) {
            Ok(candidate)
                if candidate.is_linked_worktree
                    && candidate.resolved_path == candidate.worktree_root =>
            {
                candidate
            }
            Ok(_) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Attach,
                    WorktreeDiagnosticCode::NotLinkedWorktree,
                );
            }
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Attach,
                    map_project_error(error),
                );
            }
        };
        if candidate.common_dir != source_root.common_dir {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Attach,
                WorktreeDiagnosticCode::DifferentRepository,
            );
        }
        let discovered = match list_worktrees(&source_root).await {
            Ok(discovered) => discovered,
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Attach,
                    map_git_error(error),
                );
            }
        };
        let Some(entry) = discovered
            .iter()
            .find(|entry| same_path(&entry.path, &candidate.worktree_root))
        else {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Attach,
                WorktreeDiagnosticCode::StalePreview,
            );
        };
        if entry.locked || entry.prunable {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Attach,
                WorktreeDiagnosticCode::DirectoryUnavailable,
            );
        }
        let branch_name = entry.branch_name.clone();
        let confirmation_id = Uuid::now_v7().to_string();
        let pending = PendingWorktree {
            confirmation_id: confirmation_id.clone(),
            expires_at: Instant::now() + CONFIRMATION_TTL,
            source_project_id: source_project_id.clone(),
            source_root: source_root.worktree_root,
            common_dir: source_root.common_dir,
            branch_name: branch_name.clone(),
            base_commit: None,
            operation: PendingOperation::Attach {
                candidate: candidate.clone(),
            },
        };
        if !self.replace_pending(pending) {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Attach,
                WorktreeDiagnosticCode::MetadataUnavailable,
            );
        }
        WorktreePreviewSnapshot {
            schema_version: WORKTREE_SCHEMA_VERSION,
            state: WorktreePreviewState::Ready,
            source_project_id,
            operation: WorktreeOperation::Attach,
            branch_name,
            display_path: Some(candidate.display_path),
            ownership: Some(WorktreeOwnership::Attached),
            destructive: false,
            confirmation_id: Some(confirmation_id),
            diagnostic_code: None,
        }
    }

    pub async fn preview_recover(
        &self,
        request: WorktreeRecoverPreviewRequest,
        projects: &ProjectService,
    ) -> WorktreePreviewSnapshot {
        let source_project_id = match projects.worktree_context(&request.project_id) {
            Ok(context) => context.source_project_id,
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    request.project_id,
                    WorktreeOperation::Recover,
                    map_project_error(error),
                );
            }
        };
        let Some(recoverable) = self.take_recoverable(&request.recovery_id) else {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Recover,
                WorktreeDiagnosticCode::RecoveryUnavailable,
            );
        };
        if recoverable.expires_at <= Instant::now()
            || recoverable.source_project_id != source_project_id
        {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Recover,
                WorktreeDiagnosticCode::RecoveryUnavailable,
            );
        }
        let source_root = match projects.review_root(&source_project_id) {
            Ok(root) => root,
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Recover,
                    map_project_error(error),
                );
            }
        };
        let current =
            match projects.inspect_worktree_candidate(&recoverable.candidate.selected_path) {
                Ok(current) if current == recoverable.candidate => current,
                Ok(_) => {
                    return WorktreePreviewSnapshot::unavailable(
                        source_project_id,
                        WorktreeOperation::Recover,
                        WorktreeDiagnosticCode::IdentityChanged,
                    );
                }
                Err(error) => {
                    return WorktreePreviewSnapshot::unavailable(
                        source_project_id,
                        WorktreeOperation::Recover,
                        map_project_error(error),
                    );
                }
            };
        if current.common_dir != source_root.common_dir
            || !managed_existing_destination(
                &self.storage_root,
                &source_project_id,
                &current.worktree_root,
            )
        {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Recover,
                WorktreeDiagnosticCode::RecoveryUnavailable,
            );
        }
        let discovered = match list_worktrees(&source_root).await {
            Ok(discovered) => discovered,
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Recover,
                    map_git_error(error),
                );
            }
        };
        let valid = discovered.iter().any(|entry| {
            same_path(&entry.path, &current.worktree_root)
                && entry.branch_name == recoverable.branch_name
                && !entry.locked
                && !entry.prunable
        });
        if !valid {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Recover,
                WorktreeDiagnosticCode::StalePreview,
            );
        }
        let confirmation_id = Uuid::now_v7().to_string();
        let pending = PendingWorktree {
            confirmation_id: confirmation_id.clone(),
            expires_at: Instant::now() + CONFIRMATION_TTL,
            source_project_id: source_project_id.clone(),
            source_root: source_root.worktree_root,
            common_dir: source_root.common_dir,
            branch_name: recoverable.branch_name.clone(),
            base_commit: None,
            operation: PendingOperation::Recover {
                candidate: current.clone(),
            },
        };
        if !self.replace_pending(pending) {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Recover,
                WorktreeDiagnosticCode::MetadataUnavailable,
            );
        }
        WorktreePreviewSnapshot {
            schema_version: WORKTREE_SCHEMA_VERSION,
            state: WorktreePreviewState::Ready,
            source_project_id,
            operation: WorktreeOperation::Recover,
            branch_name: recoverable.branch_name,
            display_path: Some(current.display_path),
            ownership: Some(WorktreeOwnership::Managed),
            destructive: false,
            confirmation_id: Some(confirmation_id),
            diagnostic_code: None,
        }
    }

    pub async fn preview_remove(
        &self,
        request: WorktreeRemovePreviewRequest,
        projects: &ProjectService,
    ) -> WorktreePreviewSnapshot {
        let context = match projects.worktree_context(&request.project_id) {
            Ok(context) => context,
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    request.project_id,
                    WorktreeOperation::Remove,
                    map_project_error(error),
                );
            }
        };
        let source_project_id = context.source_project_id.clone();
        if request.worktree_project_id == request.project_id
            || request.worktree_project_id == source_project_id
        {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Remove,
                WorktreeDiagnosticCode::SourceWorktree,
            );
        }
        let Some(record) = context
            .records
            .iter()
            .find(|record| record.project_id == request.worktree_project_id)
        else {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Remove,
                WorktreeDiagnosticCode::ProjectNotFound,
            );
        };
        if record.ownership != "managed" {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Remove,
                WorktreeDiagnosticCode::UnsupportedOwnership,
            );
        }
        let Some(selected_path) = record.selected_path.clone() else {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Remove,
                WorktreeDiagnosticCode::RecoveryUnavailable,
            );
        };
        let source_root = match projects.review_root(&source_project_id) {
            Ok(root) => root,
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Remove,
                    map_project_error(error),
                );
            }
        };
        if !managed_destination_path(&self.storage_root, &source_project_id, &selected_path) {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Remove,
                WorktreeDiagnosticCode::UnsupportedOwnership,
            );
        }
        let discovered = match list_worktrees(&source_root).await {
            Ok(discovered) => discovered,
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Remove,
                    map_git_error(error),
                );
            }
        };
        let discovered_entry = discovered
            .iter()
            .find(|entry| same_path(&entry.path, &selected_path));
        let (candidate, base_commit, destructive) = if selected_path.exists() {
            let Some(entry) = discovered_entry else {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Remove,
                    WorktreeDiagnosticCode::StalePreview,
                );
            };
            if entry.locked || entry.prunable || entry.branch_name != record.branch_name {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Remove,
                    WorktreeDiagnosticCode::RecoveryUnavailable,
                );
            }
            let candidate = match projects.inspect_worktree_candidate(&selected_path) {
                Ok(candidate)
                    if candidate.common_dir == source_root.common_dir
                        && candidate.worktree_root == candidate.resolved_path
                        && managed_existing_destination(
                            &self.storage_root,
                            &source_project_id,
                            &candidate.worktree_root,
                        ) =>
                {
                    candidate
                }
                Ok(_) => {
                    return WorktreePreviewSnapshot::unavailable(
                        source_project_id,
                        WorktreeOperation::Remove,
                        WorktreeDiagnosticCode::IdentityChanged,
                    );
                }
                Err(error) => {
                    return WorktreePreviewSnapshot::unavailable(
                        source_project_id,
                        WorktreeOperation::Remove,
                        map_project_error(error),
                    );
                }
            };
            let target_root = match projects.cleanup_worktree_root(&request.worktree_project_id) {
                Ok(root) => root,
                Err(error) => {
                    return WorktreePreviewSnapshot::unavailable(
                        source_project_id,
                        WorktreeOperation::Remove,
                        map_project_error(error),
                    );
                }
            };
            match worktree_clean(&target_root).await {
                Ok(true) => {}
                Ok(false) => {
                    return WorktreePreviewSnapshot::unavailable(
                        source_project_id,
                        WorktreeOperation::Remove,
                        WorktreeDiagnosticCode::WorktreeDirty,
                    );
                }
                Err(error) => {
                    return WorktreePreviewSnapshot::unavailable(
                        source_project_id,
                        WorktreeOperation::Remove,
                        map_git_error(error),
                    );
                }
            }
            let base_commit = match read_head(&target_root).await {
                Ok(base_commit) => base_commit,
                Err(error) => {
                    return WorktreePreviewSnapshot::unavailable(
                        source_project_id,
                        WorktreeOperation::Remove,
                        map_git_error(error),
                    );
                }
            };
            (Some(candidate), Some(base_commit), true)
        } else {
            if discovered_entry.is_some() {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Remove,
                    WorktreeDiagnosticCode::RecoveryUnavailable,
                );
            }
            (None, None, false)
        };
        let confirmation_id = Uuid::now_v7().to_string();
        let pending = PendingWorktree {
            confirmation_id: confirmation_id.clone(),
            expires_at: Instant::now() + CONFIRMATION_TTL,
            source_project_id: source_project_id.clone(),
            source_root: source_root.worktree_root,
            common_dir: source_root.common_dir,
            branch_name: record.branch_name.clone(),
            base_commit,
            operation: PendingOperation::Remove {
                worktree_project_id: request.worktree_project_id,
                requesting_project_id: request.project_id,
                selected_path: selected_path.clone(),
                candidate,
            },
        };
        if !self.replace_pending(pending) {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Remove,
                WorktreeDiagnosticCode::MetadataUnavailable,
            );
        }
        WorktreePreviewSnapshot {
            schema_version: WORKTREE_SCHEMA_VERSION,
            state: WorktreePreviewState::Ready,
            source_project_id,
            operation: WorktreeOperation::Remove,
            branch_name: record.branch_name.clone(),
            display_path: Some(display_path(&selected_path)),
            ownership: Some(WorktreeOwnership::Managed),
            destructive,
            confirmation_id: Some(confirmation_id),
            diagnostic_code: None,
        }
    }

    pub fn picker_unavailable(&self, project_id: String) -> WorktreePreviewSnapshot {
        WorktreePreviewSnapshot::unavailable(
            project_id,
            WorktreeOperation::Attach,
            WorktreeDiagnosticCode::PickerUnavailable,
        )
    }

    pub fn picker_cancelled(&self, project_id: String) -> WorktreePreviewSnapshot {
        WorktreePreviewSnapshot {
            schema_version: WORKTREE_SCHEMA_VERSION,
            state: WorktreePreviewState::Cancelled,
            source_project_id: project_id,
            operation: WorktreeOperation::Attach,
            branch_name: None,
            display_path: None,
            ownership: None,
            destructive: false,
            confirmation_id: None,
            diagnostic_code: None,
        }
    }

    pub fn cancel(&self, request: WorktreeCancelRequest) -> bool {
        if !valid_confirmation_id(&request.confirmation_id) {
            return false;
        }
        self.pending
            .lock()
            .map(|mut pending| {
                let matches = pending
                    .as_ref()
                    .is_some_and(|value| value.confirmation_id == request.confirmation_id);
                if matches {
                    *pending = None;
                }
                matches
            })
            .unwrap_or(false)
    }

    pub async fn confirm(
        &self,
        request: WorktreeConfirmRequest,
        projects: &ProjectService,
    ) -> WorktreeResultSnapshot {
        let Some(pending) = self.take_pending(&request.confirmation_id) else {
            return WorktreeResultSnapshot::unavailable(
                None,
                WorktreeDiagnosticCode::ConfirmationExpired,
            );
        };
        let source_project_id = pending.source_project_id.clone();
        if pending.expires_at <= Instant::now() {
            return WorktreeResultSnapshot::unavailable(
                Some(source_project_id),
                WorktreeDiagnosticCode::ConfirmationExpired,
            );
        }
        let reserved = match reserve_project_group(projects, &source_project_id) {
            Ok(reserved) => reserved,
            Err(error) => {
                return WorktreeResultSnapshot::unavailable(
                    Some(source_project_id),
                    map_project_error(error),
                );
            }
        };
        let result = self.confirm_reserved(pending, projects).await;
        for project_id in reserved {
            projects.release_execution(&project_id);
        }
        result
    }

    async fn confirm_reserved(
        &self,
        pending: PendingWorktree,
        projects: &ProjectService,
    ) -> WorktreeResultSnapshot {
        let source_project_id = pending.source_project_id.clone();
        let source_root = match projects.review_root(&source_project_id) {
            Ok(root)
                if root.worktree_root == pending.source_root
                    && root.common_dir == pending.common_dir =>
            {
                root
            }
            Ok(_) => {
                return WorktreeResultSnapshot::unavailable(
                    Some(source_project_id),
                    WorktreeDiagnosticCode::IdentityChanged,
                );
            }
            Err(error) => {
                return WorktreeResultSnapshot::unavailable(
                    Some(source_project_id),
                    map_project_error(error),
                );
            }
        };

        if matches!(&pending.operation, PendingOperation::Remove { .. }) {
            return self.confirm_remove(&pending, &source_root, projects).await;
        }

        let (selected_path, ownership) = match pending.operation {
            PendingOperation::Create { destination } => {
                let branch_name = pending
                    .branch_name
                    .as_deref()
                    .expect("create preview always has a branch name");
                match check_branch(&source_root, branch_name).await {
                    Ok(false) => {}
                    Ok(true) => {
                        return WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            WorktreeDiagnosticCode::BranchExists,
                        );
                    }
                    Err(error) => {
                        return WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            map_git_error(error),
                        );
                    }
                }
                let base_commit = match read_head(&source_root).await {
                    Ok(base_commit)
                        if pending.base_commit.as_deref() == Some(base_commit.as_str()) =>
                    {
                        base_commit
                    }
                    Ok(_) => {
                        return WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            WorktreeDiagnosticCode::StalePreview,
                        );
                    }
                    Err(error) => {
                        return WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            map_git_error(error),
                        );
                    }
                };
                if destination.exists()
                    || !managed_destination_path(
                        &self.storage_root,
                        &source_project_id,
                        &destination,
                    )
                {
                    return WorktreeResultSnapshot::unavailable(
                        Some(source_project_id),
                        WorktreeDiagnosticCode::StalePreview,
                    );
                }
                if let Some(parent) = destination.parent() {
                    if fs::create_dir_all(parent).is_err()
                        || fs::set_permissions(parent, fs::Permissions::from_mode(0o700)).is_err()
                        || fs::symlink_metadata(parent)
                            .map(|metadata| metadata.file_type().is_symlink() || !metadata.is_dir())
                            .unwrap_or(true)
                        || fs::canonicalize(parent).ok().as_deref() != Some(parent)
                    {
                        return WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            WorktreeDiagnosticCode::MetadataUnavailable,
                        );
                    }
                }
                match add_worktree(&source_root, branch_name, &destination, &base_commit).await {
                    Ok(()) => (destination, WorktreeOwnership::Managed),
                    Err(error) => {
                        let mut result = WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            if destination.exists() {
                                WorktreeDiagnosticCode::WorktreeRemains
                            } else {
                                map_git_error(error)
                            },
                        );
                        if destination.exists() {
                            result.recoverable_display_path = Some(display_path(&destination));
                        }
                        return result;
                    }
                }
            }
            PendingOperation::Attach { candidate } => {
                let current = match projects.inspect_worktree_candidate(&candidate.selected_path) {
                    Ok(current) if current == candidate => current,
                    Ok(_) => {
                        return WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            WorktreeDiagnosticCode::IdentityChanged,
                        );
                    }
                    Err(error) => {
                        return WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            map_project_error(error),
                        );
                    }
                };
                let discovered = match list_worktrees(&source_root).await {
                    Ok(discovered) => discovered,
                    Err(error) => {
                        return WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            map_git_error(error),
                        );
                    }
                };
                let valid = discovered.iter().any(|entry| {
                    same_path(&entry.path, &current.worktree_root)
                        && entry.branch_name == pending.branch_name
                        && !entry.locked
                        && !entry.prunable
                });
                if !valid || current.common_dir != pending.common_dir {
                    return WorktreeResultSnapshot::unavailable(
                        Some(source_project_id),
                        WorktreeDiagnosticCode::StalePreview,
                    );
                }
                (current.selected_path, WorktreeOwnership::Attached)
            }
            PendingOperation::Recover { candidate } => {
                let current = match projects.inspect_worktree_candidate(&candidate.selected_path) {
                    Ok(current) if current == candidate => current,
                    Ok(_) => {
                        return WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            WorktreeDiagnosticCode::IdentityChanged,
                        );
                    }
                    Err(error) => {
                        return WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            map_project_error(error),
                        );
                    }
                };
                let discovered = match list_worktrees(&source_root).await {
                    Ok(discovered) => discovered,
                    Err(error) => {
                        return WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            map_git_error(error),
                        );
                    }
                };
                let valid = current.common_dir == pending.common_dir
                    && managed_existing_destination(
                        &self.storage_root,
                        &source_project_id,
                        &current.worktree_root,
                    )
                    && discovered.iter().any(|entry| {
                        same_path(&entry.path, &current.worktree_root)
                            && entry.branch_name == pending.branch_name
                            && !entry.locked
                            && !entry.prunable
                    });
                if !valid {
                    return WorktreeResultSnapshot::unavailable(
                        Some(source_project_id),
                        WorktreeDiagnosticCode::StalePreview,
                    );
                }
                (current.selected_path, WorktreeOwnership::Managed)
            }
            PendingOperation::Remove { .. } => {
                unreachable!("remove is handled before registration")
            }
        };

        let ownership_value = ownership
            .as_storage_value()
            .expect("persisted ownership must have a storage value");
        let project_id = match projects.register_worktree_project(
            &source_project_id,
            &selected_path,
            &pending.common_dir,
            ownership_value,
            pending.branch_name.as_deref(),
        ) {
            Ok(project_id) => project_id,
            Err(error) => {
                let mut result = WorktreeResultSnapshot::unavailable(
                    Some(source_project_id),
                    map_registration_error(error),
                );
                if ownership == WorktreeOwnership::Managed {
                    result.diagnostic_code = Some(WorktreeDiagnosticCode::WorktreeRemains);
                    result.recoverable_display_path = Some(display_path(&selected_path));
                }
                return result;
            }
        };
        let workspace = self.status(source_project_id.clone(), projects).await;
        WorktreeResultSnapshot {
            schema_version: WORKTREE_SCHEMA_VERSION,
            state: WorktreeResultState::Applied,
            source_project_id: Some(source_project_id),
            project_id: Some(project_id),
            workspace: Some(workspace),
            recoverable_display_path: None,
            diagnostic_code: None,
        }
    }

    async fn confirm_remove(
        &self,
        pending: &PendingWorktree,
        source_root: &ProjectReviewRoot,
        projects: &ProjectService,
    ) -> WorktreeResultSnapshot {
        let PendingOperation::Remove {
            worktree_project_id,
            requesting_project_id,
            selected_path,
            candidate,
        } = &pending.operation
        else {
            unreachable!("remove confirmation requires a remove plan");
        };
        let source_project_id = pending.source_project_id.clone();
        if !managed_destination_path(&self.storage_root, &source_project_id, selected_path) {
            return WorktreeResultSnapshot::unavailable(
                Some(source_project_id),
                WorktreeDiagnosticCode::IdentityChanged,
            );
        }
        let current_context = match projects.worktree_context(&source_project_id) {
            Ok(context) => context,
            Err(error) => {
                return WorktreeResultSnapshot::unavailable(
                    Some(source_project_id),
                    map_project_error(error),
                );
            }
        };
        let relation_still_matches = current_context.records.iter().any(|record| {
            record.project_id == *worktree_project_id
                && record.ownership == "managed"
                && record.selected_path.as_deref() == Some(selected_path.as_path())
                && record.branch_name == pending.branch_name
        });
        if !relation_still_matches {
            return WorktreeResultSnapshot::unavailable(
                Some(source_project_id),
                WorktreeDiagnosticCode::StalePreview,
            );
        }
        let discovered = match list_worktrees(source_root).await {
            Ok(discovered) => discovered,
            Err(error) => {
                return WorktreeResultSnapshot::unavailable(
                    Some(source_project_id),
                    map_git_error(error),
                );
            }
        };

        if let Some(candidate) = candidate {
            let current = match projects.inspect_worktree_candidate(&candidate.selected_path) {
                Ok(current) if current == *candidate => current,
                Ok(_) => {
                    return WorktreeResultSnapshot::unavailable(
                        Some(source_project_id),
                        WorktreeDiagnosticCode::IdentityChanged,
                    );
                }
                Err(error) => {
                    return WorktreeResultSnapshot::unavailable(
                        Some(source_project_id),
                        map_project_error(error),
                    );
                }
            };
            if current.common_dir != pending.common_dir
                || !managed_existing_destination(
                    &self.storage_root,
                    &source_project_id,
                    &current.worktree_root,
                )
            {
                return WorktreeResultSnapshot::unavailable(
                    Some(source_project_id),
                    WorktreeDiagnosticCode::IdentityChanged,
                );
            }
            let valid = discovered.iter().any(|entry| {
                same_path(&entry.path, &current.worktree_root)
                    && entry.branch_name == pending.branch_name
                    && !entry.locked
                    && !entry.prunable
            });
            if !valid {
                return WorktreeResultSnapshot::unavailable(
                    Some(source_project_id),
                    WorktreeDiagnosticCode::StalePreview,
                );
            }
            let target_root = match projects.cleanup_worktree_root(worktree_project_id) {
                Ok(root)
                    if root.worktree_root == current.worktree_root
                        && root.common_dir == pending.common_dir =>
                {
                    root
                }
                Ok(_) => {
                    return WorktreeResultSnapshot::unavailable(
                        Some(source_project_id),
                        WorktreeDiagnosticCode::IdentityChanged,
                    );
                }
                Err(error) => {
                    return WorktreeResultSnapshot::unavailable(
                        Some(source_project_id),
                        map_project_error(error),
                    );
                }
            };
            match worktree_clean(&target_root).await {
                Ok(true) => {}
                Ok(false) => {
                    return WorktreeResultSnapshot::unavailable(
                        Some(source_project_id),
                        WorktreeDiagnosticCode::WorktreeDirty,
                    );
                }
                Err(error) => {
                    return WorktreeResultSnapshot::unavailable(
                        Some(source_project_id),
                        map_git_error(error),
                    );
                }
            }
            match read_head(&target_root).await {
                Ok(head) if pending.base_commit.as_deref() == Some(head.as_str()) => {}
                Ok(_) => {
                    return WorktreeResultSnapshot::unavailable(
                        Some(source_project_id),
                        WorktreeDiagnosticCode::StalePreview,
                    );
                }
                Err(error) => {
                    return WorktreeResultSnapshot::unavailable(
                        Some(source_project_id),
                        map_git_error(error),
                    );
                }
            }
            if let Err(error) = remove_worktree(source_root, selected_path).await {
                return WorktreeResultSnapshot::unavailable(
                    Some(source_project_id),
                    map_git_error(error),
                );
            }
            let after = match list_worktrees(source_root).await {
                Ok(discovered) => discovered,
                Err(error) => {
                    return WorktreeResultSnapshot::unavailable(
                        Some(source_project_id),
                        map_git_error(error),
                    );
                }
            };
            let branch_preserved = match pending.branch_name.as_deref() {
                Some(branch_name) => check_branch(source_root, branch_name).await == Ok(true),
                None => false,
            };
            if selected_path.exists()
                || after
                    .iter()
                    .any(|entry| same_path(&entry.path, selected_path))
                || !branch_preserved
            {
                return WorktreeResultSnapshot::unavailable(
                    Some(source_project_id),
                    WorktreeDiagnosticCode::CleanupIncomplete,
                );
            }
        } else if selected_path.exists()
            || discovered
                .iter()
                .any(|entry| same_path(&entry.path, selected_path))
        {
            return WorktreeResultSnapshot::unavailable(
                Some(source_project_id),
                WorktreeDiagnosticCode::RecoveryUnavailable,
            );
        }

        if let Err(error) =
            projects.retire_worktree_project(&source_project_id, worktree_project_id, "managed")
        {
            let mut result = WorktreeResultSnapshot::unavailable(
                Some(source_project_id.clone()),
                if matches!(error, ProjectExecutionError::MetadataUnavailable) {
                    WorktreeDiagnosticCode::CleanupIncomplete
                } else {
                    map_project_error(error)
                },
            );
            result.workspace = Some(self.status(requesting_project_id.clone(), projects).await);
            return result;
        }
        WorktreeResultSnapshot {
            schema_version: WORKTREE_SCHEMA_VERSION,
            state: WorktreeResultState::Applied,
            source_project_id: Some(source_project_id),
            project_id: Some(requesting_project_id.clone()),
            workspace: Some(self.status(requesting_project_id.clone(), projects).await),
            recoverable_display_path: None,
            diagnostic_code: None,
        }
    }

    fn refresh_recovery_candidates(
        &self,
        context: &ProjectWorktreeContext,
        source_root: &ProjectReviewRoot,
        discovered: &[DiscoveredWorktree],
        projects: &ProjectService,
    ) -> Result<HashMap<PathBuf, String>, WorktreeDiagnosticCode> {
        let registered: Vec<PathBuf> = context
            .records
            .iter()
            .filter_map(|record| {
                record
                    .selected_path
                    .as_ref()
                    .map(|path| normalized_path(path))
            })
            .collect();
        let mut candidates = HashMap::new();
        let mut ids = HashMap::new();
        for entry in discovered {
            if same_path(&entry.path, &source_root.worktree_root)
                || registered.iter().any(|path| same_path(path, &entry.path))
                || entry.locked
                || entry.prunable
                || !managed_existing_destination(
                    &self.storage_root,
                    &context.source_project_id,
                    &entry.path,
                )
            {
                continue;
            }
            let candidate = match projects.inspect_worktree_candidate(&entry.path) {
                Ok(candidate)
                    if candidate.common_dir == source_root.common_dir
                        && candidate.worktree_root == candidate.resolved_path =>
                {
                    candidate
                }
                _ => continue,
            };
            let recovery_id = Uuid::now_v7().to_string();
            ids.insert(normalized_path(&entry.path), recovery_id.clone());
            candidates.insert(
                recovery_id,
                RecoverableWorktree {
                    expires_at: Instant::now() + CONFIRMATION_TTL,
                    source_project_id: context.source_project_id.clone(),
                    candidate,
                    branch_name: entry.branch_name.clone(),
                },
            );
        }
        let mut recoverable = self
            .recoverable
            .lock()
            .map_err(|_| WorktreeDiagnosticCode::MetadataUnavailable)?;
        *recoverable = candidates;
        Ok(ids)
    }

    fn take_recoverable(&self, recovery_id: &str) -> Option<RecoverableWorktree> {
        if !valid_confirmation_id(recovery_id) {
            return None;
        }
        self.recoverable
            .lock()
            .ok()
            .and_then(|mut recoverable| recoverable.remove(recovery_id))
    }

    fn replace_pending(&self, pending: PendingWorktree) -> bool {
        self.pending
            .lock()
            .map(|mut current| *current = Some(pending))
            .is_ok()
    }

    fn take_pending(&self, confirmation_id: &str) -> Option<PendingWorktree> {
        if !valid_confirmation_id(confirmation_id) {
            return None;
        }
        self.pending.lock().ok().and_then(|mut current| {
            let matches = current
                .as_ref()
                .is_some_and(|pending| pending.confirmation_id == confirmation_id);
            if matches {
                current.take()
            } else {
                None
            }
        })
    }
}

fn prepare_storage_root(path: &Path) -> Result<PathBuf, ()> {
    if !path.is_absolute() || path.to_str().is_none() {
        return Err(());
    }
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(());
        }
    } else {
        fs::create_dir_all(path).map_err(|_| ())?;
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|_| ())?;
    fs::canonicalize(path).map_err(|_| ())
}

fn generated_destination(root: &Path, source_project_id: &str, branch_name: &str) -> PathBuf {
    let slug: String = branch_name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '-'
            }
        })
        .take(64)
        .collect();
    let suffix = Uuid::now_v7().simple().to_string();
    root.join(source_project_id)
        .join(format!("{slug}-{}", &suffix[..8]))
}

fn managed_destination_path(
    root: &Option<PathBuf>,
    source_project_id: &str,
    destination: &Path,
) -> bool {
    root.as_ref().is_some_and(|root| {
        destination.is_absolute()
            && destination.to_str().is_some()
            && destination.parent() == Some(root.join(source_project_id).as_path())
    })
}

fn managed_existing_destination(
    root: &Option<PathBuf>,
    source_project_id: &str,
    destination: &Path,
) -> bool {
    if !managed_destination_path(root, source_project_id, destination) {
        return false;
    }
    let Some(root) = root.as_ref() else {
        return false;
    };
    let Some(parent) = destination.parent() else {
        return false;
    };
    let valid_type = |path: &Path| {
        fs::symlink_metadata(path)
            .map(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink())
            .unwrap_or(false)
    };
    valid_type(parent)
        && valid_type(destination)
        && fs::canonicalize(parent).ok().as_deref() == Some(root.join(source_project_id).as_path())
        && fs::canonicalize(destination).ok().as_deref() == Some(destination)
}

fn valid_branch_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_BRANCH_BYTES
        && value.is_ascii()
        && !value.starts_with('-')
        && !value.starts_with('/')
        && !value.ends_with('/')
        && !value.ends_with('.')
        && !value.ends_with(".lock")
        && value != "HEAD"
        && !value.contains("..")
        && !value.contains("@{")
        && !value.contains("//")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"/._-".contains(&byte))
}

fn valid_confirmation_id(value: &str) -> bool {
    value.len() == 36 && Uuid::parse_str(value).is_ok()
}

fn reserve_project_group(
    projects: &ProjectService,
    source_project_id: &str,
) -> Result<Vec<String>, ProjectExecutionError> {
    let context = projects.worktree_context(source_project_id)?;
    let mut project_ids = vec![context.source_project_id];
    project_ids.extend(context.records.into_iter().map(|record| record.project_id));
    project_ids.sort();
    project_ids.dedup();
    let mut reserved: Vec<String> = Vec::with_capacity(project_ids.len());
    for project_id in project_ids {
        if let Err(error) = projects.reserve_execution(&project_id) {
            for reserved_id in &reserved {
                projects.release_execution(reserved_id);
            }
            return Err(error);
        }
        reserved.push(project_id);
    }
    Ok(reserved)
}

fn build_workspace(
    current_project_id: String,
    context: ProjectWorktreeContext,
    source_worktree_root: PathBuf,
    discovered: Vec<DiscoveredWorktree>,
    recovery_ids: &HashMap<PathBuf, String>,
) -> WorktreeWorkspaceSnapshot {
    let mut records: HashMap<PathBuf, _> = context
        .records
        .into_iter()
        .filter_map(|record| {
            record
                .selected_path
                .clone()
                .map(|path| (normalized_path(&path), record))
        })
        .collect();
    let mut entries = Vec::new();
    for discovered_entry in discovered {
        let normalized = normalized_path(&discovered_entry.path);
        let source = same_path(&discovered_entry.path, &source_worktree_root);
        let record = records.remove(&normalized);
        let (project_id, display_name, ownership, branch_name, archived) = if source {
            (
                Some(context.source_project_id.clone()),
                context.source_display_name.clone(),
                WorktreeOwnership::Source,
                discovered_entry.branch_name.clone(),
                false,
            )
        } else if let Some(record) = record {
            let ownership = WorktreeOwnership::from_storage_value(&record.ownership)
                .unwrap_or(WorktreeOwnership::External);
            (
                Some(record.project_id),
                record.display_name,
                ownership,
                record.branch_name.or(discovered_entry.branch_name.clone()),
                record.archived,
            )
        } else {
            (
                None,
                discovered_entry
                    .branch_name
                    .clone()
                    .unwrap_or_else(|| directory_name(&discovered_entry.path)),
                WorktreeOwnership::External,
                discovered_entry.branch_name.clone(),
                false,
            )
        };
        let state = if archived {
            WorktreeEntryState::Archived
        } else if discovered_entry.prunable {
            WorktreeEntryState::Prunable
        } else if discovered_entry.locked {
            WorktreeEntryState::Locked
        } else if discovered_entry.detached {
            WorktreeEntryState::Detached
        } else {
            WorktreeEntryState::Ready
        };
        entries.push(WorktreeEntry {
            current: project_id.as_deref() == Some(&current_project_id),
            project_id,
            recovery_id: recovery_ids.get(&normalized).cloned(),
            display_name,
            display_path: display_path(&discovered_entry.path),
            branch_name,
            ownership,
            state,
        });
    }
    for (_, record) in records {
        let Some(path) = record.selected_path else {
            continue;
        };
        let ownership = WorktreeOwnership::from_storage_value(&record.ownership)
            .unwrap_or(WorktreeOwnership::External);
        entries.push(WorktreeEntry {
            current: record.project_id == current_project_id,
            project_id: Some(record.project_id),
            recovery_id: None,
            display_name: record.display_name,
            display_path: display_path(&path),
            branch_name: record.branch_name,
            ownership,
            state: if record.archived {
                WorktreeEntryState::Archived
            } else {
                WorktreeEntryState::Missing
            },
        });
    }
    entries.sort_by(|left, right| {
        ownership_rank(left.ownership)
            .cmp(&ownership_rank(right.ownership))
            .then_with(|| left.display_name.cmp(&right.display_name))
    });
    let truncated = entries.len() > MAX_WORKTREES;
    entries.truncate(MAX_WORKTREES);
    WorktreeWorkspaceSnapshot {
        schema_version: WORKTREE_SCHEMA_VERSION,
        state: if entries.is_empty() {
            WorktreeWorkspaceState::Empty
        } else {
            WorktreeWorkspaceState::Ready
        },
        source_project_id: Some(context.source_project_id),
        worktrees: entries,
        truncated,
        diagnostic_code: None,
    }
}

fn ownership_rank(ownership: WorktreeOwnership) -> u8 {
    match ownership {
        WorktreeOwnership::Source => 0,
        WorktreeOwnership::Managed => 1,
        WorktreeOwnership::Attached => 2,
        WorktreeOwnership::External => 3,
    }
}

fn directory_name(path: &Path) -> String {
    path.file_name()
        .and_then(OsStr::to_str)
        .filter(|value| !value.is_empty())
        .unwrap_or("Detached worktree")
        .chars()
        .take(120)
        .collect()
}

fn normalized_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn same_path(left: &Path, right: &Path) -> bool {
    normalized_path(left) == normalized_path(right)
}

fn display_path(path: &Path) -> String {
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        if path == home {
            return "~".to_owned();
        }
        if let Ok(relative) = path.strip_prefix(home) {
            return format!("~/{}", relative.to_string_lossy());
        }
    }
    path.to_string_lossy().into_owned()
}

async fn check_branch(root: &ProjectReviewRoot, branch_name: &str) -> Result<bool, GitRunError> {
    let checked = run_git(
        &root.attached_root,
        [
            OsString::from("check-ref-format"),
            OsString::from("--branch"),
            OsString::from(branch_name),
        ],
        false,
    )
    .await?;
    if !checked.success {
        return Err(GitRunError::Failed);
    }
    let reference = format!("refs/heads/{branch_name}");
    let existing = run_git(
        &root.attached_root,
        [
            OsString::from("show-ref"),
            OsString::from("--verify"),
            OsString::from("--quiet"),
            OsString::from(reference),
        ],
        false,
    )
    .await?;
    match existing.code {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(GitRunError::Failed),
    }
}

async fn add_worktree(
    root: &ProjectReviewRoot,
    branch_name: &str,
    destination: &Path,
    base_commit: &str,
) -> Result<(), GitRunError> {
    let filter_overrides = checkout_filter_overrides(root).await?;
    let output = run_git_with_config(
        &root.attached_root,
        [
            OsString::from("worktree"),
            OsString::from("add"),
            OsString::from("--no-track"),
            OsString::from("-b"),
            OsString::from(branch_name),
            destination.as_os_str().to_os_string(),
            OsString::from(base_commit),
        ],
        true,
        &filter_overrides,
    )
    .await?;
    if output.success {
        Ok(())
    } else {
        Err(GitRunError::Failed)
    }
}

async fn worktree_clean(root: &ProjectReviewRoot) -> Result<bool, GitRunError> {
    let filter_overrides = checkout_filter_overrides(root).await?;
    let output = run_git_with_config(
        &root.attached_root,
        [
            OsString::from("status"),
            OsString::from("--porcelain=v2"),
            OsString::from("--untracked-files=all"),
            OsString::from("--ignore-submodules=none"),
            OsString::from("-z"),
        ],
        false,
        &filter_overrides,
    )
    .await?;
    if !output.success {
        return Err(GitRunError::Failed);
    }
    Ok(output.stdout.is_empty())
}

async fn remove_worktree(root: &ProjectReviewRoot, destination: &Path) -> Result<(), GitRunError> {
    let filter_overrides = checkout_filter_overrides(root).await?;
    let output = run_git_with_config(
        &root.attached_root,
        [
            OsString::from("worktree"),
            OsString::from("remove"),
            destination.as_os_str().to_os_string(),
        ],
        true,
        &filter_overrides,
    )
    .await?;
    if output.success {
        Ok(())
    } else {
        Err(GitRunError::Failed)
    }
}

async fn checkout_filter_overrides(root: &ProjectReviewRoot) -> Result<Vec<String>, GitRunError> {
    let output = run_git(
        &root.attached_root,
        [
            OsString::from("config"),
            OsString::from("--null"),
            OsString::from("--name-only"),
            OsString::from("--get-regexp"),
            OsString::from(r"^filter\..*\.(clean|smudge|process|required)$"),
        ],
        false,
    )
    .await?;
    if output.code == Some(1) {
        return Ok(Vec::new());
    }
    if !output.success {
        return Err(GitRunError::Failed);
    }
    let mut drivers = Vec::new();
    for record in output.stdout.split(|byte| *byte == 0) {
        if record.is_empty() {
            continue;
        }
        let key = std::str::from_utf8(record).map_err(|_| GitRunError::Failed)?;
        let normalized = key.to_ascii_lowercase();
        let Some(remainder) = normalized.strip_prefix("filter.") else {
            return Err(GitRunError::Failed);
        };
        let Some((driver, property)) = remainder.rsplit_once('.') else {
            return Err(GitRunError::Failed);
        };
        if driver.is_empty()
            || driver.len() > 128
            || !driver
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
            || !matches!(property, "clean" | "smudge" | "process" | "required")
        {
            return Err(GitRunError::Failed);
        }
        drivers.push(driver.to_owned());
    }
    drivers.sort();
    drivers.dedup();
    if drivers.len() > 64 {
        return Err(GitRunError::TooLarge);
    }
    let mut overrides = Vec::with_capacity(drivers.len() * 4);
    for driver in drivers {
        overrides.extend([
            format!("filter.{driver}.clean=/bin/cat"),
            format!("filter.{driver}.smudge=/bin/cat"),
            format!("filter.{driver}.process="),
            format!("filter.{driver}.required=false"),
        ]);
    }
    Ok(overrides)
}

async fn read_head(root: &ProjectReviewRoot) -> Result<String, GitRunError> {
    let output = run_git(
        &root.attached_root,
        [
            OsString::from("rev-parse"),
            OsString::from("--verify"),
            OsString::from("HEAD"),
        ],
        false,
    )
    .await?;
    if !output.success {
        return Err(GitRunError::Failed);
    }
    let value = std::str::from_utf8(&output.stdout)
        .map_err(|_| GitRunError::Failed)?
        .trim();
    if !matches!(value.len(), 40 | 64) || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(GitRunError::Failed);
    }
    Ok(value.to_ascii_lowercase())
}

async fn list_worktrees(root: &ProjectReviewRoot) -> Result<Vec<DiscoveredWorktree>, GitRunError> {
    let output = run_git(
        &root.attached_root,
        [
            OsString::from("worktree"),
            OsString::from("list"),
            OsString::from("--porcelain"),
            OsString::from("-z"),
        ],
        false,
    )
    .await?;
    if !output.success {
        return Err(GitRunError::Failed);
    }
    parse_worktree_list(&output.stdout)
}

fn parse_worktree_list(bytes: &[u8]) -> Result<Vec<DiscoveredWorktree>, GitRunError> {
    let mut worktrees = Vec::new();
    let mut current: Option<DiscoveredWorktree> = None;
    for record in bytes.split(|byte| *byte == 0) {
        if record.is_empty() {
            if let Some(entry) = current.take() {
                worktrees.push(entry);
                if worktrees.len() > MAX_DISCOVERED_WORKTREES {
                    return Err(GitRunError::TooLarge);
                }
            }
            continue;
        }
        let text = std::str::from_utf8(record).map_err(|_| GitRunError::Failed)?;
        if let Some(path) = text.strip_prefix("worktree ") {
            if current.is_some() || !valid_inventory_path(path) {
                return Err(GitRunError::Failed);
            }
            current = Some(DiscoveredWorktree {
                path: PathBuf::from(path),
                branch_name: None,
                detached: false,
                locked: false,
                prunable: false,
            });
        } else if let Some(entry) = current.as_mut() {
            if let Some(branch) = text.strip_prefix("branch refs/heads/") {
                if valid_branch_name(branch) {
                    entry.branch_name = Some(branch.to_owned());
                } else {
                    return Err(GitRunError::Failed);
                }
            } else if text == "detached" {
                entry.detached = true;
            } else if text == "locked" || text.starts_with("locked ") {
                entry.locked = true;
            } else if text == "prunable" || text.starts_with("prunable ") {
                entry.prunable = true;
            } else if !text.starts_with("HEAD ") && !text.starts_with("bare") {
                return Err(GitRunError::Failed);
            }
        } else {
            return Err(GitRunError::Failed);
        }
    }
    if let Some(entry) = current {
        worktrees.push(entry);
    }
    if worktrees.len() > MAX_DISCOVERED_WORKTREES {
        return Err(GitRunError::TooLarge);
    }
    if worktrees.iter().any(|entry| !entry.path.is_absolute()) {
        return Err(GitRunError::Failed);
    }
    Ok(worktrees)
}

fn valid_inventory_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 4096
        && !value.contains('\\')
        && !value.chars().any(|character| {
            character.is_control()
                || matches!(
                    character,
                    '\u{061c}'
                        | '\u{200e}'
                        | '\u{200f}'
                        | '\u{202a}'..='\u{202e}'
                        | '\u{2066}'..='\u{2069}'
                )
        })
}

async fn run_git<const N: usize>(
    cwd: &Path,
    arguments: [OsString; N],
    mutation: bool,
) -> Result<GitOutput, GitRunError> {
    run_git_with_config(cwd, arguments, mutation, &[]).await
}

async fn run_git_with_config<const N: usize>(
    cwd: &Path,
    arguments: [OsString; N],
    mutation: bool,
    extra_config: &[String],
) -> Result<GitOutput, GitRunError> {
    let mut command = Command::new("git");
    command
        .current_dir(cwd)
        .env_clear()
        .env("PATH", "/usr/local/bin:/usr/bin:/bin")
        .env("HOME", "/nonexistent")
        .env("XDG_CONFIG_HOME", "/nonexistent")
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_ATTR_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_LITERAL_PATHSPECS", "1")
        .env("GIT_PAGER", "cat")
        .env("PAGER", "cat")
        .arg("--no-pager")
        .args(["-c", "core.quotepath=false"])
        .args(["-c", "color.ui=false"])
        .args(["-c", "core.hooksPath=/dev/null"])
        .args(["-c", "core.fsmonitor=false"])
        .args(["-c", "credential.helper="])
        .args(["-c", "submodule.recurse=false"]);
    for value in extra_config {
        command.arg("-c").arg(value);
    }
    if !mutation {
        command.env("GIT_OPTIONAL_LOCKS", "0");
    }
    command
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = command.spawn().map_err(|_| GitRunError::Unavailable)?;
    let stdout = child.stdout.take().ok_or(GitRunError::Unavailable)?;
    let stderr = child.stderr.take().ok_or(GitRunError::Unavailable)?;
    let result = timeout(GIT_TIMEOUT, async {
        let (stdout, stderr, status) = tokio::join!(
            read_bounded(stdout, MAX_OUTPUT_BYTES),
            read_bounded(stderr, MAX_STDERR_BYTES),
            child.wait(),
        );
        (stdout, stderr, status)
    })
    .await;
    let (stdout, stderr, status) = match result {
        Ok(result) => result,
        Err(_) => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            return Err(GitRunError::TimedOut);
        }
    };
    let stdout = stdout?;
    let _ = stderr?;
    let status = status.map_err(|_| GitRunError::Failed)?;
    Ok(GitOutput {
        stdout,
        success: status.success(),
        code: status.code(),
    })
}

async fn read_bounded(
    reader: impl tokio::io::AsyncRead + Unpin,
    limit: usize,
) -> Result<Vec<u8>, GitRunError> {
    let mut bytes = Vec::with_capacity(limit.min(16 * 1024));
    reader
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| GitRunError::Failed)?;
    if bytes.len() > limit {
        return Err(GitRunError::TooLarge);
    }
    Ok(bytes)
}

fn map_project_error(error: ProjectExecutionError) -> WorktreeDiagnosticCode {
    match error {
        ProjectExecutionError::InvalidProjectId | ProjectExecutionError::ProjectNotFound => {
            WorktreeDiagnosticCode::ProjectNotFound
        }
        ProjectExecutionError::MetadataUnavailable => WorktreeDiagnosticCode::MetadataUnavailable,
        ProjectExecutionError::DirectoryUnavailable | ProjectExecutionError::NotWritable => {
            WorktreeDiagnosticCode::DirectoryUnavailable
        }
        ProjectExecutionError::IdentityChanged => WorktreeDiagnosticCode::IdentityChanged,
        ProjectExecutionError::NotRepository => WorktreeDiagnosticCode::NotRepository,
        ProjectExecutionError::ProjectBusy => WorktreeDiagnosticCode::ProjectBusy,
    }
}

fn map_git_error(error: GitRunError) -> WorktreeDiagnosticCode {
    match error {
        GitRunError::Unavailable => WorktreeDiagnosticCode::GitUnavailable,
        GitRunError::TooLarge => WorktreeDiagnosticCode::OutputTooLarge,
        GitRunError::Failed | GitRunError::TimedOut => WorktreeDiagnosticCode::GitFailed,
    }
}

fn map_registration_error(error: WorktreeRegistrationError) -> WorktreeDiagnosticCode {
    match error {
        WorktreeRegistrationError::Project(error) => map_project_error(error),
        WorktreeRegistrationError::DuplicateDirectory => WorktreeDiagnosticCode::DuplicateDirectory,
        WorktreeRegistrationError::NotLinkedWorktree => WorktreeDiagnosticCode::NotLinkedWorktree,
        WorktreeRegistrationError::DifferentRepository => {
            WorktreeDiagnosticCode::DifferentRepository
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        os::unix::fs::{symlink, PermissionsExt},
        process::Command as StdCommand,
    };

    use uuid::Uuid;

    use super::{
        parse_worktree_list, types::WorktreeDiagnosticCode, types::WorktreeOwnership,
        types::WorktreePreviewState, types::WorktreeResultState, types::WorktreeWorkspaceState,
        valid_branch_name, WorktreeOperation, WorktreeService,
    };
    use crate::project::ProjectService;
    use crate::worktree::types::{
        WorktreeConfirmRequest, WorktreeCreatePreviewRequest, WorktreeRecoverPreviewRequest,
        WorktreeRemovePreviewRequest,
    };

    fn temporary_directory(label: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("quireforge-{label}-{}", Uuid::now_v7()));
        fs::create_dir_all(&path).expect("temporary directory must be created");
        path
    }

    fn repository_fixture() -> (std::path::PathBuf, ProjectService, String) {
        let root = temporary_directory("worktree-repository");
        assert!(StdCommand::new("git")
            .args(["init", "--quiet"])
            .current_dir(&root)
            .status()
            .expect("Git must run")
            .success());
        assert!(StdCommand::new("git")
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--allow-empty",
                "--quiet",
                "-m",
                "initial",
            ])
            .current_dir(&root)
            .status()
            .expect("Git must run")
            .success());
        let projects = ProjectService::in_memory();
        projects.prepare_attachment(root.clone());
        let attached = projects.confirm_pending();
        (root, projects, attached.projects[0].id.clone())
    }

    #[test]
    fn validates_a_bounded_non_option_branch_contract() {
        assert!(valid_branch_name("feature/worktree-11a"));
        for invalid in [
            "", "-force", "HEAD", "a..b", "a@{b", "a.lock", "a b", "a\\b",
        ] {
            assert!(!valid_branch_name(invalid), "{invalid} must be rejected");
        }
    }

    #[test]
    fn parses_porcelain_without_retaining_object_ids() {
        let bytes = b"worktree /tmp/source\0HEAD 0123456789abcdef\0branch refs/heads/main\0\0worktree /tmp/linked\0HEAD fedcba9876543210\0detached\0locked reason\0\0";
        let parsed = parse_worktree_list(bytes).expect("fixture must parse");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].branch_name.as_deref(), Some("main"));
        assert!(parsed[1].detached);
        assert!(parsed[1].locked);
        let serialized = format!("{parsed:?}");
        assert!(!serialized.contains("0123456789abcdef"));
        assert!(!serialized.contains("fedcba9876543210"));
    }

    #[tokio::test]
    async fn creates_a_confirmed_managed_worktree_and_registers_a_project() {
        let (repository, projects, project_id) = repository_fixture();
        let app_data = temporary_directory("worktree-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));
        let preview = service
            .preview_create(
                WorktreeCreatePreviewRequest {
                    project_id: project_id.clone(),
                    branch_name: "feature/managed-fixture".to_owned(),
                },
                &projects,
            )
            .await;
        assert_eq!(preview.state, WorktreePreviewState::Ready);
        assert!(!preview.destructive);
        let result = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: preview.confirmation_id.expect("confirmation must exist"),
                },
                &projects,
            )
            .await;
        assert_eq!(result.state, WorktreeResultState::Applied);
        let workspace = result.workspace.expect("workspace must be returned");
        assert_eq!(workspace.state, WorktreeWorkspaceState::Ready);
        assert!(workspace.worktrees.iter().any(|entry| {
            entry.ownership == WorktreeOwnership::Managed
                && entry.branch_name.as_deref() == Some("feature/managed-fixture")
        }));
        drop(service);
        drop(projects);
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("app data fixture must be removed");
    }

    #[tokio::test]
    async fn recognizes_the_source_when_the_project_attaches_a_repository_subdirectory() {
        let (repository, projects, project_id) = repository_fixture();
        let subdirectory = repository.join("nested-project");
        fs::create_dir(&subdirectory).expect("project subdirectory must exist");
        projects.prepare_relink(project_id.clone(), subdirectory);
        let relinked = projects.confirm_pending();
        assert!(relinked.diagnostic_code.is_none());
        let app_data = temporary_directory("worktree-subdirectory-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));

        let workspace = service.status(project_id.clone(), &projects).await;

        assert_eq!(workspace.state, WorktreeWorkspaceState::Ready);
        let source = workspace
            .worktrees
            .iter()
            .find(|entry| entry.ownership == WorktreeOwnership::Source)
            .expect("repository source must be identified");
        assert_eq!(source.project_id.as_deref(), Some(project_id.as_str()));
        assert!(source.current);
        drop(service);
        drop(projects);
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("app data fixture must be removed");
    }

    #[tokio::test]
    async fn keeps_external_worktrees_unselectable_until_native_picker_attachment() {
        let (repository, projects, project_id) = repository_fixture();
        let external = repository.with_extension(format!("linked-{}", Uuid::now_v7()));
        assert!(StdCommand::new("git")
            .args([
                "worktree",
                "add",
                "--quiet",
                "-b",
                "feature/external-fixture",
            ])
            .arg(&external)
            .arg("HEAD")
            .current_dir(&repository)
            .status()
            .expect("Git must run")
            .success());
        let app_data = temporary_directory("worktree-attach-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));

        let before = service.status(project_id.clone(), &projects).await;
        let external_entry = before
            .worktrees
            .iter()
            .find(|entry| entry.branch_name.as_deref() == Some("feature/external-fixture"))
            .expect("external worktree must be discovered");
        assert_eq!(external_entry.ownership, WorktreeOwnership::External);
        assert!(external_entry.project_id.is_none());

        let preview = service
            .preview_attach(project_id, external.clone(), &projects)
            .await;
        assert_eq!(preview.state, WorktreePreviewState::Ready);
        assert_eq!(preview.ownership, Some(WorktreeOwnership::Attached));
        let result = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: preview.confirmation_id.expect("confirmation must exist"),
                },
                &projects,
            )
            .await;
        assert_eq!(result.state, WorktreeResultState::Applied);
        assert!(result
            .workspace
            .expect("workspace must be returned")
            .worktrees
            .iter()
            .any(|entry| {
                entry.ownership == WorktreeOwnership::Attached
                    && entry.project_id.is_some()
                    && entry.branch_name.as_deref() == Some("feature/external-fixture")
            }));

        drop(service);
        drop(projects);
        fs::remove_dir_all(external).expect("external worktree fixture must be removed");
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("app data fixture must be removed");
    }

    #[tokio::test]
    async fn leaves_a_created_worktree_recoverable_when_metadata_registration_fails() {
        let (repository, projects, project_id) = repository_fixture();
        let app_data = temporary_directory("worktree-recovery-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));
        let preview = service
            .preview_create(
                WorktreeCreatePreviewRequest {
                    project_id,
                    branch_name: "feature/recoverable-fixture".to_owned(),
                },
                &projects,
            )
            .await;
        projects.fail_worktree_registration_for_test();

        let result = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: preview.confirmation_id.expect("confirmation must exist"),
                },
                &projects,
            )
            .await;

        assert_eq!(result.state, WorktreeResultState::Unavailable);
        assert_eq!(
            result.diagnostic_code,
            Some(WorktreeDiagnosticCode::WorktreeRemains)
        );
        assert!(result.recoverable_display_path.is_some());
        let managed_parent = app_data.join("worktrees");
        assert!(managed_parent
            .read_dir()
            .expect("managed root must exist")
            .next()
            .is_some());

        drop(service);
        drop(projects);
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("recoverable fixture must be removed explicitly");
    }

    #[tokio::test]
    async fn consumes_confirmation_tokens_once() {
        let (repository, projects, project_id) = repository_fixture();
        let app_data = temporary_directory("worktree-token-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));
        let preview = service
            .preview_create(
                WorktreeCreatePreviewRequest {
                    project_id,
                    branch_name: "feature/one-use".to_owned(),
                },
                &projects,
            )
            .await;
        let confirmation_id = preview.confirmation_id.expect("confirmation must exist");
        let first = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: confirmation_id.clone(),
                },
                &projects,
            )
            .await;
        assert_eq!(first.state, WorktreeResultState::Applied);
        let second = service
            .confirm(WorktreeConfirmRequest { confirmation_id }, &projects)
            .await;
        assert_eq!(
            second.diagnostic_code,
            Some(WorktreeDiagnosticCode::ConfirmationExpired)
        );
        drop(service);
        drop(projects);
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("app data fixture must be removed");
    }

    #[tokio::test]
    async fn refuses_creation_when_head_changes_after_preview() {
        let (repository, projects, project_id) = repository_fixture();
        let app_data = temporary_directory("worktree-stale-head-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));
        let preview = service
            .preview_create(
                WorktreeCreatePreviewRequest {
                    project_id,
                    branch_name: "feature/stale-head".to_owned(),
                },
                &projects,
            )
            .await;
        assert!(StdCommand::new("git")
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--allow-empty",
                "--quiet",
                "-m",
                "changed after preview",
            ])
            .current_dir(&repository)
            .status()
            .expect("Git must run")
            .success());

        let result = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: preview.confirmation_id.expect("confirmation must exist"),
                },
                &projects,
            )
            .await;

        assert_eq!(result.state, WorktreeResultState::Unavailable);
        assert_eq!(
            result.diagnostic_code,
            Some(WorktreeDiagnosticCode::StalePreview)
        );
        assert!(!StdCommand::new("git")
            .args([
                "show-ref",
                "--verify",
                "--quiet",
                "refs/heads/feature/stale-head"
            ])
            .current_dir(&repository)
            .status()
            .expect("Git must run")
            .success());
        drop(service);
        drop(projects);
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("app data fixture must be removed");
    }

    #[tokio::test]
    async fn creates_without_running_repository_checkout_filters() {
        let (repository, projects, project_id) = repository_fixture();
        fs::write(
            repository.join(".gitattributes"),
            "payload.txt filter=fixture\n",
        )
        .expect("attributes fixture must be written");
        fs::write(repository.join("payload.txt"), "safe payload\n")
            .expect("payload fixture must be written");
        assert!(StdCommand::new("git")
            .args(["add", ".gitattributes", "payload.txt"])
            .current_dir(&repository)
            .status()
            .expect("Git must run")
            .success());
        assert!(StdCommand::new("git")
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--quiet",
                "-m",
                "filter fixture",
            ])
            .current_dir(&repository)
            .status()
            .expect("Git must run")
            .success());
        let marker = std::env::temp_dir().join(format!("quireforge-filter-{}", Uuid::now_v7()));
        let hook_marker = std::env::temp_dir().join(format!("quireforge-hook-{}", Uuid::now_v7()));
        let filter_command = format!("touch {}; cat", marker.display());
        assert!(StdCommand::new("git")
            .args(["config", "--local", "filter.fixture.smudge"])
            .arg(&filter_command)
            .current_dir(&repository)
            .status()
            .expect("Git must run")
            .success());
        assert!(StdCommand::new("git")
            .args(["config", "--local", "filter.fixture.clean"])
            .arg(&filter_command)
            .current_dir(&repository)
            .status()
            .expect("Git must run")
            .success());
        assert!(StdCommand::new("git")
            .args(["config", "--local", "filter.fixture.process"])
            .arg(&filter_command)
            .current_dir(&repository)
            .status()
            .expect("Git must run")
            .success());
        let hook = repository.join(".git/hooks/post-checkout");
        fs::write(
            &hook,
            format!("#!/bin/sh\n: > '{}'\n", hook_marker.display()),
        )
        .expect("hook fixture must be written");
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o755))
            .expect("hook fixture must be executable");
        assert!(StdCommand::new("git")
            .args(["config", "--local", "filter.fixture.required", "true"])
            .current_dir(&repository)
            .status()
            .expect("Git must run")
            .success());
        let app_data = temporary_directory("worktree-filter-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));
        let preview = service
            .preview_create(
                WorktreeCreatePreviewRequest {
                    project_id: project_id.clone(),
                    branch_name: "feature/no-filter".to_owned(),
                },
                &projects,
            )
            .await;

        let result = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: preview.confirmation_id.expect("confirmation must exist"),
                },
                &projects,
            )
            .await;

        assert_eq!(result.state, WorktreeResultState::Applied);
        assert!(!marker.exists(), "checkout filter must not execute");
        let worktree_project_id = result.project_id.expect("managed project must exist");
        let target = projects
            .execution_cwd(&worktree_project_id)
            .expect("managed cwd must resolve");
        let target_root = projects
            .review_root(&worktree_project_id)
            .expect("managed review root must resolve");
        let overrides = super::checkout_filter_overrides(&target_root)
            .await
            .expect("filter overrides must be generated");
        assert!(overrides
            .iter()
            .any(|value| value == "filter.fixture.clean=/bin/cat"));
        fs::write(target.join("payload.txt"), "safe payload\n")
            .expect("payload mtime must be refreshed");
        let cleanup = service
            .preview_remove(
                WorktreeRemovePreviewRequest {
                    project_id,
                    worktree_project_id,
                },
                &projects,
            )
            .await;
        assert_eq!(cleanup.state, WorktreePreviewState::Ready);
        let removed = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: cleanup
                        .confirmation_id
                        .expect("cleanup confirmation must exist"),
                },
                &projects,
            )
            .await;
        assert_eq!(removed.state, WorktreeResultState::Applied);
        let filter_executed = marker.exists();
        if filter_executed {
            fs::remove_file(&marker).expect("unexpected filter marker must be removed");
        }
        let hook_executed = hook_marker.exists();
        if hook_executed {
            fs::remove_file(&hook_marker).expect("unexpected hook marker must be removed");
        }
        assert!(
            !filter_executed,
            "checkout or status filters must not execute"
        );
        assert!(!hook_executed, "checkout hook must not execute");
        drop(service);
        drop(projects);
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("app data fixture must be removed");
    }

    #[tokio::test]
    async fn removes_a_clean_archived_managed_worktree_and_preserves_its_branch() {
        let (repository, projects, source_project_id) = repository_fixture();
        let app_data = temporary_directory("worktree-remove-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));
        let created = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: service
                        .preview_create(
                            WorktreeCreatePreviewRequest {
                                project_id: source_project_id.clone(),
                                branch_name: "feature/remove-clean".to_owned(),
                            },
                            &projects,
                        )
                        .await
                        .confirmation_id
                        .expect("create confirmation must exist"),
                },
                &projects,
            )
            .await;
        let worktree_project_id = created
            .project_id
            .expect("managed project must be registered");
        let managed_path = projects
            .execution_cwd(&worktree_project_id)
            .expect("managed cwd must resolve");
        let archived = projects.archive(worktree_project_id.clone());
        assert!(archived.diagnostic_code.is_none());
        let preview = service
            .preview_remove(
                WorktreeRemovePreviewRequest {
                    project_id: source_project_id.clone(),
                    worktree_project_id: worktree_project_id.clone(),
                },
                &projects,
            )
            .await;
        assert_eq!(preview.state, WorktreePreviewState::Ready);
        assert_eq!(preview.operation, WorktreeOperation::Remove);
        assert!(preview.destructive);

        let removed = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: preview
                        .confirmation_id
                        .expect("remove confirmation must exist"),
                },
                &projects,
            )
            .await;

        assert_eq!(removed.state, WorktreeResultState::Applied);
        assert_eq!(
            removed.project_id.as_deref(),
            Some(source_project_id.as_str())
        );
        assert!(!removed
            .workspace
            .expect("refreshed workspace must be returned")
            .worktrees
            .iter()
            .any(|entry| entry.project_id.as_deref() == Some(&worktree_project_id)));
        assert!(StdCommand::new("git")
            .args([
                "show-ref",
                "--verify",
                "--quiet",
                "refs/heads/feature/remove-clean",
            ])
            .current_dir(&repository)
            .status()
            .expect("Git must run")
            .success());
        assert!(!managed_path.exists());
        drop(service);
        drop(projects);
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("app data fixture must be removed");
    }

    #[tokio::test]
    async fn refuses_cleanup_when_the_managed_worktree_becomes_dirty() {
        let (repository, projects, source_project_id) = repository_fixture();
        let app_data = temporary_directory("worktree-dirty-remove-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));
        let created = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: service
                        .preview_create(
                            WorktreeCreatePreviewRequest {
                                project_id: source_project_id.clone(),
                                branch_name: "feature/refuse-dirty".to_owned(),
                            },
                            &projects,
                        )
                        .await
                        .confirmation_id
                        .expect("create confirmation must exist"),
                },
                &projects,
            )
            .await;
        let worktree_project_id = created.project_id.expect("managed project must exist");
        let target = projects
            .execution_cwd(&worktree_project_id)
            .expect("managed cwd must resolve");
        let preview = service
            .preview_remove(
                WorktreeRemovePreviewRequest {
                    project_id: source_project_id,
                    worktree_project_id,
                },
                &projects,
            )
            .await;
        fs::write(target.join("untracked-after-preview.txt"), "preserve me\n")
            .expect("dirty fixture must be written");

        let refused = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: preview
                        .confirmation_id
                        .expect("remove confirmation must exist"),
                },
                &projects,
            )
            .await;

        assert_eq!(
            refused.diagnostic_code,
            Some(WorktreeDiagnosticCode::WorktreeDirty)
        );
        assert!(target.join("untracked-after-preview.txt").exists());
        drop(service);
        drop(projects);
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("dirty fixture must be removed explicitly");
    }

    #[tokio::test]
    async fn recovers_an_unregistered_worktree_only_through_an_opaque_status_id() {
        let (repository, projects, source_project_id) = repository_fixture();
        let app_data = temporary_directory("worktree-adopt-recovery-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));
        let create_preview = service
            .preview_create(
                WorktreeCreatePreviewRequest {
                    project_id: source_project_id.clone(),
                    branch_name: "feature/adopt-recovery".to_owned(),
                },
                &projects,
            )
            .await;
        projects.fail_worktree_registration_for_test();
        let retained = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: create_preview
                        .confirmation_id
                        .expect("create confirmation must exist"),
                },
                &projects,
            )
            .await;
        assert_eq!(
            retained.diagnostic_code,
            Some(WorktreeDiagnosticCode::WorktreeRemains)
        );
        projects.allow_worktree_registration_for_test();
        let inventory = service.status(source_project_id.clone(), &projects).await;
        let candidate = inventory
            .worktrees
            .iter()
            .find(|entry| entry.branch_name.as_deref() == Some("feature/adopt-recovery"))
            .expect("retained worktree must be inventoried");
        assert_eq!(candidate.ownership, WorktreeOwnership::External);
        let recovery_id = candidate
            .recovery_id
            .clone()
            .expect("native status must issue a recovery ID");
        let preview = service
            .preview_recover(
                WorktreeRecoverPreviewRequest {
                    project_id: source_project_id,
                    recovery_id,
                },
                &projects,
            )
            .await;
        assert_eq!(preview.operation, WorktreeOperation::Recover);
        assert!(!preview.destructive);

        let recovered = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: preview
                        .confirmation_id
                        .expect("recovery confirmation must exist"),
                },
                &projects,
            )
            .await;

        assert_eq!(recovered.state, WorktreeResultState::Applied);
        assert!(recovered
            .workspace
            .expect("workspace must be returned")
            .worktrees
            .iter()
            .any(|entry| {
                entry.ownership == WorktreeOwnership::Managed
                    && entry.branch_name.as_deref() == Some("feature/adopt-recovery")
                    && entry.recovery_id.is_none()
            }));
        drop(service);
        drop(projects);
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("recovery fixture must be removed explicitly");
    }

    #[tokio::test]
    async fn finalizes_metadata_after_a_post_git_cleanup_failure() {
        let (repository, projects, source_project_id) = repository_fixture();
        let app_data = temporary_directory("worktree-finalize-cleanup-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));
        let created = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: service
                        .preview_create(
                            WorktreeCreatePreviewRequest {
                                project_id: source_project_id.clone(),
                                branch_name: "feature/finalize-cleanup".to_owned(),
                            },
                            &projects,
                        )
                        .await
                        .confirmation_id
                        .expect("create confirmation must exist"),
                },
                &projects,
            )
            .await;
        let worktree_project_id = created.project_id.expect("managed project must exist");
        projects.fail_worktree_retirement_for_test();
        let preview = service
            .preview_remove(
                WorktreeRemovePreviewRequest {
                    project_id: source_project_id.clone(),
                    worktree_project_id: worktree_project_id.clone(),
                },
                &projects,
            )
            .await;
        let incomplete = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: preview
                        .confirmation_id
                        .expect("remove confirmation must exist"),
                },
                &projects,
            )
            .await;
        assert_eq!(
            incomplete.diagnostic_code,
            Some(WorktreeDiagnosticCode::CleanupIncomplete)
        );
        assert!(incomplete
            .workspace
            .expect("incomplete result must refresh inventory")
            .worktrees
            .iter()
            .any(|entry| {
                entry.project_id.as_deref() == Some(&worktree_project_id)
                    && entry.state == super::types::WorktreeEntryState::Missing
            }));

        projects.allow_worktree_retirement_for_test();
        let finalize = service
            .preview_remove(
                WorktreeRemovePreviewRequest {
                    project_id: source_project_id,
                    worktree_project_id,
                },
                &projects,
            )
            .await;
        assert_eq!(finalize.state, WorktreePreviewState::Ready);
        assert!(!finalize.destructive);
        let finalized = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: finalize
                        .confirmation_id
                        .expect("finalization confirmation must exist"),
                },
                &projects,
            )
            .await;
        assert_eq!(finalized.state, WorktreeResultState::Applied);
        drop(service);
        drop(projects);
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("app data fixture must be removed");
    }

    #[tokio::test]
    async fn never_offers_filesystem_cleanup_for_an_attached_worktree() {
        let (repository, projects, source_project_id) = repository_fixture();
        let external = repository.with_extension(format!("attached-{}", Uuid::now_v7()));
        assert!(StdCommand::new("git")
            .args(["worktree", "add", "--quiet", "-b", "feature/attached-safe"])
            .arg(&external)
            .arg("HEAD")
            .current_dir(&repository)
            .status()
            .expect("Git must run")
            .success());
        let app_data = temporary_directory("worktree-attached-cleanup-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));
        let attach = service
            .preview_attach(source_project_id.clone(), external.clone(), &projects)
            .await;
        let attached = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: attach
                        .confirmation_id
                        .expect("attach confirmation must exist"),
                },
                &projects,
            )
            .await;
        let preview = service
            .preview_remove(
                WorktreeRemovePreviewRequest {
                    project_id: source_project_id,
                    worktree_project_id: attached.project_id.expect("attached project must exist"),
                },
                &projects,
            )
            .await;

        assert_eq!(preview.state, WorktreePreviewState::Unavailable);
        assert_eq!(
            preview.diagnostic_code,
            Some(WorktreeDiagnosticCode::UnsupportedOwnership)
        );
        assert!(external.exists());
        drop(service);
        drop(projects);
        fs::remove_dir_all(external).expect("attached fixture must be removed explicitly");
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("app data fixture must be removed");
    }

    #[tokio::test]
    async fn refuses_cleanup_while_any_related_project_is_reserved() {
        let (repository, projects, source_project_id) = repository_fixture();
        let app_data = temporary_directory("worktree-busy-cleanup-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));
        let created = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: service
                        .preview_create(
                            WorktreeCreatePreviewRequest {
                                project_id: source_project_id.clone(),
                                branch_name: "feature/busy-cleanup".to_owned(),
                            },
                            &projects,
                        )
                        .await
                        .confirmation_id
                        .expect("create confirmation must exist"),
                },
                &projects,
            )
            .await;
        let worktree_project_id = created.project_id.expect("managed project must exist");
        let target = projects
            .execution_cwd(&worktree_project_id)
            .expect("managed cwd must resolve");
        let preview = service
            .preview_remove(
                WorktreeRemovePreviewRequest {
                    project_id: source_project_id.clone(),
                    worktree_project_id,
                },
                &projects,
            )
            .await;
        projects
            .reserve_execution(&source_project_id)
            .expect("source reservation must succeed");
        let refused = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: preview
                        .confirmation_id
                        .expect("remove confirmation must exist"),
                },
                &projects,
            )
            .await;
        projects.release_execution(&source_project_id);

        assert_eq!(
            refused.diagnostic_code,
            Some(WorktreeDiagnosticCode::ProjectBusy)
        );
        assert!(target.exists());
        drop(service);
        drop(projects);
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("busy fixture must be removed explicitly");
    }

    #[tokio::test]
    async fn refuses_cleanup_after_the_reviewed_path_becomes_a_symlink() {
        let (repository, projects, source_project_id) = repository_fixture();
        let app_data = temporary_directory("worktree-symlink-cleanup-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));
        let created = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: service
                        .preview_create(
                            WorktreeCreatePreviewRequest {
                                project_id: source_project_id.clone(),
                                branch_name: "feature/symlink-cleanup".to_owned(),
                            },
                            &projects,
                        )
                        .await
                        .confirmation_id
                        .expect("create confirmation must exist"),
                },
                &projects,
            )
            .await;
        let worktree_project_id = created.project_id.expect("managed project must exist");
        let target = projects
            .execution_cwd(&worktree_project_id)
            .expect("managed cwd must resolve");
        let preview = service
            .preview_remove(
                WorktreeRemovePreviewRequest {
                    project_id: source_project_id,
                    worktree_project_id,
                },
                &projects,
            )
            .await;
        let preserved = app_data.join("preserved-worktree");
        fs::rename(&target, &preserved).expect("reviewed worktree must move");
        symlink(&repository, &target).expect("replacement symlink must be created");

        let refused = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: preview
                        .confirmation_id
                        .expect("remove confirmation must exist"),
                },
                &projects,
            )
            .await;

        assert_eq!(
            refused.diagnostic_code,
            Some(WorktreeDiagnosticCode::IdentityChanged)
        );
        assert!(preserved.exists());
        assert!(fs::symlink_metadata(&target)
            .expect("replacement must remain")
            .file_type()
            .is_symlink());
        drop(service);
        drop(projects);
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("symlink fixture must be removed explicitly");
    }
}
