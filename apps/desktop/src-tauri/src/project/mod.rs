mod identity;
mod package_validation;
mod storage;
pub mod types;

use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};

use package_validation::{
    InstalledHostValidationOutcome, PackageValidationController, PackageValidationControllerError,
};
use storage::{
    ProjectRepository, StorageError, StoredAssociation, StoredProject, StoredWorktreeRelation,
};
pub(crate) use storage::{StoredConversationReference, StoredTerminalSession};
use types::{
    DirectoryAccessibilityState, DirectorySummary, GitSummary,
    LocalReviewActivityPresentationEvidencePreview, LocalReviewActivityPresentationEvidenceRequest,
    LocalReviewAnnotationCreateRequest, LocalReviewAnnotationEditRequest,
    LocalReviewAnnotationMutationRequest, LocalReviewApprovalPresentationEvidencePreview,
    LocalReviewApprovalPresentationEvidenceRequest, LocalReviewCollectionCreateRequest,
    LocalReviewCollectionMutationRequest, LocalReviewComparisonCreateRequest,
    LocalReviewComparisonDiscardRequest, LocalReviewComparisonReadRequest,
    LocalReviewDiagnosticCode, LocalReviewEvidenceArtifactKind, LocalReviewEvidenceArtifactState,
    LocalReviewEvidenceSource, LocalReviewEvidenceWorkspaceState,
    LocalReviewGitStatusDiffSummaryDetails, LocalReviewGitStatusDiffSummaryEvidencePreview,
    LocalReviewGitStatusDiffSummaryEvidenceRequest, LocalReviewImagePickRequest,
    LocalReviewImagePreview, LocalReviewImagePreviewRequest, LocalReviewItemDiscardRequest,
    LocalReviewListRequest, LocalReviewM48ArtifactCopyRequest,
    LocalReviewM48GeneratedArtifactMetadataDetails,
    LocalReviewM48GeneratedArtifactMetadataEvidencePreview,
    LocalReviewM48GeneratedArtifactMetadataEvidenceRequest, LocalReviewManualEvidenceCreateRequest,
    LocalReviewManualEvidenceCreateResult, LocalReviewManualEvidencePreview,
    LocalReviewPackageManifestSummaryEvidencePreview,
    LocalReviewPackageManifestSummaryEvidenceRequest, LocalReviewPromotionCandidate,
    LocalReviewPromotionPrepareRequest, LocalReviewPromotionReservationRequest,
    LocalReviewPromotionReservationState, LocalReviewSafePreviewMetadataDetails,
    LocalReviewSafePreviewMetadataEvidencePreview, LocalReviewSafePreviewMetadataEvidenceRequest,
    LocalReviewSnapshot, LocalReviewTextItemCreateRequest, LocalReviewTextPreview,
    LocalReviewTextPreviewRequest, PendingAttachmentKind, PendingAttachmentPreview,
    ProjectDiagnosticCode, ProjectPreflightSnapshot, ProjectSummary, ProjectWorkspaceSnapshot,
    ProjectWorkspaceState, TaskCatalogListRequest, TaskCatalogSnapshot, TaskCatalogState,
    TaskDiagnosticCode, LOCAL_REVIEW_SCHEMA_VERSION, PROJECT_SCHEMA_VERSION,
    TASK_RECORD_SCHEMA_VERSION,
};
use uuid::Uuid;

use crate::advisor_generated_artifact::{
    AdvisorGeneratedArtifactService, GeneratedArtifactClaimRequest, GeneratedArtifactClass,
    GeneratedArtifactCreateRequest, GeneratedArtifactManifestV1, GeneratedArtifactSourceKind,
    GeneratedArtifactState, MAX_ARTIFACTS,
};
use crate::git::{
    types::{GitChangeKind, GitWorkspaceSnapshot, GitWorkspaceState},
    GitService,
};

use crate::advisor::{
    AdvisorApprovalSnapshot, AdvisorDispatchProposal, AdvisorDispatchState,
    AdvisorFoundationSnapshot, AdvisorWorkspaceSnapshot,
};

use self::identity::{
    disconnected_state, display_path, inspect_directory, DirectoryIdentity,
    DirectoryInspectionError,
};

#[derive(Clone)]
struct PendingAttachment {
    kind: PendingAttachmentKind,
    project_id: Option<String>,
    display_name: String,
    identity: DirectoryIdentity,
}

pub struct ProjectService {
    repository: Mutex<Option<ProjectRepository>>,
    pending: Mutex<Option<PendingAttachment>>,
    active_executions: Mutex<HashSet<String>>,
    active_terminals: Mutex<HashMap<String, usize>>,
    promotion_reservations: Mutex<VecDeque<LocalReviewPromotionReservation>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InstalledHostHeadlessStatus {
    Created,
    Existing,
    Failed,
    Unavailable,
}

struct LocalReviewPromotionReservation {
    candidate: LocalReviewPromotionCandidate,
    observed_plan_updated_at_ms: Option<i64>,
    content: String,
    class: GeneratedArtifactClass,
    created: Instant,
}

const LOCAL_REVIEW_PROMOTION_RESERVATION_LIMIT: usize = 16;
const LOCAL_REVIEW_PROMOTION_RESERVATION_TTL: Duration = Duration::from_secs(5 * 60);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProjectExecutionError {
    InvalidProjectId,
    MetadataUnavailable,
    ProjectNotFound,
    DirectoryUnavailable,
    IdentityChanged,
    NotRepository,
    NotWritable,
    ProjectBusy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectReviewRoot {
    pub attached_root: PathBuf,
    pub worktree_root: PathBuf,
    pub git_dir: PathBuf,
    pub common_dir: PathBuf,
    pub writable: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct ProjectWorktreeRecord {
    pub project_id: String,
    pub display_name: String,
    pub selected_path: Option<PathBuf>,
    pub ownership: String,
    pub branch_name: Option<String>,
    pub archived: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct ProjectWorktreeContext {
    pub source_project_id: String,
    pub source_display_name: String,
    pub records: Vec<ProjectWorktreeRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectWorktreeCandidate {
    pub selected_path: PathBuf,
    pub resolved_path: PathBuf,
    pub display_path: String,
    pub worktree_root: PathBuf,
    pub common_dir: PathBuf,
    pub is_linked_worktree: bool,
    pub device_id: u64,
    pub inode: u64,
    pub mount_id: Option<u64>,
    pub filesystem_type: Option<String>,
    pub has_agents_guidance: bool,
    pub has_codex_config: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorktreeRegistrationError {
    Project(ProjectExecutionError),
    DuplicateDirectory,
    NotLinkedWorktree,
    DifferentRepository,
}

pub(crate) struct ConversationReference<'a> {
    pub conversation_id: &'a str,
    pub project_id: &'a str,
    pub codex_thread_id: &'a str,
    pub model_id: &'a str,
    pub reasoning_effort: &'a str,
    pub sandbox_mode: &'a str,
    pub approval_policy: &'a str,
    pub parent_conversation_id: Option<&'a str>,
    pub selection: ConversationSelectionMetadata<'a>,
}

/// Bounded local metadata for a no-project conversation. This intentionally
/// excludes prompts, responses, authentication data, and any filesystem or
/// project association.
pub(crate) struct ChatConversationMetadata<'a> {
    pub conversation_id: &'a str,
    pub codex_thread_id: &'a str,
}

/// Bounded metadata for an Advisor conversation. The thread remains owned by
/// Codex; QuireForge stores neither prompt nor response text.
pub(crate) struct AdvisorConversationMetadata<'a> {
    pub conversation_id: &'a str,
    pub codex_thread_id: &'a str,
}

pub(crate) struct ConversationSelectionMetadata<'a> {
    pub availability: &'a str,
    pub ownership: &'a str,
    pub user_locked: bool,
    pub allowed_model_ids_json: &'a str,
    pub reasoning_ceiling: Option<&'a str>,
    pub pending: Option<ConversationPendingSelection<'a>>,
}

pub(crate) struct ConversationPendingSelection<'a> {
    pub model_id: &'a str,
    pub reasoning_effort: &'a str,
    pub rationale: &'a str,
    pub provenance: &'a str,
    pub application: &'a str,
    pub requested_at_ms: i64,
}

impl ProjectService {
    pub fn local_review(&self, request: LocalReviewListRequest) -> LocalReviewSnapshot {
        let selected = request.selected_collection_id.as_deref();
        let result = self.repository.lock().ok().and_then(|mut repository| {
            let repository = repository.as_mut()?;
            let snapshot = repository.local_review_snapshot(selected).ok()?;
            let comparisons = selected
                .and_then(|id| repository.local_review_comparisons(id).ok())
                .unwrap_or_default();
            let package_manifest_summary_available =
                snapshot.1.as_ref().is_some_and(|collection| {
                    repository.package_manifest_summary_available_for_local_review(
                        &collection.collection_id,
                    )
                });
            let git_status_diff_summary_available = snapshot.1.as_ref().is_some_and(|collection| {
                repository
                    .git_status_diff_summary_project_for_local_review(&collection.collection_id)
                    .is_ok()
            });
            let activity_presentation_available = snapshot.1.as_ref().is_some_and(|collection| {
                repository
                    .activity_presentation_available_for_local_review(&collection.collection_id)
            });
            let approval_presentation_available = snapshot.1.as_ref().is_some_and(|collection| {
                repository
                    .approval_presentation_available_for_local_review(&collection.collection_id)
            });
            Some((
                snapshot,
                comparisons,
                package_manifest_summary_available,
                git_status_diff_summary_available,
                activity_presentation_available,
                approval_presentation_available,
            ))
        });
        match result {
            Some((
                (collections, selected_collection, items, collection_count, payload_bytes, warning),
                comparisons,
                package_manifest_summary_available,
                git_status_diff_summary_available,
                activity_presentation_available,
                approval_presentation_available,
            )) => LocalReviewSnapshot {
                schema_version: LOCAL_REVIEW_SCHEMA_VERSION,
                collections,
                selected_collection,
                items,
                comparisons,
                collection_count,
                payload_bytes,
                warning,
                package_manifest_summary_available,
                git_status_diff_summary_available,
                activity_presentation_available,
                approval_presentation_available,
                diagnostic_code: None,
            },
            None => local_review_unavailable(LocalReviewDiagnosticCode::MetadataUnavailable),
        }
    }

    pub fn create_local_review_text_comparison(
        &self,
        request: LocalReviewComparisonCreateRequest,
    ) -> LocalReviewSnapshot {
        let collection_id = request.collection_id.clone();
        let result = self.repository.lock().ok().and_then(|mut repository| {
            repository
                .as_mut()?
                .create_local_review_text_comparison(
                    &request.collection_id,
                    &request.left_item_id,
                    &request.right_item_id,
                    request.expected_collection_updated_at_ms,
                )
                .ok()
        });
        let mut snapshot = self.local_review(LocalReviewListRequest {
            selected_collection_id: Some(collection_id),
        });
        if result.is_none() {
            snapshot.diagnostic_code = Some(LocalReviewDiagnosticCode::InvalidRequest);
        }
        snapshot
    }

    pub fn discard_local_review_text_comparison(
        &self,
        request: LocalReviewComparisonDiscardRequest,
    ) -> LocalReviewSnapshot {
        let collection_id = request.collection_id.clone();
        let result = self.repository.lock().ok().and_then(|mut repository| {
            repository
                .as_mut()?
                .discard_local_review_text_comparison(
                    &request.collection_id,
                    &request.comparison_id,
                    request.expected_collection_updated_at_ms,
                )
                .ok()
        });
        let mut snapshot = self.local_review(LocalReviewListRequest {
            selected_collection_id: Some(collection_id),
        });
        if result.is_none() {
            snapshot.diagnostic_code = Some(LocalReviewDiagnosticCode::InvalidRequest);
        }
        snapshot
    }

    pub fn local_review_line_comparison(
        &self,
        request: LocalReviewComparisonReadRequest,
    ) -> Result<types::LocalReviewLineComparison, ()> {
        self.repository
            .lock()
            .ok()
            .and_then(|repository| {
                let repository = repository.as_ref()?;
                let comparisons = repository
                    .local_review_comparisons(&request.collection_id)
                    .ok()?;
                comparisons
                    .iter()
                    .any(|comparison| comparison.comparison_id == request.comparison_id)
                    .then(|| {
                        repository
                            .local_review_line_comparison(&request.comparison_id)
                            .ok()
                    })
                    .flatten()
            })
            .ok_or(())
    }

    pub fn prepare_local_review_promotion(
        &self,
        request: LocalReviewPromotionPrepareRequest,
        artifacts: &AdvisorGeneratedArtifactService,
    ) -> Result<LocalReviewPromotionCandidate, ()> {
        let source = self
            .repository
            .lock()
            .ok()
            .and_then(|mut repository| {
                repository
                    .as_mut()?
                    .local_review_promotion_source(
                        &request.collection_id,
                        &request.item_id,
                        Some(request.expected_collection_updated_at_ms),
                    )
                    .ok()
            })
            .ok_or(())?;
        if artifacts.snapshot().artifacts.len() >= MAX_ARTIFACTS {
            return Err(());
        }
        let class = promotion_class(source.text_format);
        let now = Instant::now();
        let mut reservations = self.promotion_reservations.lock().map_err(|_| ())?;
        expire_promotion_reservations(&mut reservations);
        if reservations.len() >= LOCAL_REVIEW_PROMOTION_RESERVATION_LIMIT {
            return Err(());
        }
        let candidate = LocalReviewPromotionCandidate {
            reservation_id: Uuid::now_v7().to_string(),
            collection_id: source.collection_id,
            item_id: source.item_id,
            title: source.title,
            sha256: source.sha256,
            text_format: source.text_format,
            destination_class: promotion_class_name(class).to_owned(),
            task_id: source.task_id,
            plan_id: source.plan_id,
            created_at_ms: 0,
            expires_at_ms: LOCAL_REVIEW_PROMOTION_RESERVATION_TTL.as_millis() as u64,
            state: LocalReviewPromotionReservationState::Prepared,
        };
        reservations.push_back(LocalReviewPromotionReservation {
            candidate: candidate.clone(),
            observed_plan_updated_at_ms: source.observed_plan_updated_at_ms,
            content: source.content,
            class,
            created: now,
        });
        Ok(candidate)
    }

    pub fn confirm_local_review_promotion(
        &self,
        request: LocalReviewPromotionReservationRequest,
        artifacts: &AdvisorGeneratedArtifactService,
    ) -> Result<GeneratedArtifactManifestV1, ()> {
        let reservation = {
            let mut reservations = self.promotion_reservations.lock().map_err(|_| ())?;
            expire_promotion_reservations(&mut reservations);
            let index = reservations
                .iter()
                .position(|value| value.candidate.reservation_id == request.reservation_id)
                .ok_or(())?;
            reservations.remove(index).ok_or(())?
        };
        let source = self
            .repository
            .lock()
            .ok()
            .and_then(|mut repository| {
                repository
                    .as_mut()?
                    .local_review_promotion_source(
                        &reservation.candidate.collection_id,
                        &reservation.candidate.item_id,
                        None,
                    )
                    .ok()
            })
            .ok_or(())?;
        if source.sha256 != reservation.candidate.sha256
            || source.content != reservation.content
            || source.text_format != reservation.candidate.text_format
            || source.observed_plan_updated_at_ms != reservation.observed_plan_updated_at_ms
            || artifacts.snapshot().artifacts.len() >= MAX_ARTIFACTS
        {
            return Err(());
        }
        artifacts
            .create(GeneratedArtifactCreateRequest {
                class: reservation.class,
                source_kind: GeneratedArtifactSourceKind::ExplicitReviewPromotion,
                display_label: source.title,
                suggested_filename: format!("review-promotion{}", reservation.class.suffix()),
                content: source.content,
            })
            .map_err(|_| ())
    }

    pub fn cancel_local_review_promotion(
        &self,
        request: LocalReviewPromotionReservationRequest,
    ) -> Result<LocalReviewPromotionCandidate, ()> {
        let mut reservations = self.promotion_reservations.lock().map_err(|_| ())?;
        expire_promotion_reservations(&mut reservations);
        let index = reservations
            .iter()
            .position(|value| value.candidate.reservation_id == request.reservation_id)
            .ok_or(())?;
        let mut reservation = reservations.remove(index).ok_or(())?.candidate;
        reservation.state = LocalReviewPromotionReservationState::Expired;
        Ok(reservation)
    }

    pub fn create_local_review_collection(
        &self,
        request: LocalReviewCollectionCreateRequest,
    ) -> LocalReviewSnapshot {
        let result = self.repository.lock().ok().and_then(|mut repository| {
            repository
                .as_mut()?
                .create_local_review_collection(
                    &request.task_id,
                    request.plan_id.as_deref(),
                    &request.title,
                )
                .ok()
        });
        match result {
            Some(id) => self.local_review(LocalReviewListRequest {
                selected_collection_id: Some(id),
            }),
            None => local_review_unavailable(LocalReviewDiagnosticCode::InvalidRequest),
        }
    }

    pub fn create_local_review_text_item(
        &self,
        request: LocalReviewTextItemCreateRequest,
    ) -> LocalReviewSnapshot {
        let collection_id = request.collection_id.clone();
        let result = self.repository.lock().ok().and_then(|mut repository| {
            repository
                .as_mut()?
                .create_local_review_text_item(
                    &request.collection_id,
                    request.expected_collection_updated_at_ms,
                    &request.title,
                    request.text_format,
                    &request.content,
                )
                .ok()
        });
        let mut snapshot = self.local_review(LocalReviewListRequest {
            selected_collection_id: Some(collection_id),
        });
        if result.is_none() {
            snapshot.diagnostic_code = Some(LocalReviewDiagnosticCode::InvalidRequest);
        }
        snapshot
    }

    pub fn create_local_review_m48_artifact_copy(
        &self,
        request: LocalReviewM48ArtifactCopyRequest,
        artifacts: &AdvisorGeneratedArtifactService,
    ) -> LocalReviewSnapshot {
        let collection_id = request.collection_id.clone();
        let source = artifacts
            .local_review_copy_source(&GeneratedArtifactClaimRequest {
                artifact_id: request.artifact_id,
                manifest_sha256: request.manifest_sha256,
            })
            .ok();
        let result = source.and_then(|source| {
            let format = local_review_format(source.class);
            let content = String::from_utf8(source.bytes).ok()?;
            self.repository.lock().ok().and_then(|mut repository| {
                repository
                    .as_mut()?
                    .create_local_review_m48_artifact_copy(
                        &collection_id,
                        request.expected_collection_updated_at_ms,
                        &source.artifact_id,
                        &source.sha256,
                        &source.display_label,
                        format,
                        &content,
                    )
                    .ok()
            })
        });
        let mut snapshot = self.local_review(LocalReviewListRequest {
            selected_collection_id: Some(collection_id),
        });
        if result.is_none() {
            snapshot.diagnostic_code = Some(LocalReviewDiagnosticCode::InvalidRequest);
        }
        snapshot
    }

    pub fn create_local_review_m48_generated_artifact_metadata_evidence(
        &self,
        request: LocalReviewM48GeneratedArtifactMetadataEvidenceRequest,
        artifacts: &AdvisorGeneratedArtifactService,
    ) -> LocalReviewManualEvidenceCreateResult {
        let collection_id = request.collection_id.clone();
        let source = artifacts
            .local_review_metadata_source(&GeneratedArtifactClaimRequest {
                artifact_id: request.artifact_id,
                manifest_sha256: request.manifest_sha256,
            })
            .ok();
        let result = source.and_then(|source| {
            let title = format!("Generated artifact metadata: {}", source.display_label);
            let summary = "Captured live generated-artifact metadata only.";
            let details = LocalReviewM48GeneratedArtifactMetadataDetails {
                artifact_state: match source.state {
                    GeneratedArtifactState::Ready => LocalReviewEvidenceArtifactState::Ready,
                    GeneratedArtifactState::Saving => LocalReviewEvidenceArtifactState::Saving,
                    GeneratedArtifactState::Expired => LocalReviewEvidenceArtifactState::Expired,
                    GeneratedArtifactState::Saved => LocalReviewEvidenceArtifactState::Saved,
                },
                artifact_kind: match source.class {
                    GeneratedArtifactClass::Text => LocalReviewEvidenceArtifactKind::Text,
                    GeneratedArtifactClass::Markdown => LocalReviewEvidenceArtifactKind::Markdown,
                    GeneratedArtifactClass::Json => LocalReviewEvidenceArtifactKind::Json,
                    GeneratedArtifactClass::Csv => LocalReviewEvidenceArtifactKind::Csv,
                    GeneratedArtifactClass::Python => LocalReviewEvidenceArtifactKind::Python,
                },
                format: local_review_format(source.class),
                byte_length: u32::try_from(source.byte_size).ok()?,
                truncated: false,
                manifest_sha256: source.sha256,
            };
            self.repository.lock().ok().and_then(|mut repository| {
                repository
                    .as_mut()?
                    .create_local_review_m48_generated_artifact_metadata_evidence_item(
                        &collection_id,
                        request.expected_collection_updated_at_ms,
                        &title,
                        summary,
                        &details,
                    )
                    .ok()
            })
        });
        let mut snapshot = self.local_review(LocalReviewListRequest {
            selected_collection_id: Some(collection_id),
        });
        if let Some(created_item_id) = result {
            if snapshot.items.iter().any(|item| {
                item.item_id == created_item_id
                    && item.evidence_source
                        == Some(LocalReviewEvidenceSource::M48GeneratedArtifactMetadata)
            }) {
                return LocalReviewManualEvidenceCreateResult::Created {
                    created_item_id,
                    source: LocalReviewEvidenceSource::M48GeneratedArtifactMetadata,
                    snapshot,
                };
            }
        }
        if snapshot.diagnostic_code.is_none() {
            snapshot.diagnostic_code = Some(LocalReviewDiagnosticCode::InvalidRequest);
        }
        LocalReviewManualEvidenceCreateResult::Failed { snapshot }
    }

    pub fn create_local_review_safe_preview_metadata_evidence(
        &self,
        request: LocalReviewSafePreviewMetadataEvidenceRequest,
        details: LocalReviewSafePreviewMetadataDetails,
    ) -> LocalReviewManualEvidenceCreateResult {
        let collection_id = request.collection_id.clone();
        let result = self.repository.lock().ok().and_then(|mut repository| {
            repository
                .as_mut()?
                .create_local_review_safe_preview_metadata_evidence_item(
                    &collection_id,
                    request.expected_collection_updated_at_ms,
                    "Safe preview metadata",
                    "Captured current safe-preview metadata only.",
                    &details,
                )
                .ok()
        });
        let mut snapshot = self.local_review(LocalReviewListRequest {
            selected_collection_id: Some(collection_id),
        });
        if let Some(created_item_id) = result {
            if snapshot.items.iter().any(|item| {
                item.item_id == created_item_id
                    && item.evidence_source == Some(LocalReviewEvidenceSource::SafePreviewMetadata)
            }) {
                return LocalReviewManualEvidenceCreateResult::Created {
                    created_item_id,
                    source: LocalReviewEvidenceSource::SafePreviewMetadata,
                    snapshot,
                };
            }
        }
        if snapshot.diagnostic_code.is_none() {
            snapshot.diagnostic_code = Some(LocalReviewDiagnosticCode::InvalidRequest);
        }
        LocalReviewManualEvidenceCreateResult::Failed { snapshot }
    }

    pub fn create_local_review_package_manifest_summary_evidence(
        &self,
        request: LocalReviewPackageManifestSummaryEvidenceRequest,
    ) -> LocalReviewManualEvidenceCreateResult {
        let collection_id = request.collection_id.clone();
        let result = self.repository.lock().ok().and_then(|mut repository| {
            repository
                .as_mut()?
                .create_local_review_package_manifest_summary_evidence_item(
                    &collection_id,
                    request.expected_collection_updated_at_ms,
                )
                .ok()
        });
        let mut snapshot = self.local_review(LocalReviewListRequest {
            selected_collection_id: Some(collection_id),
        });
        if let Some(created_item_id) = result {
            if snapshot.items.iter().any(|item| {
                item.item_id == created_item_id
                    && item.evidence_source
                        == Some(LocalReviewEvidenceSource::PackageManifestSummary)
            }) {
                return LocalReviewManualEvidenceCreateResult::Created {
                    created_item_id,
                    source: LocalReviewEvidenceSource::PackageManifestSummary,
                    snapshot,
                };
            }
        }
        if snapshot.diagnostic_code.is_none() {
            snapshot.diagnostic_code = Some(LocalReviewDiagnosticCode::InvalidRequest);
        }
        LocalReviewManualEvidenceCreateResult::Failed { snapshot }
    }

    pub async fn create_local_review_git_status_diff_summary_evidence(
        &self,
        request: LocalReviewGitStatusDiffSummaryEvidenceRequest,
    ) -> LocalReviewManualEvidenceCreateResult {
        let collection_id = request.collection_id.clone();
        let project_id = self.repository.lock().ok().and_then(|repository| {
            repository
                .as_ref()?
                .git_status_diff_summary_project_for_local_review(&collection_id)
                .ok()
        });
        let result = if let Some(project_id) = project_id {
            let workspace = GitService::default().status(project_id.clone(), self).await;
            git_status_diff_summary_details(&workspace).and_then(|details| {
                self.repository.lock().ok().and_then(|mut repository| {
                    repository
                        .as_mut()?
                        .create_local_review_git_status_diff_summary_evidence_item(
                            &collection_id,
                            request.expected_collection_updated_at_ms,
                            &project_id,
                            &details,
                        )
                        .ok()
                })
            })
        } else {
            None
        };
        let mut snapshot = self.local_review(LocalReviewListRequest {
            selected_collection_id: Some(collection_id),
        });
        if let Some(created_item_id) = result {
            if snapshot.items.iter().any(|item| {
                item.item_id == created_item_id
                    && item.evidence_source == Some(LocalReviewEvidenceSource::GitStatusDiffSummary)
            }) {
                return LocalReviewManualEvidenceCreateResult::Created {
                    created_item_id,
                    source: LocalReviewEvidenceSource::GitStatusDiffSummary,
                    snapshot,
                };
            }
        }
        if snapshot.diagnostic_code.is_none() {
            snapshot.diagnostic_code = Some(LocalReviewDiagnosticCode::InvalidRequest);
        }
        LocalReviewManualEvidenceCreateResult::Failed { snapshot }
    }

    pub fn create_local_review_activity_presentation_evidence(
        &self,
        request: LocalReviewActivityPresentationEvidenceRequest,
    ) -> LocalReviewManualEvidenceCreateResult {
        let collection_id = request.collection_id.clone();
        let result = self.repository.lock().ok().and_then(|mut repository| {
            repository
                .as_mut()?
                .create_local_review_activity_presentation_evidence_item(
                    &collection_id,
                    request.expected_collection_updated_at_ms,
                )
                .ok()
        });
        let mut snapshot = self.local_review(LocalReviewListRequest {
            selected_collection_id: Some(collection_id),
        });
        if let Some(created_item_id) = result {
            if snapshot.items.iter().any(|item| {
                item.item_id == created_item_id
                    && item.evidence_source == Some(LocalReviewEvidenceSource::ActivityPresentation)
            }) {
                return LocalReviewManualEvidenceCreateResult::Created {
                    created_item_id,
                    source: LocalReviewEvidenceSource::ActivityPresentation,
                    snapshot,
                };
            }
        }
        snapshot.diagnostic_code = Some(LocalReviewDiagnosticCode::InvalidRequest);
        LocalReviewManualEvidenceCreateResult::Failed { snapshot }
    }

    pub fn create_local_review_approval_presentation_evidence(
        &self,
        request: LocalReviewApprovalPresentationEvidenceRequest,
    ) -> LocalReviewManualEvidenceCreateResult {
        let collection_id = request.collection_id.clone();
        let result = self.repository.lock().ok().and_then(|mut repository| {
            repository
                .as_mut()?
                .create_local_review_approval_presentation_evidence_item(
                    &collection_id,
                    request.expected_collection_updated_at_ms,
                )
                .ok()
        });
        let mut snapshot = self.local_review(LocalReviewListRequest {
            selected_collection_id: Some(collection_id),
        });
        if let Some(created_item_id) = result {
            if snapshot.items.iter().any(|item| {
                item.item_id == created_item_id
                    && item.evidence_source == Some(LocalReviewEvidenceSource::ApprovalPresentation)
            }) {
                return LocalReviewManualEvidenceCreateResult::Created {
                    created_item_id,
                    source: LocalReviewEvidenceSource::ApprovalPresentation,
                    snapshot,
                };
            }
        }
        snapshot.diagnostic_code = Some(LocalReviewDiagnosticCode::InvalidRequest);
        LocalReviewManualEvidenceCreateResult::Failed { snapshot }
    }

    pub fn create_local_review_annotation(
        &self,
        request: LocalReviewAnnotationCreateRequest,
    ) -> LocalReviewSnapshot {
        let collection_id = request.collection_id.clone();
        let result = self.repository.lock().ok().and_then(|mut repository| {
            repository
                .as_mut()?
                .create_local_review_annotation(
                    &request.collection_id,
                    &request.item_id,
                    request.expected_collection_updated_at_ms,
                    &request.text,
                )
                .ok()
        });
        let mut snapshot = self.local_review(LocalReviewListRequest {
            selected_collection_id: Some(collection_id),
        });
        if result.is_none() {
            snapshot.diagnostic_code = Some(LocalReviewDiagnosticCode::InvalidRequest);
        }
        snapshot
    }

    pub fn edit_local_review_annotation(
        &self,
        request: LocalReviewAnnotationEditRequest,
    ) -> LocalReviewSnapshot {
        let collection_id = request.collection_id.clone();
        let result = self.repository.lock().ok().and_then(|mut repository| {
            repository
                .as_mut()?
                .edit_local_review_annotation(
                    &request.collection_id,
                    &request.item_id,
                    &request.annotation_id,
                    request.expected_collection_updated_at_ms,
                    &request.text,
                )
                .ok()
        });
        self.local_review_annotation_mutation_snapshot(collection_id, result.is_some())
    }

    pub fn resolve_local_review_annotation(
        &self,
        request: LocalReviewAnnotationMutationRequest,
    ) -> LocalReviewSnapshot {
        let collection_id = request.collection_id.clone();
        let result = self.repository.lock().ok().and_then(|mut repository| {
            repository
                .as_mut()?
                .resolve_local_review_annotation(
                    &request.collection_id,
                    &request.item_id,
                    &request.annotation_id,
                    request.expected_collection_updated_at_ms,
                )
                .ok()
        });
        self.local_review_annotation_mutation_snapshot(collection_id, result.is_some())
    }

    pub fn reopen_local_review_annotation(
        &self,
        request: LocalReviewAnnotationMutationRequest,
    ) -> LocalReviewSnapshot {
        let collection_id = request.collection_id.clone();
        let result = self.repository.lock().ok().and_then(|mut repository| {
            repository
                .as_mut()?
                .reopen_local_review_annotation(
                    &request.collection_id,
                    &request.item_id,
                    &request.annotation_id,
                    request.expected_collection_updated_at_ms,
                )
                .ok()
        });
        self.local_review_annotation_mutation_snapshot(collection_id, result.is_some())
    }

    pub fn delete_local_review_annotation(
        &self,
        request: LocalReviewAnnotationMutationRequest,
    ) -> LocalReviewSnapshot {
        let collection_id = request.collection_id.clone();
        let result = self.repository.lock().ok().and_then(|mut repository| {
            repository
                .as_mut()?
                .delete_local_review_annotation(
                    &request.collection_id,
                    &request.item_id,
                    &request.annotation_id,
                    request.expected_collection_updated_at_ms,
                )
                .ok()
        });
        self.local_review_annotation_mutation_snapshot(collection_id, result.is_some())
    }

    fn local_review_annotation_mutation_snapshot(
        &self,
        collection_id: String,
        succeeded: bool,
    ) -> LocalReviewSnapshot {
        let mut snapshot = self.local_review(LocalReviewListRequest {
            selected_collection_id: Some(collection_id),
        });
        if !succeeded {
            snapshot.diagnostic_code = Some(LocalReviewDiagnosticCode::InvalidRequest);
        }
        snapshot
    }

    pub fn resume_local_review_collection(
        &self,
        request: LocalReviewCollectionMutationRequest,
    ) -> LocalReviewSnapshot {
        let id = request.collection_id.clone();
        let result = self.repository.lock().ok().and_then(|mut repository| {
            repository
                .as_mut()?
                .resume_local_review_collection(&id, request.expected_collection_updated_at_ms)
                .ok()
        });
        let mut snapshot = self.local_review(LocalReviewListRequest {
            selected_collection_id: Some(id),
        });
        if result.is_none() {
            snapshot.diagnostic_code = Some(LocalReviewDiagnosticCode::InvalidRequest);
        }
        snapshot
    }

    pub fn discard_local_review_collection(
        &self,
        request: LocalReviewCollectionMutationRequest,
    ) -> LocalReviewSnapshot {
        let result = self.repository.lock().ok().and_then(|mut repository| {
            repository
                .as_mut()?
                .discard_local_review_collection(
                    &request.collection_id,
                    request.expected_collection_updated_at_ms,
                )
                .ok()
        });
        let mut snapshot = self.local_review(LocalReviewListRequest {
            selected_collection_id: None,
        });
        if result.is_none() {
            snapshot.diagnostic_code = Some(LocalReviewDiagnosticCode::InvalidRequest);
        }
        snapshot
    }

    pub fn discard_local_review_item(
        &self,
        request: LocalReviewItemDiscardRequest,
    ) -> LocalReviewSnapshot {
        let collection_id = request.collection_id.clone();
        let result = self.repository.lock().ok().and_then(|mut repository| {
            repository
                .as_mut()?
                .discard_local_review_item(
                    &collection_id,
                    &request.item_id,
                    request.expected_collection_updated_at_ms,
                )
                .ok()
        });
        let mut snapshot = self.local_review(LocalReviewListRequest {
            selected_collection_id: Some(collection_id),
        });
        if result.is_none() {
            snapshot.diagnostic_code = Some(LocalReviewDiagnosticCode::InvalidRequest);
        }
        snapshot
    }
    pub fn create_local_review_image_item(
        &self,
        request: LocalReviewImagePickRequest,
        bytes: Vec<u8>,
    ) -> LocalReviewSnapshot {
        let collection_id = request.collection_id.clone();
        let result = self.repository.lock().ok().and_then(|mut repository| {
            repository
                .as_mut()?
                .create_local_review_image_item(
                    &collection_id,
                    request.expected_collection_updated_at_ms,
                    &request.title,
                    &bytes,
                )
                .ok()
        });
        let mut snapshot = self.local_review(LocalReviewListRequest {
            selected_collection_id: Some(collection_id),
        });
        if result.is_none() {
            snapshot.diagnostic_code = Some(LocalReviewDiagnosticCode::InvalidRequest);
        }
        snapshot
    }
    pub fn local_review_image_preview(
        &self,
        request: LocalReviewImagePreviewRequest,
    ) -> Result<LocalReviewImagePreview, ()> {
        self.repository
            .lock()
            .map_err(|_| ())?
            .as_ref()
            .ok_or(())?
            .local_review_image_preview(&request.item_id, &request.sha256)
            .map_err(|_| ())
    }
    pub fn local_review_text_preview(
        &self,
        request: LocalReviewTextPreviewRequest,
    ) -> Result<LocalReviewTextPreview, ()> {
        if !valid_id(&request.collection_id)
            || !valid_id(&request.item_id)
            || !valid_sha256(&request.sha256)
        {
            return Err(());
        }
        Ok(self
            .repository
            .lock()
            .ok()
            .and_then(|repository| {
                repository
                    .as_ref()?
                    .local_review_text_preview(
                        &request.collection_id,
                        &request.item_id,
                        &request.sha256,
                    )
                    .ok()
            })
            .unwrap_or_else(|| local_review_text_preview_unavailable(&request)))
    }
    pub fn create_local_review_manual_evidence(
        &self,
        request: LocalReviewManualEvidenceCreateRequest,
    ) -> LocalReviewManualEvidenceCreateResult {
        let collection_id = request.collection_id.clone();
        let result = self.repository.lock().ok().and_then(|mut repository| {
            repository
                .as_mut()?
                .create_local_review_manual_evidence_item(
                    &collection_id,
                    request.expected_collection_updated_at_ms,
                    &request.title,
                    &request.summary,
                )
                .ok()
        });
        let mut snapshot = self.local_review(LocalReviewListRequest {
            selected_collection_id: Some(collection_id),
        });
        if let Some(created_item_id) = result {
            if snapshot.items.iter().any(|item| {
                item.item_id == created_item_id
                    && item.evidence_source
                        == Some(LocalReviewEvidenceSource::ManualValidationSummary)
            }) {
                return LocalReviewManualEvidenceCreateResult::Created {
                    created_item_id,
                    source: LocalReviewEvidenceSource::ManualValidationSummary,
                    snapshot,
                };
            }
        }
        if snapshot.diagnostic_code.is_none() {
            snapshot.diagnostic_code = Some(LocalReviewDiagnosticCode::InvalidRequest);
        }
        LocalReviewManualEvidenceCreateResult::Failed { snapshot }
    }
    pub fn local_review_manual_evidence_preview(
        &self,
        item_id: String,
        sha256: String,
    ) -> Result<LocalReviewManualEvidencePreview, ()> {
        self.repository
            .lock()
            .map_err(|_| ())?
            .as_ref()
            .ok_or(())?
            .local_review_manual_evidence_preview(&item_id, &sha256)
            .map_err(|_| ())
    }
    pub fn local_review_m48_generated_artifact_metadata_evidence_preview(
        &self,
        item_id: String,
        sha256: String,
    ) -> Result<LocalReviewM48GeneratedArtifactMetadataEvidencePreview, ()> {
        self.repository
            .lock()
            .map_err(|_| ())?
            .as_ref()
            .ok_or(())?
            .local_review_m48_generated_artifact_metadata_evidence_preview(&item_id, &sha256)
            .map_err(|_| ())
    }
    pub fn local_review_safe_preview_metadata_evidence_preview(
        &self,
        item_id: String,
        sha256: String,
    ) -> Result<LocalReviewSafePreviewMetadataEvidencePreview, ()> {
        self.repository
            .lock()
            .map_err(|_| ())?
            .as_ref()
            .ok_or(())?
            .local_review_safe_preview_metadata_evidence_preview(&item_id, &sha256)
            .map_err(|_| ())
    }
    pub fn local_review_package_manifest_summary_evidence_preview(
        &self,
        item_id: String,
        sha256: String,
    ) -> Result<LocalReviewPackageManifestSummaryEvidencePreview, ()> {
        self.repository
            .lock()
            .map_err(|_| ())?
            .as_ref()
            .ok_or(())?
            .local_review_package_manifest_summary_evidence_preview(&item_id, &sha256)
            .map_err(|_| ())
    }
    pub fn local_review_git_status_diff_summary_evidence_preview(
        &self,
        item_id: String,
        sha256: String,
    ) -> Result<LocalReviewGitStatusDiffSummaryEvidencePreview, ()> {
        self.repository
            .lock()
            .map_err(|_| ())?
            .as_ref()
            .ok_or(())?
            .local_review_git_status_diff_summary_evidence_preview(&item_id, &sha256)
            .map_err(|_| ())
    }
    pub fn local_review_activity_presentation_evidence_preview(
        &self,
        item_id: String,
        sha256: String,
    ) -> Result<LocalReviewActivityPresentationEvidencePreview, ()> {
        self.repository
            .lock()
            .map_err(|_| ())?
            .as_ref()
            .ok_or(())?
            .local_review_activity_presentation_evidence_preview(&item_id, &sha256)
            .map_err(|_| ())
    }
    pub fn local_review_approval_presentation_evidence_preview(
        &self,
        item_id: String,
        sha256: String,
    ) -> Result<LocalReviewApprovalPresentationEvidencePreview, ()> {
        self.repository
            .lock()
            .map_err(|_| ())?
            .as_ref()
            .ok_or(())?
            .local_review_approval_presentation_evidence_preview(&item_id, &sha256)
            .map_err(|_| ())
    }
    pub fn task_catalog(&self, request: TaskCatalogListRequest) -> TaskCatalogSnapshot {
        if request
            .selected_task_id
            .as_deref()
            .is_some_and(|id| !valid_id(id))
        {
            return TaskCatalogSnapshot {
                diagnostic_code: Some(TaskDiagnosticCode::InvalidRequest),
                ..task_catalog_unavailable()
            };
        }
        let mut repository = match self.repository.lock() {
            Ok(value) => value,
            Err(_) => return task_catalog_unavailable(),
        };
        let Some(repository) = repository.as_mut() else {
            return task_catalog_unavailable();
        };
        let selected = request
            .selected_task_id
            .as_deref()
            .filter(|id| valid_id(id));
        match repository.task_catalog(selected, request.include_archived, request.query.as_deref())
        {
            Ok((tasks, selected_task, plans, task_count, payload_bytes, corrupt_rows)) => {
                TaskCatalogSnapshot {
                    schema_version: TASK_RECORD_SCHEMA_VERSION,
                    state: if tasks.is_empty() {
                        TaskCatalogState::Empty
                    } else {
                        TaskCatalogState::Ready
                    },
                    tasks,
                    selected_task,
                    plans,
                    task_count,
                    payload_bytes,
                    warning: task_count >= 160 || payload_bytes >= 6 * 1024 * 1024,
                    diagnostic_code: corrupt_rows.then_some(TaskDiagnosticCode::InvalidStoredValue),
                }
            }
            Err(error) => TaskCatalogSnapshot {
                diagnostic_code: Some(map_task_storage_error(&error)),
                ..task_catalog_unavailable()
            },
        }
    }

    pub fn create_task_record(&self) -> TaskCatalogSnapshot {
        let result = {
            let mut repository = match self.repository.lock() {
                Ok(value) => value,
                Err(_) => return task_catalog_unavailable(),
            };
            let Some(repository) = repository.as_mut() else {
                return task_catalog_unavailable();
            };
            repository.create_task()
        };
        match result {
            Ok(id) => self.task_catalog(TaskCatalogListRequest {
                query: None,
                include_archived: false,
                selected_task_id: Some(id),
            }),
            Err(error) => self.task_snapshot_with_diagnostic(None, error),
        }
    }

    /// Creates a project-bound task only from a native persisted conversation
    /// context. The request identifies the context, never the project.
    pub fn create_task_record_from_conversation(
        &self,
        conversation_id: String,
    ) -> TaskCatalogSnapshot {
        if !valid_id(&conversation_id) {
            return TaskCatalogSnapshot {
                diagnostic_code: Some(TaskDiagnosticCode::InvalidRequest),
                ..task_catalog_unavailable()
            };
        }
        let result = {
            let mut repository = match self.repository.lock() {
                Ok(value) => value,
                Err(_) => return task_catalog_unavailable(),
            };
            let Some(repository) = repository.as_mut() else {
                return task_catalog_unavailable();
            };
            repository.create_task_from_conversation_context(&conversation_id)
        };
        match result {
            Ok(id) => self.task_catalog(TaskCatalogListRequest {
                query: None,
                include_archived: false,
                selected_task_id: Some(id),
            }),
            Err(error) => self.task_snapshot_with_diagnostic(None, error),
        }
    }
    pub fn task_action(
        &self,
        task_id: String,
        action: impl FnOnce(&mut ProjectRepository, &str) -> Result<(), StorageError>,
    ) -> TaskCatalogSnapshot {
        if !valid_id(&task_id) {
            return TaskCatalogSnapshot {
                diagnostic_code: Some(TaskDiagnosticCode::InvalidRequest),
                ..task_catalog_unavailable()
            };
        }
        let result = {
            let mut guard = match self.repository.lock() {
                Ok(value) => value,
                Err(_) => return task_catalog_unavailable(),
            };
            let Some(repository) = guard.as_mut() else {
                return task_catalog_unavailable();
            };
            action(repository, &task_id)
        };
        match result {
            Ok(()) => self.task_catalog(TaskCatalogListRequest {
                query: None,
                include_archived: false,
                selected_task_id: Some(task_id),
            }),
            Err(error) => self.task_snapshot_with_diagnostic(Some(task_id), error),
        }
    }
    pub fn create_task_plan(
        &self,
        task_id: String,
        copy_primary_body: bool,
    ) -> TaskCatalogSnapshot {
        if !valid_id(&task_id) {
            return TaskCatalogSnapshot {
                diagnostic_code: Some(TaskDiagnosticCode::InvalidRequest),
                ..task_catalog_unavailable()
            };
        }
        let result = {
            let mut guard = match self.repository.lock() {
                Ok(value) => value,
                Err(_) => return task_catalog_unavailable(),
            };
            let Some(repo) = guard.as_mut() else {
                return task_catalog_unavailable();
            };
            repo.create_plan(&task_id, copy_primary_body)
        };
        match result {
            Ok(_) => self.task_catalog(TaskCatalogListRequest {
                query: None,
                include_archived: false,
                selected_task_id: Some(task_id),
            }),
            Err(error) => self.task_snapshot_with_diagnostic(Some(task_id), error),
        }
    }

    fn task_snapshot_with_diagnostic(
        &self,
        selected_task_id: Option<String>,
        error: StorageError,
    ) -> TaskCatalogSnapshot {
        let diagnostic = map_task_storage_error(&error);
        let mut snapshot = self.task_catalog(TaskCatalogListRequest {
            query: None,
            include_archived: false,
            selected_task_id,
        });
        snapshot.diagnostic_code = Some(diagnostic);
        snapshot
    }
    pub fn unavailable() -> Self {
        Self {
            repository: Mutex::new(None),
            pending: Mutex::new(None),
            active_executions: Mutex::new(HashSet::new()),
            active_terminals: Mutex::new(HashMap::new()),
            promotion_reservations: Mutex::new(VecDeque::new()),
        }
    }

    pub fn open(database_path: &Path) -> Self {
        Self {
            repository: Mutex::new(ProjectRepository::open(database_path).ok()),
            pending: Mutex::new(None),
            active_executions: Mutex::new(HashSet::new()),
            active_terminals: Mutex::new(HashMap::new()),
            promotion_reservations: Mutex::new(VecDeque::new()),
        }
    }

    /// Fixed native executable path for the restricted installed-host phase.
    /// It resolves every authority input from durable metadata and delegates
    /// recording exclusively to the package-validation controller.
    pub(crate) fn complete_installed_host_validation(&self) -> InstalledHostHeadlessStatus {
        let project_id = match self.repository.lock().ok().and_then(|guard| {
            guard.as_ref().and_then(|repository| {
                repository
                    .installed_host_headless_context_project_id_for_internal()
                    .ok()
            })
        }) {
            Some(value) => value,
            None => return InstalledHostHeadlessStatus::Unavailable,
        };
        let context =
            match PackageValidationController::trusted_context_from_live_project(self, &project_id)
            {
                Ok(context) => context,
                Err(_) => return InstalledHostHeadlessStatus::Unavailable,
            };
        let mut repository_guard = match self.repository.lock() {
            Ok(guard) => guard,
            Err(_) => return InstalledHostHeadlessStatus::Unavailable,
        };
        let Some(repository) = repository_guard.as_mut() else {
            return InstalledHostHeadlessStatus::Unavailable;
        };
        let mut controller = PackageValidationController::default();
        if !PackageValidationController::installed_debian_version_is("0.1.0~beta.51")
            .unwrap_or(false)
        {
            return InstalledHostHeadlessStatus::Unavailable;
        }
        if controller
            .run_and_record(repository, context.clone())
            .is_err()
        {
            return InstalledHostHeadlessStatus::Failed;
        }
        let predecessor = match repository.installed_host_headless_predecessor_for_internal() {
            Ok((resolved_project_id, predecessor)) if resolved_project_id == project_id => {
                predecessor
            }
            _ => return InstalledHostHeadlessStatus::Failed,
        };
        match controller.run_installed_host_and_record(repository, context, &predecessor.id) {
            Ok(InstalledHostValidationOutcome::Created) => InstalledHostHeadlessStatus::Created,
            Ok(InstalledHostValidationOutcome::Existing) => InstalledHostHeadlessStatus::Existing,
            Ok(InstalledHostValidationOutcome::Failed) => InstalledHostHeadlessStatus::Failed,
            Ok(InstalledHostValidationOutcome::Unavailable)
            | Err(PackageValidationControllerError::Unavailable)
            | Err(PackageValidationControllerError::ContextUnavailable) => {
                InstalledHostHeadlessStatus::Unavailable
            }
            Err(_) => InstalledHostHeadlessStatus::Failed,
        }
    }

    #[cfg(test)]
    pub(crate) fn in_memory() -> Self {
        Self {
            repository: Mutex::new(ProjectRepository::in_memory().ok()),
            pending: Mutex::new(None),
            active_executions: Mutex::new(HashSet::new()),
            active_terminals: Mutex::new(HashMap::new()),
            promotion_reservations: Mutex::new(VecDeque::new()),
        }
    }

    #[cfg(test)]
    pub(crate) fn fail_worktree_registration_for_test(&self) {
        self.repository
            .lock()
            .expect("test repository lock must be available")
            .as_ref()
            .expect("test repository must be available")
            .fail_worktree_registration_for_test()
            .expect("test failure trigger must install");
    }

    #[cfg(test)]
    pub(crate) fn allow_worktree_registration_for_test(&self) {
        self.repository
            .lock()
            .expect("test repository lock must be available")
            .as_ref()
            .expect("test repository must be available")
            .allow_worktree_registration_for_test()
            .expect("test failure trigger must be removed");
    }

    #[cfg(test)]
    pub(crate) fn fail_worktree_retirement_for_test(&self) {
        self.repository
            .lock()
            .expect("test repository lock must be available")
            .as_ref()
            .expect("test repository must be available")
            .fail_worktree_retirement_for_test()
            .expect("test retirement failure trigger must install");
    }

    #[cfg(test)]
    pub(crate) fn allow_worktree_retirement_for_test(&self) {
        self.repository
            .lock()
            .expect("test repository lock must be available")
            .as_ref()
            .expect("test repository must be available")
            .allow_worktree_retirement_for_test()
            .expect("test retirement failure trigger must be removed");
    }

    pub fn status(&self) -> ProjectWorkspaceSnapshot {
        self.build_snapshot(None)
    }

    /// Returns only QuireForge-owned, reference-only Advisor metadata. No
    /// project identity, filesystem, Git, Codex, or execution state is read.
    pub fn advisor_snapshot(&self) -> Result<AdvisorFoundationSnapshot, ()> {
        let repository = self.repository.lock().map_err(|_| ())?;
        repository
            .as_ref()
            .ok_or(())?
            .advisor_snapshot()
            .map_err(|_| ())
    }

    pub fn advisor_workspace_snapshot(&self) -> Result<AdvisorWorkspaceSnapshot, ()> {
        Ok(self.advisor_snapshot()?.workspace_snapshot())
    }

    pub(crate) fn create_advisor_dispatch_proposal(
        &self,
        proposal: &AdvisorDispatchProposal,
    ) -> Result<AdvisorApprovalSnapshot, ProjectExecutionError> {
        let mut repository = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository
            .as_mut()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .insert_advisor_dispatch_proposal(proposal)
            .map_err(map_project_execution_storage_error)?;
        Ok(AdvisorApprovalSnapshot {
            proposal_id: proposal.id.clone(),
            state: proposal.state,
            expires_at_ms: proposal.expires_at_ms,
            dispatch_available: false,
        })
    }

    pub(crate) fn decide_advisor_dispatch_proposal(
        &self,
        proposal_id: &str,
        decision: AdvisorDispatchState,
    ) -> Result<AdvisorApprovalSnapshot, ProjectExecutionError> {
        let mut repository = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository
            .as_mut()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        let proposal = repository
            .decide_advisor_dispatch_proposal(proposal_id, decision)
            .map_err(map_project_execution_storage_error)?;
        Ok(AdvisorApprovalSnapshot {
            proposal_id: proposal.id,
            state: proposal.state,
            expires_at_ms: proposal.expires_at_ms,
            dispatch_available: false,
        })
    }

    pub(crate) fn advisor_dispatch_proposal(
        &self,
        proposal_id: &str,
    ) -> Result<AdvisorDispatchProposal, ProjectExecutionError> {
        let repository = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        repository
            .as_ref()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?
            .advisor_dispatch_proposal(proposal_id)
            .map_err(map_project_execution_storage_error)
    }

    pub(crate) fn claim_advisor_dispatch_proposal(
        &self,
        proposal_id: &str,
    ) -> Result<(), ProjectExecutionError> {
        let mut repository = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        repository
            .as_mut()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?
            .claim_advisor_dispatch_proposal(proposal_id)
            .map_err(map_project_execution_storage_error)
    }

    pub(crate) fn finish_advisor_dispatch_proposal(
        &self,
        proposal_id: &str,
        conversation_id: Option<&str>,
    ) -> Result<(), ProjectExecutionError> {
        let mut repository = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        repository
            .as_mut()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?
            .finish_advisor_dispatch_proposal(proposal_id, conversation_id)
            .map_err(map_project_execution_storage_error)
    }

    pub fn picker_unavailable(&self) -> ProjectWorkspaceSnapshot {
        self.build_snapshot(Some(ProjectDiagnosticCode::PickerUnavailable))
    }

    pub fn prepare_attachment(&self, selected_path: PathBuf) -> ProjectWorkspaceSnapshot {
        self.prepare(PendingAttachmentKind::Attach, None, selected_path)
    }

    pub fn prepare_relink(
        &self,
        project_id: String,
        selected_path: PathBuf,
    ) -> ProjectWorkspaceSnapshot {
        if !valid_id(&project_id) {
            return self.build_snapshot(Some(ProjectDiagnosticCode::ProjectNotFound));
        }
        if self.execution_active(&project_id) {
            return self.build_snapshot(Some(ProjectDiagnosticCode::ProjectBusy));
        }
        self.prepare(
            PendingAttachmentKind::Relink,
            Some(project_id),
            selected_path,
        )
    }

    pub fn cancel_pending(&self) -> ProjectWorkspaceSnapshot {
        if let Ok(mut pending) = self.pending.lock() {
            *pending = None;
        }
        self.status()
    }

    pub fn confirm_pending(&self) -> ProjectWorkspaceSnapshot {
        let pending = self
            .pending
            .lock()
            .ok()
            .and_then(|mut pending| pending.take());
        let Some(pending) = pending else {
            return self.build_snapshot(Some(ProjectDiagnosticCode::AttachmentNotPending));
        };
        if pending
            .project_id
            .as_deref()
            .is_some_and(|project_id| self.execution_active(project_id))
        {
            return self.build_snapshot(Some(ProjectDiagnosticCode::ProjectBusy));
        }

        let current_identity = match inspect_directory(&pending.identity.selected_path) {
            Ok(identity) => identity,
            Err(_) => return self.build_snapshot(Some(ProjectDiagnosticCode::IdentityChanged)),
        };
        if !same_identity(&pending.identity, &current_identity) {
            return self.build_snapshot(Some(ProjectDiagnosticCode::IdentityChanged));
        }

        let mut repository_guard = match self.repository.lock() {
            Ok(repository) => repository,
            Err(_) => {
                return ProjectWorkspaceSnapshot::unavailable(
                    ProjectDiagnosticCode::MetadataUnavailable,
                );
            }
        };
        let Some(repository) = repository_guard.as_mut() else {
            return ProjectWorkspaceSnapshot::unavailable(
                ProjectDiagnosticCode::MetadataUnavailable,
            );
        };
        let result = match pending.kind {
            PendingAttachmentKind::Attach => repository
                .insert_project(&pending.display_name, &current_identity)
                .map(|_| ()),
            PendingAttachmentKind::Relink => repository.relink_project(
                pending
                    .project_id
                    .as_deref()
                    .expect("relink pending state always has a project ID"),
                &current_identity,
            ),
        };
        drop(repository_guard);
        match result {
            Ok(()) => self.status(),
            Err(error) => self.build_snapshot(Some(map_storage_error(&error))),
        }
    }

    pub fn detach(&self, project_id: String) -> ProjectWorkspaceSnapshot {
        self.metadata_action(&project_id, |repository, project_id| {
            repository.detach_project(project_id)
        })
    }

    pub fn archive(&self, project_id: String) -> ProjectWorkspaceSnapshot {
        self.metadata_action(&project_id, |repository, project_id| {
            repository.archive_project(project_id)
        })
    }

    pub fn preflight(&self, project_id: String) -> ProjectPreflightSnapshot {
        if !valid_id(&project_id) {
            return unavailable_preflight(project_id, ProjectDiagnosticCode::ProjectNotFound);
        }
        let repository_guard = match self.repository.lock() {
            Ok(repository) => repository,
            Err(_) => {
                return unavailable_preflight(
                    project_id,
                    ProjectDiagnosticCode::MetadataUnavailable,
                );
            }
        };
        let Some(repository) = repository_guard.as_ref() else {
            return unavailable_preflight(project_id, ProjectDiagnosticCode::MetadataUnavailable);
        };
        let project = match repository.project(&project_id) {
            Ok(project) => project,
            Err(StorageError::ProjectNotFound) => {
                return unavailable_preflight(project_id, ProjectDiagnosticCode::ProjectNotFound);
            }
            Err(_) => {
                return unavailable_preflight(
                    project_id,
                    ProjectDiagnosticCode::MetadataUnavailable,
                );
            }
        };
        drop(repository_guard);
        let Some(association) = project.association else {
            return ProjectPreflightSnapshot {
                schema_version: PROJECT_SCHEMA_VERSION,
                project_id,
                cwd_ready: false,
                display_path: None,
                state: DirectoryAccessibilityState::MissingOrMoved,
                diagnostic_code: None,
            };
        };

        let selected_path = PathBuf::from(&association.selected_path);
        match inspect_directory(&selected_path) {
            Ok(identity) if same_stored_identity(&association, &identity) => {
                let cwd_ready =
                    identity.accessibility == DirectoryAccessibilityState::ConnectedAccessible;
                ProjectPreflightSnapshot {
                    schema_version: PROJECT_SCHEMA_VERSION,
                    project_id,
                    cwd_ready,
                    display_path: Some(identity.selected_display_path),
                    state: identity.accessibility,
                    diagnostic_code: None,
                }
            }
            Ok(_) => ProjectPreflightSnapshot {
                schema_version: PROJECT_SCHEMA_VERSION,
                project_id,
                cwd_ready: false,
                display_path: Some(display_path(&selected_path)),
                state: DirectoryAccessibilityState::IdentityChanged,
                diagnostic_code: Some(ProjectDiagnosticCode::IdentityChanged),
            },
            Err(error) => ProjectPreflightSnapshot {
                schema_version: PROJECT_SCHEMA_VERSION,
                project_id,
                cwd_ready: false,
                display_path: Some(display_path(&selected_path)),
                state: preflight_failure_state(&association, error),
                diagnostic_code: None,
            },
        }
    }

    pub(crate) fn execution_cwd(&self, project_id: &str) -> Result<PathBuf, ProjectExecutionError> {
        if !valid_id(project_id) {
            return Err(ProjectExecutionError::InvalidProjectId);
        }
        let repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_ref()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        let project = repository
            .project(project_id)
            .map_err(|error| match error {
                StorageError::ProjectNotFound => ProjectExecutionError::ProjectNotFound,
                _ => ProjectExecutionError::MetadataUnavailable,
            })?;
        if project.archived {
            return Err(ProjectExecutionError::ProjectNotFound);
        }
        let association = project
            .association
            .ok_or(ProjectExecutionError::DirectoryUnavailable)?;
        drop(repository_guard);

        let identity = inspect_directory(Path::new(&association.selected_path))
            .map_err(|_| ProjectExecutionError::DirectoryUnavailable)?;
        if !same_stored_identity(&association, &identity) {
            return Err(ProjectExecutionError::IdentityChanged);
        }
        if identity.accessibility != DirectoryAccessibilityState::ConnectedAccessible {
            return Err(ProjectExecutionError::NotWritable);
        }
        Ok(identity.resolved_path)
    }

    pub(crate) fn review_root(
        &self,
        project_id: &str,
    ) -> Result<ProjectReviewRoot, ProjectExecutionError> {
        self.review_root_with_archived(project_id, false)
    }

    pub(crate) fn content_root(&self, project_id: &str) -> Result<PathBuf, ProjectExecutionError> {
        if !valid_id(project_id) {
            return Err(ProjectExecutionError::InvalidProjectId);
        }
        let repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_ref()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        let project = repository
            .project(project_id)
            .map_err(|error| match error {
                StorageError::ProjectNotFound => ProjectExecutionError::ProjectNotFound,
                _ => ProjectExecutionError::MetadataUnavailable,
            })?;
        if project.archived {
            return Err(ProjectExecutionError::ProjectNotFound);
        }
        let association = project
            .association
            .ok_or(ProjectExecutionError::DirectoryUnavailable)?;
        drop(repository_guard);

        let identity = inspect_directory(Path::new(&association.selected_path))
            .map_err(|_| ProjectExecutionError::DirectoryUnavailable)?;
        if !same_stored_identity(&association, &identity) {
            return Err(ProjectExecutionError::IdentityChanged);
        }
        if !matches!(
            identity.accessibility,
            DirectoryAccessibilityState::ConnectedAccessible
                | DirectoryAccessibilityState::ConnectedReadOnly
        ) {
            return Err(ProjectExecutionError::DirectoryUnavailable);
        }
        Ok(identity.resolved_path)
    }

    pub(crate) fn cleanup_worktree_root(
        &self,
        project_id: &str,
    ) -> Result<ProjectReviewRoot, ProjectExecutionError> {
        self.review_root_with_archived(project_id, true)
    }

    fn review_root_with_archived(
        &self,
        project_id: &str,
        allow_archived: bool,
    ) -> Result<ProjectReviewRoot, ProjectExecutionError> {
        if !valid_id(project_id) {
            return Err(ProjectExecutionError::InvalidProjectId);
        }
        let repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_ref()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        let project = repository
            .project(project_id)
            .map_err(|error| match error {
                StorageError::ProjectNotFound => ProjectExecutionError::ProjectNotFound,
                _ => ProjectExecutionError::MetadataUnavailable,
            })?;
        if project.archived && !allow_archived {
            return Err(ProjectExecutionError::ProjectNotFound);
        }
        let association = project
            .association
            .ok_or(ProjectExecutionError::DirectoryUnavailable)?;
        drop(repository_guard);

        let identity = inspect_directory(Path::new(&association.selected_path))
            .map_err(|_| ProjectExecutionError::DirectoryUnavailable)?;
        if !same_stored_identity(&association, &identity) {
            return Err(ProjectExecutionError::IdentityChanged);
        }
        if !matches!(
            identity.accessibility,
            DirectoryAccessibilityState::ConnectedAccessible
                | DirectoryAccessibilityState::ConnectedReadOnly
        ) {
            return Err(ProjectExecutionError::DirectoryUnavailable);
        }
        let git = identity.git.ok_or(ProjectExecutionError::NotRepository)?;
        if !identity.resolved_path.starts_with(&git.worktree_root) {
            return Err(ProjectExecutionError::IdentityChanged);
        }
        Ok(ProjectReviewRoot {
            attached_root: identity.resolved_path,
            worktree_root: git.worktree_root,
            git_dir: git.git_dir,
            common_dir: git.common_dir,
            writable: identity.accessibility == DirectoryAccessibilityState::ConnectedAccessible,
        })
    }

    pub(crate) fn worktree_context(
        &self,
        project_id: &str,
    ) -> Result<ProjectWorktreeContext, ProjectExecutionError> {
        if !valid_id(project_id) {
            return Err(ProjectExecutionError::InvalidProjectId);
        }
        let repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_ref()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        let source_project_id = repository
            .worktree_source_project_id(project_id)
            .map_err(map_project_execution_storage_error)?;
        let source = repository
            .project(&source_project_id)
            .map_err(map_project_execution_storage_error)?;
        if source.archived {
            return Err(ProjectExecutionError::ProjectNotFound);
        }
        let relations = repository
            .list_worktree_relations(&source_project_id)
            .map_err(map_project_execution_storage_error)?;
        let mut records = Vec::with_capacity(relations.len());
        for relation in relations {
            let project = repository
                .project(&relation.worktree_project_id)
                .map_err(map_project_execution_storage_error)?;
            records.push(worktree_record(relation, project));
        }
        Ok(ProjectWorktreeContext {
            source_project_id,
            source_display_name: source.display_name,
            records,
        })
    }

    pub(crate) fn register_worktree_project(
        &self,
        source_project_id: &str,
        selected_path: &Path,
        expected_common_dir: &Path,
        ownership: &str,
        branch_name: Option<&str>,
    ) -> Result<String, WorktreeRegistrationError> {
        if !valid_id(source_project_id) {
            return Err(WorktreeRegistrationError::Project(
                ProjectExecutionError::InvalidProjectId,
            ));
        }
        let identity = inspect_directory(selected_path).map_err(|_| {
            WorktreeRegistrationError::Project(ProjectExecutionError::DirectoryUnavailable)
        })?;
        if identity.accessibility != DirectoryAccessibilityState::ConnectedAccessible {
            return Err(WorktreeRegistrationError::Project(
                ProjectExecutionError::NotWritable,
            ));
        }
        let git = identity
            .git
            .as_ref()
            .ok_or(WorktreeRegistrationError::NotLinkedWorktree)?;
        if !git.is_linked_worktree {
            return Err(WorktreeRegistrationError::NotLinkedWorktree);
        }
        if git.common_dir != expected_common_dir {
            return Err(WorktreeRegistrationError::DifferentRepository);
        }
        let display_name = branch_name
            .map(str::to_owned)
            .unwrap_or_else(|| directory_display_name(selected_path));
        let mut repository_guard = self.repository.lock().map_err(|_| {
            WorktreeRegistrationError::Project(ProjectExecutionError::MetadataUnavailable)
        })?;
        let repository = repository_guard
            .as_mut()
            .ok_or(WorktreeRegistrationError::Project(
                ProjectExecutionError::MetadataUnavailable,
            ))?;
        repository
            .insert_worktree_project(
                source_project_id,
                &display_name,
                &identity,
                ownership,
                branch_name,
            )
            .map_err(|error| match error {
                StorageError::DuplicateDirectory => WorktreeRegistrationError::DuplicateDirectory,
                error => {
                    WorktreeRegistrationError::Project(map_project_execution_storage_error(error))
                }
            })
    }

    pub(crate) fn retire_worktree_project(
        &self,
        source_project_id: &str,
        worktree_project_id: &str,
        expected_ownership: &str,
    ) -> Result<(), ProjectExecutionError> {
        if !valid_id(source_project_id) || !valid_id(worktree_project_id) {
            return Err(ProjectExecutionError::InvalidProjectId);
        }
        let mut repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_mut()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .retire_worktree_project(source_project_id, worktree_project_id, expected_ownership)
            .map_err(map_project_execution_storage_error)
    }

    pub(crate) fn inspect_worktree_candidate(
        &self,
        selected_path: &Path,
    ) -> Result<ProjectWorktreeCandidate, ProjectExecutionError> {
        let identity = inspect_directory(selected_path)
            .map_err(|_| ProjectExecutionError::DirectoryUnavailable)?;
        if identity.accessibility != DirectoryAccessibilityState::ConnectedAccessible {
            return Err(ProjectExecutionError::NotWritable);
        }
        let git = identity
            .git
            .as_ref()
            .ok_or(ProjectExecutionError::NotRepository)?;
        Ok(ProjectWorktreeCandidate {
            selected_path: identity.selected_path,
            resolved_path: identity.resolved_path,
            display_path: identity.selected_display_path,
            worktree_root: git.worktree_root.clone(),
            common_dir: git.common_dir.clone(),
            is_linked_worktree: git.is_linked_worktree,
            device_id: identity.device_id,
            inode: identity.inode,
            mount_id: identity.mount_id,
            filesystem_type: identity.filesystem_type,
            has_agents_guidance: identity.has_agents_guidance,
            has_codex_config: identity.has_codex_config,
        })
    }

    pub(crate) fn reserve_execution(&self, project_id: &str) -> Result<(), ProjectExecutionError> {
        if !valid_id(project_id) {
            return Err(ProjectExecutionError::InvalidProjectId);
        }
        let mut active = self
            .active_executions
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let terminals = self
            .active_terminals
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        if terminals.contains_key(project_id) || !active.insert(project_id.to_owned()) {
            return Err(ProjectExecutionError::ProjectBusy);
        }
        Ok(())
    }

    pub(crate) fn release_execution(&self, project_id: &str) {
        if let Ok(mut active) = self.active_executions.lock() {
            active.remove(project_id);
        }
    }

    pub(crate) fn reserve_terminal(&self, project_id: &str) -> Result<(), ProjectExecutionError> {
        if !valid_id(project_id) {
            return Err(ProjectExecutionError::InvalidProjectId);
        }
        let active = self
            .active_executions
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        if active.contains(project_id) {
            return Err(ProjectExecutionError::ProjectBusy);
        }
        let mut terminals = self
            .active_terminals
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        *terminals.entry(project_id.to_owned()).or_default() += 1;
        Ok(())
    }

    pub(crate) fn release_terminal(&self, project_id: &str) {
        if let Ok(mut terminals) = self.active_terminals.lock() {
            if let Some(count) = terminals.get_mut(project_id) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    terminals.remove(project_id);
                }
            }
        }
    }

    pub(crate) fn record_conversation_reference(
        &self,
        reference: ConversationReference<'_>,
    ) -> Result<(), ProjectExecutionError> {
        let mut repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_mut()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .insert_conversation_reference(&reference)
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)
    }

    pub(crate) fn record_chat_conversation_metadata(
        &self,
        metadata: ChatConversationMetadata<'_>,
    ) -> Result<(), ProjectExecutionError> {
        if !valid_id(metadata.conversation_id) || !valid_id(metadata.codex_thread_id) {
            return Err(ProjectExecutionError::InvalidProjectId);
        }
        let mut repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_mut()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .insert_chat_conversation_metadata(&metadata)
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)
    }

    pub(crate) fn record_advisor_conversation_metadata(
        &self,
        metadata: AdvisorConversationMetadata<'_>,
    ) -> Result<(), ProjectExecutionError> {
        if !valid_id(metadata.conversation_id) || !valid_id(metadata.codex_thread_id) {
            return Err(ProjectExecutionError::InvalidProjectId);
        }
        let mut repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_mut()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .insert_advisor_conversation_metadata(&metadata)
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)
    }

    pub(crate) fn conversation_reference(
        &self,
        conversation_id: &str,
    ) -> Result<StoredConversationReference, ProjectExecutionError> {
        if !valid_id(conversation_id) {
            return Err(ProjectExecutionError::InvalidProjectId);
        }
        let repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_ref()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .conversation_reference(conversation_id)
            .map_err(|error| match error {
                StorageError::InvalidStoredValue => ProjectExecutionError::ProjectNotFound,
                _ => ProjectExecutionError::MetadataUnavailable,
            })
    }

    pub(crate) fn conversation_references(
        &self,
        project_id: Option<&str>,
    ) -> Result<Vec<StoredConversationReference>, ProjectExecutionError> {
        if project_id.is_some_and(|value| !valid_id(value)) {
            return Err(ProjectExecutionError::InvalidProjectId);
        }
        let repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_ref()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .list_conversation_references(project_id)
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)
    }

    pub(crate) fn record_conversation_turn(
        &self,
        conversation_id: &str,
        active_turn_id: &str,
    ) -> Result<(), ProjectExecutionError> {
        let mut repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_mut()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .update_conversation_turn(conversation_id, active_turn_id)
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)
    }

    pub(crate) fn record_conversation_status(
        &self,
        conversation_id: &str,
        status: &str,
    ) -> Result<(), ProjectExecutionError> {
        if !matches!(
            status,
            "stopping" | "completed" | "interrupted" | "blocked" | "failed"
        ) {
            return Err(ProjectExecutionError::MetadataUnavailable);
        }
        let mut repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_mut()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .update_conversation_status(conversation_id, status)
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)
    }

    pub(crate) fn record_conversation_archived(
        &self,
        conversation_id: &str,
        archived: bool,
    ) -> Result<(), ProjectExecutionError> {
        let mut repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_mut()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .update_conversation_archived(conversation_id, archived)
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)
    }

    pub(crate) fn record_model_selection(
        &self,
        conversation_id: &str,
        effective: Option<(&str, &str)>,
        selection: ConversationSelectionMetadata<'_>,
    ) -> Result<(), ProjectExecutionError> {
        if !valid_id(conversation_id) {
            return Err(ProjectExecutionError::InvalidProjectId);
        }
        let mut repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_mut()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .update_model_selection(conversation_id, effective, &selection)
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)
    }

    pub(crate) fn record_terminal_start(
        &self,
        terminal_id: &str,
        project_id: &str,
        title: &str,
        columns: u16,
        rows: u16,
    ) -> Result<(), ProjectExecutionError> {
        let mut repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_mut()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .insert_terminal_session(terminal_id, project_id, title, columns, rows)
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)
    }

    pub(crate) fn record_terminal_state(
        &self,
        terminal_id: &str,
        status: &str,
        columns: u16,
        rows: u16,
        exit_code: Option<i32>,
    ) -> Result<(), ProjectExecutionError> {
        let mut repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_mut()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .update_terminal_session(terminal_id, status, columns, rows, exit_code)
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)
    }

    pub(crate) fn remove_terminal_record(
        &self,
        terminal_id: &str,
    ) -> Result<(), ProjectExecutionError> {
        let mut repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_mut()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .delete_terminal_session(terminal_id)
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)
    }

    pub(crate) fn terminal_records(
        &self,
    ) -> Result<Vec<StoredTerminalSession>, ProjectExecutionError> {
        let repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_ref()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .list_terminal_sessions()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)
    }

    fn prepare(
        &self,
        kind: PendingAttachmentKind,
        project_id: Option<String>,
        selected_path: PathBuf,
    ) -> ProjectWorkspaceSnapshot {
        let identity = match inspect_directory(&selected_path) {
            Ok(identity)
                if matches!(
                    identity.accessibility,
                    DirectoryAccessibilityState::ConnectedAccessible
                        | DirectoryAccessibilityState::ConnectedReadOnly
                ) =>
            {
                identity
            }
            Ok(_) | Err(_) => {
                return self.build_snapshot(Some(ProjectDiagnosticCode::DirectoryUnavailable));
            }
        };

        let repository_guard = match self.repository.lock() {
            Ok(repository) => repository,
            Err(_) => {
                return ProjectWorkspaceSnapshot::unavailable(
                    ProjectDiagnosticCode::MetadataUnavailable,
                );
            }
        };
        let Some(repository) = repository_guard.as_ref() else {
            return ProjectWorkspaceSnapshot::unavailable(
                ProjectDiagnosticCode::MetadataUnavailable,
            );
        };
        let availability = (|| {
            let excluding_association = project_id
                .as_deref()
                .map(|project_id| {
                    repository
                        .project(project_id)
                        .map(|project| project.association.map(|association| association.id))
                })
                .transpose()?
                .flatten();
            repository.ensure_directory_available(&identity, excluding_association.as_deref())
        })();
        drop(repository_guard);
        if let Err(error) = availability {
            return self.build_snapshot(Some(map_storage_error(&error)));
        }

        let display_name = directory_display_name(&identity.selected_path);
        if let Ok(mut pending) = self.pending.lock() {
            *pending = Some(PendingAttachment {
                kind,
                project_id,
                display_name,
                identity,
            });
        } else {
            return ProjectWorkspaceSnapshot::unavailable(
                ProjectDiagnosticCode::MetadataUnavailable,
            );
        }
        self.status()
    }

    fn metadata_action<F>(&self, project_id: &str, action: F) -> ProjectWorkspaceSnapshot
    where
        F: FnOnce(&mut ProjectRepository, &str) -> Result<(), StorageError>,
    {
        if !valid_id(project_id) {
            return self.build_snapshot(Some(ProjectDiagnosticCode::ProjectNotFound));
        }
        if self.execution_active(project_id) {
            return self.build_snapshot(Some(ProjectDiagnosticCode::ProjectBusy));
        }
        let result =
            self.repository.lock().ok().and_then(|mut repository| {
                repository.as_mut().map(|repo| action(repo, project_id))
            });
        match result {
            Some(Ok(())) => self.status(),
            Some(Err(error)) => self.build_snapshot(Some(map_storage_error(&error))),
            None => {
                ProjectWorkspaceSnapshot::unavailable(ProjectDiagnosticCode::MetadataUnavailable)
            }
        }
    }

    fn execution_active(&self, project_id: &str) -> bool {
        let execution_active = self
            .active_executions
            .lock()
            .map(|active| active.contains(project_id))
            .unwrap_or(true);
        execution_active
            || self
                .active_terminals
                .lock()
                .map(|active| active.contains_key(project_id))
                .unwrap_or(true)
    }

    fn build_snapshot(
        &self,
        diagnostic_code: Option<ProjectDiagnosticCode>,
    ) -> ProjectWorkspaceSnapshot {
        let projects = match self.repository.lock() {
            Ok(repository) => match repository.as_ref() {
                Some(repository) => match repository.list_projects() {
                    Ok(projects) => projects,
                    Err(_) => {
                        return ProjectWorkspaceSnapshot::unavailable(
                            ProjectDiagnosticCode::MetadataUnavailable,
                        );
                    }
                },
                None => {
                    return ProjectWorkspaceSnapshot::unavailable(
                        ProjectDiagnosticCode::MetadataUnavailable,
                    );
                }
            },
            Err(_) => {
                return ProjectWorkspaceSnapshot::unavailable(
                    ProjectDiagnosticCode::MetadataUnavailable,
                );
            }
        };
        let projects: Vec<_> = projects.into_iter().map(project_summary).collect();
        let pending_attachment = self
            .pending
            .lock()
            .ok()
            .and_then(|pending| pending.as_ref().map(pending_preview));
        ProjectWorkspaceSnapshot {
            schema_version: PROJECT_SCHEMA_VERSION,
            state: if projects.is_empty() {
                ProjectWorkspaceState::Empty
            } else {
                ProjectWorkspaceState::Ready
            },
            projects,
            pending_attachment,
            diagnostic_code,
        }
    }
}

fn project_summary(project: StoredProject) -> ProjectSummary {
    ProjectSummary {
        id: project.id,
        display_name: project.display_name,
        archived: project.archived,
        directory: project.association.map(directory_summary),
    }
}

fn worktree_record(
    relation: StoredWorktreeRelation,
    project: StoredProject,
) -> ProjectWorktreeRecord {
    debug_assert_eq!(relation.source_project_id.len(), 36);
    ProjectWorktreeRecord {
        project_id: project.id,
        display_name: project.display_name,
        selected_path: project
            .association
            .map(|association| PathBuf::from(association.selected_path)),
        ownership: relation.ownership,
        branch_name: relation.branch_name,
        archived: project.archived,
    }
}

fn directory_summary(association: StoredAssociation) -> DirectorySummary {
    let selected_path = PathBuf::from(&association.selected_path);
    let stored_resolved_path = PathBuf::from(&association.resolved_path);
    match inspect_directory(&selected_path) {
        Ok(identity) if same_stored_identity(&association, &identity) => {
            let git = identity.git_summary();
            DirectorySummary {
                association_id: association.id,
                display_path: identity.selected_display_path,
                resolved_display_path: Some(identity.resolved_display_path),
                state: identity.accessibility,
                expected_access: association.expected_access,
                is_primary: true,
                git,
                has_agents_guidance: identity.has_agents_guidance,
                has_codex_config: identity.has_codex_config,
            }
        }
        Ok(identity) => {
            let git = identity.git_summary();
            DirectorySummary {
                association_id: association.id,
                display_path: display_path(&selected_path),
                resolved_display_path: Some(identity.resolved_display_path),
                state: DirectoryAccessibilityState::IdentityChanged,
                expected_access: association.expected_access,
                is_primary: true,
                git,
                has_agents_guidance: identity.has_agents_guidance,
                has_codex_config: identity.has_codex_config,
            }
        }
        Err(error) => {
            let state = preflight_failure_state(&association, error);
            DirectorySummary {
                association_id: association.id,
                display_path: display_path(&selected_path),
                resolved_display_path: Some(display_path(&stored_resolved_path)),
                state,
                expected_access: association.expected_access,
                is_primary: true,
                git: GitSummary {
                    is_repository: association.git_common_dir.is_some(),
                    is_linked_worktree: association.git_is_linked_worktree,
                },
                has_agents_guidance: association.has_agents_guidance,
                has_codex_config: association.has_codex_config,
            }
        }
    }
}

fn pending_preview(pending: &PendingAttachment) -> PendingAttachmentPreview {
    PendingAttachmentPreview {
        operation: pending.kind,
        project_id: pending.project_id.clone(),
        display_name: pending.display_name.clone(),
        selected_display_path: pending.identity.selected_display_path.clone(),
        resolved_display_path: pending.identity.resolved_display_path.clone(),
        state: pending.identity.accessibility,
        git: pending.identity.git_summary(),
        has_agents_guidance: pending.identity.has_agents_guidance,
        has_codex_config: pending.identity.has_codex_config,
    }
}

fn same_identity(expected: &DirectoryIdentity, current: &DirectoryIdentity) -> bool {
    expected.resolved_path == current.resolved_path
        && expected.device_id == current.device_id
        && expected.inode == current.inode
        && expected.mount_id == current.mount_id
        && expected.filesystem_type == current.filesystem_type
        && expected.git == current.git
        && expected.accessibility == current.accessibility
        && expected.has_agents_guidance == current.has_agents_guidance
        && expected.has_codex_config == current.has_codex_config
}

fn same_stored_identity(stored: &StoredAssociation, current: &DirectoryIdentity) -> bool {
    stored.resolved_path == current.resolved_path.to_string_lossy()
        && stored.device_id == Some(current.device_id)
        && stored.inode == Some(current.inode)
        && stored.mount_id == current.mount_id
        && stored.filesystem_type == current.filesystem_type
        && stored.git_common_dir.as_deref()
            == current.git.as_ref().and_then(|git| git.common_dir.to_str())
        && stored.git_worktree_root.as_deref()
            == current
                .git
                .as_ref()
                .and_then(|git| git.worktree_root.to_str())
        && stored.git_is_linked_worktree
            == current
                .git
                .as_ref()
                .is_some_and(|git| git.is_linked_worktree)
}

fn preflight_failure_state(
    stored: &StoredAssociation,
    failure: DirectoryInspectionError,
) -> DirectoryAccessibilityState {
    if failure.state == DirectoryAccessibilityState::MissingOrMoved {
        disconnected_state(stored.filesystem_type.as_deref())
    } else {
        failure.state
    }
}

fn directory_display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty() && !name.chars().any(char::is_control))
        .map(|name| name.chars().take(120).collect())
        .unwrap_or_else(|| "Local project".to_owned())
}

fn git_status_diff_summary_details(
    workspace: &GitWorkspaceSnapshot,
) -> Option<LocalReviewGitStatusDiffSummaryDetails> {
    let state = match workspace.state {
        GitWorkspaceState::Clean => LocalReviewEvidenceWorkspaceState::Clean,
        GitWorkspaceState::Ready => LocalReviewEvidenceWorkspaceState::Ready,
        GitWorkspaceState::Unavailable => return None,
    };
    if workspace.project_id.is_none() || workspace.diagnostic_code.is_some() {
        return None;
    }
    let mut details = LocalReviewGitStatusDiffSummaryDetails {
        workspace_state: state,
        dirty: !workspace.changes.is_empty(),
        staged_count: 0,
        modified_count: 0,
        added_count: 0,
        deleted_count: 0,
        renamed_count: 0,
        untracked_count: 0,
        conflicted_count: 0,
        changed_file_count: u32::try_from(workspace.changes.len()).ok()?,
        additions: 0,
        deletions: 0,
        diff_available: false,
        diff_truncated: workspace.truncated,
    };
    for change in &workspace.changes {
        details.staged_count += u32::from(change.staged.is_some());
        details.conflicted_count += u32::from(change.conflict);
        for kind in [change.staged, change.worktree].into_iter().flatten() {
            match kind {
                GitChangeKind::Modified | GitChangeKind::TypeChanged => details.modified_count += 1,
                GitChangeKind::Added => details.added_count += 1,
                GitChangeKind::Deleted => details.deleted_count += 1,
                GitChangeKind::Renamed => details.renamed_count += 1,
                GitChangeKind::Untracked => details.untracked_count += 1,
                GitChangeKind::Copied | GitChangeKind::Unmerged => {}
            }
        }
    }
    Some(details)
}

fn valid_id(value: &str) -> bool {
    value.len() == 36
        && value == value.to_ascii_lowercase()
        && Uuid::parse_str(value).is_ok_and(|id| id.get_version_num() == 7)
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn task_catalog_unavailable() -> TaskCatalogSnapshot {
    TaskCatalogSnapshot {
        schema_version: TASK_RECORD_SCHEMA_VERSION,
        state: TaskCatalogState::Unavailable,
        tasks: Vec::new(),
        selected_task: None,
        plans: Vec::new(),
        task_count: 0,
        payload_bytes: 0,
        warning: false,
        diagnostic_code: Some(TaskDiagnosticCode::MetadataUnavailable),
    }
}

fn local_review_unavailable(diagnostic_code: LocalReviewDiagnosticCode) -> LocalReviewSnapshot {
    LocalReviewSnapshot {
        schema_version: LOCAL_REVIEW_SCHEMA_VERSION,
        collections: Vec::new(),
        selected_collection: None,
        items: Vec::new(),
        comparisons: Vec::new(),
        collection_count: 0,
        payload_bytes: 0,
        warning: false,
        package_manifest_summary_available: false,
        git_status_diff_summary_available: false,
        activity_presentation_available: false,
        approval_presentation_available: false,
        diagnostic_code: Some(diagnostic_code),
    }
}

fn local_review_text_preview_unavailable(
    request: &LocalReviewTextPreviewRequest,
) -> LocalReviewTextPreview {
    LocalReviewTextPreview {
        schema_version: 1,
        collection_id: request.collection_id.clone(),
        item_id: request.item_id.clone(),
        title: None,
        text_format: None,
        byte_size: None,
        sha256: None,
        created_at_ms: None,
        state: types::LocalReviewItemState::Unavailable,
        text: None,
        projected_byte_size: 0,
        projected_line_count: 0,
        projected_code_point_count: 0,
        truncated: false,
        diagnostic_code: Some(LocalReviewDiagnosticCode::MetadataUnavailable),
    }
}

fn promotion_class(format: types::LocalReviewTextFormat) -> GeneratedArtifactClass {
    match format {
        types::LocalReviewTextFormat::Plain => GeneratedArtifactClass::Text,
        types::LocalReviewTextFormat::Markdown => GeneratedArtifactClass::Markdown,
        types::LocalReviewTextFormat::Json => GeneratedArtifactClass::Json,
        types::LocalReviewTextFormat::Csv => GeneratedArtifactClass::Csv,
        types::LocalReviewTextFormat::Python => GeneratedArtifactClass::Python,
    }
}

fn local_review_format(class: GeneratedArtifactClass) -> types::LocalReviewTextFormat {
    match class {
        GeneratedArtifactClass::Text => types::LocalReviewTextFormat::Plain,
        GeneratedArtifactClass::Markdown => types::LocalReviewTextFormat::Markdown,
        GeneratedArtifactClass::Json => types::LocalReviewTextFormat::Json,
        GeneratedArtifactClass::Csv => types::LocalReviewTextFormat::Csv,
        GeneratedArtifactClass::Python => types::LocalReviewTextFormat::Python,
    }
}

fn promotion_class_name(class: GeneratedArtifactClass) -> &'static str {
    match class {
        GeneratedArtifactClass::Text => "text",
        GeneratedArtifactClass::Markdown => "markdown",
        GeneratedArtifactClass::Json => "json",
        GeneratedArtifactClass::Csv => "csv",
        GeneratedArtifactClass::Python => "python",
    }
}

fn expire_promotion_reservations(reservations: &mut VecDeque<LocalReviewPromotionReservation>) {
    reservations.retain(|reservation| {
        reservation.created.elapsed() < LOCAL_REVIEW_PROMOTION_RESERVATION_TTL
    });
}

fn map_task_storage_error(error: &StorageError) -> TaskDiagnosticCode {
    match error {
        StorageError::TaskCapacity | StorageError::PlanCapacity => {
            TaskDiagnosticCode::CapacityReached
        }
        StorageError::TaskNotFound => TaskDiagnosticCode::TaskNotFound,
        StorageError::TaskArchived => TaskDiagnosticCode::TaskArchived,
        StorageError::PlanNotFound => TaskDiagnosticCode::PlanNotFound,
        StorageError::InvalidStatusTransition => TaskDiagnosticCode::InvalidStatusTransition,
        StorageError::DuplicateId => TaskDiagnosticCode::DuplicateId,
        StorageError::InvalidStoredValue => TaskDiagnosticCode::InvalidStoredValue,
        StorageError::DuplicateDirectory
        | StorageError::ProjectNotFound
        | StorageError::FutureSchema
        | StorageError::Filesystem
        | StorageError::Sqlite(_) => TaskDiagnosticCode::MetadataUnavailable,
    }
}

fn map_storage_error(error: &StorageError) -> ProjectDiagnosticCode {
    match error {
        StorageError::DuplicateDirectory => ProjectDiagnosticCode::DuplicateDirectory,
        StorageError::ProjectNotFound => ProjectDiagnosticCode::ProjectNotFound,
        StorageError::InvalidStoredValue
        | StorageError::FutureSchema
        | StorageError::Filesystem
        | StorageError::Sqlite(_) => ProjectDiagnosticCode::MetadataUnavailable,
        StorageError::TaskCapacity
        | StorageError::PlanCapacity
        | StorageError::TaskArchived
        | StorageError::PlanNotFound
        | StorageError::InvalidStatusTransition
        | StorageError::DuplicateId => ProjectDiagnosticCode::MetadataUnavailable,
        StorageError::TaskNotFound => ProjectDiagnosticCode::ProjectNotFound,
    }
}

fn map_project_execution_storage_error(error: StorageError) -> ProjectExecutionError {
    match error {
        StorageError::ProjectNotFound => ProjectExecutionError::ProjectNotFound,
        StorageError::DuplicateDirectory
        | StorageError::InvalidStoredValue
        | StorageError::FutureSchema
        | StorageError::Filesystem
        | StorageError::Sqlite(_)
        | StorageError::TaskCapacity
        | StorageError::PlanCapacity
        | StorageError::TaskNotFound
        | StorageError::TaskArchived
        | StorageError::PlanNotFound
        | StorageError::InvalidStatusTransition
        | StorageError::DuplicateId => ProjectExecutionError::MetadataUnavailable,
    }
}

fn unavailable_preflight(
    project_id: String,
    diagnostic_code: ProjectDiagnosticCode,
) -> ProjectPreflightSnapshot {
    ProjectPreflightSnapshot {
        schema_version: PROJECT_SCHEMA_VERSION,
        project_id,
        cwd_ready: false,
        display_path: None,
        state: DirectoryAccessibilityState::VerificationUnknown,
        diagnostic_code: Some(diagnostic_code),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        os::unix::fs::{symlink, PermissionsExt},
        sync::Arc,
        thread,
    };

    use uuid::Uuid;

    use super::{
        types::{
            DirectoryAccessibilityState, LocalReviewEvidenceSource,
            LocalReviewM48ArtifactCopyRequest,
            LocalReviewM48GeneratedArtifactMetadataEvidenceRequest,
            LocalReviewManualEvidenceCreateRequest, LocalReviewManualEvidenceCreateResult,
            LocalReviewPromotionPrepareRequest, LocalReviewPromotionReservationRequest,
            LocalReviewTextFormat, ProjectDiagnosticCode, ProjectWorkspaceState,
        },
        ProjectExecutionError, ProjectService, LOCAL_REVIEW_PROMOTION_RESERVATION_TTL,
    };
    use crate::advisor_generated_artifact::{
        AdvisorGeneratedArtifactService, GeneratedArtifactClass, GeneratedArtifactCreateRequest,
        GeneratedArtifactSourceKind,
    };
    use std::time::Instant;

    fn temporary_directory(label: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("quireforge-{label}-{}", Uuid::now_v7()));
        fs::create_dir_all(&path).expect("temporary directory must be created");
        path
    }

    #[test]
    fn serialized_empty_workspace_matches_the_shared_frontend_fixture() {
        let service = ProjectService::in_memory();
        let actual =
            serde_json::to_value(service.status()).expect("workspace snapshot must serialize");
        let expected: serde_json::Value =
            serde_json::from_str(include_str!("../../../fixtures/project-workspace.json"))
                .expect("shared workspace fixture must parse");

        assert_eq!(actual, expected);
    }

    #[test]
    fn local_review_promotion_prepare_is_digest_task_plan_and_class_bound() {
        let service = ProjectService::in_memory();
        let artifacts = AdvisorGeneratedArtifactService::default();
        let (collection, item, updated_at_ms) = {
            let mut repository = service.repository.lock().expect("repository lock");
            let repository = repository.as_mut().expect("repository");
            let task = repository.create_task().expect("task");
            let plan = repository
                .task_catalog(Some(&task), false, None)
                .expect("catalog")
                .1
                .expect("task")
                .selected_plan_id;
            let collection = repository
                .create_local_review_collection(&task, Some(&plan), "Promotion")
                .expect("collection");
            let updated = repository
                .local_review_snapshot(Some(&collection))
                .expect("snapshot")
                .1
                .expect("collection")
                .updated_at_ms;
            let item = repository
                .create_local_review_text_item(
                    &collection,
                    updated,
                    "Text",
                    LocalReviewTextFormat::Plain,
                    "promotion text",
                )
                .expect("item");
            let updated = repository
                .local_review_snapshot(Some(&collection))
                .expect("snapshot")
                .1
                .expect("collection")
                .updated_at_ms;
            (collection, item, updated)
        };
        let candidate = service
            .prepare_local_review_promotion(
                LocalReviewPromotionPrepareRequest {
                    collection_id: collection.clone(),
                    item_id: item.clone(),
                    expected_collection_updated_at_ms: updated_at_ms,
                },
                &artifacts,
            )
            .expect("prepare");
        assert_eq!(candidate.destination_class, "text");
        assert!(
            Uuid::parse_str(&candidate.reservation_id)
                .expect("uuid")
                .get_version_num()
                == 7
        );
        assert!(artifacts.snapshot().artifacts.is_empty());
        let manifest = service
            .confirm_local_review_promotion(
                LocalReviewPromotionReservationRequest {
                    reservation_id: candidate.reservation_id.clone(),
                },
                &artifacts,
            )
            .expect("confirm");
        assert_eq!(manifest.class, GeneratedArtifactClass::Text);
        assert_eq!(
            manifest.source_kind,
            GeneratedArtifactSourceKind::ExplicitReviewPromotion
        );
        assert_eq!(manifest.sha256, candidate.sha256);
        assert!(service
            .confirm_local_review_promotion(
                LocalReviewPromotionReservationRequest {
                    reservation_id: candidate.reservation_id
                },
                &artifacts
            )
            .is_err());
        let snapshot = service.local_review(super::types::LocalReviewListRequest {
            selected_collection_id: Some(collection),
        });
        assert!(snapshot
            .items
            .iter()
            .any(|candidate| candidate.item_id == item));
    }

    #[test]
    fn local_review_m48_artifact_copy_is_digest_bound_and_non_mutating() {
        let service = ProjectService::in_memory();
        let artifacts = AdvisorGeneratedArtifactService::default();
        let (collection, updated_at_ms) = {
            let mut repository = service.repository.lock().expect("repository lock");
            let repository = repository.as_mut().expect("repository");
            let task = repository.create_task().expect("task");
            let plan = repository
                .task_catalog(Some(&task), false, None)
                .expect("catalog")
                .1
                .expect("task")
                .selected_plan_id;
            let collection = repository
                .create_local_review_collection(&task, Some(&plan), "Copied artifacts")
                .expect("collection");
            let updated_at_ms = repository
                .local_review_snapshot(Some(&collection))
                .expect("snapshot")
                .1
                .expect("collection")
                .updated_at_ms;
            (collection, updated_at_ms)
        };
        let manifest = artifacts
            .create(GeneratedArtifactCreateRequest {
                class: GeneratedArtifactClass::Markdown,
                source_kind: GeneratedArtifactSourceKind::VisibleFencedBlock,
                display_label: "Live artifact".to_owned(),
                suggested_filename: "live.md".to_owned(),
                content: "line one\r\nline two".to_owned(),
            })
            .expect("artifact");
        let source_before = artifacts.snapshot();
        let copied = service.create_local_review_m48_artifact_copy(
            LocalReviewM48ArtifactCopyRequest {
                collection_id: collection.clone(),
                expected_collection_updated_at_ms: updated_at_ms,
                artifact_id: manifest.artifact_id.clone(),
                manifest_sha256: manifest.sha256.clone(),
            },
            &artifacts,
        );
        assert_eq!(copied.diagnostic_code, None);
        let item = copied.items.first().expect("copied item");
        assert_eq!(item.class, super::types::LocalReviewItemClass::Text);
        assert_eq!(
            item.source_kind,
            super::types::LocalReviewSourceKind::M48ArtifactCopy
        );
        assert_eq!(item.text_format, Some(LocalReviewTextFormat::Markdown));
        assert_eq!(item.sha256, manifest.sha256);
        assert_eq!(item.byte_size, "line one\nline two".len() as u64);
        assert_eq!(
            Uuid::parse_str(&item.item_id)
                .expect("item UUID")
                .get_version_num(),
            7
        );
        assert_eq!(artifacts.snapshot(), source_before);
        assert!(serde_json::to_string(item)
            .expect("item serializes")
            .contains("m48-artifact-copy"));
        let stale = service.create_local_review_m48_artifact_copy(
            LocalReviewM48ArtifactCopyRequest {
                collection_id: collection,
                expected_collection_updated_at_ms: updated_at_ms,
                artifact_id: manifest.artifact_id,
                manifest_sha256: "0".repeat(64),
            },
            &artifacts,
        );
        assert_eq!(
            stale.diagnostic_code,
            Some(super::types::LocalReviewDiagnosticCode::InvalidRequest)
        );
        assert_eq!(stale.items.len(), 1);
    }

    #[test]
    fn local_review_m48_metadata_evidence_is_manifest_bound_and_content_free() {
        let service = ProjectService::in_memory();
        let artifacts = AdvisorGeneratedArtifactService::default();
        let (collection, updated_at_ms) = {
            let mut repository = service.repository.lock().expect("repository lock");
            let repository = repository.as_mut().expect("repository");
            let task = repository.create_task().expect("task");
            let plan = repository
                .task_catalog(Some(&task), false, None)
                .expect("catalog")
                .1
                .expect("task")
                .selected_plan_id;
            let collection = repository
                .create_local_review_collection(&task, Some(&plan), "Metadata")
                .expect("collection");
            let updated = repository
                .local_review_snapshot(Some(&collection))
                .expect("snapshot")
                .1
                .expect("collection")
                .updated_at_ms;
            (collection, updated)
        };
        let manifest = artifacts
            .create(GeneratedArtifactCreateRequest {
                class: GeneratedArtifactClass::Markdown,
                source_kind: GeneratedArtifactSourceKind::VisibleFencedBlock,
                display_label: "Safe artifact".to_owned(),
                suggested_filename: "secret.md".to_owned(),
                content: "generated content must not be copied".to_owned(),
            })
            .expect("artifact");
        let source_before = artifacts.snapshot();
        let result = service.create_local_review_m48_generated_artifact_metadata_evidence(
            LocalReviewM48GeneratedArtifactMetadataEvidenceRequest {
                collection_id: collection.clone(),
                expected_collection_updated_at_ms: updated_at_ms,
                artifact_id: manifest.artifact_id.clone(),
                manifest_sha256: manifest.sha256.clone(),
            },
            &artifacts,
        );
        let (created_item_id, snapshot) = match result {
            LocalReviewManualEvidenceCreateResult::Created {
                created_item_id,
                source,
                snapshot,
            } => {
                assert_eq!(
                    source,
                    LocalReviewEvidenceSource::M48GeneratedArtifactMetadata
                );
                (created_item_id, snapshot)
            }
            LocalReviewManualEvidenceCreateResult::Failed { .. } => panic!("metadata evidence"),
        };
        let item = snapshot
            .items
            .iter()
            .find(|item| item.item_id == created_item_id)
            .expect("created item");
        assert_eq!(
            item.evidence_source,
            Some(LocalReviewEvidenceSource::M48GeneratedArtifactMetadata)
        );
        assert!(
            Uuid::parse_str(&item.item_id)
                .expect("uuid")
                .get_version_num()
                == 7
        );
        let preview = service
            .local_review_m48_generated_artifact_metadata_evidence_preview(
                item.item_id.clone(),
                item.sha256.clone(),
            )
            .expect("stored preview");
        assert_eq!(preview.details.manifest_sha256, manifest.sha256);
        assert_eq!(
            preview.details.artifact_kind,
            super::types::LocalReviewEvidenceArtifactKind::Markdown
        );
        assert_eq!(preview.details.format, LocalReviewTextFormat::Markdown);
        let stored = serde_json::to_string(&preview).expect("preview JSON");
        assert!(!stored.contains("generated content"));
        assert!(!stored.contains("secret.md"));
        assert_eq!(artifacts.snapshot(), source_before);

        let stale = service.create_local_review_m48_generated_artifact_metadata_evidence(
            LocalReviewM48GeneratedArtifactMetadataEvidenceRequest {
                collection_id: collection,
                expected_collection_updated_at_ms: updated_at_ms,
                artifact_id: manifest.artifact_id,
                manifest_sha256: manifest.sha256,
            },
            &artifacts,
        );
        assert!(matches!(
            stale,
            LocalReviewManualEvidenceCreateResult::Failed { .. }
        ));
    }

    #[test]
    fn local_review_manual_evidence_result_identifies_only_the_authoritative_created_item() {
        let service = ProjectService::in_memory();
        let (collection, updated_at_ms) = {
            let mut repository = service.repository.lock().expect("repository lock");
            let repository = repository.as_mut().expect("repository");
            let task = repository.create_task().expect("task");
            let collection = repository
                .create_local_review_collection(&task, None, "Evidence")
                .expect("collection");
            let updated_at_ms = repository
                .local_review_snapshot(Some(&collection))
                .expect("snapshot")
                .1
                .expect("collection")
                .updated_at_ms;
            (collection, updated_at_ms)
        };
        let result =
            service.create_local_review_manual_evidence(LocalReviewManualEvidenceCreateRequest {
                collection_id: collection.clone(),
                expected_collection_updated_at_ms: updated_at_ms,
                title: "Validation".to_owned(),
                summary: "passed".to_owned(),
            });
        let (created_item_id, snapshot) = match result {
            LocalReviewManualEvidenceCreateResult::Created {
                created_item_id,
                source,
                snapshot,
            } => {
                assert_eq!(source, LocalReviewEvidenceSource::ManualValidationSummary);
                (created_item_id, snapshot)
            }
            LocalReviewManualEvidenceCreateResult::Failed { .. } => {
                panic!("creation must identify success")
            }
        };
        assert!(snapshot
            .items
            .iter()
            .any(|item| item.item_id == created_item_id
                && item.evidence_source
                    == Some(LocalReviewEvidenceSource::ManualValidationSummary)));
        let failed =
            service.create_local_review_manual_evidence(LocalReviewManualEvidenceCreateRequest {
                collection_id: collection.clone(),
                expected_collection_updated_at_ms: updated_at_ms,
                title: "Again".to_owned(),
                summary: "failed".to_owned(),
            });
        assert!(matches!(
            failed,
            LocalReviewManualEvidenceCreateResult::Failed { .. }
        ));
    }

    #[test]
    fn local_review_m48_artifact_copy_rejects_stale_lifecycle_and_claims_atomically() {
        let service = ProjectService::in_memory();
        let artifacts = AdvisorGeneratedArtifactService::default();
        let (task, collection, updated_at_ms) = {
            let mut repository = service.repository.lock().expect("repository lock");
            let repository = repository.as_mut().expect("repository");
            let task = repository.create_task().expect("task");
            let collection = repository
                .create_local_review_collection(&task, None, "Copied artifacts")
                .expect("collection");
            let updated_at_ms = repository
                .local_review_snapshot(Some(&collection))
                .expect("snapshot")
                .1
                .expect("collection")
                .updated_at_ms;
            (task, collection, updated_at_ms)
        };
        let manifest = artifacts
            .create(GeneratedArtifactCreateRequest {
                class: GeneratedArtifactClass::Text,
                source_kind: GeneratedArtifactSourceKind::VisibleCompletedReply,
                display_label: "Live artifact".to_owned(),
                suggested_filename: "live.txt".to_owned(),
                content: "copy text".to_owned(),
            })
            .expect("artifact");
        let request = LocalReviewM48ArtifactCopyRequest {
            collection_id: collection.clone(),
            expected_collection_updated_at_ms: updated_at_ms,
            artifact_id: manifest.artifact_id.clone(),
            manifest_sha256: manifest.sha256.clone(),
        };
        let missing = service.create_local_review_m48_artifact_copy(
            LocalReviewM48ArtifactCopyRequest {
                artifact_id: Uuid::now_v7().to_string(),
                ..request.clone()
            },
            &artifacts,
        );
        assert_eq!(
            missing.diagnostic_code,
            Some(super::types::LocalReviewDiagnosticCode::InvalidRequest)
        );
        assert!(missing.items.is_empty());
        {
            let mut repository = service.repository.lock().expect("repository lock");
            repository
                .as_mut()
                .expect("repository")
                .set_task_status(&task, super::types::TaskStatus::Completed)
                .expect("complete task");
        }
        let frozen = service.create_local_review_m48_artifact_copy(request, &artifacts);
        assert_eq!(
            frozen.diagnostic_code,
            Some(super::types::LocalReviewDiagnosticCode::InvalidRequest)
        );
        assert!(frozen.items.is_empty());
        assert_eq!(artifacts.snapshot().artifacts, vec![manifest]);
    }

    #[test]
    fn local_review_promotion_reservations_are_bounded_expiring_and_one_use() {
        let service = ProjectService::in_memory();
        let artifacts = AdvisorGeneratedArtifactService::default();
        let (collection, item, updated_at_ms) = {
            let mut repository = service.repository.lock().expect("repository lock");
            let repository = repository.as_mut().expect("repository");
            let task = repository.create_task().expect("task");
            let plan = repository
                .task_catalog(Some(&task), false, None)
                .expect("catalog")
                .1
                .expect("task")
                .selected_plan_id;
            let collection = repository
                .create_local_review_collection(&task, Some(&plan), "Promotion")
                .expect("collection");
            let updated = repository
                .local_review_snapshot(Some(&collection))
                .expect("snapshot")
                .1
                .expect("collection")
                .updated_at_ms;
            let item = repository
                .create_local_review_text_item(
                    &collection,
                    updated,
                    "Text",
                    LocalReviewTextFormat::Plain,
                    "promotion text",
                )
                .expect("item");
            let updated = repository
                .local_review_snapshot(Some(&collection))
                .expect("snapshot")
                .1
                .expect("collection")
                .updated_at_ms;
            (collection, item, updated)
        };
        let request = LocalReviewPromotionPrepareRequest {
            collection_id: collection,
            item_id: item,
            expected_collection_updated_at_ms: updated_at_ms,
        };
        let first = service
            .prepare_local_review_promotion(request.clone(), &artifacts)
            .expect("first");
        for _ in 1..16 {
            service
                .prepare_local_review_promotion(request.clone(), &artifacts)
                .expect("reservation");
        }
        assert!(service
            .prepare_local_review_promotion(request.clone(), &artifacts)
            .is_err());
        service
            .cancel_local_review_promotion(LocalReviewPromotionReservationRequest {
                reservation_id: first.reservation_id,
            })
            .expect("cancel");
        let expiring = service
            .prepare_local_review_promotion(request, &artifacts)
            .expect("replacement");
        {
            let mut reservations = service.promotion_reservations.lock().expect("reservations");
            let reservation = reservations
                .iter_mut()
                .find(|value| value.candidate.reservation_id == expiring.reservation_id)
                .expect("reservation");
            reservation.created = Instant::now() - LOCAL_REVIEW_PROMOTION_RESERVATION_TTL;
        }
        assert!(service
            .confirm_local_review_promotion(
                LocalReviewPromotionReservationRequest {
                    reservation_id: expiring.reservation_id
                },
                &artifacts
            )
            .is_err());
    }

    #[test]
    fn attaches_and_preflights_the_original_directory_in_place() {
        let directory = temporary_directory("attach");
        fs::write(directory.join("kept-in-place.txt"), "original").expect("marker must be written");
        let service = ProjectService::in_memory();

        let pending = service.prepare_attachment(directory.clone());
        assert!(pending.pending_attachment.is_some());
        let attached = service.confirm_pending();

        assert_eq!(attached.state, ProjectWorkspaceState::Ready);
        assert_eq!(attached.projects.len(), 1);
        let preflight = service.preflight(attached.projects[0].id.clone());
        assert!(preflight.cwd_ready);
        assert!(directory.join("kept-in-place.txt").is_file());
        fs::remove_dir_all(directory).expect("temporary directory must be removed");
    }

    #[test]
    fn persists_project_metadata_across_service_restarts() {
        let root = temporary_directory("persistence");
        let directory = root.join("project");
        let database = root.join("app-data/metadata.sqlite3");
        fs::create_dir(&directory).expect("project directory must be created");
        let service = ProjectService::open(&database);
        service.prepare_attachment(directory.clone());
        let attached = service.confirm_pending();
        let project_id = attached.projects[0].id.clone();
        drop(service);

        let reopened = ProjectService::open(&database);
        let status = reopened.status();
        let preflight = reopened.preflight(project_id);

        assert_eq!(status.projects.len(), 1);
        assert!(preflight.cwd_ready);
        drop(reopened);
        fs::remove_dir_all(root).expect("temporary directory must be removed");
    }

    #[test]
    fn rejects_duplicate_resolved_directories() {
        let directory = temporary_directory("duplicate");
        let alias = directory.with_extension("alias");
        symlink(&directory, &alias).expect("alias must be created");
        let service = ProjectService::in_memory();
        service.prepare_attachment(directory.clone());
        service.confirm_pending();

        let duplicate = service.prepare_attachment(alias.clone());

        assert_eq!(
            duplicate.diagnostic_code,
            Some(ProjectDiagnosticCode::DuplicateDirectory)
        );
        fs::remove_file(alias).expect("alias must be removed");
        fs::remove_dir_all(directory).expect("temporary directory must be removed");
    }

    #[test]
    fn fails_closed_when_a_symlink_changes_after_confirmation_preview() {
        let root = temporary_directory("retarget");
        let first = root.join("first");
        let second = root.join("second");
        let selected = root.join("selected");
        fs::create_dir(&first).expect("first target must exist");
        fs::create_dir(&second).expect("second target must exist");
        symlink(&first, &selected).expect("selected symlink must exist");
        let service = ProjectService::in_memory();
        service.prepare_attachment(selected.clone());
        fs::remove_file(&selected).expect("old symlink must be removed");
        symlink(&second, &selected).expect("new symlink must exist");

        let result = service.confirm_pending();

        assert_eq!(
            result.diagnostic_code,
            Some(ProjectDiagnosticCode::IdentityChanged)
        );
        assert!(result.projects.is_empty());
        fs::remove_dir_all(root).expect("temporary directory must be removed");
    }

    #[test]
    fn fails_closed_when_project_configuration_changes_after_preview() {
        let directory = temporary_directory("config-retarget");
        let service = ProjectService::in_memory();
        service.prepare_attachment(directory.clone());
        fs::create_dir(directory.join(".codex")).expect("configuration directory must be created");

        let result = service.confirm_pending();

        assert_eq!(
            result.diagnostic_code,
            Some(ProjectDiagnosticCode::IdentityChanged)
        );
        assert!(result.projects.is_empty());
        fs::remove_dir_all(directory).expect("temporary directory must be removed");
    }

    #[test]
    fn relinks_an_existing_project_without_touching_either_directory() {
        let first = temporary_directory("relink-first");
        let second = temporary_directory("relink-second");
        fs::write(first.join("first.txt"), "first").expect("first marker must be written");
        fs::write(second.join("second.txt"), "second").expect("second marker must be written");
        let service = ProjectService::in_memory();
        service.prepare_attachment(first.clone());
        let attached = service.confirm_pending();
        let project_id = attached.projects[0].id.clone();

        let pending = service.prepare_relink(project_id.clone(), second.clone());
        assert!(pending.pending_attachment.is_some());
        let relinked = service.confirm_pending();
        let preflight = service.preflight(project_id);

        assert_eq!(relinked.projects.len(), 1);
        assert!(preflight.cwd_ready);
        assert_eq!(preflight.display_path, Some(second.display().to_string()));
        assert!(first.join("first.txt").is_file());
        assert!(second.join("second.txt").is_file());
        fs::remove_dir_all(first).expect("first directory must be removed");
        fs::remove_dir_all(second).expect("second directory must be removed");
    }

    #[test]
    fn attaches_read_only_directories_but_refuses_them_as_a_working_cwd() {
        let directory = temporary_directory("read-only");
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o555))
            .expect("directory must become read-only");
        let service = ProjectService::in_memory();

        let pending = service.prepare_attachment(directory.clone());
        assert_eq!(
            pending
                .pending_attachment
                .as_ref()
                .expect("attachment must be pending")
                .state,
            DirectoryAccessibilityState::ConnectedReadOnly
        );
        let attached = service.confirm_pending();
        let preflight = service.preflight(attached.projects[0].id.clone());

        assert!(!preflight.cwd_ready);
        assert_eq!(
            preflight.state,
            DirectoryAccessibilityState::ConnectedReadOnly
        );
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o755))
            .expect("directory permissions must be restored");
        fs::remove_dir_all(directory).expect("temporary directory must be removed");
    }

    #[test]
    fn git_review_accepts_a_revalidated_read_only_repository() {
        let directory = temporary_directory("read-only-review");
        let initialized = std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&directory)
            .status()
            .expect("git must start for the project test");
        assert!(initialized.success());
        let service = ProjectService::in_memory();
        service.prepare_attachment(directory.clone());
        let attached = service.confirm_pending();
        let project_id = attached.projects[0].id.clone();
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o555))
            .expect("directory must become read-only");

        let root = service
            .review_root(&project_id)
            .expect("read-only repository must remain reviewable");
        assert_eq!(root.attached_root, directory);
        assert_eq!(root.worktree_root, root.attached_root);

        fs::set_permissions(&directory, fs::Permissions::from_mode(0o755))
            .expect("directory permissions must be restored");
        fs::remove_dir_all(directory).expect("temporary directory must be removed");
    }

    #[test]
    fn git_review_refuses_an_attached_non_repository() {
        let directory = temporary_directory("non-repository-review");
        let service = ProjectService::in_memory();
        service.prepare_attachment(directory.clone());
        let attached = service.confirm_pending();

        assert_eq!(
            service.review_root(&attached.projects[0].id),
            Err(ProjectExecutionError::NotRepository)
        );

        fs::remove_dir_all(directory).expect("temporary directory must be removed");
    }

    #[test]
    fn detach_and_archive_never_delete_source_content() {
        let directory = temporary_directory("detach");
        let marker = directory.join("source.txt");
        fs::write(&marker, "keep").expect("marker must be written");
        let service = ProjectService::in_memory();
        service.prepare_attachment(directory.clone());
        let attached = service.confirm_pending();
        let project_id = attached.projects[0].id.clone();

        service.archive(project_id.clone());
        service.detach(project_id);

        assert_eq!(
            fs::read_to_string(&marker).expect("marker must remain readable"),
            "keep"
        );
        fs::remove_dir_all(directory).expect("temporary directory must be removed");
    }

    #[test]
    fn concurrent_status_reads_are_serialized_without_state_drift() {
        let service = Arc::new(ProjectService::in_memory());
        let readers: Vec<_> = (0..8)
            .map(|_| {
                let service = Arc::clone(&service);
                thread::spawn(move || service.status())
            })
            .collect();

        for reader in readers {
            assert_eq!(
                reader.join().expect("status reader must finish").state,
                ProjectWorkspaceState::Empty
            );
        }
    }

    #[test]
    fn terminal_and_controlled_execution_reservations_fail_closed() {
        let service = ProjectService::in_memory();
        let project_id = Uuid::now_v7().to_string();

        service
            .reserve_terminal(&project_id)
            .expect("first terminal must reserve the project");
        service
            .reserve_terminal(&project_id)
            .expect("multiple app-owned terminals may share a project");
        assert_eq!(
            service.reserve_execution(&project_id),
            Err(ProjectExecutionError::ProjectBusy)
        );

        service.release_terminal(&project_id);
        assert_eq!(
            service.reserve_execution(&project_id),
            Err(ProjectExecutionError::ProjectBusy)
        );
        service.release_terminal(&project_id);
        service
            .reserve_execution(&project_id)
            .expect("controlled execution must proceed after terminal cleanup");
        assert_eq!(
            service.reserve_terminal(&project_id),
            Err(ProjectExecutionError::ProjectBusy)
        );
        service.release_execution(&project_id);
        service
            .reserve_terminal(&project_id)
            .expect("terminal must proceed after controlled execution cleanup");
        service.release_terminal(&project_id);
    }

    #[test]
    fn missing_directory_preflight_never_falls_back() {
        let directory = temporary_directory("missing");
        let service = ProjectService::in_memory();
        service.prepare_attachment(directory.clone());
        let attached = service.confirm_pending();
        let project_id = attached.projects[0].id.clone();
        fs::remove_dir_all(directory).expect("temporary directory must be removed");

        let preflight = service.preflight(project_id);

        assert!(!preflight.cwd_ready);
        assert_eq!(preflight.state, DirectoryAccessibilityState::MissingOrMoved);
    }

    #[test]
    fn rejects_malformed_ids_and_distinguishes_unavailable_metadata() {
        let service = ProjectService::in_memory();

        let malformed = service.detach("not-an-opaque-id".to_owned());
        assert_eq!(
            malformed.diagnostic_code,
            Some(ProjectDiagnosticCode::ProjectNotFound)
        );
        let malformed_preflight = service.preflight("not-an-opaque-id".to_owned());
        assert_eq!(
            malformed_preflight.diagnostic_code,
            Some(ProjectDiagnosticCode::ProjectNotFound)
        );

        let unavailable = ProjectService::unavailable().preflight(Uuid::now_v7().to_string());
        assert_eq!(
            unavailable.diagnostic_code,
            Some(ProjectDiagnosticCode::MetadataUnavailable)
        );
        assert!(!unavailable.cwd_ready);
    }
}
