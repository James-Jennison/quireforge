mod action_card;
pub mod advisor;
mod advisor_archive_attachment;
mod advisor_attachment;
mod advisor_binary_attachment;
mod advisor_document_attachment;
mod advisor_generated_artifact;
mod advisor_image_attachment;
mod attachment;
mod codex;
#[allow(dead_code)]
mod connector_foundation;
mod context_assembly;
mod contract;
mod controlled_browser_verification;
mod desktop;
mod dynamic_analysis;
mod git;
mod local_chat;
mod local_runtime;
mod mock_inference;
mod preview;
mod project;
pub mod project_state;
#[allow(dead_code)]
mod provider_capability_registry;
#[allow(dead_code)]
mod provider_interaction_protocol;
mod task_handoff;
mod terminal;
mod worktree;

pub use codex::integration;

use advisor_archive_attachment::{
    AdvisorArchiveAttachmentClaimRequest, AdvisorArchiveAttachmentService,
    AdvisorArchiveAttachmentSnapshot,
};
use advisor_attachment::{
    AdvisorTextAttachmentClaimRequest, AdvisorTextAttachmentService, AdvisorTextAttachmentSnapshot,
    AdvisorTextExportRequest,
};
use advisor_binary_attachment::{
    AdvisorBinaryAttachmentClaimRequest, AdvisorBinaryAttachmentService,
    AdvisorBinaryAttachmentSnapshot,
};
use advisor_document_attachment::{
    AdvisorDocumentAttachmentClaimRequest, AdvisorDocumentAttachmentService,
    AdvisorDocumentAttachmentSnapshot,
};
use advisor_generated_artifact::{
    save_reserved, AdvisorGeneratedArtifactService, GeneratedArtifactClaimRequest,
    GeneratedArtifactCreateRequest, GeneratedArtifactPreviewV1, GeneratedArtifactSaveReceiptV1,
    GeneratedArtifactSnapshotV1,
};
use advisor_image_attachment::{
    AdvisorImageAttachmentClaimRequest, AdvisorImageAttachmentService,
    AdvisorImageAttachmentSnapshot,
};
use attachment::{
    types::{
        ConversationAttachmentCancelRequest, ConversationAttachmentDropRequest,
        ConversationAttachmentSnapshot, ConversationAttachmentState,
    },
    ClaimedConversationAttachments, ConversationAttachmentService,
};
use codex::conversation_mode::{chat_authentication_snapshot, ChatAuthenticationSnapshot};
use codex::{
    types::CodexRuntimeSnapshot, AdvisorConversationDiagnosticCode, AdvisorConversationService,
    AdvisorConversationSnapshot, AdvisorConversationStartRequest, AuthLoginMethod,
    ChatConversationService, ChatConversationSnapshot, ChatConversationStartRequest,
    CodexAuthService, CodexAuthSnapshot, CodexRuntimeService, CodexUsageService,
    CodexUsageSnapshot, ConversationApprovalDecisionRequest, ConversationApprovalPolicy,
    ConversationContinueRequest, ConversationDiagnosticCode, ConversationRegistrySnapshot,
    ConversationSandboxMode, ConversationService, ConversationSnapshot, ConversationStartRequest,
    IntegrationCatalogService, IntegrationControlService, IntegrationMutationService,
    ModelSelectionDiagnosticCode, ModelSelectionPolicy, ModelSelectionSnapshot,
    ModelSelectionUpdateRequest, SessionLifecycleSnapshot,
};
use contract::DesktopBootstrap;
use desktop::{
    DesktopNotificationRequest, DesktopNotificationResult, DesktopNotificationService,
    DesktopNotificationStatus,
};
use dynamic_analysis::{
    DynamicAnalysisRunRequest, DynamicAnalysisService, DynamicAnalysisSnapshot,
};
use sha2::{Digest, Sha256};
use std::{
    ffi::OsString,
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

const COMPLETE_INSTALLED_HOST_VALIDATION_FLAG: &str = "--complete-installed-host-validation";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HeadlessDispatch {
    Gui,
    CompleteInstalledHostValidation,
    Rejected,
}

fn headless_dispatch(arguments: impl IntoIterator<Item = OsString>) -> HeadlessDispatch {
    let mut arguments = arguments.into_iter();
    let _program = arguments.next();
    let remaining = arguments.collect::<Vec<_>>();
    match remaining.as_slice() {
        [] => HeadlessDispatch::Gui,
        [flag] if flag == COMPLETE_INSTALLED_HOST_VALIDATION_FLAG => {
            HeadlessDispatch::CompleteInstalledHostValidation
        }
        _ if remaining
            .iter()
            .any(|argument| argument == COMPLETE_INSTALLED_HOST_VALIDATION_FLAG) =>
        {
            HeadlessDispatch::Rejected
        }
        _ => HeadlessDispatch::Gui,
    }
}

fn complete_installed_host_validation_at(
    data_directory: Option<PathBuf>,
) -> project::InstalledHostHeadlessStatus {
    let Some(data_directory) = data_directory else {
        return project::InstalledHostHeadlessStatus::Unavailable;
    };
    ProjectService::open(
        &data_directory
            .join("io.github.codeframe78.QuireForge")
            .join("metadata.sqlite3"),
    )
    .complete_installed_host_validation()
}

/// Handles the single supported headless invocation before any Tauri state is
/// constructed. `true` means main must exit without initializing the GUI.
pub fn run_complete_installed_host_validation_from_env() -> bool {
    if controlled_browser_verification::run_fixture_helper_from_env() {
        return true;
    }
    match headless_dispatch(std::env::args_os()) {
        HeadlessDispatch::Gui => false,
        HeadlessDispatch::Rejected => {
            println!("failed");
            true
        }
        HeadlessDispatch::CompleteInstalledHostValidation => {
            let status = complete_installed_host_validation_at(dirs::data_dir());
            println!(
                "{}",
                match status {
                    project::InstalledHostHeadlessStatus::Created => "created",
                    project::InstalledHostHeadlessStatus::Existing => "existing",
                    project::InstalledHostHeadlessStatus::Failed => "failed",
                    project::InstalledHostHeadlessStatus::Unavailable => "unavailable",
                }
            );
            true
        }
    }
}

#[cfg(test)]
mod headless_dispatch_tests {
    use super::{headless_dispatch, HeadlessDispatch, COMPLETE_INSTALLED_HOST_VALIDATION_FLAG};
    use std::ffi::OsString;

    fn args(values: &[&str]) -> Vec<OsString> {
        std::iter::once(OsString::from("quireforge"))
            .chain(values.iter().map(OsString::from))
            .collect()
    }

    #[test]
    fn complete_installed_host_validation_accepts_only_its_single_flag_before_gui() {
        assert_eq!(
            headless_dispatch(args(&[COMPLETE_INSTALLED_HOST_VALIDATION_FLAG])),
            HeadlessDispatch::CompleteInstalledHostValidation
        );
        assert_eq!(headless_dispatch(args(&[])), HeadlessDispatch::Gui);
        assert_eq!(
            headless_dispatch(args(&["--tauri-debug"])),
            HeadlessDispatch::Gui
        );
    }

    #[test]
    fn complete_installed_host_validation_rejects_duplicate_additional_and_conflicting_arguments() {
        for values in [
            vec![
                COMPLETE_INSTALLED_HOST_VALIDATION_FLAG,
                COMPLETE_INSTALLED_HOST_VALIDATION_FLAG,
            ],
            vec![COMPLETE_INSTALLED_HOST_VALIDATION_FLAG, "--tauri-debug"],
            vec!["--tauri-debug", COMPLETE_INSTALLED_HOST_VALIDATION_FLAG],
        ] {
            assert_eq!(headless_dispatch(args(&values)), HeadlessDispatch::Rejected);
        }
    }
}

use git::{
    repository_state::{
        ArtifactVerificationMode, RepositoryRemoteMode, RepositoryStateDiagnosticSeverity,
        RepositoryStateReadRequest, RepositoryStateReadSnapshot, RepositoryStateReader,
    },
    types::{
        GitDiffRequest, GitDiffSnapshot, GitMutationConfirmRequest, GitMutationPreviewRequest,
        GitMutationPreviewSnapshot, GitMutationResultSnapshot, GitOpenFileRequest,
        GitRecoveryRequest, GitWorkspaceSnapshot,
    },
    GitService,
};
use preview::{
    types::{FilePreviewHandoffRequest, FilePreviewSnapshot},
    FilePreviewService,
};
use project::{
    types::{
        ArtifactReferenceConfirmRequest, ArtifactReferenceDeleteConfirmRequest,
        ArtifactReferenceDeletePrepareRequest, ArtifactReferencePreparation,
        ArtifactReferencePrepareRequest, ArtifactReferenceProjectRequest,
        DurableSourceArtifactPrepareRequest, DurableSourceConfirmRequest,
        DurableSourceDeleteConfirmRequest, DurableSourceFilePrepareRequest,
        DurableSourceManualPrepareRequest, DurableSourcePreparation, DurableSourceProjectRequest,
        DurableSourceReadRequest, DurableSourceSnapshot, KnowledgeLedgerSnapshot,
        KnowledgeRecordBindingRequest, KnowledgeRecordCreateRequest, KnowledgeRecordProjectRequest,
        LocalReviewAnnotationCreateRequest, LocalReviewAnnotationEditRequest,
        LocalReviewAnnotationMutationRequest, LocalReviewCollectionCreateRequest,
        LocalReviewCollectionMutationRequest, LocalReviewComparisonCreateRequest,
        LocalReviewComparisonDiscardRequest, LocalReviewComparisonReadRequest,
        LocalReviewImagePickOutcome, LocalReviewImagePickRequest, LocalReviewImagePreview,
        LocalReviewImagePreviewRequest, LocalReviewItemDiscardRequest, LocalReviewListRequest,
        LocalReviewM48ArtifactCopyRequest, LocalReviewManualEvidenceCreateRequest,
        LocalReviewManualEvidenceCreateResult, LocalReviewManualEvidencePreview,
        LocalReviewPromotionPrepareRequest, LocalReviewPromotionReservationRequest,
        LocalReviewSnapshot, LocalReviewTextItemCreateRequest, LocalReviewTextPreview,
        LocalReviewTextPreviewRequest, PlanCreateRequest, PlanEditRequest, PlanIdRequest,
        ProjectPreflightSnapshot, ProjectWorkspaceSnapshot, TaskCatalogContextCreateRequest,
        TaskCatalogCreateRequest, TaskCatalogListRequest, TaskCatalogSnapshot, TaskIdRequest,
        TaskStatusRequest, TaskTitleRequest,
    },
    FictionalConnectorOperationRecord, ProjectService,
};
use task_handoff::{
    TaskHandoffCreateRequest, TaskHandoffDirection, TaskHandoffReceiptRequest, TaskHandoffService,
    TaskHandoffSnapshot,
};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;
use terminal::{
    types::{
        TerminalCloseRequest, TerminalPollRequest, TerminalRegistrySnapshot, TerminalResizeRequest,
        TerminalSnapshot, TerminalStartRequest, TerminalWriteRequest,
    },
    TerminalService,
};
use worktree::{
    types::{
        WorktreeCancelRequest, WorktreeConfirmRequest, WorktreeCreatePreviewRequest,
        WorktreePreviewSnapshot, WorktreeRecoverPreviewRequest, WorktreeRemovePreviewRequest,
        WorktreeResultSnapshot, WorktreeWorkspaceSnapshot,
    },
    WorktreeService,
};

#[tauri::command]
async fn task_handoff_status(
    service: tauri::State<'_, TaskHandoffService>,
) -> Result<TaskHandoffSnapshot, ()> {
    Ok(service.status().await)
}

#[tauri::command]
async fn task_handoff_prepare_advisor_brief(
    request: TaskHandoffCreateRequest,
    service: tauri::State<'_, TaskHandoffService>,
) -> Result<TaskHandoffSnapshot, ()> {
    Ok(service.prepare_advisor_brief(request).await)
}

#[tauri::command]
async fn task_handoff_prepare_completion_receipt(
    request: TaskHandoffReceiptRequest,
    service: tauri::State<'_, TaskHandoffService>,
) -> Result<TaskHandoffSnapshot, ()> {
    Ok(service.prepare_completion_receipt(request).await)
}

#[tauri::command]
async fn task_handoff_accept(
    direction: TaskHandoffDirection,
    service: tauri::State<'_, TaskHandoffService>,
) -> Result<TaskHandoffSnapshot, ()> {
    Ok(service.accept(direction).await)
}

#[tauri::command]
async fn task_handoff_cancel(
    service: tauri::State<'_, TaskHandoffService>,
) -> Result<TaskHandoffSnapshot, ()> {
    Ok(service.cancel().await)
}

#[tauri::command]
fn desktop_bootstrap() -> DesktopBootstrap {
    DesktopBootstrap::current()
}

/// M39 is deliberately separate from Advisor, Approval/Dispatch, project
/// execution, and terminals. This command only reports the transient client
/// state; it never starts a VM or reaches the worker.
#[tauri::command]
fn dynamic_analysis_status(
    service: tauri::State<'_, DynamicAnalysisService>,
) -> DynamicAnalysisSnapshot {
    service.snapshot()
}

#[tauri::command]
async fn dynamic_analysis_pick(
    app: tauri::AppHandle,
    service: tauri::State<'_, DynamicAnalysisService>,
) -> Result<DynamicAnalysisSnapshot, ()> {
    let selection = app
        .dialog()
        .file()
        .set_title("Select one static ELF64 sample for isolated analysis")
        .blocking_pick_file();
    Ok(match selection {
        Some(file) => match file.into_path() {
            Ok(path) => service.stage_path(path),
            Err(_) => DynamicAnalysisSnapshot {
                schema_version: 1,
                state: dynamic_analysis::DynamicAnalysisState::Empty,
                manifest: None,
                result: None,
                diagnostic_code: None,
            },
        },
        None => service.snapshot(),
    })
}

#[tauri::command]
fn dynamic_analysis_clear(
    service: tauri::State<'_, DynamicAnalysisService>,
) -> DynamicAnalysisSnapshot {
    service.clear()
}

#[tauri::command]
fn dynamic_analysis_run(
    request: DynamicAnalysisRunRequest,
    service: tauri::State<'_, DynamicAnalysisService>,
) -> DynamicAnalysisSnapshot {
    service.run(request)
}

#[tauri::command]
async fn codex_runtime_probe(
    service: tauri::State<'_, CodexRuntimeService>,
) -> Result<CodexRuntimeSnapshot, ()> {
    Ok(service.snapshot().await)
}

#[tauri::command]
async fn integration_catalog_read(
    service: tauri::State<'_, IntegrationCatalogService>,
) -> Result<integration::IntegrationCatalogSnapshot, ()> {
    Ok(service.snapshot().await)
}

#[tauri::command]
async fn integration_catalog_refresh(
    service: tauri::State<'_, IntegrationCatalogService>,
) -> Result<integration::IntegrationCatalogSnapshot, ()> {
    Ok(service.refresh().await)
}

#[tauri::command]
async fn integration_control_preview(
    request: integration::IntegrationControlPreviewRequest,
    service: tauri::State<'_, IntegrationControlService>,
    catalog: tauri::State<'_, IntegrationCatalogService>,
) -> Result<integration::IntegrationControlPreviewSnapshot, ()> {
    let snapshot = catalog.refresh().await;
    Ok(service.preview(request, &snapshot).await)
}

#[tauri::command]
async fn integration_control_confirm(
    request: integration::IntegrationControlConfirmationRequest,
    service: tauri::State<'_, IntegrationControlService>,
    catalog: tauri::State<'_, IntegrationCatalogService>,
) -> Result<integration::IntegrationControlResultSnapshot, ()> {
    let snapshot = catalog.refresh().await;
    let result = service.confirm(request, &snapshot).await;
    if result.catalog_refresh_required {
        let _ = catalog.refresh().await;
    }
    Ok(result)
}

#[tauri::command]
async fn integration_control_open_browser(
    request: integration::IntegrationControlActionRequest,
    app: tauri::AppHandle,
    service: tauri::State<'_, IntegrationControlService>,
) -> Result<integration::IntegrationControlResultSnapshot, ()> {
    let (url, result) = service.claim_handoff(&request).await.map_err(|_| ())?;
    if app.opener().open_url(url, None::<&str>).is_err() {
        service.restore_handoff(&request).await;
        return Err(());
    }
    Ok(result)
}

#[tauri::command]
async fn integration_control_status(
    request: integration::IntegrationControlActionRequest,
    service: tauri::State<'_, IntegrationControlService>,
    catalog: tauri::State<'_, IntegrationCatalogService>,
) -> Result<integration::IntegrationControlResultSnapshot, ()> {
    let result = service.status(request).await;
    if result.catalog_refresh_required {
        let _ = catalog.refresh().await;
    }
    Ok(result)
}

#[tauri::command]
async fn integration_mutation_preview(
    request: integration::IntegrationMutationPreviewRequest,
    service: tauri::State<'_, IntegrationMutationService>,
    catalog: tauri::State<'_, IntegrationCatalogService>,
) -> Result<integration::IntegrationMutationPreviewSnapshot, ()> {
    let snapshot = catalog.refresh().await;
    Ok(service.preview(request, &snapshot).await)
}

#[tauri::command]
async fn integration_mutation_confirm(
    request: integration::IntegrationMutationConfirmRequest,
    service: tauri::State<'_, IntegrationMutationService>,
    catalog: tauri::State<'_, IntegrationCatalogService>,
) -> Result<integration::IntegrationMutationResultSnapshot, ()> {
    let snapshot = catalog.refresh().await;
    let result = service.confirm(request, &snapshot).await;
    if result.state == integration::IntegrationMutationResultState::Applied {
        let _ = catalog.refresh().await;
    }
    Ok(result)
}

#[tauri::command]
async fn codex_auth_status(
    service: tauri::State<'_, CodexAuthService>,
) -> Result<CodexAuthSnapshot, ()> {
    Ok(service.status().await)
}

#[tauri::command]
async fn codex_auth_refresh(
    service: tauri::State<'_, CodexAuthService>,
) -> Result<CodexAuthSnapshot, ()> {
    Ok(service.refresh().await)
}

#[tauri::command]
async fn chat_authentication_status(
    service: tauri::State<'_, CodexAuthService>,
) -> Result<ChatAuthenticationSnapshot, ()> {
    Ok(chat_authentication_snapshot(&service.status().await))
}

#[tauri::command]
async fn chat_conversation_status(
    service: tauri::State<'_, ChatConversationService>,
) -> Result<ChatConversationSnapshot, ()> {
    Ok(service.status().await)
}

#[tauri::command]
async fn chat_conversation_start(
    request: ChatConversationStartRequest,
    service: tauri::State<'_, ChatConversationService>,
    authentication: tauri::State<'_, CodexAuthService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<ChatConversationSnapshot, ()> {
    Ok(service
        .start(request, &authentication.status().await, &projects)
        .await)
}

#[tauri::command]
async fn chat_conversation_poll(
    conversation_id: String,
    service: tauri::State<'_, ChatConversationService>,
) -> Result<ChatConversationSnapshot, ()> {
    Ok(service.poll(conversation_id).await)
}

#[tauri::command]
async fn chat_conversation_interrupt(
    conversation_id: String,
    service: tauri::State<'_, ChatConversationService>,
) -> Result<ChatConversationSnapshot, ()> {
    Ok(service.interrupt(conversation_id).await)
}

#[tauri::command]
async fn advisor_conversation_status(
    service: tauri::State<'_, AdvisorConversationService>,
) -> Result<AdvisorConversationSnapshot, ()> {
    Ok(service.status().await)
}

#[tauri::command]
// Tauri injects each narrowly typed service state separately; combining them
// would obscure the text/image/document one-use authority boundaries.
#[allow(clippy::too_many_arguments)]
async fn advisor_conversation_start(
    request: AdvisorConversationStartRequest,
    service: tauri::State<'_, AdvisorConversationService>,
    authentication: tauri::State<'_, CodexAuthService>,
    projects: tauri::State<'_, ProjectService>,
    reader: tauri::State<'_, RepositoryStateReader>,
    attachments: tauri::State<'_, AdvisorTextAttachmentService>,
    image_attachments: tauri::State<'_, AdvisorImageAttachmentService>,
    document_attachments: tauri::State<'_, AdvisorDocumentAttachmentService>,
    archive_attachments: tauri::State<'_, AdvisorArchiveAttachmentService>,
    binary_attachments: tauri::State<'_, AdvisorBinaryAttachmentService>,
) -> Result<AdvisorConversationSnapshot, ()> {
    if !request.is_valid() {
        return Ok(AdvisorConversationSnapshot::unavailable(
            AdvisorConversationDiagnosticCode::InvalidRequest,
        ));
    }
    macro_rules! requested_attachment_bytes {
        ($id:expr, $hash:expr, $snapshot:expr) => {{
            match ($id, $hash) {
                (None, None) => Ok(0_u64),
                (Some(attachment_id), Some(manifest_sha256)) => $snapshot
                    .attachment
                    .filter(|attachment| {
                        attachment.attachment_id == *attachment_id
                            && attachment.sha256 == *manifest_sha256
                    })
                    .map(|attachment| attachment.byte_size)
                    .ok_or(()),
                _ => Err(()),
            }
        }};
    }
    let source_bytes = [
        requested_attachment_bytes!(
            request.attachment_id.as_ref(),
            request.attachment_manifest_sha256.as_ref(),
            attachments.snapshot()
        ),
        requested_attachment_bytes!(
            request.image_attachment_id.as_ref(),
            request.image_attachment_manifest_sha256.as_ref(),
            image_attachments.snapshot()
        ),
        requested_attachment_bytes!(
            request.document_attachment_id.as_ref(),
            request.document_attachment_manifest_sha256.as_ref(),
            document_attachments.snapshot()
        ),
        requested_attachment_bytes!(
            request.archive_attachment_id.as_ref(),
            request.archive_attachment_manifest_sha256.as_ref(),
            archive_attachments.snapshot()
        ),
        requested_attachment_bytes!(
            request.binary_attachment_id.as_ref(),
            request.binary_attachment_manifest_sha256.as_ref(),
            binary_attachments.snapshot()
        ),
    ]
    .into_iter()
    .try_fold(0_u64, |total, bytes| {
        bytes.and_then(|bytes| total.checked_add(bytes).ok_or(()))
    });
    let Ok(source_bytes) = source_bytes else {
        return Ok(AdvisorConversationSnapshot::unavailable(
            AdvisorConversationDiagnosticCode::AttachmentUnavailable,
        ));
    };
    if source_bytes > 40 * 1024 * 1024 {
        return Ok(AdvisorConversationSnapshot::unavailable(
            AdvisorConversationDiagnosticCode::AttachmentUnavailable,
        ));
    }
    let selected_project_state = if let Some(project_id) = request.project_id.as_ref() {
        let snapshot = reader
            .read(
                RepositoryStateReadRequest {
                    project_id: project_id.clone(),
                    remote_mode: RepositoryRemoteMode::LocalOnly,
                    artifact_verification: ArtifactVerificationMode::MetadataOnly,
                },
                &projects,
            )
            .await;
        if snapshot
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == RepositoryStateDiagnosticSeverity::Error)
        {
            return Ok(AdvisorConversationSnapshot::unavailable(
                AdvisorConversationDiagnosticCode::ContextUnavailable,
            ));
        }
        let selected_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ())?
            .as_millis()
            .try_into()
            .map_err(|_| ())?;
        Some(
            advisor::AdvisorSelectedProjectStateSnapshot::from_repository_snapshot(
                snapshot,
                selected_at_ms,
            ),
        )
    } else {
        None
    };
    let attachment = match (
        &request.attachment_id,
        &request.attachment_manifest_sha256,
        request.attachment_confirmation,
    ) {
        (None, None, None) => None,
        (Some(attachment_id), Some(manifest_sha256), Some(confirmation)) => {
            match attachments.claim(&AdvisorTextAttachmentClaimRequest {
                attachment_id: attachment_id.clone(),
                manifest_sha256: manifest_sha256.clone(),
                confirmation,
            }) {
                Ok(attachment) => Some(attachment),
                Err(_) => {
                    return Ok(AdvisorConversationSnapshot::unavailable(
                        AdvisorConversationDiagnosticCode::AttachmentUnavailable,
                    ))
                }
            }
        }
        _ => {
            return Ok(AdvisorConversationSnapshot::unavailable(
                AdvisorConversationDiagnosticCode::InvalidRequest,
            ))
        }
    };
    let image_attachment = match (
        &request.image_attachment_id,
        &request.image_attachment_manifest_sha256,
        request.image_attachment_confirmation,
    ) {
        (None, None, None) => None,
        (Some(attachment_id), Some(manifest_sha256), Some(confirmation)) => match image_attachments
            .claim(&AdvisorImageAttachmentClaimRequest {
                attachment_id: attachment_id.clone(),
                manifest_sha256: manifest_sha256.clone(),
                confirmation,
            }) {
            Ok(attachment) => Some(attachment),
            Err(_) => {
                return Ok(AdvisorConversationSnapshot::unavailable(
                    AdvisorConversationDiagnosticCode::AttachmentUnavailable,
                ))
            }
        },
        _ => {
            return Ok(AdvisorConversationSnapshot::unavailable(
                AdvisorConversationDiagnosticCode::InvalidRequest,
            ))
        }
    };
    let document_attachment = match (
        &request.document_attachment_id,
        &request.document_attachment_manifest_sha256,
        request.document_attachment_confirmation,
    ) {
        (None, None, None) => None,
        (Some(attachment_id), Some(manifest_sha256), Some(confirmation)) => {
            match document_attachments.claim(&AdvisorDocumentAttachmentClaimRequest {
                attachment_id: attachment_id.clone(),
                manifest_sha256: manifest_sha256.clone(),
                confirmation,
            }) {
                Ok(attachment) => Some(attachment),
                Err(_) => {
                    return Ok(AdvisorConversationSnapshot::unavailable(
                        AdvisorConversationDiagnosticCode::AttachmentUnavailable,
                    ))
                }
            }
        }
        _ => {
            return Ok(AdvisorConversationSnapshot::unavailable(
                AdvisorConversationDiagnosticCode::InvalidRequest,
            ))
        }
    };
    let archive_attachment = match (
        &request.archive_attachment_id,
        &request.archive_attachment_manifest_sha256,
        request.archive_attachment_confirmation,
    ) {
        (None, None, None) => None,
        (Some(attachment_id), Some(manifest_sha256), Some(confirmation)) => {
            match archive_attachments.claim(&AdvisorArchiveAttachmentClaimRequest {
                attachment_id: attachment_id.clone(),
                manifest_sha256: manifest_sha256.clone(),
                confirmation,
            }) {
                Ok(attachment) => Some(attachment),
                Err(_) => {
                    return Ok(AdvisorConversationSnapshot::unavailable(
                        AdvisorConversationDiagnosticCode::AttachmentUnavailable,
                    ))
                }
            }
        }
        _ => {
            return Ok(AdvisorConversationSnapshot::unavailable(
                AdvisorConversationDiagnosticCode::InvalidRequest,
            ))
        }
    };
    let binary_attachment = match (
        &request.binary_attachment_id,
        &request.binary_attachment_manifest_sha256,
        request.binary_attachment_confirmation,
    ) {
        (None, None, None) => None,
        (Some(attachment_id), Some(manifest_sha256), Some(confirmation)) => {
            match binary_attachments.claim(&AdvisorBinaryAttachmentClaimRequest {
                attachment_id: attachment_id.clone(),
                manifest_sha256: manifest_sha256.clone(),
                confirmation,
            }) {
                Ok(attachment) => Some(attachment),
                Err(_) => {
                    return Ok(AdvisorConversationSnapshot::unavailable(
                        AdvisorConversationDiagnosticCode::AttachmentUnavailable,
                    ))
                }
            }
        }
        _ => {
            return Ok(AdvisorConversationSnapshot::unavailable(
                AdvisorConversationDiagnosticCode::InvalidRequest,
            ))
        }
    };
    Ok(service
        .start(
            request,
            &authentication.status().await,
            &projects,
            selected_project_state,
            attachment,
            image_attachment,
            document_attachment,
            archive_attachment,
            binary_attachment,
        )
        .await)
}

#[tauri::command]
fn advisor_text_attachment_status(
    service: tauri::State<'_, AdvisorTextAttachmentService>,
) -> AdvisorTextAttachmentSnapshot {
    service.snapshot()
}

#[tauri::command]
async fn advisor_text_attachment_pick(
    app: tauri::AppHandle,
    service: tauri::State<'_, AdvisorTextAttachmentService>,
) -> Result<AdvisorTextAttachmentSnapshot, ()> {
    let selection = app
        .dialog()
        .file()
        .set_title("Attach one text or data file to the next Advisor message")
        .add_filter("Text and data", &["txt", "md", "csv", "json", "py"])
        .blocking_pick_file();
    Ok(match selection {
        Some(file) => match file.into_path() {
            Ok(path) => service.stage_path(path),
            Err(_) => AdvisorTextAttachmentSnapshot::empty(),
        },
        None => service.snapshot(),
    })
}

#[tauri::command]
fn advisor_text_attachment_cancel(
    service: tauri::State<'_, AdvisorTextAttachmentService>,
) -> AdvisorTextAttachmentSnapshot {
    service.clear()
}

#[tauri::command]
fn advisor_image_attachment_status(
    service: tauri::State<'_, AdvisorImageAttachmentService>,
) -> AdvisorImageAttachmentSnapshot {
    service.snapshot()
}

#[tauri::command]
async fn advisor_image_attachment_pick(
    app: tauri::AppHandle,
    service: tauri::State<'_, AdvisorImageAttachmentService>,
) -> Result<AdvisorImageAttachmentSnapshot, ()> {
    let selection = app
        .dialog()
        .file()
        .set_title("Attach one PNG or JPEG image to the next Advisor message")
        .add_filter("PNG or JPEG image", &["png", "jpg", "jpeg"])
        .blocking_pick_file();
    Ok(match selection {
        Some(file) => match file.into_path() {
            Ok(path) => service.stage_path(path),
            Err(_) => AdvisorImageAttachmentSnapshot::empty(),
        },
        None => service.snapshot(),
    })
}

#[tauri::command]
fn advisor_image_attachment_cancel(
    service: tauri::State<'_, AdvisorImageAttachmentService>,
) -> AdvisorImageAttachmentSnapshot {
    service.clear()
}

#[tauri::command]
fn advisor_document_attachment_status(
    service: tauri::State<'_, AdvisorDocumentAttachmentService>,
) -> AdvisorDocumentAttachmentSnapshot {
    service.snapshot()
}
#[tauri::command]
async fn advisor_document_attachment_pick(
    app: tauri::AppHandle,
    service: tauri::State<'_, AdvisorDocumentAttachmentService>,
) -> Result<AdvisorDocumentAttachmentSnapshot, ()> {
    let selection = app
        .dialog()
        .file()
        .set_title("Attach one PDF document to the next Advisor message")
        .add_filter("PDF document", &["pdf"])
        .blocking_pick_file();
    Ok(match selection {
        Some(file) => match file.into_path() {
            Ok(path) => service.stage_path(path),
            Err(_) => AdvisorDocumentAttachmentSnapshot::empty(),
        },
        None => service.snapshot(),
    })
}
#[tauri::command]
fn advisor_document_attachment_cancel(
    service: tauri::State<'_, AdvisorDocumentAttachmentService>,
) -> AdvisorDocumentAttachmentSnapshot {
    service.clear()
}

#[tauri::command]
fn advisor_archive_attachment_status(
    service: tauri::State<'_, AdvisorArchiveAttachmentService>,
) -> AdvisorArchiveAttachmentSnapshot {
    service.snapshot()
}
#[tauri::command]
async fn advisor_archive_attachment_pick(
    app: tauri::AppHandle,
    service: tauri::State<'_, AdvisorArchiveAttachmentService>,
) -> Result<AdvisorArchiveAttachmentSnapshot, ()> {
    let selection = app
        .dialog()
        .file()
        .set_title("Attach one ZIP archive to the next Advisor message")
        .add_filter("ZIP archive", &["zip"])
        .blocking_pick_file();
    Ok(match selection {
        Some(file) => match file.into_path() {
            Ok(path) => service.stage_path(path),
            Err(_) => AdvisorArchiveAttachmentSnapshot::empty(),
        },
        None => service.snapshot(),
    })
}
#[tauri::command]
fn advisor_archive_attachment_cancel(
    service: tauri::State<'_, AdvisorArchiveAttachmentService>,
) -> AdvisorArchiveAttachmentSnapshot {
    service.clear()
}

#[tauri::command]
fn advisor_binary_attachment_status(
    service: tauri::State<'_, AdvisorBinaryAttachmentService>,
) -> AdvisorBinaryAttachmentSnapshot {
    service.snapshot()
}
#[tauri::command]
async fn advisor_binary_attachment_pick(
    app: tauri::AppHandle,
    service: tauri::State<'_, AdvisorBinaryAttachmentService>,
) -> Result<AdvisorBinaryAttachmentSnapshot, ()> {
    let selection = app
        .dialog()
        .file()
        .set_title("Attach one ELF file to the next Advisor message")
        .blocking_pick_file();
    Ok(match selection {
        Some(file) => match file.into_path() {
            Ok(path) => service.stage_path(path),
            Err(_) => AdvisorBinaryAttachmentSnapshot::empty(),
        },
        None => service.snapshot(),
    })
}
#[tauri::command]
fn advisor_binary_attachment_cancel(
    service: tauri::State<'_, AdvisorBinaryAttachmentService>,
) -> AdvisorBinaryAttachmentSnapshot {
    service.clear()
}

#[tauri::command]
async fn advisor_text_export_save(
    request: AdvisorTextExportRequest,
    app: tauri::AppHandle,
) -> Result<(), ()> {
    let extension = match request.content_type {
        advisor_attachment::AdvisorContentType::Text => "txt",
        advisor_attachment::AdvisorContentType::Markdown => "md",
        advisor_attachment::AdvisorContentType::Csv => "csv",
        advisor_attachment::AdvisorContentType::Json => "json",
        advisor_attachment::AdvisorContentType::Python => "py",
    };
    let selected = app
        .dialog()
        .file()
        .set_title("Save Advisor output")
        .add_filter("Selected output", &[extension])
        .set_file_name(&request.suggested_name)
        .blocking_save_file();
    let Some(selected) = selected else {
        return Ok(());
    };
    let path = selected.into_path().map_err(|_| ())?;
    advisor_attachment::save_export(path, &request).map_err(|_| ())
}

#[tauri::command]
fn advisor_generated_artifact_create(
    request: GeneratedArtifactCreateRequest,
    service: tauri::State<'_, AdvisorGeneratedArtifactService>,
) -> Result<advisor_generated_artifact::GeneratedArtifactManifestV1, ()> {
    service.create(request).map_err(|_| ())
}

#[tauri::command]
fn advisor_generated_artifact_snapshot(
    service: tauri::State<'_, AdvisorGeneratedArtifactService>,
) -> GeneratedArtifactSnapshotV1 {
    service.snapshot()
}

#[tauri::command]
fn advisor_generated_artifact_preview(
    request: GeneratedArtifactClaimRequest,
    service: tauri::State<'_, AdvisorGeneratedArtifactService>,
) -> Result<GeneratedArtifactPreviewV1, ()> {
    service.preview(&request).map_err(|_| ())
}

#[tauri::command]
fn advisor_generated_artifact_discard(
    request: GeneratedArtifactClaimRequest,
    service: tauri::State<'_, AdvisorGeneratedArtifactService>,
) -> Result<GeneratedArtifactSnapshotV1, ()> {
    service.discard(request).map_err(|_| ())?;
    Ok(service.snapshot())
}

#[tauri::command]
async fn advisor_generated_artifact_save(
    request: GeneratedArtifactClaimRequest,
    app: tauri::AppHandle,
    service: tauri::State<'_, AdvisorGeneratedArtifactService>,
) -> Result<Option<GeneratedArtifactSaveReceiptV1>, ()> {
    let reservation = service.reserve(&request).map_err(|_| ())?;
    let extension = reservation.class().suffix().trim_start_matches('.');
    let selected = app
        .dialog()
        .file()
        .set_title("Save generated Advisor artifact")
        .add_filter("Generated artifact", &[extension])
        .set_file_name(reservation.suggested_filename())
        .blocking_save_file();
    let Some(selected) = selected else {
        service.release(&reservation);
        return Ok(None);
    };
    let path = match selected.into_path() {
        Ok(path) => path,
        Err(_) => {
            service.release(&reservation);
            return Err(());
        }
    };
    match save_reserved(&reservation, &path) {
        Ok(filename) => service
            .consume(&reservation, filename)
            .map(Some)
            .map_err(|_| ()),
        Err(_) => {
            service.release(&reservation);
            Err(())
        }
    }
}

#[tauri::command]
async fn advisor_conversation_poll(
    conversation_id: String,
    service: tauri::State<'_, AdvisorConversationService>,
) -> Result<AdvisorConversationSnapshot, ()> {
    Ok(service.poll(conversation_id).await)
}

#[tauri::command]
async fn advisor_conversation_interrupt(
    conversation_id: String,
    service: tauri::State<'_, AdvisorConversationService>,
) -> Result<AdvisorConversationSnapshot, ()> {
    Ok(service.interrupt(conversation_id).await)
}

const ADVISOR_APPROVAL_TTL: Duration = Duration::from_secs(15 * 60);

/// Creates a digest-only Phase A approval draft. This command intentionally
/// has no Codex, terminal, Git, project-read, or execution-service dependency.
#[tauri::command]
fn advisor_draft_create(
    request: advisor::AdvisorDraftCreateRequest,
    projects: tauri::State<'_, ProjectService>,
) -> Result<advisor::AdvisorApprovalSnapshot, ()> {
    if !valid_advisor_draft_request(&request) {
        return Err(());
    }
    let now: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ())?
        .as_millis()
        .try_into()
        .map_err(|_| ())?;
    let expires_at_ms = now
        .checked_add(
            ADVISOR_APPROVAL_TTL
                .as_millis()
                .try_into()
                .map_err(|_| ())?,
        )
        .ok_or(())?;
    let context_manifest =
        serde_json::to_string(&request.selected_project_state).map_err(|_| ())?;
    let capability_manifest =
        serde_json::to_string(&request.declared_capabilities).map_err(|_| ())?;
    let proposal = advisor::AdvisorDispatchProposal {
        id: Uuid::now_v7().to_string(),
        advisor_conversation_id: request.advisor_conversation_id,
        target_project_id: request.target_project_id,
        prompt_sha256: sha256(&request.prompt),
        context_manifest_sha256: sha256(&context_manifest),
        capability_manifest_sha256: sha256(&capability_manifest),
        state: advisor::AdvisorDispatchState::Draft,
        requires_explicit_approval: true,
        requested_model: Some(request.requested_model),
        requested_reasoning_effort: Some(request.requested_reasoning_effort),
        created_at_ms: now,
        updated_at_ms: now,
        decided_at_ms: None,
        expires_at_ms,
        execution_dispatch_state: None,
        execution_conversation_id: None,
        provenance: advisor::AdvisorProvenance {
            trust: advisor::AdvisorTrust::Reported,
            source: advisor::AdvisorProvenanceSource::UserSelection,
            source_ref: Some("advisor-phase-a-draft".to_owned()),
            source_commit: None,
            observed_at_ms: Some(now),
            note: Some("Approval record only; dispatch is unavailable.".to_owned()),
        },
    };
    projects
        .create_advisor_dispatch_proposal(&proposal)
        .map_err(|_| ())
}

/// Records an explicit Phase A decision for an unexpired digest-only draft.
/// It cannot invoke the Codex execution boundary.
#[tauri::command]
fn advisor_draft_decide(
    request: advisor::AdvisorApprovalDecisionRequest,
    projects: tauri::State<'_, ProjectService>,
) -> Result<advisor::AdvisorApprovalSnapshot, ()> {
    if !valid_uuid_v7(&request.proposal_id)
        || matches!(request.decision, advisor::AdvisorDispatchState::Draft)
        || !valid_advisor_draft_request(&request.binding)
    {
        return Err(());
    }
    let proposal = projects
        .advisor_dispatch_proposal(&request.proposal_id)
        .map_err(|_| ())?;
    if proposal.state != advisor::AdvisorDispatchState::Draft
        || proposal.advisor_conversation_id != request.binding.advisor_conversation_id
        || proposal.target_project_id != request.binding.target_project_id
        || proposal.prompt_sha256 != sha256(&request.binding.prompt)
        || proposal.context_manifest_sha256
            != sha256(
                &serde_json::to_string(&request.binding.selected_project_state).map_err(|_| ())?,
            )
        || proposal.capability_manifest_sha256
            != sha256(
                &serde_json::to_string(&request.binding.declared_capabilities).map_err(|_| ())?,
            )
        || proposal.requested_model.as_deref() != Some(request.binding.requested_model.as_str())
        || proposal.requested_reasoning_effort.as_deref()
            != Some(request.binding.requested_reasoning_effort.as_str())
        || projects
            .execution_cwd(&request.binding.target_project_id)
            .is_err()
    {
        return Err(());
    }
    projects
        .decide_advisor_dispatch_proposal(&request.proposal_id, request.decision)
        .map_err(|_| ())
}

/// Hands one explicit, revalidated approval to the existing project-bound
/// execution workspace. Advisor never receives an execution handle or output.
#[tauri::command]
async fn advisor_dispatch_once(
    request: advisor::AdvisorDispatchRequest,
    projects: tauri::State<'_, ProjectService>,
    conversations: tauri::State<'_, ConversationService>,
) -> Result<advisor::AdvisorDispatchSnapshot, ()> {
    if !valid_uuid_v7(&request.proposal_id) || !valid_advisor_draft_request(&request.binding) {
        return Err(());
    }
    let proposal = projects
        .advisor_dispatch_proposal(&request.proposal_id)
        .map_err(|_| ())?;
    if proposal.state != advisor::AdvisorDispatchState::Approved
        || proposal.execution_dispatch_state.is_some()
        || proposal.advisor_conversation_id != request.binding.advisor_conversation_id
        || proposal.target_project_id != request.binding.target_project_id
        || proposal.prompt_sha256 != sha256(&request.binding.prompt)
        || proposal.context_manifest_sha256
            != sha256(
                &serde_json::to_string(&request.binding.selected_project_state).map_err(|_| ())?,
            )
        || proposal.capability_manifest_sha256
            != sha256(
                &serde_json::to_string(&request.binding.declared_capabilities).map_err(|_| ())?,
            )
        || proposal.requested_model.as_deref() != Some(request.binding.requested_model.as_str())
        || proposal.requested_reasoning_effort.as_deref()
            != Some(request.binding.requested_reasoning_effort.as_str())
        || projects
            .execution_cwd(&request.binding.target_project_id)
            .is_err()
    {
        return Err(());
    }
    let (sandbox_mode, approval_policy) = match request.binding.declared_capabilities.as_slice() {
        [advisor::AdvisorDeclaredCapability::ReadOnly] => (
            ConversationSandboxMode::ReadOnly,
            ConversationApprovalPolicy::Untrusted,
        ),
        [advisor::AdvisorDeclaredCapability::WorkspaceWrite] => (
            ConversationSandboxMode::WorkspaceWrite,
            ConversationApprovalPolicy::OnRequest,
        ),
        _ => return Err(()),
    };
    projects
        .claim_advisor_dispatch_proposal(&request.proposal_id)
        .map_err(|_| ())?;
    let snapshot = conversations
        .start_with_mentions(
            ConversationStartRequest {
                project_id: request.binding.target_project_id,
                prompt: request.binding.prompt,
                attachment_ids: Vec::new(),
                integration_entry_ids: Vec::new(),
                model_id: request.binding.requested_model,
                reasoning_effort: request.binding.requested_reasoning_effort,
                selection_policy: ModelSelectionPolicy::default(),
                sandbox_mode,
                approval_policy,
            },
            &projects,
            Vec::new(),
            Vec::new(),
        )
        .await;
    let conversation_id = snapshot.conversation_id.clone();
    projects
        .finish_advisor_dispatch_proposal(&request.proposal_id, conversation_id.as_deref())
        .map_err(|_| ())?;
    Ok(advisor::AdvisorDispatchSnapshot {
        proposal_id: request.proposal_id,
        state: if conversation_id.is_some() {
            advisor::AdvisorExecutionDispatchState::Started
        } else {
            advisor::AdvisorExecutionDispatchState::FailedToStart
        },
        execution_conversation_id: conversation_id,
    })
}

#[tauri::command]
fn advisor_completion_report(
    request: advisor::AdvisorCompletionReportRequest,
    projects: tauri::State<'_, ProjectService>,
) -> Result<advisor::AdvisorCompletionReportSnapshot, ()> {
    let observed_at_ms: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ())?
        .as_millis()
        .try_into()
        .map_err(|_| ())?;
    if !valid_uuid_v7(&request.proposal_id) {
        return Err(());
    }
    let proposal = projects
        .advisor_dispatch_proposal(&request.proposal_id)
        .map_err(|_| ())?;
    let unavailable = || advisor::AdvisorCompletionReportSnapshot {
        proposal_id: request.proposal_id.clone(),
        status: advisor::AdvisorCompletionStatus::Unavailable,
        observed_at_ms,
        execution_conversation_id: None,
        diagnostic_code: Some("report-unavailable"),
    };
    let Some(conversation_id) = proposal.execution_conversation_id.clone() else {
        return Ok(unavailable());
    };
    let reference = match projects.conversation_reference(&conversation_id) {
        Ok(value) => value,
        Err(_) => return Ok(unavailable()),
    };
    if reference.project_id != proposal.target_project_id {
        return Ok(unavailable());
    }
    let status = match reference.status.as_str() {
        "running" | "stopping" => advisor::AdvisorCompletionStatus::Started,
        "completed" => advisor::AdvisorCompletionStatus::Completed,
        "failed" => advisor::AdvisorCompletionStatus::Failed,
        "interrupted" => advisor::AdvisorCompletionStatus::Cancelled,
        "blocked" => advisor::AdvisorCompletionStatus::Blocked,
        _ => advisor::AdvisorCompletionStatus::Unavailable,
    };
    Ok(advisor::AdvisorCompletionReportSnapshot {
        proposal_id: request.proposal_id,
        status,
        observed_at_ms,
        execution_conversation_id: Some(conversation_id),
        diagnostic_code: None,
    })
}

fn valid_advisor_draft_request(request: &advisor::AdvisorDraftCreateRequest) -> bool {
    valid_uuid_v7(&request.advisor_conversation_id)
        && valid_uuid_v7(&request.target_project_id)
        && request
            .selected_project_state
            .as_ref()
            .is_none_or(advisor::AdvisorSelectedProjectStateSnapshot::is_valid)
        && !request.prompt.trim().is_empty()
        && request.prompt.len() <= 64 * 1024
        && !request.prompt.contains('\0')
        && !request.declared_capabilities.is_empty()
        && request.declared_capabilities.len() <= 3
        && !request.requested_model.trim().is_empty()
        && request.requested_model.len() <= 128
        && !request.requested_model.chars().any(char::is_control)
        && !request.requested_reasoning_effort.trim().is_empty()
        && request.requested_reasoning_effort.len() <= 64
        && !request
            .requested_reasoning_effort
            .chars()
            .any(char::is_control)
}

fn valid_uuid_v7(value: &str) -> bool {
    value
        .parse::<Uuid>()
        .is_ok_and(|uuid| uuid.get_version_num() == 7)
}

fn sha256(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

#[tauri::command]
async fn codex_auth_start(
    method: AuthLoginMethod,
    service: tauri::State<'_, CodexAuthService>,
) -> Result<CodexAuthSnapshot, ()> {
    Ok(service.start_login(method).await)
}

#[tauri::command]
async fn codex_auth_cancel(
    service: tauri::State<'_, CodexAuthService>,
) -> Result<CodexAuthSnapshot, ()> {
    Ok(service.cancel_login().await)
}

#[tauri::command]
async fn codex_auth_logout(
    service: tauri::State<'_, CodexAuthService>,
) -> Result<CodexAuthSnapshot, ()> {
    Ok(service.logout().await)
}

#[tauri::command]
async fn codex_auth_open_browser(
    app: tauri::AppHandle,
    service: tauri::State<'_, CodexAuthService>,
) -> Result<(), ()> {
    let url = service.handoff_url().await.ok_or(())?;
    app.opener().open_url(url, None::<&str>).map_err(|_| ())
}

#[tauri::command]
async fn codex_usage_status(
    service: tauri::State<'_, CodexUsageService>,
) -> Result<CodexUsageSnapshot, ()> {
    Ok(service.snapshot().await)
}

#[tauri::command]
async fn codex_usage_refresh(
    service: tauri::State<'_, CodexUsageService>,
) -> Result<CodexUsageSnapshot, ()> {
    Ok(service.refresh().await)
}

#[tauri::command]
fn project_workspace_status(service: tauri::State<'_, ProjectService>) -> ProjectWorkspaceSnapshot {
    service.status()
}

#[tauri::command]
fn task_catalog_status(
    request: TaskCatalogListRequest,
    service: tauri::State<'_, ProjectService>,
) -> TaskCatalogSnapshot {
    service.task_catalog(request)
}

#[tauri::command]
fn knowledge_ledger_status(
    request: KnowledgeRecordProjectRequest,
    service: tauri::State<'_, ProjectService>,
) -> KnowledgeLedgerSnapshot {
    service.knowledge_ledger(request)
}
#[tauri::command]
fn knowledge_ledger_create(
    request: KnowledgeRecordCreateRequest,
    service: tauri::State<'_, ProjectService>,
) -> KnowledgeLedgerSnapshot {
    service.create_knowledge_record(request)
}
#[tauri::command]
fn knowledge_ledger_bind(
    request: KnowledgeRecordBindingRequest,
    service: tauri::State<'_, ProjectService>,
) -> KnowledgeLedgerSnapshot {
    service.bind_knowledge_record(request)
}

#[tauri::command]
fn task_catalog_create(
    request: TaskCatalogCreateRequest,
    service: tauri::State<'_, ProjectService>,
) -> TaskCatalogSnapshot {
    service.create_task_record(request.project_id, request.title)
}

#[tauri::command]
fn task_catalog_create_from_conversation(
    request: TaskCatalogContextCreateRequest,
    service: tauri::State<'_, ProjectService>,
) -> TaskCatalogSnapshot {
    service.create_task_record_from_conversation(request.conversation_id)
}
#[tauri::command]
fn task_catalog_rename(
    request: TaskTitleRequest,
    service: tauri::State<'_, ProjectService>,
) -> TaskCatalogSnapshot {
    service.task_action(request.task_id, |repo, id| {
        repo.rename_task(id, &request.title)
    })
}
#[tauri::command]
fn task_catalog_status_set(
    request: TaskStatusRequest,
    service: tauri::State<'_, ProjectService>,
) -> TaskCatalogSnapshot {
    service.task_action(request.task_id, |repo, id| {
        repo.set_task_status(id, request.status)
    })
}
#[tauri::command]
fn task_catalog_archive(
    request: TaskIdRequest,
    service: tauri::State<'_, ProjectService>,
) -> TaskCatalogSnapshot {
    service.task_action(request.task_id, |repo, id| repo.archive_task(id, false))
}
#[tauri::command]
fn task_catalog_restore(
    request: TaskIdRequest,
    service: tauri::State<'_, ProjectService>,
) -> TaskCatalogSnapshot {
    service.task_action(request.task_id, |repo, id| repo.archive_task(id, true))
}
#[tauri::command]
fn task_catalog_delete(
    request: TaskIdRequest,
    service: tauri::State<'_, ProjectService>,
) -> TaskCatalogSnapshot {
    service.task_action(request.task_id, |repo, id| repo.delete_task(id))
}

#[tauri::command]
fn durable_source_prepare_manual(
    request: DurableSourceManualPrepareRequest,
    service: tauri::State<'_, ProjectService>,
) -> DurableSourcePreparation {
    service.durable_source_prepare_manual(request)
}

#[tauri::command]
async fn durable_source_prepare_local_text_file(
    request: DurableSourceFilePrepareRequest,
    app: tauri::AppHandle,
    service: tauri::State<'_, ProjectService>,
) -> Result<DurableSourcePreparation, String> {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title("Select one UTF-8 text source to copy into QuireForge")
        .add_filter("Text", &["txt", "md", "csv", "json", "log", "py"])
        .pick_file(move |selection| {
            let _ = sender.send(selection.and_then(|file| file.into_path().ok()));
        });
    let selection = tokio::time::timeout(std::time::Duration::from_secs(60), receiver).await;
    Ok(match selection.ok().and_then(Result::ok).flatten() {
        Some(path) => service.durable_source_prepare_file(request, path),
        None => project::types::DurableSourcePreparation {
            schema_version: 1,
            preparation_id: String::new(),
            nonce: String::new(),
            expires_at_ms: 0,
            project_id: String::new(),
            task_id: None,
            source_class: project::types::DurableSourceClass::LocalTextFile,
            title: String::new(),
            origin_display: None,
            sha256: String::new(),
            byte_size: 0,
            line_count: 0,
            preview: String::new(),
            diagnostic_code: Some(project::types::DurableSourceDiagnosticCode::SourceUnavailable),
        },
    })
}

#[tauri::command]
fn durable_source_cancel_admission(
    request: project::types::DurableSourceCancelRequest,
    service: tauri::State<'_, ProjectService>,
) -> Result<(), project::types::DurableSourceDiagnosticCode> {
    service.durable_source_cancel(request)
}

#[tauri::command]
fn durable_source_prepare_reviewed_artifact_text(
    request: DurableSourceArtifactPrepareRequest,
    service: tauri::State<'_, ProjectService>,
    artifacts: tauri::State<'_, AdvisorGeneratedArtifactService>,
) -> DurableSourcePreparation {
    service.durable_source_prepare_artifact(request, &artifacts)
}

#[tauri::command]
fn durable_source_confirm_admission(
    request: DurableSourceConfirmRequest,
    service: tauri::State<'_, ProjectService>,
) -> Result<project::types::DurableSourceSummary, project::types::DurableSourceDiagnosticCode> {
    service.durable_source_confirm(request)
}

#[tauri::command]
fn durable_source_list_active(
    request: DurableSourceProjectRequest,
    service: tauri::State<'_, ProjectService>,
) -> DurableSourceSnapshot {
    service.durable_sources(request)
}

#[tauri::command]
fn durable_source_read_details(
    request: DurableSourceReadRequest,
    service: tauri::State<'_, ProjectService>,
) -> Option<project::types::DurableSourceSummary> {
    service.durable_source_read(request)
}

#[tauri::command]
fn durable_source_prepare_deletion(
    request: DurableSourceReadRequest,
    service: tauri::State<'_, ProjectService>,
) -> DurableSourcePreparation {
    service.durable_source_prepare_delete(request)
}

#[tauri::command]
fn durable_source_confirm_deletion(
    request: DurableSourceDeleteConfirmRequest,
    service: tauri::State<'_, ProjectService>,
) -> Result<(), project::types::DurableSourceDiagnosticCode> {
    service.durable_source_confirm_delete(request)
}

#[tauri::command]
fn artifact_reference_prepare(
    request: ArtifactReferencePrepareRequest,
    service: tauri::State<'_, ProjectService>,
    artifacts: tauri::State<'_, AdvisorGeneratedArtifactService>,
) -> ArtifactReferencePreparation {
    service.artifact_reference_prepare(request, &artifacts)
}
#[tauri::command]
fn artifact_reference_confirm(
    request: ArtifactReferenceConfirmRequest,
    service: tauri::State<'_, ProjectService>,
    artifacts: tauri::State<'_, AdvisorGeneratedArtifactService>,
) -> Result<project::types::ArtifactReferenceSummary, project::types::ArtifactReferenceDiagnosticCode>
{
    service.artifact_reference_confirm(request, &artifacts)
}
#[tauri::command]
fn artifact_reference_list(
    request: ArtifactReferenceProjectRequest,
    service: tauri::State<'_, ProjectService>,
    artifacts: tauri::State<'_, AdvisorGeneratedArtifactService>,
) -> project::types::ArtifactReferenceSnapshot {
    service.artifact_references(request, &artifacts)
}
#[tauri::command]
fn artifact_reference_prepare_deletion(
    request: ArtifactReferenceDeletePrepareRequest,
    service: tauri::State<'_, ProjectService>,
) -> ArtifactReferencePreparation {
    service.artifact_reference_prepare_delete(request)
}
#[tauri::command]
fn artifact_reference_confirm_deletion(
    request: ArtifactReferenceDeleteConfirmRequest,
    service: tauri::State<'_, ProjectService>,
) -> Result<(), project::types::ArtifactReferenceDiagnosticCode> {
    service.artifact_reference_confirm_delete(request)
}

#[tauri::command]
fn task_template_catalog(
    service: tauri::State<'_, ProjectService>,
) -> project::types::TaskTemplateCatalogSnapshot {
    service.task_template_catalog()
}
#[tauri::command]
fn task_template_inspect(
    request: project::types::TaskTemplateIdRequest,
    service: tauri::State<'_, ProjectService>,
) -> project::types::TaskTemplateInspectionSnapshot {
    service.task_template_inspect(request)
}
#[tauri::command]
fn task_template_create(
    request: project::types::TaskTemplateContentRequest,
    service: tauri::State<'_, ProjectService>,
) -> project::types::TaskTemplateInspectionSnapshot {
    service.task_template_create(request)
}
#[tauri::command]
fn task_template_edit(
    request: project::types::TaskTemplateEditRequest,
    service: tauri::State<'_, ProjectService>,
) -> project::types::TaskTemplateInspectionSnapshot {
    service.task_template_edit(request)
}
#[tauri::command]
fn task_template_duplicate(
    request: project::types::TaskTemplateMutationRequest,
    service: tauri::State<'_, ProjectService>,
) -> project::types::TaskTemplateInspectionSnapshot {
    service.task_template_duplicate(request)
}
#[tauri::command]
fn task_template_archive(
    request: project::types::TaskTemplateMutationRequest,
    service: tauri::State<'_, ProjectService>,
) -> project::types::TaskTemplateInspectionSnapshot {
    service.task_template_archive(request)
}
#[tauri::command]
fn task_template_restore(
    request: project::types::TaskTemplateMutationRequest,
    service: tauri::State<'_, ProjectService>,
) -> project::types::TaskTemplateInspectionSnapshot {
    service.task_template_restore(request)
}
#[tauri::command]
fn task_template_delete(
    request: project::types::TaskTemplateDeleteRequest,
    service: tauri::State<'_, ProjectService>,
) -> project::types::TaskTemplateApplicationOutcome {
    service.task_template_delete(request)
}
#[tauri::command]
fn task_template_preview(
    request: project::types::TaskTemplatePreviewRequest,
    service: tauri::State<'_, ProjectService>,
) -> project::types::TaskTemplatePreviewSnapshot {
    service.task_template_preview(request)
}
#[tauri::command]
fn task_template_confirm(
    request: project::types::TaskTemplateConfirmRequest,
    service: tauri::State<'_, ProjectService>,
) -> project::types::TaskTemplateApplicationOutcome {
    service.task_template_confirm(request)
}
#[tauri::command]
fn task_template_cancel(
    request: project::types::TaskTemplateCancelRequest,
    service: tauri::State<'_, ProjectService>,
) -> project::types::TaskTemplateApplicationOutcome {
    service.task_template_cancel(request)
}

#[tauri::command]
fn mock_inference_catalog(
    service: tauri::State<'_, mock_inference::MockInferenceService>,
) -> mock_inference::MockInferenceCatalog {
    service.catalog()
}

#[tauri::command]
fn connector_governance_catalog(
    service: tauri::State<'_, connector_foundation::ConnectorGovernanceService>,
) -> connector_foundation::ConnectorSnapshot {
    service.catalog()
}

#[tauri::command]
fn connector_governance_prepare(
    request: connector_foundation::ConnectorPrepareRequest,
    service: tauri::State<'_, connector_foundation::ConnectorGovernanceService>,
    projects: tauri::State<'_, ProjectService>,
) -> connector_foundation::ConnectorSnapshot {
    let Some(binding) = projects.task_mock_inference_binding(&request.task_id) else {
        return service.unavailable();
    };
    let snapshot = service.prepare(request, binding.project_id);
    if let (
        Some(project_id),
        Some(task_id),
        Some(binding_id),
        Some(operation_id),
        Some(operation),
        Some(descriptor_id),
        Some(descriptor_version),
        Some(descriptor_sha256),
        Some(scope_digest),
        Some(request_digest),
        Some(expires_at_ms),
    ) = (
        snapshot.project_id.as_deref(),
        snapshot.task_id.as_deref(),
        snapshot.binding_id.as_deref(),
        snapshot.operation_id.as_deref(),
        snapshot.operation.as_deref(),
        snapshot.descriptor_id.as_deref(),
        snapshot.descriptor_version,
        snapshot.descriptor_sha256.as_deref(),
        snapshot.scope_digest.as_deref(),
        snapshot.request_digest.as_deref(),
        snapshot.expires_at_ms,
    ) {
        let state = match snapshot.state.as_str() {
            "succeeded" => "completed",
            "prepared" => "prepared",
            _ => "rejected",
        };
        if !projects.record_fictional_connector_operation(FictionalConnectorOperationRecord {
            project_id,
            task_id,
            binding_id,
            operation_id,
            authorization_id: snapshot.authorization_id.as_deref(),
            operation_class: operation,
            state,
            descriptor_id,
            descriptor_version,
            descriptor_sha256,
            scope_digest,
            request_digest,
            expires_at_ms: expires_at_ms as i64,
        }) {
            return service.unavailable();
        }
    }
    snapshot
}

#[tauri::command]
fn connector_governance_confirm(
    request: connector_foundation::ConnectorConfirmRequest,
    service: tauri::State<'_, connector_foundation::ConnectorGovernanceService>,
    projects: tauri::State<'_, ProjectService>,
) -> connector_foundation::ConnectorSnapshot {
    let snapshot = service.confirm(request);
    if let (Some(project_id), Some(task_id), Some(operation_id)) = (
        snapshot.project_id.as_deref(),
        snapshot.task_id.as_deref(),
        snapshot.operation_id.as_deref(),
    ) {
        let state = if snapshot.state == "outcome-unknown" {
            "outcome-unknown"
        } else if snapshot.state == "succeeded" {
            "completed"
        } else {
            "rejected"
        };
        if !projects.complete_fictional_connector_operation(
            project_id,
            task_id,
            operation_id,
            state,
            &sha256(&format!("{}:{}:{}", project_id, operation_id, state)),
        ) {
            return service.unavailable();
        }
    }
    snapshot
}

#[tauri::command]
fn connector_governance_cancel(
    request: connector_foundation::ConnectorCancelRequest,
    service: tauri::State<'_, connector_foundation::ConnectorGovernanceService>,
    projects: tauri::State<'_, ProjectService>,
) -> connector_foundation::ConnectorSnapshot {
    let snapshot = service.cancel(request);
    if let (Some(project_id), Some(task_id), Some(operation_id)) = (
        snapshot.project_id.as_deref(),
        snapshot.task_id.as_deref(),
        snapshot.operation_id.as_deref(),
    ) {
        if !projects.complete_fictional_connector_operation(
            project_id,
            task_id,
            operation_id,
            "cancelled",
            &sha256(&format!("{}:{}:cancelled", project_id, operation_id)),
        ) {
            return service.unavailable();
        }
    }
    snapshot
}

#[tauri::command]
fn connector_governance_revoke(
    request: connector_foundation::ConnectorOperationRequest,
    service: tauri::State<'_, connector_foundation::ConnectorGovernanceService>,
    projects: tauri::State<'_, ProjectService>,
) -> connector_foundation::ConnectorSnapshot {
    let snapshot = service.revoke(request);
    if let (Some(project_id), Some(task_id), Some(operation_id)) = (
        snapshot.project_id.as_deref(),
        snapshot.task_id.as_deref(),
        snapshot.operation_id.as_deref(),
    ) {
        if !projects.invalidate_fictional_connector_operation(
            project_id,
            task_id,
            operation_id,
            "revoked",
            "revoked",
            &sha256(&format!("{}:{}:revoked", project_id, operation_id)),
        ) {
            return service.unavailable();
        }
    }
    snapshot
}

#[tauri::command]
fn controlled_browser_verification_status(
    service: tauri::State<
        '_,
        controlled_browser_verification::ControlledBrowserVerificationService,
    >,
) -> controlled_browser_verification::BrowserVerificationSnapshot {
    service.status()
}

#[tauri::command]
fn controlled_browser_verification_prepare(
    request: controlled_browser_verification::BrowserVerificationPrepareRequest,
    service: tauri::State<
        '_,
        controlled_browser_verification::ControlledBrowserVerificationService,
    >,
    projects: tauri::State<'_, ProjectService>,
) -> controlled_browser_verification::BrowserVerificationSnapshot {
    let project_id = request.project_id.clone();
    if let Some(task_id) = request.task_id.as_deref() {
        let Some(binding) = projects.task_mock_inference_binding(task_id) else {
            return service.status();
        };
        if binding.project_id != project_id {
            return service.status();
        }
    }
    let snapshot = service.prepare(request, project_id);
    if let (
        Some(attempt_id),
        Some(project_id),
        Some(authorization_id),
        Some(request_digest),
        Some(expires_at_ms),
    ) = (
        snapshot.attempt_id.as_deref(),
        snapshot.project_id.as_deref(),
        snapshot.authorization_id.as_deref(),
        snapshot.request_digest.as_deref(),
        snapshot.expires_at_ms,
    ) {
        if !projects.record_controlled_browser_verification(
            project::ControlledBrowserVerificationRecord {
                attempt_id,
                project_id,
                task_id: snapshot.task_id.as_deref(),
                target_digest: &sha256(snapshot.target.as_deref().unwrap_or_default()),
                request_digest,
                authorization_id,
                expires_at_ms: expires_at_ms as i64,
            },
        ) {
            return service.status();
        }
    }
    snapshot
}

#[tauri::command]
fn controlled_browser_verification_confirm(
    request: controlled_browser_verification::BrowserVerificationConfirmRequest,
    service: tauri::State<
        '_,
        controlled_browser_verification::ControlledBrowserVerificationService,
    >,
    projects: tauri::State<'_, ProjectService>,
) -> controlled_browser_verification::BrowserVerificationSnapshot {
    let snapshot = service.confirm(request);
    if let Some(attempt_id) = snapshot.attempt_id.as_deref() {
        let evidence = snapshot.evidence_digest.as_deref();
        if !projects.complete_controlled_browser_verification(attempt_id, &snapshot.state, evidence)
        {
            return service.status();
        }
    }
    snapshot
}

#[tauri::command]
fn controlled_browser_verification_cancel(
    request: controlled_browser_verification::BrowserVerificationAttemptRequest,
    service: tauri::State<
        '_,
        controlled_browser_verification::ControlledBrowserVerificationService,
    >,
    projects: tauri::State<'_, ProjectService>,
) -> controlled_browser_verification::BrowserVerificationSnapshot {
    let snapshot = service.cancel(request);
    if let Some(attempt_id) = snapshot.attempt_id.as_deref() {
        let _ =
            projects.complete_controlled_browser_verification(attempt_id, &snapshot.state, None);
    }
    snapshot
}

#[tauri::command]
fn controlled_browser_verification_revoke(
    request: controlled_browser_verification::BrowserVerificationAttemptRequest,
    service: tauri::State<
        '_,
        controlled_browser_verification::ControlledBrowserVerificationService,
    >,
    projects: tauri::State<'_, ProjectService>,
) -> controlled_browser_verification::BrowserVerificationSnapshot {
    let snapshot = service.revoke(request);
    if let Some(attempt_id) = snapshot.attempt_id.as_deref() {
        let _ =
            projects.complete_controlled_browser_verification(attempt_id, &snapshot.state, None);
    }
    snapshot
}

#[tauri::command]
fn context_assembly_status(
    service: tauri::State<'_, context_assembly::ContextAssemblyService>,
) -> context_assembly::ContextSnapshot {
    service.status()
}

#[tauri::command]
fn context_authority_ledger(
    project_id: String,
    service: tauri::State<'_, ProjectService>,
) -> project::types::ContextLedgerSnapshot {
    service.context_ledger(project_id)
}

#[tauri::command]
fn context_assembly_prepare(
    request: context_assembly::ContextPrepareRequest,
    service: tauri::State<'_, context_assembly::ContextAssemblyService>,
    projects: tauri::State<'_, ProjectService>,
) -> context_assembly::ContextSnapshot {
    let mut materials = match projects.context_durable_materials(
        &request.project_id,
        request.task_id.as_deref(),
        &request.durable_source_ids,
    ) {
        Ok(value) => value,
        Err(_) => return service.status(),
    };
    match projects.context_selected_plan_material(
        &request.project_id,
        request.task_id.as_deref(),
        request.selected_plan_id.as_deref(),
    ) {
        Ok(Some(plan)) => materials.push(plan),
        Ok(None) => {}
        Err(_) => return service.status(),
    }
    match projects.context_review_evidence_materials(
        &request.project_id,
        request.task_id.as_deref(),
        &request.review_evidence_ids,
    ) {
        Ok(items) => materials.extend(items),
        Err(_) => return service.status(),
    }
    if request.include_scope_metadata {
        match projects
            .context_scope_metadata_material(&request.project_id, request.task_id.as_deref())
        {
            Ok(material) => materials.push(material),
            Err(_) => return service.status(),
        }
    }
    let snapshot = service.prepare(request, materials);
    if let (
        Some(bundle_id),
        Some(project_id),
        Some(authorization_id),
        Some(bundle_digest),
        Some(expires_at_ms),
    ) = (
        snapshot.bundle_id.as_deref(),
        snapshot.project_id.as_deref(),
        snapshot.authorization_id.as_deref(),
        snapshot.bundle_digest.as_deref(),
        snapshot.expires_at_ms,
    ) {
        let Some(canonical_bytes) = service.canonical_bytes(bundle_id) else {
            return service.status();
        };
        if !projects.record_context_bundle(project::ContextBundleRecord {
            bundle_id,
            project_id,
            task_id: snapshot.task_id.as_deref(),
            bundle_digest,
            authorization_id,
            expires_at_ms: expires_at_ms as i64,
            items: &snapshot.items,
            canonical_bytes: &canonical_bytes,
        }) {
            return service.status();
        }
    }
    snapshot
}

#[tauri::command]
fn context_assembly_confirm(
    request: context_assembly::ContextConfirmRequest,
    service: tauri::State<'_, context_assembly::ContextAssemblyService>,
    projects: tauri::State<'_, ProjectService>,
) -> context_assembly::ContextSnapshot {
    let preflight = service.preflight_confirm(&request);
    if preflight.state == "expired" {
        if let Some(bundle_id) = preflight.bundle_id.as_deref() {
            if !projects.complete_context_bundle(bundle_id, "expired") {
                return context_assembly::ContextSnapshot::storage_failure();
            }
        }
        return preflight;
    }
    if preflight.state == "rejected" {
        return preflight;
    }
    let expected_terminal = preflight.state.clone();
    if !projects.complete_context_bundle(&request.bundle_id, &expected_terminal) {
        return context_assembly::ContextSnapshot::storage_failure();
    }
    let snapshot = service.confirm(request);
    if snapshot.bundle_id.is_some() && snapshot.state != expected_terminal {
        return context_assembly::ContextSnapshot::storage_failure();
    }
    snapshot
}

#[tauri::command]
fn context_assembly_local_runtime_availability(
    runtime: tauri::State<'_, Arc<local_runtime::LocalRuntimeService>>,
) -> local_runtime::LocalRuntimeAvailability {
    runtime.availability()
}

#[tauri::command]
async fn context_assembly_run_local_runtime(
    request: context_assembly::ContextConfirmRequest,
    assemblies: tauri::State<'_, context_assembly::ContextAssemblyService>,
    runtime: tauri::State<'_, Arc<local_runtime::LocalRuntimeService>>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<local_runtime::LocalRuntimeSnapshot, ()> {
    // Validate the exact in-memory review before consuming its durable record.
    // Otherwise a mismatched authorization could turn a valid reviewed bundle
    // into a dispatching/failed attempt without ever reaching the local model.
    let preflight = assemblies.preflight_confirm(&request);
    if preflight.state == "expired" {
        if let Some(bundle_id) = preflight.bundle_id.as_deref() {
            let _ = projects.complete_context_bundle(bundle_id, "expired");
        }
        return Ok(local_runtime::LocalRuntimeSnapshot {
            schema_version: 1,
            local_only: true,
            state: "failed".into(),
            output: None,
            diagnostic: Some("authorization-expired".into()),
            input_token_limit: 4096,
            output_token_limit: 512,
            deadline_seconds: 60,
            memory_ceiling_mib: 6144,
        });
    }
    if preflight.state == "rejected" {
        return Ok(local_runtime::LocalRuntimeSnapshot {
            schema_version: 1,
            local_only: true,
            state: "failed".into(),
            output: None,
            diagnostic: Some(
                preflight
                    .diagnostic
                    .unwrap_or_else(|| "authorization-replayed-or-mismatched".into()),
            ),
            input_token_limit: 4096,
            output_token_limit: 512,
            deadline_seconds: 60,
            memory_ceiling_mib: 6144,
        });
    }
    // The browser view preflights this content-free state too, but the native
    // command remains authoritative: an unavailable supervisor-owned model
    // must leave this exact one-use review untouched even if the command is
    // invoked without the view.
    if !runtime.availability().available {
        return Ok(local_runtime::LocalRuntimeService::unavailable_snapshot());
    }
    // Reserve the one shared CPU slot before consuming the durable reviewed
    // bundle. A concurrent attempt must leave its exact review available
    // rather than turn it into a failed one-use dispatch.
    let reservation = match runtime.reserve_available_model(&request.bundle_id) {
        Ok(reservation) => reservation,
        Err(()) => {
            return Ok(local_runtime::LocalRuntimeSnapshot {
                schema_version: 1,
                local_only: true,
                state: "failed".into(),
                output: None,
                diagnostic: Some("runtime-busy".into()),
                input_token_limit: 4096,
                output_token_limit: 512,
                deadline_seconds: 60,
                memory_ceiling_mib: 6144,
            });
        }
    };
    if !projects.start_local_runtime_context_bundle(&request.bundle_id) {
        return Ok(local_runtime::LocalRuntimeSnapshot {
            schema_version: 1,
            local_only: true,
            state: "failed".into(),
            output: None,
            diagnostic: Some("authorization-replayed-or-mismatched".into()),
            input_token_limit: 4096,
            output_token_limit: 512,
            deadline_seconds: 60,
            memory_ceiling_mib: 6144,
        });
    }
    let canonical_bytes = match assemblies.claim_for_local_runtime(&request) {
        Ok(bytes) => bytes,
        Err(snapshot) => {
            let terminal = if snapshot.state == "expired" {
                "expired"
            } else {
                "failed"
            };
            let _ = projects.complete_context_bundle(&request.bundle_id, terminal);
            return Ok(local_runtime::LocalRuntimeSnapshot {
                schema_version: 1,
                local_only: true,
                state: "failed".into(),
                output: None,
                diagnostic: snapshot.diagnostic,
                input_token_limit: 4096,
                output_token_limit: 512,
                deadline_seconds: 60,
                memory_ceiling_mib: 6144,
            });
        }
    };
    // Keep the Tauri command executor free while the approved in-process model
    // runs. This leaves the exact-bundle cancellation command responsive; the
    // shared runtime service still enforces M63's one-attempt slot.
    let snapshot = tauri::async_runtime::spawn_blocking(move || reservation.run(&canonical_bytes))
        .await
        .unwrap_or_else(|_| local_runtime::LocalRuntimeSnapshot {
            schema_version: 1,
            local_only: true,
            state: "failed".into(),
            output: None,
            diagnostic: Some("runtime-unavailable".into()),
            input_token_limit: 4096,
            output_token_limit: 512,
            deadline_seconds: 60,
            memory_ceiling_mib: 6144,
        });
    let terminal = if snapshot.state == "completed" {
        "closed"
    } else if snapshot.state == "cancelled" {
        "cancelled"
    } else {
        "failed"
    };
    if !projects.complete_context_bundle(&request.bundle_id, terminal) {
        return Ok(local_runtime::LocalRuntimeSnapshot {
            schema_version: 1,
            local_only: true,
            state: "failed".into(),
            output: None,
            diagnostic: Some("durable-audit-unavailable".into()),
            input_token_limit: 4096,
            output_token_limit: 512,
            deadline_seconds: 60,
            memory_ceiling_mib: 6144,
        });
    }
    Ok(snapshot)
}

#[tauri::command]
fn context_assembly_cancel_local_runtime(
    request: context_assembly::ContextAttemptRequest,
    runtime: tauri::State<'_, Arc<local_runtime::LocalRuntimeService>>,
) -> bool {
    runtime.request_cancel(&request.bundle_id)
}

#[tauri::command]
async fn local_chat_run(
    request: local_chat::LocalChatRequest,
    service: tauri::State<'_, Arc<local_chat::LocalChatService>>,
) -> Result<local_runtime::LocalRuntimeSnapshot, ()> {
    let service = Arc::clone(&service);
    tauri::async_runtime::spawn_blocking(move || service.run(request))
        .await
        .map_err(|_| ())
}

#[tauri::command]
fn local_chat_cancel(service: tauri::State<'_, Arc<local_chat::LocalChatService>>) -> bool {
    service.cancel()
}

#[tauri::command]
fn action_card_prepare(
    request: action_card::ActionCardPrepareRequest,
    service: tauri::State<'_, action_card::ActionCardService>,
) -> Result<action_card::ActionCardSnapshot, ()> {
    service.prepare(request)
}

#[tauri::command]
fn action_card_approve(
    request: action_card::ActionCardDecisionRequest,
    service: tauri::State<'_, action_card::ActionCardService>,
) -> Result<action_card::ActionCardSnapshot, ()> {
    service.approve(request)
}

#[tauri::command]
fn action_card_revoke(
    request: action_card::ActionCardDecisionRequest,
    service: tauri::State<'_, action_card::ActionCardService>,
) -> Result<action_card::ActionCardSnapshot, ()> {
    service.revoke(request)
}

#[tauri::command]
fn context_assembly_review(
    request: context_assembly::ContextAttemptRequest,
    service: tauri::State<'_, context_assembly::ContextAssemblyService>,
    projects: tauri::State<'_, ProjectService>,
) -> context_assembly::ContextSnapshot {
    if !projects.review_context_bundle(&request.bundle_id) {
        return service.status();
    }
    service.review(request)
}

#[tauri::command]
fn context_assembly_acknowledge_review(
    request: context_assembly::ContextAttemptRequest,
    service: tauri::State<'_, context_assembly::ContextAssemblyService>,
    projects: tauri::State<'_, ProjectService>,
) -> context_assembly::ContextSnapshot {
    if !projects.acknowledge_context_bundle_review(&request.bundle_id) {
        return context_assembly::ContextSnapshot::storage_failure();
    }
    service.acknowledge_review(request)
}

#[tauri::command]
fn context_assembly_cancel(
    request: context_assembly::ContextAttemptRequest,
    service: tauri::State<'_, context_assembly::ContextAssemblyService>,
    projects: tauri::State<'_, ProjectService>,
) -> context_assembly::ContextSnapshot {
    let prospective = service.terminal_state(&request.bundle_id, "cancelled");
    if prospective == "rejected"
        || !projects.complete_context_bundle(&request.bundle_id, prospective)
    {
        return context_assembly::ContextSnapshot::storage_failure();
    }
    let snapshot = service.cancel(request);
    if snapshot.bundle_id.is_some() && snapshot.state != prospective {
        return context_assembly::ContextSnapshot::storage_failure();
    }
    snapshot
}

#[tauri::command]
fn context_assembly_revoke(
    request: context_assembly::ContextAttemptRequest,
    service: tauri::State<'_, context_assembly::ContextAssemblyService>,
    projects: tauri::State<'_, ProjectService>,
) -> context_assembly::ContextSnapshot {
    let prospective = service.terminal_state(&request.bundle_id, "revoked");
    if prospective == "rejected"
        || !projects.complete_context_bundle(&request.bundle_id, prospective)
    {
        return context_assembly::ContextSnapshot::storage_failure();
    }
    let snapshot = service.revoke(request);
    if snapshot.bundle_id.is_some() && snapshot.state != prospective {
        return context_assembly::ContextSnapshot::storage_failure();
    }
    snapshot
}

#[tauri::command]
fn mock_inference_prepare(
    request: mock_inference::MockInferencePrepareRequest,
    service: tauri::State<'_, mock_inference::MockInferenceService>,
    projects: tauri::State<'_, ProjectService>,
) -> mock_inference::MockInferenceSnapshot {
    let Some(binding) = projects.task_mock_inference_binding(&request.task_id) else {
        return mock_inference::diagnostic(
            mock_inference::MockAttemptState::Invalidated,
            mock_inference::MockDiagnostic::TaskUnavailable,
        );
    };
    service.prepare(request, binding)
}

#[tauri::command]
fn mock_inference_authorize(
    request: mock_inference::MockInferenceAuthorizationRequest,
    service: tauri::State<'_, mock_inference::MockInferenceService>,
    projects: tauri::State<'_, ProjectService>,
) -> mock_inference::MockInferenceSnapshot {
    let Some(binding) = projects.task_mock_inference_binding(&request.task_id) else {
        return mock_inference::diagnostic(
            mock_inference::MockAttemptState::Invalidated,
            mock_inference::MockDiagnostic::TaskUnavailable,
        );
    };
    service.authorize(request, &binding)
}

#[tauri::command]
fn mock_inference_submit(
    request: mock_inference::MockInferenceAuthorizationRequest,
    service: tauri::State<'_, mock_inference::MockInferenceService>,
    projects: tauri::State<'_, ProjectService>,
) -> mock_inference::MockInferenceSnapshot {
    let Some(binding) = projects.task_mock_inference_binding(&request.task_id) else {
        return mock_inference::diagnostic(
            mock_inference::MockAttemptState::Invalidated,
            mock_inference::MockDiagnostic::TaskUnavailable,
        );
    };
    service.submit(request, &binding)
}

#[tauri::command]
fn mock_inference_cancel(
    request: mock_inference::MockInferenceAttemptRequest,
    service: tauri::State<'_, mock_inference::MockInferenceService>,
    projects: tauri::State<'_, ProjectService>,
) -> mock_inference::MockInferenceSnapshot {
    let Some(binding) = projects.task_mock_inference_binding(&request.task_id) else {
        return mock_inference::diagnostic(
            mock_inference::MockAttemptState::Invalidated,
            mock_inference::MockDiagnostic::TaskUnavailable,
        );
    };
    service.cancel(request, &binding)
}

#[tauri::command]
fn mock_inference_poll(
    request: mock_inference::MockInferenceAttemptRequest,
    service: tauri::State<'_, mock_inference::MockInferenceService>,
    projects: tauri::State<'_, ProjectService>,
) -> mock_inference::MockInferenceSnapshot {
    let Some(binding) = projects.task_mock_inference_binding(&request.task_id) else {
        return mock_inference::diagnostic(
            mock_inference::MockAttemptState::Invalidated,
            mock_inference::MockDiagnostic::TaskUnavailable,
        );
    };
    service.poll(request, &binding)
}

#[tauri::command]
fn local_review_status(
    request: LocalReviewListRequest,
    service: tauri::State<'_, ProjectService>,
) -> LocalReviewSnapshot {
    service.local_review(request)
}

#[tauri::command]
fn local_review_collection_create(
    request: LocalReviewCollectionCreateRequest,
    service: tauri::State<'_, ProjectService>,
) -> LocalReviewSnapshot {
    service.create_local_review_collection(request)
}

#[tauri::command]
fn local_review_text_item_create(
    request: LocalReviewTextItemCreateRequest,
    service: tauri::State<'_, ProjectService>,
) -> LocalReviewSnapshot {
    service.create_local_review_text_item(request)
}

#[tauri::command]
fn local_review_m48_artifact_copy(
    request: LocalReviewM48ArtifactCopyRequest,
    projects: tauri::State<'_, ProjectService>,
    artifacts: tauri::State<'_, AdvisorGeneratedArtifactService>,
) -> LocalReviewSnapshot {
    projects.create_local_review_m48_artifact_copy(request, &artifacts)
}
#[tauri::command]
fn local_review_m48_generated_artifact_metadata_evidence_create(
    request: project::types::LocalReviewM48GeneratedArtifactMetadataEvidenceRequest,
    projects: tauri::State<'_, ProjectService>,
    artifacts: tauri::State<'_, AdvisorGeneratedArtifactService>,
) -> project::types::LocalReviewManualEvidenceCreateResult {
    projects.create_local_review_m48_generated_artifact_metadata_evidence(request, &artifacts)
}
#[tauri::command]
fn local_review_safe_preview_metadata_claim(
    previews: tauri::State<'_, FilePreviewService>,
) -> Result<preview::SafePreviewMetadataClaim, ()> {
    previews.issue_safe_metadata_claim().ok_or(())
}
#[tauri::command]
fn local_review_safe_preview_metadata_evidence_create(
    request: project::types::LocalReviewSafePreviewMetadataEvidenceRequest,
    projects: tauri::State<'_, ProjectService>,
    previews: tauri::State<'_, FilePreviewService>,
) -> project::types::LocalReviewManualEvidenceCreateResult {
    let collection_id = request.collection_id.clone();
    let failed = || project::types::LocalReviewManualEvidenceCreateResult::Failed {
        snapshot: projects.local_review(project::types::LocalReviewListRequest {
            selected_collection_id: Some(collection_id.clone()),
        }),
    };
    let Some(claim) =
        previews.safe_metadata_claim(&request.preview_claim_id, &request.claim_sha256)
    else {
        return failed();
    };
    let media_type = match claim.media_type.as_str() {
        "text/plain; charset=utf-8" => project::types::LocalReviewEvidenceMediaType::TextPlain,
        "image/png" => project::types::LocalReviewEvidenceMediaType::ImagePng,
        "image/jpeg" => project::types::LocalReviewEvidenceMediaType::ImageJpeg,
        "application/pdf" => project::types::LocalReviewEvidenceMediaType::ApplicationPdf,
        _ => return failed(),
    };
    let byte_length = match u32::try_from(claim.byte_length) {
        Ok(value) => value,
        Err(_) => return failed(),
    };
    let details = project::types::LocalReviewSafePreviewMetadataDetails {
        preview_state: project::types::LocalReviewEvidencePreviewState::Ready,
        kind: match claim.kind {
            preview::types::FilePreviewKind::Text => {
                project::types::LocalReviewEvidencePreviewKind::Text
            }
            preview::types::FilePreviewKind::Image => {
                project::types::LocalReviewEvidencePreviewKind::Image
            }
            preview::types::FilePreviewKind::Pdf => {
                project::types::LocalReviewEvidencePreviewKind::Pdf
            }
        },
        rendering: match claim.rendering {
            preview::types::FilePreviewRendering::NormalizedText => {
                project::types::LocalReviewEvidencePreviewRendering::NormalizedText
            }
            preview::types::FilePreviewRendering::BoundedImage => {
                project::types::LocalReviewEvidencePreviewRendering::BoundedImage
            }
            preview::types::FilePreviewRendering::MetadataOnly => {
                project::types::LocalReviewEvidencePreviewRendering::MetadataOnly
            }
        },
        media_type,
        byte_length,
        truncated: claim.truncated,
        width_px: claim.width_px,
        height_px: claim.height_px,
    };
    let result =
        projects.create_local_review_safe_preview_metadata_evidence(request.clone(), details);
    if matches!(
        result,
        project::types::LocalReviewManualEvidenceCreateResult::Created { .. }
    ) {
        let _ =
            previews.consume_safe_metadata_claim(&request.preview_claim_id, &request.claim_sha256);
    }
    result
}
#[tauri::command]
fn local_review_package_manifest_summary_evidence_create(
    request: project::types::LocalReviewPackageManifestSummaryEvidenceRequest,
    projects: tauri::State<'_, ProjectService>,
) -> project::types::LocalReviewManualEvidenceCreateResult {
    projects.create_local_review_package_manifest_summary_evidence(request)
}
#[tauri::command]
async fn local_review_git_status_diff_summary_evidence_create(
    request: project::types::LocalReviewGitStatusDiffSummaryEvidenceRequest,
    projects: tauri::State<'_, ProjectService>,
) -> Result<project::types::LocalReviewManualEvidenceCreateResult, ()> {
    Ok(projects
        .create_local_review_git_status_diff_summary_evidence(request)
        .await)
}
#[tauri::command]
fn local_review_activity_presentation_evidence_create(
    request: project::types::LocalReviewActivityPresentationEvidenceRequest,
    projects: tauri::State<'_, ProjectService>,
) -> project::types::LocalReviewManualEvidenceCreateResult {
    projects.create_local_review_activity_presentation_evidence(request)
}
#[tauri::command]
fn local_review_approval_presentation_evidence_create(
    request: project::types::LocalReviewApprovalPresentationEvidenceRequest,
    projects: tauri::State<'_, ProjectService>,
) -> project::types::LocalReviewManualEvidenceCreateResult {
    projects.create_local_review_approval_presentation_evidence(request)
}

#[tauri::command]
fn local_review_collection_resume(
    request: LocalReviewCollectionMutationRequest,
    service: tauri::State<'_, ProjectService>,
) -> LocalReviewSnapshot {
    service.resume_local_review_collection(request)
}
#[tauri::command]
fn local_review_collection_discard(
    request: LocalReviewCollectionMutationRequest,
    service: tauri::State<'_, ProjectService>,
) -> LocalReviewSnapshot {
    service.discard_local_review_collection(request)
}
#[tauri::command]
fn local_review_item_discard(
    request: LocalReviewItemDiscardRequest,
    service: tauri::State<'_, ProjectService>,
) -> LocalReviewSnapshot {
    service.discard_local_review_item(request)
}
#[tauri::command]
async fn local_review_image_pick(
    app: tauri::AppHandle,
    request: LocalReviewImagePickRequest,
    service: tauri::State<'_, ProjectService>,
) -> Result<LocalReviewImagePickOutcome, ()> {
    let selection = app
        .dialog()
        .file()
        .set_title("Add one static PNG or JPEG local review mockup")
        .add_filter("PNG or JPEG image", &["png", "jpg", "jpeg"])
        .blocking_pick_file();
    let Some(file) = selection else {
        return Ok(LocalReviewImagePickOutcome::Canceled {
            snapshot: service.local_review(LocalReviewListRequest {
                selected_collection_id: Some(request.collection_id),
            }),
        });
    };
    let path = file.into_path().map_err(|_| ())?;
    let metadata = std::fs::symlink_metadata(&path).map_err(|_| ())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 1024 * 1024 {
        return Ok(LocalReviewImagePickOutcome::Created {
            snapshot: service.create_local_review_image_item(request, Vec::new()),
        });
    }
    let bytes = std::fs::read(path).map_err(|_| ())?;
    Ok(LocalReviewImagePickOutcome::Created {
        snapshot: service.create_local_review_image_item(request, bytes),
    })
}
#[tauri::command]
fn local_review_image_preview(
    request: LocalReviewImagePreviewRequest,
    service: tauri::State<'_, ProjectService>,
) -> Result<LocalReviewImagePreview, ()> {
    service.local_review_image_preview(request)
}
#[tauri::command]
fn local_review_text_preview(
    request: LocalReviewTextPreviewRequest,
    service: tauri::State<'_, ProjectService>,
) -> Result<LocalReviewTextPreview, ()> {
    service.local_review_text_preview(request)
}
#[tauri::command]
fn local_review_manual_evidence_create(
    request: LocalReviewManualEvidenceCreateRequest,
    service: tauri::State<'_, ProjectService>,
) -> LocalReviewManualEvidenceCreateResult {
    service.create_local_review_manual_evidence(request)
}
#[tauri::command]
fn local_review_manual_evidence_preview(
    item_id: String,
    sha256: String,
    service: tauri::State<'_, ProjectService>,
) -> Result<LocalReviewManualEvidencePreview, ()> {
    service.local_review_manual_evidence_preview(item_id, sha256)
}
#[tauri::command]
fn local_review_m48_generated_artifact_metadata_evidence_preview(
    item_id: String,
    sha256: String,
    service: tauri::State<'_, ProjectService>,
) -> Result<project::types::LocalReviewM48GeneratedArtifactMetadataEvidencePreview, ()> {
    service.local_review_m48_generated_artifact_metadata_evidence_preview(item_id, sha256)
}
#[tauri::command]
fn local_review_safe_preview_metadata_evidence_preview(
    item_id: String,
    sha256: String,
    service: tauri::State<'_, ProjectService>,
) -> Result<project::types::LocalReviewSafePreviewMetadataEvidencePreview, ()> {
    service.local_review_safe_preview_metadata_evidence_preview(item_id, sha256)
}
#[tauri::command]
fn local_review_package_manifest_summary_evidence_preview(
    item_id: String,
    sha256: String,
    service: tauri::State<'_, ProjectService>,
) -> Result<project::types::LocalReviewPackageManifestSummaryEvidencePreview, ()> {
    service.local_review_package_manifest_summary_evidence_preview(item_id, sha256)
}
#[tauri::command]
fn local_review_git_status_diff_summary_evidence_preview(
    item_id: String,
    sha256: String,
    service: tauri::State<'_, ProjectService>,
) -> Result<project::types::LocalReviewGitStatusDiffSummaryEvidencePreview, ()> {
    service.local_review_git_status_diff_summary_evidence_preview(item_id, sha256)
}
#[tauri::command]
fn local_review_activity_presentation_evidence_preview(
    item_id: String,
    sha256: String,
    service: tauri::State<'_, ProjectService>,
) -> Result<project::types::LocalReviewActivityPresentationEvidencePreview, ()> {
    service.local_review_activity_presentation_evidence_preview(item_id, sha256)
}
#[tauri::command]
fn local_review_approval_presentation_evidence_preview(
    item_id: String,
    sha256: String,
    service: tauri::State<'_, ProjectService>,
) -> Result<project::types::LocalReviewApprovalPresentationEvidencePreview, ()> {
    service.local_review_approval_presentation_evidence_preview(item_id, sha256)
}
#[tauri::command]
fn local_review_annotation_create(
    request: LocalReviewAnnotationCreateRequest,
    service: tauri::State<'_, ProjectService>,
) -> LocalReviewSnapshot {
    service.create_local_review_annotation(request)
}
#[tauri::command]
fn local_review_annotation_edit(
    request: LocalReviewAnnotationEditRequest,
    service: tauri::State<'_, ProjectService>,
) -> LocalReviewSnapshot {
    service.edit_local_review_annotation(request)
}
#[tauri::command]
fn local_review_annotation_resolve(
    request: LocalReviewAnnotationMutationRequest,
    service: tauri::State<'_, ProjectService>,
) -> LocalReviewSnapshot {
    service.resolve_local_review_annotation(request)
}
#[tauri::command]
fn local_review_annotation_reopen(
    request: LocalReviewAnnotationMutationRequest,
    service: tauri::State<'_, ProjectService>,
) -> LocalReviewSnapshot {
    service.reopen_local_review_annotation(request)
}
#[tauri::command]
fn local_review_annotation_delete(
    request: LocalReviewAnnotationMutationRequest,
    service: tauri::State<'_, ProjectService>,
) -> LocalReviewSnapshot {
    service.delete_local_review_annotation(request)
}
#[tauri::command]
fn local_review_comparison_create(
    request: LocalReviewComparisonCreateRequest,
    service: tauri::State<'_, ProjectService>,
) -> LocalReviewSnapshot {
    service.create_local_review_text_comparison(request)
}
#[tauri::command]
fn local_review_comparison_read(
    request: LocalReviewComparisonReadRequest,
    service: tauri::State<'_, ProjectService>,
) -> Result<project::types::LocalReviewLineComparison, ()> {
    service.local_review_line_comparison(request)
}
#[tauri::command]
fn local_review_comparison_discard(
    request: LocalReviewComparisonDiscardRequest,
    service: tauri::State<'_, ProjectService>,
) -> LocalReviewSnapshot {
    service.discard_local_review_text_comparison(request)
}
#[tauri::command]
fn local_review_promotion_prepare(
    request: LocalReviewPromotionPrepareRequest,
    projects: tauri::State<'_, ProjectService>,
    artifacts: tauri::State<'_, AdvisorGeneratedArtifactService>,
) -> Result<project::types::LocalReviewPromotionCandidate, ()> {
    projects.prepare_local_review_promotion(request, &artifacts)
}
#[tauri::command]
fn local_review_promotion_confirm(
    request: LocalReviewPromotionReservationRequest,
    projects: tauri::State<'_, ProjectService>,
    artifacts: tauri::State<'_, AdvisorGeneratedArtifactService>,
) -> Result<advisor_generated_artifact::GeneratedArtifactManifestV1, ()> {
    projects.confirm_local_review_promotion(request, &artifacts)
}
#[tauri::command]
fn local_review_promotion_cancel(
    request: LocalReviewPromotionReservationRequest,
    projects: tauri::State<'_, ProjectService>,
) -> Result<project::types::LocalReviewPromotionCandidate, ()> {
    projects.cancel_local_review_promotion(request)
}
#[tauri::command]
fn task_plan_create(
    request: PlanCreateRequest,
    service: tauri::State<'_, ProjectService>,
) -> TaskCatalogSnapshot {
    service.create_task_plan(request.task_id, request.copy_primary_body)
}
#[tauri::command]
fn task_plan_select(
    request: PlanIdRequest,
    service: tauri::State<'_, ProjectService>,
) -> TaskCatalogSnapshot {
    let plan = request.plan_id;
    service.task_action(request.task_id, |repo, task| repo.select_plan(task, &plan))
}
#[tauri::command]
fn task_plan_edit(
    request: PlanEditRequest,
    service: tauri::State<'_, ProjectService>,
) -> TaskCatalogSnapshot {
    let (plan, label, body) = (request.plan_id, request.label, request.body);
    service.task_action(request.task_id, |repo, task| {
        repo.edit_plan(task, &plan, &label, &body)
    })
}
#[tauri::command]
fn task_plan_delete(
    request: PlanIdRequest,
    service: tauri::State<'_, ProjectService>,
) -> TaskCatalogSnapshot {
    let plan = request.plan_id;
    service.task_action(request.task_id, |repo, task| repo.delete_plan(task, &plan))
}

/// Reads QuireForge-owned Advisor reference metadata only. It accepts no
/// caller input and cannot read a project, start a turn, or dispatch work.
#[tauri::command]
fn advisor_snapshot_read(
    service: tauri::State<'_, ProjectService>,
) -> Result<advisor::AdvisorWorkspaceSnapshot, ()> {
    service.advisor_workspace_snapshot()
}

/// Reads one explicitly selected attached project's normalized local state,
/// then returns only the Advisor-safe summary projection. The remote and
/// artifact modes are intentionally fixed; this command cannot browse paths,
/// fetch, verify packages, or persist Advisor context.
#[tauri::command]
async fn advisor_project_state_snapshot_read(
    request: advisor::AdvisorProjectStateReadRequest,
    reader: tauri::State<'_, RepositoryStateReader>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<advisor::AdvisorSelectedProjectStateSnapshot, ()> {
    if !request.is_valid() {
        return Err(());
    }
    let snapshot = reader
        .read(
            RepositoryStateReadRequest {
                project_id: request.project_id,
                remote_mode: RepositoryRemoteMode::LocalOnly,
                artifact_verification: ArtifactVerificationMode::MetadataOnly,
            },
            &projects,
        )
        .await;
    if snapshot
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == RepositoryStateDiagnosticSeverity::Error)
    {
        return Err(());
    }
    let selected_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ())?
        .as_millis()
        .try_into()
        .map_err(|_| ())?;
    Ok(
        advisor::AdvisorSelectedProjectStateSnapshot::from_repository_snapshot(
            snapshot,
            selected_at_ms,
        ),
    )
}

#[tauri::command]
async fn project_pick_directory(
    app: tauri::AppHandle,
    service: tauri::State<'_, ProjectService>,
) -> Result<ProjectWorkspaceSnapshot, ()> {
    let selection = app
        .dialog()
        .file()
        .set_title("Attach a local project")
        .blocking_pick_folder();
    Ok(match selection {
        Some(path) => match path.into_path() {
            Ok(path) => service.prepare_attachment(path),
            Err(_) => service.picker_unavailable(),
        },
        None => service.cancel_pending(),
    })
}

#[tauri::command]
async fn project_pick_relink(
    project_id: String,
    app: tauri::AppHandle,
    service: tauri::State<'_, ProjectService>,
) -> Result<ProjectWorkspaceSnapshot, ()> {
    let selection = app
        .dialog()
        .file()
        .set_title("Relink the local project")
        .blocking_pick_folder();
    Ok(match selection {
        Some(path) => match path.into_path() {
            Ok(path) => service.prepare_relink(project_id, path),
            Err(_) => service.picker_unavailable(),
        },
        None => service.cancel_pending(),
    })
}

#[tauri::command]
fn project_confirm_attachment(
    service: tauri::State<'_, ProjectService>,
) -> ProjectWorkspaceSnapshot {
    service.confirm_pending()
}

#[tauri::command]
fn project_cancel_attachment(
    service: tauri::State<'_, ProjectService>,
) -> ProjectWorkspaceSnapshot {
    service.cancel_pending()
}

#[tauri::command]
fn project_detach(
    project_id: String,
    service: tauri::State<'_, ProjectService>,
) -> ProjectWorkspaceSnapshot {
    service.detach(project_id)
}

#[tauri::command]
fn project_archive(
    project_id: String,
    service: tauri::State<'_, ProjectService>,
) -> ProjectWorkspaceSnapshot {
    service.archive(project_id)
}

#[tauri::command]
fn project_preflight(
    project_id: String,
    service: tauri::State<'_, ProjectService>,
) -> ProjectPreflightSnapshot {
    service.preflight(project_id)
}

#[tauri::command]
async fn file_preview_pick(
    project_id: String,
    app: tauri::AppHandle,
    service: tauri::State<'_, FilePreviewService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<FilePreviewSnapshot, ()> {
    if !preview::valid_project_id(&project_id) {
        return Ok(FilePreviewSnapshot::unavailable(
            None,
            preview::types::FilePreviewDiagnosticCode::InvalidRequest,
        ));
    }
    let selection = app
        .dialog()
        .file()
        .set_title("Preview a project file")
        .blocking_pick_file();
    Ok(match selection {
        Some(path) => match path.into_path() {
            Ok(path) => service.preview_selected(project_id, path, &projects),
            Err(_) => {
                service.clear_project(&project_id);
                FilePreviewSnapshot::unavailable(
                    Some(project_id),
                    preview::types::FilePreviewDiagnosticCode::PickerUnavailable,
                )
            }
        },
        None => {
            service.clear_project(&project_id);
            FilePreviewSnapshot::empty(Some(project_id))
        }
    })
}

#[tauri::command]
async fn file_preview_open(
    request: FilePreviewHandoffRequest,
    app: tauri::AppHandle,
    service: tauri::State<'_, FilePreviewService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<(), preview::types::FilePreviewDiagnosticCode> {
    let claimed = service.claim_handoff(&request)?;
    let path = match claimed.path(&projects) {
        Ok(path) => path,
        Err(error) => return Err(error),
    };
    let Some(path) = path.to_str() else {
        return Err(preview::types::FilePreviewDiagnosticCode::UnsafePath);
    };
    if app.opener().open_path(path, None::<&str>).is_err() {
        service.restore_handoff(claimed);
        return Err(preview::types::FilePreviewDiagnosticCode::OpenFailed);
    }
    Ok(())
}

#[tauri::command]
fn file_preview_cancel(
    request: FilePreviewHandoffRequest,
    service: tauri::State<'_, FilePreviewService>,
) -> bool {
    service.cancel_handoff(&request)
}

#[tauri::command]
async fn conversation_notify(
    request: DesktopNotificationRequest,
    app: tauri::AppHandle,
    conversations: tauri::State<'_, ConversationService>,
    notifications: tauri::State<'_, DesktopNotificationService>,
) -> Result<DesktopNotificationResult, ()> {
    let Some(candidate) = conversations
        .notification_candidate(&request.conversation_id)
        .await
    else {
        return Ok(DesktopNotificationResult::new(
            DesktopNotificationStatus::Ineligible,
        ));
    };
    let Some(window) = app.get_webview_window("main") else {
        return Ok(DesktopNotificationResult::new(
            DesktopNotificationStatus::Unavailable,
        ));
    };
    if window.is_focused().unwrap_or(true) {
        return Ok(DesktopNotificationResult::new(
            DesktopNotificationStatus::Foreground,
        ));
    }
    let prepared = match notifications.prepare(candidate) {
        Ok(Some(prepared)) => prepared,
        Ok(None) => {
            return Ok(DesktopNotificationResult::new(
                DesktopNotificationStatus::Duplicate,
            ));
        }
        Err(()) => {
            return Ok(DesktopNotificationResult::new(
                DesktopNotificationStatus::Unavailable,
            ));
        }
    };
    if app
        .notification()
        .builder()
        .title(prepared.title())
        .body(prepared.body())
        .show()
        .is_err()
    {
        notifications.restore(prepared);
        return Ok(DesktopNotificationResult::new(
            DesktopNotificationStatus::Unavailable,
        ));
    }
    notifications.complete(prepared);
    Ok(DesktopNotificationResult::new(
        DesktopNotificationStatus::Sent,
    ))
}

#[tauri::command]
fn conversation_attachment_status(
    project_id: String,
    service: tauri::State<'_, ConversationAttachmentService>,
    projects: tauri::State<'_, ProjectService>,
) -> ConversationAttachmentSnapshot {
    service.status(project_id, &projects)
}

#[tauri::command]
async fn conversation_attachment_pick(
    project_id: String,
    app: tauri::AppHandle,
    service: tauri::State<'_, ConversationAttachmentService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<ConversationAttachmentSnapshot, ()> {
    let status = service.status(project_id.clone(), &projects);
    if status.state == ConversationAttachmentState::Unavailable {
        return Ok(status);
    }
    let selection = app
        .dialog()
        .file()
        .set_title("Attach images to the next Codex turn")
        .add_filter("Supported images", &["png", "jpg", "jpeg"])
        .blocking_pick_files();
    let Some(selection) = selection else {
        return Ok(status);
    };
    let selected_paths = selection
        .into_iter()
        .map(|path| path.into_path())
        .collect::<Result<Vec<_>, _>>();
    Ok(match selected_paths {
        Ok(paths) => service.stage_picker_paths(project_id, paths, &projects),
        Err(_) => ConversationAttachmentSnapshot::unavailable(
            Some(project_id),
            attachment::types::ConversationAttachmentDiagnosticCode::InvalidRequest,
        ),
    })
}

#[tauri::command]
fn conversation_attachment_stage_drop(
    request: ConversationAttachmentDropRequest,
    service: tauri::State<'_, ConversationAttachmentService>,
    projects: tauri::State<'_, ProjectService>,
) -> ConversationAttachmentSnapshot {
    service.stage_drop(request, &projects)
}

#[tauri::command]
fn conversation_attachment_stage_native_drop(
    project_id: String,
    service: tauri::State<'_, ConversationAttachmentService>,
    projects: tauri::State<'_, ProjectService>,
) -> ConversationAttachmentSnapshot {
    service.stage_native_drop(project_id, &projects)
}

#[tauri::command]
fn conversation_attachment_cancel(
    request: ConversationAttachmentCancelRequest,
    service: tauri::State<'_, ConversationAttachmentService>,
) -> ConversationAttachmentSnapshot {
    service.cancel(request)
}

#[tauri::command]
async fn worktree_status(
    project_id: String,
    service: tauri::State<'_, WorktreeService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<WorktreeWorkspaceSnapshot, ()> {
    Ok(service.status(project_id, &projects).await)
}

#[tauri::command]
async fn worktree_create_preview(
    request: WorktreeCreatePreviewRequest,
    service: tauri::State<'_, WorktreeService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<WorktreePreviewSnapshot, ()> {
    Ok(service.preview_create(request, &projects).await)
}

#[tauri::command]
async fn worktree_recover_preview(
    request: WorktreeRecoverPreviewRequest,
    service: tauri::State<'_, WorktreeService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<WorktreePreviewSnapshot, ()> {
    Ok(service.preview_recover(request, &projects).await)
}

#[tauri::command]
async fn worktree_remove_preview(
    request: WorktreeRemovePreviewRequest,
    service: tauri::State<'_, WorktreeService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<WorktreePreviewSnapshot, ()> {
    Ok(service.preview_remove(request, &projects).await)
}

#[tauri::command]
async fn worktree_pick_attach(
    project_id: String,
    app: tauri::AppHandle,
    service: tauri::State<'_, WorktreeService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<WorktreePreviewSnapshot, ()> {
    let selection = app
        .dialog()
        .file()
        .set_title("Attach an existing Git worktree")
        .blocking_pick_folder();
    Ok(match selection {
        Some(path) => match path.into_path() {
            Ok(path) => service.preview_attach(project_id, path, &projects).await,
            Err(_) => service.picker_unavailable(project_id),
        },
        None => service.picker_cancelled(project_id),
    })
}

#[tauri::command]
async fn worktree_confirm(
    request: WorktreeConfirmRequest,
    service: tauri::State<'_, WorktreeService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<WorktreeResultSnapshot, ()> {
    Ok(service.confirm(request, &projects).await)
}

#[tauri::command]
fn worktree_cancel(
    request: WorktreeCancelRequest,
    service: tauri::State<'_, WorktreeService>,
) -> bool {
    service.cancel(request)
}

#[tauri::command]
async fn git_status(
    project_id: String,
    service: tauri::State<'_, GitService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<GitWorkspaceSnapshot, ()> {
    Ok(service.status(project_id, &projects).await)
}

#[tauri::command]
async fn repository_state_read(
    request: RepositoryStateReadRequest,
    service: tauri::State<'_, RepositoryStateReader>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<RepositoryStateReadSnapshot, ()> {
    Ok(service.read(request, &projects).await)
}

#[tauri::command]
async fn git_diff(
    request: GitDiffRequest,
    service: tauri::State<'_, GitService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<GitDiffSnapshot, ()> {
    Ok(service.diff(request, &projects).await)
}

#[tauri::command]
async fn git_open_file(
    request: GitOpenFileRequest,
    app: tauri::AppHandle,
    service: tauri::State<'_, GitService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<(), git::types::GitDiagnosticCode> {
    let path = service.review_file(request, &projects).await?;
    let path = path
        .to_str()
        .ok_or(git::types::GitDiagnosticCode::InvalidPath)?;
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|_| git::types::GitDiagnosticCode::DiffUnavailable)
}

#[tauri::command]
async fn git_mutation_preview(
    request: GitMutationPreviewRequest,
    service: tauri::State<'_, GitService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<GitMutationPreviewSnapshot, ()> {
    Ok(service.preview_mutation(request, &projects).await)
}

#[tauri::command]
async fn git_mutation_confirm(
    request: GitMutationConfirmRequest,
    service: tauri::State<'_, GitService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<GitMutationResultSnapshot, ()> {
    Ok(service.confirm_mutation(request, &projects).await)
}

#[tauri::command]
async fn git_mutation_recover(
    request: GitRecoveryRequest,
    service: tauri::State<'_, GitService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<GitMutationResultSnapshot, ()> {
    Ok(service.recover_mutation(request, &projects).await)
}

#[tauri::command]
async fn conversation_status(
    service: tauri::State<'_, ConversationService>,
) -> Result<ConversationSnapshot, ()> {
    Ok(service.status().await)
}

#[tauri::command]
async fn conversation_active(
    service: tauri::State<'_, ConversationService>,
) -> Result<ConversationRegistrySnapshot, ()> {
    Ok(service.active().await)
}

#[tauri::command]
async fn conversation_start(
    request: ConversationStartRequest,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
    integrations: tauri::State<'_, IntegrationControlService>,
    attachment_service: tauri::State<'_, ConversationAttachmentService>,
) -> Result<ConversationSnapshot, ()> {
    let claimed =
        match attachment_service.claim(&request.project_id, &request.attachment_ids, &projects) {
            Ok(claimed) => claimed,
            Err(_) => {
                return Ok(ConversationSnapshot::unavailable(
                    ConversationDiagnosticCode::AttachmentUnavailable,
                ));
            }
        };
    let mentions = match integrations
        .resolve_mentions(&request.integration_entry_ids)
        .await
    {
        Ok(mentions) => mentions,
        Err(_) => {
            return Ok(ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::IntegrationUnavailable,
            ))
        }
    };
    let snapshot = service
        .start_with_mentions(request, &projects, mentions, claimed.resolved())
        .await;
    if retain_claimed_attachments(&attachment_service, claimed, &snapshot).is_err() {
        return Ok(ConversationSnapshot::unavailable(
            ConversationDiagnosticCode::AttachmentUnavailable,
        ));
    }
    Ok(snapshot)
}

#[tauri::command]
async fn conversation_poll(
    conversation_id: String,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
    attachment_service: tauri::State<'_, ConversationAttachmentService>,
) -> Result<ConversationSnapshot, ()> {
    let snapshot = service.poll(conversation_id, &projects).await;
    cleanup_terminal_attachments(&attachment_service, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
async fn conversation_interrupt(
    conversation_id: String,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
    attachment_service: tauri::State<'_, ConversationAttachmentService>,
) -> Result<ConversationSnapshot, ()> {
    let snapshot = service.interrupt(conversation_id, &projects).await;
    cleanup_terminal_attachments(&attachment_service, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
async fn conversation_approval_decide(
    request: ConversationApprovalDecisionRequest,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
    attachment_service: tauri::State<'_, ConversationAttachmentService>,
) -> Result<ConversationSnapshot, ()> {
    let snapshot = service.decide_approval(request, &projects).await;
    cleanup_terminal_attachments(&attachment_service, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
async fn model_selection_update(
    request: ModelSelectionUpdateRequest,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<ModelSelectionSnapshot, ModelSelectionDiagnosticCode> {
    service.update_model_selection(request, &projects).await
}

#[tauri::command]
async fn conversation_sessions(
    request: codex::SessionListRequest,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<SessionLifecycleSnapshot, ()> {
    Ok(service.sessions(request, &projects).await)
}

#[tauri::command]
async fn conversation_resume(
    request: ConversationContinueRequest,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
    attachment_service: tauri::State<'_, ConversationAttachmentService>,
) -> Result<ConversationSnapshot, ()> {
    let project_id = match projects.conversation_reference(&request.conversation_id) {
        Ok(reference) => reference.project_id,
        Err(_) => {
            return Ok(ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::ConversationNotFound,
            ))
        }
    };
    let claimed = match attachment_service.claim(&project_id, &request.attachment_ids, &projects) {
        Ok(claimed) => claimed,
        Err(_) => {
            return Ok(ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::AttachmentUnavailable,
            ))
        }
    };
    let snapshot = service
        .resume_with_attachments(request, &projects, claimed.resolved())
        .await;
    if retain_claimed_attachments(&attachment_service, claimed, &snapshot).is_err() {
        return Ok(ConversationSnapshot::unavailable(
            ConversationDiagnosticCode::AttachmentUnavailable,
        ));
    }
    Ok(snapshot)
}

#[tauri::command]
async fn conversation_fork(
    request: ConversationContinueRequest,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
    attachment_service: tauri::State<'_, ConversationAttachmentService>,
) -> Result<ConversationSnapshot, ()> {
    let project_id = match projects.conversation_reference(&request.conversation_id) {
        Ok(reference) => reference.project_id,
        Err(_) => {
            return Ok(ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::ConversationNotFound,
            ))
        }
    };
    let claimed = match attachment_service.claim(&project_id, &request.attachment_ids, &projects) {
        Ok(claimed) => claimed,
        Err(_) => {
            return Ok(ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::AttachmentUnavailable,
            ))
        }
    };
    let snapshot = service
        .fork_with_attachments(request, &projects, claimed.resolved())
        .await;
    if retain_claimed_attachments(&attachment_service, claimed, &snapshot).is_err() {
        return Ok(ConversationSnapshot::unavailable(
            ConversationDiagnosticCode::AttachmentUnavailable,
        ));
    }
    Ok(snapshot)
}

fn retain_claimed_attachments(
    service: &ConversationAttachmentService,
    claimed: ClaimedConversationAttachments,
    snapshot: &ConversationSnapshot,
) -> Result<(), attachment::types::ConversationAttachmentDiagnosticCode> {
    if !snapshot.turn_in_flight() {
        return Ok(());
    }
    let conversation_id = snapshot
        .conversation_id
        .as_deref()
        .ok_or(attachment::types::ConversationAttachmentDiagnosticCode::InvalidRequest)?;
    service.retain_for_conversation(conversation_id, claimed)
}

fn cleanup_terminal_attachments(
    service: &ConversationAttachmentService,
    snapshot: &ConversationSnapshot,
) {
    if snapshot.turn_in_flight() {
        return;
    }
    if let Some(conversation_id) = snapshot.conversation_id.as_deref() {
        let _ = service.cleanup_conversation(conversation_id);
    }
}

#[tauri::command]
async fn conversation_archive(
    conversation_id: String,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<SessionLifecycleSnapshot, ()> {
    Ok(service.archive(conversation_id, &projects).await)
}

#[tauri::command]
async fn conversation_restore(
    conversation_id: String,
    service: tauri::State<'_, ConversationService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<SessionLifecycleSnapshot, ()> {
    Ok(service.restore(conversation_id, &projects).await)
}

#[tauri::command]
async fn terminal_status(
    service: tauri::State<'_, TerminalService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<TerminalRegistrySnapshot, ()> {
    Ok(service.status(&projects).await)
}

#[tauri::command]
async fn terminal_start(
    request: TerminalStartRequest,
    service: tauri::State<'_, TerminalService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<TerminalSnapshot, ()> {
    Ok(service.start(request, &projects).await)
}

#[tauri::command]
async fn terminal_poll(
    request: TerminalPollRequest,
    service: tauri::State<'_, TerminalService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<TerminalSnapshot, ()> {
    Ok(service.poll(request, &projects).await)
}

#[tauri::command]
async fn terminal_write(
    request: TerminalWriteRequest,
    service: tauri::State<'_, TerminalService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<TerminalSnapshot, ()> {
    Ok(service.write(request, &projects).await)
}

#[tauri::command]
async fn terminal_resize(
    request: TerminalResizeRequest,
    service: tauri::State<'_, TerminalService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<TerminalSnapshot, ()> {
    Ok(service.resize(request, &projects).await)
}

#[tauri::command]
async fn terminal_close(
    request: TerminalCloseRequest,
    service: tauri::State<'_, TerminalService>,
    projects: tauri::State<'_, ProjectService>,
) -> Result<TerminalRegistrySnapshot, ()> {
    Ok(service.close(request.terminal_id, &projects).await)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let local_runtime = Arc::new(local_runtime::LocalRuntimeService::default());
    let local_chat = Arc::new(local_chat::LocalChatService::new(Arc::clone(
        &local_runtime,
    )));
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .manage(CodexRuntimeService::default())
        .manage(CodexAuthService::default())
        .manage(CodexUsageService::default())
        .manage(IntegrationCatalogService::default())
        .manage(IntegrationControlService::default())
        .manage(IntegrationMutationService::default())
        .manage(ConversationService::default())
        .manage(ChatConversationService::default())
        .manage(AdvisorConversationService::default())
        .manage(AdvisorTextAttachmentService::default())
        .manage(AdvisorGeneratedArtifactService::default())
        .manage(AdvisorImageAttachmentService::default())
        .manage(AdvisorDocumentAttachmentService::default())
        .manage(AdvisorArchiveAttachmentService::default())
        .manage(AdvisorBinaryAttachmentService::default())
        .manage(TaskHandoffService::default())
        .manage(DynamicAnalysisService::default())
        .manage(DesktopNotificationService::default())
        .manage(GitService::default())
        .manage(RepositoryStateReader)
        .manage(FilePreviewService::default())
        .manage(TerminalService::default())
        .manage(mock_inference::MockInferenceService::default())
        .manage(connector_foundation::ConnectorGovernanceService::default())
        .manage(controlled_browser_verification::ControlledBrowserVerificationService::default())
        .manage(context_assembly::ContextAssemblyService::default())
        .manage(action_card::ActionCardService::default())
        .manage(local_runtime)
        .manage(local_chat)
        .setup(|app| {
            match app.path().app_data_dir() {
                Ok(directory) => {
                    app.manage(ProjectService::open(&directory.join("metadata.sqlite3")));
                    app.manage(WorktreeService::open(&directory.join("worktrees")));
                    app.manage(ConversationAttachmentService::open(
                        directory.join("conversation-attachments"),
                    ));
                }
                Err(_) => {
                    app.manage(ProjectService::unavailable());
                    app.manage(WorktreeService::unavailable());
                    app.manage(ConversationAttachmentService::unavailable());
                }
            }
            #[cfg(target_os = "linux")]
            attachment::install_native_drop_capture(app.handle())?;
            #[cfg(feature = "manual-notification-probe")]
            if desktop::manual_notification_probe_requested() {
                let notifications = app.state::<DesktopNotificationService>();
                let prepared = notifications
                    .prepare_manual_probe()
                    .map_err(|()| std::io::Error::other("notification probe state unavailable"))?
                    .ok_or_else(|| std::io::Error::other("notification probe already delivered"))?;
                if let Err(error) = app
                    .notification()
                    .builder()
                    .title(prepared.title())
                    .body(prepared.body())
                    .show()
                {
                    notifications.restore(prepared);
                    return Err(error.into());
                }
                notifications.complete(prepared);
                eprintln!(
                    "QuireForge native notification probe delivered fixed completed-task copy"
                );
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            desktop_bootstrap,
            dynamic_analysis_status,
            dynamic_analysis_pick,
            dynamic_analysis_clear,
            dynamic_analysis_run,
            codex_runtime_probe,
            integration_catalog_read,
            integration_catalog_refresh,
            integration_control_preview,
            integration_control_confirm,
            integration_control_open_browser,
            integration_control_status,
            integration_mutation_preview,
            integration_mutation_confirm,
            codex_auth_status,
            codex_auth_refresh,
            chat_authentication_status,
            chat_conversation_status,
            chat_conversation_start,
            chat_conversation_poll,
            chat_conversation_interrupt,
            advisor_conversation_status,
            advisor_conversation_start,
            advisor_conversation_poll,
            advisor_conversation_interrupt,
            task_handoff_status,
            task_handoff_prepare_advisor_brief,
            task_handoff_prepare_completion_receipt,
            task_handoff_accept,
            task_handoff_cancel,
            advisor_text_attachment_status,
            advisor_text_attachment_pick,
            advisor_text_attachment_cancel,
            advisor_image_attachment_status,
            advisor_image_attachment_pick,
            advisor_image_attachment_cancel,
            advisor_document_attachment_status,
            advisor_document_attachment_pick,
            advisor_document_attachment_cancel,
            advisor_archive_attachment_status,
            advisor_archive_attachment_pick,
            advisor_archive_attachment_cancel,
            advisor_binary_attachment_status,
            advisor_binary_attachment_pick,
            advisor_binary_attachment_cancel,
            advisor_text_export_save,
            advisor_generated_artifact_create,
            advisor_generated_artifact_snapshot,
            advisor_generated_artifact_preview,
            advisor_generated_artifact_discard,
            advisor_generated_artifact_save,
            codex_auth_start,
            codex_auth_cancel,
            codex_auth_logout,
            codex_auth_open_browser,
            codex_usage_status,
            codex_usage_refresh,
            project_workspace_status,
            task_catalog_status,
            knowledge_ledger_status,
            knowledge_ledger_create,
            knowledge_ledger_bind,
            task_catalog_create,
            task_catalog_create_from_conversation,
            task_catalog_rename,
            task_catalog_status_set,
            task_catalog_archive,
            task_catalog_restore,
            task_catalog_delete,
            durable_source_prepare_manual,
            durable_source_prepare_local_text_file,
            durable_source_prepare_reviewed_artifact_text,
            durable_source_confirm_admission,
            durable_source_cancel_admission,
            durable_source_list_active,
            durable_source_read_details,
            durable_source_prepare_deletion,
            durable_source_confirm_deletion,
            artifact_reference_prepare,
            artifact_reference_confirm,
            artifact_reference_list,
            artifact_reference_prepare_deletion,
            artifact_reference_confirm_deletion,
            task_template_catalog,
            task_template_inspect,
            task_template_create,
            task_template_edit,
            task_template_duplicate,
            task_template_archive,
            task_template_restore,
            task_template_delete,
            task_template_preview,
            task_template_confirm,
            task_template_cancel,
            mock_inference_catalog,
            connector_governance_catalog,
            connector_governance_prepare,
            connector_governance_confirm,
            connector_governance_cancel,
            connector_governance_revoke,
            controlled_browser_verification_status,
            controlled_browser_verification_prepare,
            controlled_browser_verification_confirm,
            controlled_browser_verification_cancel,
            controlled_browser_verification_revoke,
            context_assembly_status,
            context_authority_ledger,
            context_assembly_prepare,
            context_assembly_confirm,
            context_assembly_local_runtime_availability,
            context_assembly_run_local_runtime,
            context_assembly_cancel_local_runtime,
            local_chat_run,
            local_chat_cancel,
            action_card_prepare,
            action_card_approve,
            action_card_revoke,
            context_assembly_review,
            context_assembly_acknowledge_review,
            context_assembly_cancel,
            context_assembly_revoke,
            mock_inference_prepare,
            mock_inference_authorize,
            mock_inference_submit,
            mock_inference_cancel,
            mock_inference_poll,
            local_review_status,
            local_review_collection_create,
            local_review_text_item_create,
            local_review_m48_artifact_copy,
            local_review_m48_generated_artifact_metadata_evidence_create,
            local_review_safe_preview_metadata_claim,
            local_review_safe_preview_metadata_evidence_create,
            local_review_package_manifest_summary_evidence_create,
            local_review_git_status_diff_summary_evidence_create,
            local_review_activity_presentation_evidence_create,
            local_review_approval_presentation_evidence_create,
            local_review_collection_resume,
            local_review_collection_discard,
            local_review_item_discard,
            local_review_image_pick,
            local_review_image_preview,
            local_review_text_preview,
            local_review_manual_evidence_create,
            local_review_manual_evidence_preview,
            local_review_m48_generated_artifact_metadata_evidence_preview,
            local_review_safe_preview_metadata_evidence_preview,
            local_review_package_manifest_summary_evidence_preview,
            local_review_git_status_diff_summary_evidence_preview,
            local_review_activity_presentation_evidence_preview,
            local_review_approval_presentation_evidence_preview,
            local_review_annotation_create,
            local_review_annotation_edit,
            local_review_annotation_resolve,
            local_review_annotation_reopen,
            local_review_annotation_delete,
            local_review_comparison_create,
            local_review_comparison_read,
            local_review_comparison_discard,
            local_review_promotion_prepare,
            local_review_promotion_confirm,
            local_review_promotion_cancel,
            task_plan_create,
            task_plan_select,
            task_plan_edit,
            task_plan_delete,
            advisor_snapshot_read,
            advisor_project_state_snapshot_read,
            advisor_draft_create,
            advisor_draft_decide,
            advisor_dispatch_once,
            advisor_completion_report,
            project_pick_directory,
            project_pick_relink,
            project_confirm_attachment,
            project_cancel_attachment,
            project_detach,
            project_archive,
            project_preflight,
            file_preview_pick,
            file_preview_open,
            file_preview_cancel,
            conversation_attachment_status,
            conversation_attachment_pick,
            conversation_attachment_stage_drop,
            conversation_attachment_stage_native_drop,
            conversation_attachment_cancel,
            worktree_status,
            worktree_create_preview,
            worktree_recover_preview,
            worktree_remove_preview,
            worktree_pick_attach,
            worktree_confirm,
            worktree_cancel,
            git_status,
            repository_state_read,
            git_diff,
            git_open_file,
            git_mutation_preview,
            git_mutation_confirm,
            git_mutation_recover,
            conversation_status,
            conversation_active,
            conversation_notify,
            conversation_start,
            conversation_poll,
            conversation_interrupt,
            conversation_approval_decide,
            model_selection_update,
            conversation_sessions,
            conversation_resume,
            conversation_fork,
            conversation_archive,
            conversation_restore,
            terminal_status,
            terminal_start,
            terminal_poll,
            terminal_write,
            terminal_resize,
            terminal_close
        ])
        .run(tauri::generate_context!())
        .expect("failed to run QuireForge");
}

#[cfg(test)]
mod phase_a_tests {
    use super::*;

    #[test]
    fn phase_a_drafts_require_bounded_nonempty_bindings() {
        let request = advisor::AdvisorDraftCreateRequest {
            advisor_conversation_id: Uuid::now_v7().to_string(),
            target_project_id: Uuid::now_v7().to_string(),
            prompt: "Prepare a focused plan".to_owned(),
            selected_project_state: None,
            declared_capabilities: vec![advisor::AdvisorDeclaredCapability::WorkspaceWrite],
            requested_model: "default".to_owned(),
            requested_reasoning_effort: "default".to_owned(),
        };
        assert!(valid_advisor_draft_request(&request));

        let invalid = advisor::AdvisorDraftCreateRequest {
            declared_capabilities: Vec::new(),
            ..request
        };
        assert!(!valid_advisor_draft_request(&invalid));
    }

    #[test]
    fn approval_digest_is_content_sensitive() {
        assert_ne!(sha256("draft one"), sha256("draft two"));
    }

    #[test]
    fn task_template_commands_are_registered_once_and_are_closed() {
        let source = include_str!("lib.rs");
        let handler = source
            .split(".invoke_handler(tauri::generate_handler![")
            .nth(1)
            .and_then(|value| value.split("]).run").next())
            .expect("native command handler");
        for command in [
            "task_template_catalog",
            "task_template_inspect",
            "task_template_create",
            "task_template_edit",
            "task_template_duplicate",
            "task_template_archive",
            "task_template_restore",
            "task_template_delete",
            "task_template_preview",
            "task_template_confirm",
            "task_template_cancel",
        ] {
            assert_eq!(handler.matches(command).count(), 1, "{command}");
        }
        let command_section = source
            .split("fn task_template_catalog")
            .nth(1)
            .and_then(|value| value.split("fn context_assembly_status").next())
            .expect("task-template command declarations");
        for forbidden in [
            "git",
            "terminal",
            "attachment",
            "dispatch",
            "execute",
            "approve",
        ] {
            assert!(!command_section.contains(forbidden), "{forbidden}");
        }
    }

    #[test]
    fn mock_inference_commands_are_registered_once_and_remain_closed() {
        let source = include_str!("lib.rs");
        let handler = source
            .split(".invoke_handler(tauri::generate_handler![")
            .nth(1)
            .and_then(|value| value.split("]).run").next())
            .expect("native command handler");
        for command in [
            "mock_inference_catalog",
            "mock_inference_prepare",
            "mock_inference_authorize",
            "mock_inference_submit",
            "mock_inference_cancel",
            "mock_inference_poll",
        ] {
            assert_eq!(handler.matches(command).count(), 1, "{command}");
        }
        let command_section = source
            .split("fn mock_inference_catalog")
            .nth(1)
            .and_then(|value| value.split("fn context_assembly_status").next())
            .expect("mock inference command declarations");
        for forbidden in ["terminal", "git", "attachment", "dispatch", "execute"] {
            assert!(!command_section.contains(forbidden), "{forbidden}");
        }
    }
}
