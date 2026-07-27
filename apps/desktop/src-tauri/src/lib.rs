pub mod advisor;
mod advisor_archive_attachment;
mod advisor_attachment;
mod advisor_document_attachment;
mod advisor_image_attachment;
mod attachment;
mod codex;
mod contract;
mod desktop;
mod git;
mod preview;
mod project;
pub mod project_state;
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
use advisor_document_attachment::{
    AdvisorDocumentAttachmentClaimRequest, AdvisorDocumentAttachmentService,
    AdvisorDocumentAttachmentSnapshot,
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
use sha2::{Digest, Sha256};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

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
    types::{ProjectPreflightSnapshot, ProjectWorkspaceSnapshot},
    ProjectService,
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
fn desktop_bootstrap() -> DesktopBootstrap {
    DesktopBootstrap::current()
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
) -> Result<AdvisorConversationSnapshot, ()> {
    if !request.is_valid() {
        return Ok(AdvisorConversationSnapshot::unavailable(
            AdvisorConversationDiagnosticCode::InvalidRequest,
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
        .manage(AdvisorImageAttachmentService::default())
        .manage(AdvisorDocumentAttachmentService::default())
        .manage(AdvisorArchiveAttachmentService::default())
        .manage(DesktopNotificationService::default())
        .manage(GitService::default())
        .manage(RepositoryStateReader)
        .manage(FilePreviewService::default())
        .manage(TerminalService::default())
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
            advisor_text_export_save,
            codex_auth_start,
            codex_auth_cancel,
            codex_auth_logout,
            codex_auth_open_browser,
            codex_usage_status,
            codex_usage_refresh,
            project_workspace_status,
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
}
