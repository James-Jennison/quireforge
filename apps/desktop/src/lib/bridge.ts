import { invoke } from "@tauri-apps/api/core";
import { z } from "zod";

import {
  conversationAttachmentCancelRequestSchema,
  conversationAttachmentDropRequestSchema,
  conversationAttachmentSnapshotSchema,
  type ConversationAttachmentCancelRequest,
  type ConversationAttachmentDropRequest,
  type ConversationAttachmentSnapshot,
} from "./attachment";
import {
  gitDiffRequestSchema,
  gitDiffSchema,
  gitMutationConfirmRequestSchema,
  gitMutationPreviewRequestSchema,
  gitMutationPreviewSchema,
  gitMutationResultSchema,
  gitOpenFileRequestSchema,
  gitRecoveryRequestSchema,
  gitWorkspaceSchema,
  type GitDiffRequest,
  type GitDiffSnapshot,
  type GitMutationConfirmRequest,
  type GitMutationPreviewRequest,
  type GitMutationPreviewSnapshot,
  type GitMutationResultSnapshot,
  type GitOpenFileRequest,
  type GitRecoveryRequest,
  type GitWorkspaceSnapshot,
} from "./git";
import {
  codexAuthSchema,
  type AuthLoginMethod,
  type CodexAuthSnapshot,
} from "./auth";
import { codexRuntimeSchema, type CodexRuntimeSnapshot } from "./codex";
import { codexUsageSchema, type CodexUsageSnapshot } from "./usage";
import { desktopBootstrapSchema, type DesktopBootstrap } from "./contract";
import {
  filePreviewHandoffRequestSchema,
  filePreviewSchema,
  type FilePreviewHandoffRequest,
  type FilePreviewSnapshot,
} from "./filePreview";
import {
  desktopNotificationResultSchema,
  type DesktopNotificationResult,
} from "./desktopIntegration";
import {
  integrationCatalogSchema,
  integrationControlActionRequestSchema,
  integrationControlConfirmationRequestSchema,
  integrationControlPreviewRequestSchema,
  integrationControlPreviewSchema,
  integrationControlResultSchema,
  integrationMutationConfirmRequestSchema,
  integrationMutationPreviewRequestSchema,
  integrationMutationPreviewSchema,
  integrationMutationResultSchema,
  type IntegrationCatalogSnapshot,
  type IntegrationControlActionRequest,
  type IntegrationControlConfirmationRequest,
  type IntegrationControlPreviewRequest,
  type IntegrationControlPreviewSnapshot,
  type IntegrationControlResultSnapshot,
  type IntegrationMutationConfirmRequest,
  type IntegrationMutationPreviewRequest,
  type IntegrationMutationPreviewSnapshot,
  type IntegrationMutationResultSnapshot,
} from "./integration";
import {
  modelSelectionSnapshotSchema,
  modelSelectionUpdateRequestSchema,
  type ModelSelectionSnapshot,
  type ModelSelectionUpdateRequest,
} from "./modelSelection";
import {
  ConversationActionFailure,
  conversationSnapshotSchema,
  conversationRegistrySchema,
  conversationStartRequestSchema,
  conversationApprovalDecisionRequestSchema,
  conversationIdSchema,
  type ConversationApprovalDecisionRequest,
  type ConversationSnapshot,
  type ConversationRegistrySnapshot,
  type ConversationStartRequest,
} from "./conversation";
import {
  chatAuthenticationSnapshotSchema,
  type ChatAuthenticationSnapshot,
} from "./conversationMode";
import {
  chatConversationIdSchema,
  chatConversationSnapshotSchema,
  chatConversationStartRequestSchema,
  type ChatConversationSnapshot,
  type ChatConversationStartRequest,
} from "./chat";
import {
  advisorConversationIdSchema,
  advisorConversationSnapshotSchema,
  advisorConversationStartRequestSchema,
  type AdvisorConversationSnapshot,
  type AdvisorConversationStartRequest,
} from "./advisorConversation";
import {
  advisorTextAttachmentSnapshotSchema,
  advisorTextExportRequestSchema,
  type AdvisorTextAttachmentSnapshot,
  type AdvisorTextExportRequest,
} from "./advisorAttachment";
import {
  advisorImageAttachmentSnapshotSchema,
  type AdvisorImageAttachmentSnapshot,
} from "./advisorImageAttachment";
import {
  advisorDocumentAttachmentSnapshotSchema,
  type AdvisorDocumentAttachmentSnapshot,
} from "./advisorDocumentAttachment";
import {
  advisorArchiveAttachmentSnapshotSchema,
  type AdvisorArchiveAttachmentSnapshot,
} from "./advisorArchiveAttachment";
import {
  advisorBinaryAttachmentSnapshotSchema,
  type AdvisorBinaryAttachmentSnapshot,
} from "./advisorBinaryAttachment";
import {
  dynamicAnalysisSnapshotSchema,
  type DynamicAnalysisRunRequest,
  type DynamicAnalysisSnapshot,
} from "./dynamicAnalysis";
import {
  advisorWorkspaceSnapshotSchema,
  parseAdvisorProjectStateReadRequest,
  parseAdvisorSelectedProjectStateSnapshot,
  type AdvisorProjectStateReadRequest,
  type AdvisorSelectedProjectStateSnapshot,
  type AdvisorWorkspaceSnapshot,
} from "./advisorWorkspace";
import {
  advisorApprovalDecisionRequestSchema,
  advisorDispatchRequestSchema,
  advisorDispatchSnapshotSchema,
  advisorCompletionReportRequestSchema,
  advisorCompletionReportSnapshotSchema,
  advisorApprovalSnapshotSchema,
  advisorDraftCreateRequestSchema,
  type AdvisorApprovalDecisionRequest,
  type AdvisorApprovalSnapshot,
  type AdvisorDraftCreateRequest,
  type AdvisorDispatchRequest,
  type AdvisorDispatchSnapshot,
  type AdvisorCompletionReportRequest,
  type AdvisorCompletionReportSnapshot,
} from "./advisorApproval";
import {
  projectPreflightSchema,
  projectWorkspaceSchema,
  type ProjectPreflightSnapshot,
  type ProjectWorkspaceSnapshot,
} from "./project";
import type {
  RepositoryStateReadRequest,
  RepositoryStateReadSnapshot,
} from "./repositoryState";
import {
  conversationContinueRequestSchema,
  sessionListRequestSchema,
  sessionLifecycleSchema,
  type ConversationContinueRequest,
  type SessionListRequest,
  type SessionLifecycleSnapshot,
} from "./session";
import {
  worktreeConfirmationRequestSchema,
  worktreeCreatePreviewRequestSchema,
  worktreePreviewSchema,
  worktreeRecoverPreviewRequestSchema,
  worktreeRemovePreviewRequestSchema,
  worktreeResultSchema,
  worktreeWorkspaceSchema,
  type WorktreeConfirmationRequest,
  type WorktreeCreatePreviewRequest,
  type WorktreePreviewSnapshot,
  type WorktreeRecoverPreviewRequest,
  type WorktreeRemovePreviewRequest,
  type WorktreeResultSnapshot,
  type WorktreeWorkspaceSnapshot,
} from "./worktree";
import {
  terminalCloseRequestSchema,
  terminalPollRequestSchema,
  terminalRegistrySchema,
  terminalResizeRequestSchema,
  terminalSnapshotSchema,
  terminalStartRequestSchema,
  terminalWriteRequestSchema,
  type TerminalCloseRequest,
  type TerminalPollRequest,
  type TerminalRegistrySnapshot,
  type TerminalResizeRequest,
  type TerminalSnapshot,
  type TerminalStartRequest,
  type TerminalWriteRequest,
} from "./terminal";

export const CODEX_RUNTIME_PROBE_COMMAND = "codex_runtime_probe";
export const CODEX_AUTH_STATUS_COMMAND = "codex_auth_status";
export const CODEX_AUTH_REFRESH_COMMAND = "codex_auth_refresh";
export const CODEX_AUTH_START_COMMAND = "codex_auth_start";
export const CODEX_AUTH_CANCEL_COMMAND = "codex_auth_cancel";
export const CODEX_AUTH_LOGOUT_COMMAND = "codex_auth_logout";
export const CODEX_AUTH_OPEN_BROWSER_COMMAND = "codex_auth_open_browser";
export const CODEX_USAGE_STATUS_COMMAND = "codex_usage_status";
export const CODEX_USAGE_REFRESH_COMMAND = "codex_usage_refresh";
export const DESKTOP_BOOTSTRAP_COMMAND = "desktop_bootstrap";
export const INTEGRATION_CATALOG_READ_COMMAND = "integration_catalog_read";
export const INTEGRATION_CATALOG_REFRESH_COMMAND =
  "integration_catalog_refresh";
export const INTEGRATION_CONTROL_PREVIEW_COMMAND =
  "integration_control_preview";
export const INTEGRATION_CONTROL_CONFIRM_COMMAND =
  "integration_control_confirm";
export const INTEGRATION_CONTROL_OPEN_BROWSER_COMMAND =
  "integration_control_open_browser";
export const INTEGRATION_CONTROL_STATUS_COMMAND = "integration_control_status";
export const INTEGRATION_MUTATION_PREVIEW_COMMAND =
  "integration_mutation_preview";
export const INTEGRATION_MUTATION_CONFIRM_COMMAND =
  "integration_mutation_confirm";
export const PROJECT_WORKSPACE_STATUS_COMMAND = "project_workspace_status";
export const ADVISOR_SNAPSHOT_READ_COMMAND = "advisor_snapshot_read";
export const ADVISOR_PROJECT_STATE_SNAPSHOT_READ_COMMAND =
  "advisor_project_state_snapshot_read";
export const ADVISOR_DRAFT_CREATE_COMMAND = "advisor_draft_create";
export const ADVISOR_DRAFT_DECIDE_COMMAND = "advisor_draft_decide";
export const ADVISOR_DISPATCH_ONCE_COMMAND = "advisor_dispatch_once";
export const ADVISOR_COMPLETION_REPORT_COMMAND = "advisor_completion_report";
export const PROJECT_PICK_DIRECTORY_COMMAND = "project_pick_directory";
export const PROJECT_PICK_RELINK_COMMAND = "project_pick_relink";
export const PROJECT_CONFIRM_ATTACHMENT_COMMAND = "project_confirm_attachment";
export const PROJECT_CANCEL_ATTACHMENT_COMMAND = "project_cancel_attachment";
export const PROJECT_DETACH_COMMAND = "project_detach";
export const PROJECT_ARCHIVE_COMMAND = "project_archive";
export const PROJECT_PREFLIGHT_COMMAND = "project_preflight";
export const FILE_PREVIEW_PICK_COMMAND = "file_preview_pick";
export const FILE_PREVIEW_OPEN_COMMAND = "file_preview_open";
export const FILE_PREVIEW_CANCEL_COMMAND = "file_preview_cancel";
export const CONVERSATION_ATTACHMENT_STATUS_COMMAND =
  "conversation_attachment_status";
export const CONVERSATION_ATTACHMENT_PICK_COMMAND =
  "conversation_attachment_pick";
export const CONVERSATION_ATTACHMENT_STAGE_DROP_COMMAND =
  "conversation_attachment_stage_drop";
export const CONVERSATION_ATTACHMENT_STAGE_NATIVE_DROP_COMMAND =
  "conversation_attachment_stage_native_drop";
export const CONVERSATION_ATTACHMENT_CANCEL_COMMAND =
  "conversation_attachment_cancel";
export const WORKTREE_STATUS_COMMAND = "worktree_status";
export const WORKTREE_CREATE_PREVIEW_COMMAND = "worktree_create_preview";
export const WORKTREE_RECOVER_PREVIEW_COMMAND = "worktree_recover_preview";
export const WORKTREE_REMOVE_PREVIEW_COMMAND = "worktree_remove_preview";
export const WORKTREE_PICK_ATTACH_COMMAND = "worktree_pick_attach";
export const WORKTREE_CONFIRM_COMMAND = "worktree_confirm";
export const WORKTREE_CANCEL_COMMAND = "worktree_cancel";
export const GIT_STATUS_COMMAND = "git_status";
export const REPOSITORY_STATE_READ_COMMAND = "repository_state_read";
export const GIT_DIFF_COMMAND = "git_diff";
export const GIT_OPEN_FILE_COMMAND = "git_open_file";
export const GIT_MUTATION_PREVIEW_COMMAND = "git_mutation_preview";
export const GIT_MUTATION_CONFIRM_COMMAND = "git_mutation_confirm";
export const GIT_MUTATION_RECOVER_COMMAND = "git_mutation_recover";
export const CONVERSATION_STATUS_COMMAND = "conversation_status";
export const CHAT_AUTHENTICATION_STATUS_COMMAND = "chat_authentication_status";
export const CHAT_CONVERSATION_STATUS_COMMAND = "chat_conversation_status";
export const CHAT_CONVERSATION_START_COMMAND = "chat_conversation_start";
export const CHAT_CONVERSATION_POLL_COMMAND = "chat_conversation_poll";
export const CHAT_CONVERSATION_INTERRUPT_COMMAND =
  "chat_conversation_interrupt";
export const ADVISOR_CONVERSATION_STATUS_COMMAND =
  "advisor_conversation_status";
export const ADVISOR_CONVERSATION_START_COMMAND = "advisor_conversation_start";
export const ADVISOR_CONVERSATION_POLL_COMMAND = "advisor_conversation_poll";
export const ADVISOR_CONVERSATION_INTERRUPT_COMMAND =
  "advisor_conversation_interrupt";
export const ADVISOR_TEXT_ATTACHMENT_STATUS_COMMAND =
  "advisor_text_attachment_status";
export const ADVISOR_TEXT_ATTACHMENT_PICK_COMMAND =
  "advisor_text_attachment_pick";
export const ADVISOR_TEXT_ATTACHMENT_CANCEL_COMMAND =
  "advisor_text_attachment_cancel";
export const ADVISOR_TEXT_EXPORT_SAVE_COMMAND = "advisor_text_export_save";
export const ADVISOR_IMAGE_ATTACHMENT_STATUS_COMMAND =
  "advisor_image_attachment_status";
export const ADVISOR_IMAGE_ATTACHMENT_PICK_COMMAND =
  "advisor_image_attachment_pick";
export const ADVISOR_IMAGE_ATTACHMENT_CANCEL_COMMAND =
  "advisor_image_attachment_cancel";
export const ADVISOR_DOCUMENT_ATTACHMENT_STATUS_COMMAND =
  "advisor_document_attachment_status";
export const ADVISOR_DOCUMENT_ATTACHMENT_PICK_COMMAND =
  "advisor_document_attachment_pick";
export const ADVISOR_DOCUMENT_ATTACHMENT_CANCEL_COMMAND =
  "advisor_document_attachment_cancel";
export const ADVISOR_ARCHIVE_ATTACHMENT_STATUS_COMMAND =
  "advisor_archive_attachment_status";
export const ADVISOR_ARCHIVE_ATTACHMENT_PICK_COMMAND =
  "advisor_archive_attachment_pick";
export const ADVISOR_ARCHIVE_ATTACHMENT_CANCEL_COMMAND =
  "advisor_archive_attachment_cancel";
export const ADVISOR_BINARY_ATTACHMENT_STATUS_COMMAND =
  "advisor_binary_attachment_status";
export const ADVISOR_BINARY_ATTACHMENT_PICK_COMMAND =
  "advisor_binary_attachment_pick";
export const ADVISOR_BINARY_ATTACHMENT_CANCEL_COMMAND =
  "advisor_binary_attachment_cancel";
export const DYNAMIC_ANALYSIS_STATUS_COMMAND = "dynamic_analysis_status";
export const DYNAMIC_ANALYSIS_PICK_COMMAND = "dynamic_analysis_pick";
export const DYNAMIC_ANALYSIS_CLEAR_COMMAND = "dynamic_analysis_clear";
export const DYNAMIC_ANALYSIS_RUN_COMMAND = "dynamic_analysis_run";
export const CONVERSATION_ACTIVE_COMMAND = "conversation_active";
export const CONVERSATION_NOTIFY_COMMAND = "conversation_notify";
export const CONVERSATION_START_COMMAND = "conversation_start";
export const CONVERSATION_POLL_COMMAND = "conversation_poll";
export const CONVERSATION_INTERRUPT_COMMAND = "conversation_interrupt";
export const CONVERSATION_APPROVAL_DECIDE_COMMAND =
  "conversation_approval_decide";
export const MODEL_SELECTION_UPDATE_COMMAND = "model_selection_update";
export const CONVERSATION_SESSIONS_COMMAND = "conversation_sessions";
export const CONVERSATION_RESUME_COMMAND = "conversation_resume";
export const CONVERSATION_FORK_COMMAND = "conversation_fork";
export const CONVERSATION_ARCHIVE_COMMAND = "conversation_archive";
export const CONVERSATION_RESTORE_COMMAND = "conversation_restore";
export const TERMINAL_STATUS_COMMAND = "terminal_status";
export const TERMINAL_START_COMMAND = "terminal_start";
export const TERMINAL_POLL_COMMAND = "terminal_poll";
export const TERMINAL_WRITE_COMMAND = "terminal_write";
export const TERMINAL_RESIZE_COMMAND = "terminal_resize";
export const TERMINAL_CLOSE_COMMAND = "terminal_close";

export type InvokeFunction = (
  command: string,
  args?: Record<string, unknown>,
) => Promise<unknown>;

const invokeTauri: InvokeFunction = (command, args) =>
  invoke<unknown>(command, args);

export async function loadDesktopBootstrap(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<DesktopBootstrap> {
  const payload = await invokeFunction(DESKTOP_BOOTSTRAP_COMMAND);
  return desktopBootstrapSchema.parse(payload);
}

export async function loadCodexRuntime(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<CodexRuntimeSnapshot> {
  const payload = await invokeFunction(CODEX_RUNTIME_PROBE_COMMAND);
  return codexRuntimeSchema.parse(payload);
}

export async function loadIntegrationCatalog(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<IntegrationCatalogSnapshot> {
  const payload = await invokeFunction(INTEGRATION_CATALOG_READ_COMMAND);
  return integrationCatalogSchema.parse(payload);
}

export async function refreshIntegrationCatalog(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<IntegrationCatalogSnapshot> {
  const payload = await invokeFunction(INTEGRATION_CATALOG_REFRESH_COMMAND);
  return integrationCatalogSchema.parse(payload);
}

export async function previewIntegrationControl(
  request: IntegrationControlPreviewRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<IntegrationControlPreviewSnapshot> {
  const reviewedRequest = integrationControlPreviewRequestSchema.parse(request);
  const payload = await invokeFunction(INTEGRATION_CONTROL_PREVIEW_COMMAND, {
    request: reviewedRequest,
  });
  return integrationControlPreviewSchema.parse(payload);
}

export async function confirmIntegrationControl(
  request: IntegrationControlConfirmationRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<IntegrationControlResultSnapshot> {
  const reviewedRequest =
    integrationControlConfirmationRequestSchema.parse(request);
  const payload = await invokeFunction(INTEGRATION_CONTROL_CONFIRM_COMMAND, {
    request: reviewedRequest,
  });
  return integrationControlResultSchema.parse(payload);
}

export async function openIntegrationControlBrowser(
  request: IntegrationControlActionRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<IntegrationControlResultSnapshot> {
  const reviewedRequest = integrationControlActionRequestSchema.parse(request);
  const payload = await invokeFunction(
    INTEGRATION_CONTROL_OPEN_BROWSER_COMMAND,
    { request: reviewedRequest },
  );
  return integrationControlResultSchema.parse(payload);
}

export async function pollIntegrationControl(
  request: IntegrationControlActionRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<IntegrationControlResultSnapshot> {
  const reviewedRequest = integrationControlActionRequestSchema.parse(request);
  const payload = await invokeFunction(INTEGRATION_CONTROL_STATUS_COMMAND, {
    request: reviewedRequest,
  });
  return integrationControlResultSchema.parse(payload);
}

export async function previewIntegrationMutation(
  request: IntegrationMutationPreviewRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<IntegrationMutationPreviewSnapshot> {
  const reviewedRequest =
    integrationMutationPreviewRequestSchema.parse(request);
  const payload = await invokeFunction(INTEGRATION_MUTATION_PREVIEW_COMMAND, {
    request: reviewedRequest,
  });
  return integrationMutationPreviewSchema.parse(payload);
}

export async function confirmIntegrationMutation(
  request: IntegrationMutationConfirmRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<IntegrationMutationResultSnapshot> {
  const reviewedRequest =
    integrationMutationConfirmRequestSchema.parse(request);
  const payload = await invokeFunction(INTEGRATION_MUTATION_CONFIRM_COMMAND, {
    request: reviewedRequest,
  });
  return integrationMutationResultSchema.parse(payload);
}

export async function loadCodexAuth(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<CodexAuthSnapshot> {
  const payload = await invokeFunction(CODEX_AUTH_STATUS_COMMAND);
  return codexAuthSchema.parse(payload);
}

export async function loadChatAuthentication(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ChatAuthenticationSnapshot> {
  const payload = await invokeFunction(CHAT_AUTHENTICATION_STATUS_COMMAND);
  return chatAuthenticationSnapshotSchema.parse(payload);
}

export async function loadChatConversation(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ChatConversationSnapshot> {
  return chatConversationSnapshotSchema.parse(
    await invokeFunction(CHAT_CONVERSATION_STATUS_COMMAND),
  );
}

export async function startChatConversation(
  request: ChatConversationStartRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ChatConversationSnapshot> {
  return chatConversationSnapshotSchema.parse(
    await invokeFunction(
      CHAT_CONVERSATION_START_COMMAND,
      chatConversationStartRequestSchema.parse(request),
    ),
  );
}

export async function pollChatConversation(
  conversationId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ChatConversationSnapshot> {
  return chatConversationSnapshotSchema.parse(
    await invokeFunction(CHAT_CONVERSATION_POLL_COMMAND, {
      conversationId: chatConversationIdSchema.parse(conversationId),
    }),
  );
}

export async function interruptChatConversation(
  conversationId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ChatConversationSnapshot> {
  return chatConversationSnapshotSchema.parse(
    await invokeFunction(CHAT_CONVERSATION_INTERRUPT_COMMAND, {
      conversationId: chatConversationIdSchema.parse(conversationId),
    }),
  );
}

export async function loadAdvisorConversation(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorConversationSnapshot> {
  return advisorConversationSnapshotSchema.parse(
    await invokeFunction(ADVISOR_CONVERSATION_STATUS_COMMAND),
  );
}

export async function startAdvisorConversation(
  request: AdvisorConversationStartRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorConversationSnapshot> {
  return advisorConversationSnapshotSchema.parse(
    await invokeFunction(ADVISOR_CONVERSATION_START_COMMAND, {
      request: advisorConversationStartRequestSchema.parse(request),
    }),
  );
}

export async function pollAdvisorConversation(
  conversationId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorConversationSnapshot> {
  return advisorConversationSnapshotSchema.parse(
    await invokeFunction(ADVISOR_CONVERSATION_POLL_COMMAND, {
      conversationId: advisorConversationIdSchema.parse(conversationId),
    }),
  );
}

export async function interruptAdvisorConversation(
  conversationId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorConversationSnapshot> {
  return advisorConversationSnapshotSchema.parse(
    await invokeFunction(ADVISOR_CONVERSATION_INTERRUPT_COMMAND, {
      conversationId: advisorConversationIdSchema.parse(conversationId),
    }),
  );
}

export async function loadAdvisorTextAttachment(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorTextAttachmentSnapshot> {
  return advisorTextAttachmentSnapshotSchema.parse(
    await invokeFunction(ADVISOR_TEXT_ATTACHMENT_STATUS_COMMAND),
  );
}

export async function pickAdvisorTextAttachment(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorTextAttachmentSnapshot> {
  return advisorTextAttachmentSnapshotSchema.parse(
    await invokeFunction(ADVISOR_TEXT_ATTACHMENT_PICK_COMMAND),
  );
}

export async function cancelAdvisorTextAttachment(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorTextAttachmentSnapshot> {
  return advisorTextAttachmentSnapshotSchema.parse(
    await invokeFunction(ADVISOR_TEXT_ATTACHMENT_CANCEL_COMMAND),
  );
}

export async function saveAdvisorTextExport(
  request: AdvisorTextExportRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<void> {
  await invokeFunction(ADVISOR_TEXT_EXPORT_SAVE_COMMAND, {
    request: advisorTextExportRequestSchema.parse(request),
  });
}

export async function loadAdvisorImageAttachment(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorImageAttachmentSnapshot> {
  return advisorImageAttachmentSnapshotSchema.parse(
    await invokeFunction(ADVISOR_IMAGE_ATTACHMENT_STATUS_COMMAND),
  );
}

export async function pickAdvisorImageAttachment(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorImageAttachmentSnapshot> {
  return advisorImageAttachmentSnapshotSchema.parse(
    await invokeFunction(ADVISOR_IMAGE_ATTACHMENT_PICK_COMMAND),
  );
}

export async function cancelAdvisorImageAttachment(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorImageAttachmentSnapshot> {
  return advisorImageAttachmentSnapshotSchema.parse(
    await invokeFunction(ADVISOR_IMAGE_ATTACHMENT_CANCEL_COMMAND),
  );
}
export async function loadAdvisorDocumentAttachment(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorDocumentAttachmentSnapshot> {
  return advisorDocumentAttachmentSnapshotSchema.parse(
    await invokeFunction(ADVISOR_DOCUMENT_ATTACHMENT_STATUS_COMMAND),
  );
}
export async function pickAdvisorDocumentAttachment(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorDocumentAttachmentSnapshot> {
  return advisorDocumentAttachmentSnapshotSchema.parse(
    await invokeFunction(ADVISOR_DOCUMENT_ATTACHMENT_PICK_COMMAND),
  );
}
export async function cancelAdvisorDocumentAttachment(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorDocumentAttachmentSnapshot> {
  return advisorDocumentAttachmentSnapshotSchema.parse(
    await invokeFunction(ADVISOR_DOCUMENT_ATTACHMENT_CANCEL_COMMAND),
  );
}
export async function loadAdvisorArchiveAttachment(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorArchiveAttachmentSnapshot> {
  return advisorArchiveAttachmentSnapshotSchema.parse(
    await invokeFunction(ADVISOR_ARCHIVE_ATTACHMENT_STATUS_COMMAND),
  );
}
export async function pickAdvisorArchiveAttachment(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorArchiveAttachmentSnapshot> {
  return advisorArchiveAttachmentSnapshotSchema.parse(
    await invokeFunction(ADVISOR_ARCHIVE_ATTACHMENT_PICK_COMMAND),
  );
}
export async function cancelAdvisorArchiveAttachment(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorArchiveAttachmentSnapshot> {
  return advisorArchiveAttachmentSnapshotSchema.parse(
    await invokeFunction(ADVISOR_ARCHIVE_ATTACHMENT_CANCEL_COMMAND),
  );
}
export async function loadAdvisorBinaryAttachment(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorBinaryAttachmentSnapshot> {
  return advisorBinaryAttachmentSnapshotSchema.parse(
    await invokeFunction(ADVISOR_BINARY_ATTACHMENT_STATUS_COMMAND),
  );
}
export async function pickAdvisorBinaryAttachment(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorBinaryAttachmentSnapshot> {
  return advisorBinaryAttachmentSnapshotSchema.parse(
    await invokeFunction(ADVISOR_BINARY_ATTACHMENT_PICK_COMMAND),
  );
}
export async function cancelAdvisorBinaryAttachment(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorBinaryAttachmentSnapshot> {
  return advisorBinaryAttachmentSnapshotSchema.parse(
    await invokeFunction(ADVISOR_BINARY_ATTACHMENT_CANCEL_COMMAND),
  );
}

export async function loadDynamicAnalysis(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<DynamicAnalysisSnapshot> {
  return dynamicAnalysisSnapshotSchema.parse(
    await invokeFunction(DYNAMIC_ANALYSIS_STATUS_COMMAND),
  );
}

export async function pickDynamicAnalysis(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<DynamicAnalysisSnapshot> {
  return dynamicAnalysisSnapshotSchema.parse(
    await invokeFunction(DYNAMIC_ANALYSIS_PICK_COMMAND),
  );
}

export async function clearDynamicAnalysis(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<DynamicAnalysisSnapshot> {
  return dynamicAnalysisSnapshotSchema.parse(
    await invokeFunction(DYNAMIC_ANALYSIS_CLEAR_COMMAND),
  );
}

export async function runDynamicAnalysis(
  request: DynamicAnalysisRunRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<DynamicAnalysisSnapshot> {
  return dynamicAnalysisSnapshotSchema.parse(
    await invokeFunction(DYNAMIC_ANALYSIS_RUN_COMMAND, { request }),
  );
}

export async function refreshCodexAuth(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<CodexAuthSnapshot> {
  const payload = await invokeFunction(CODEX_AUTH_REFRESH_COMMAND);
  return codexAuthSchema.parse(payload);
}

export async function startCodexAuth(
  method: AuthLoginMethod,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<CodexAuthSnapshot> {
  const payload = await invokeFunction(CODEX_AUTH_START_COMMAND, { method });
  return codexAuthSchema.parse(payload);
}

export async function cancelCodexAuth(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<CodexAuthSnapshot> {
  const payload = await invokeFunction(CODEX_AUTH_CANCEL_COMMAND);
  return codexAuthSchema.parse(payload);
}

export async function logoutCodexAuth(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<CodexAuthSnapshot> {
  const payload = await invokeFunction(CODEX_AUTH_LOGOUT_COMMAND);
  return codexAuthSchema.parse(payload);
}

export async function openCodexAuthBrowser(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<void> {
  await invokeFunction(CODEX_AUTH_OPEN_BROWSER_COMMAND);
}

export async function loadCodexUsage(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<CodexUsageSnapshot> {
  const payload = await invokeFunction(CODEX_USAGE_STATUS_COMMAND);
  return codexUsageSchema.parse(payload);
}

export async function refreshCodexUsage(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<CodexUsageSnapshot> {
  const payload = await invokeFunction(CODEX_USAGE_REFRESH_COMMAND);
  return codexUsageSchema.parse(payload);
}

async function invokeProjectWorkspace(
  command: string,
  args?: Record<string, unknown>,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ProjectWorkspaceSnapshot> {
  const payload = await invokeFunction(command, args);
  return projectWorkspaceSchema.parse(payload);
}

export function loadProjectWorkspace(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ProjectWorkspaceSnapshot> {
  return invokeProjectWorkspace(
    PROJECT_WORKSPACE_STATUS_COMMAND,
    undefined,
    invokeFunction,
  );
}

/**
 * Reads only local Advisor reference metadata. This fixed command has no
 * request shape, project selection, prompt, or execution capability.
 */
export async function loadAdvisorSnapshot(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorWorkspaceSnapshot> {
  return advisorWorkspaceSnapshotSchema.parse(
    await invokeFunction(ADVISOR_SNAPSHOT_READ_COMMAND),
  );
}

/**
 * Reads a single attached project's normalized state through the fixed
 * local-only/metadata-only native boundary. Callers cannot choose a path,
 * remote mode, artifact-verification mode, or source document.
 */
export async function readAdvisorProjectStateSnapshot(
  request: AdvisorProjectStateReadRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorSelectedProjectStateSnapshot> {
  const reviewedRequest = parseAdvisorProjectStateReadRequest(request);
  return parseAdvisorSelectedProjectStateSnapshot(
    await invokeFunction(ADVISOR_PROJECT_STATE_SNAPSHOT_READ_COMMAND, {
      request: reviewedRequest,
    }),
  );
}

export async function createAdvisorDraft(
  request: AdvisorDraftCreateRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorApprovalSnapshot> {
  return advisorApprovalSnapshotSchema.parse(
    await invokeFunction(ADVISOR_DRAFT_CREATE_COMMAND, {
      request: advisorDraftCreateRequestSchema.parse(request),
    }),
  );
}

export async function decideAdvisorDraft(
  request: AdvisorApprovalDecisionRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorApprovalSnapshot> {
  return advisorApprovalSnapshotSchema.parse(
    await invokeFunction(ADVISOR_DRAFT_DECIDE_COMMAND, {
      request: advisorApprovalDecisionRequestSchema.parse(request),
    }),
  );
}

export async function dispatchAdvisorOnce(
  request: AdvisorDispatchRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorDispatchSnapshot> {
  return advisorDispatchSnapshotSchema.parse(
    await invokeFunction(ADVISOR_DISPATCH_ONCE_COMMAND, {
      request: advisorDispatchRequestSchema.parse(request),
    }),
  );
}

export async function loadAdvisorCompletionReport(
  request: AdvisorCompletionReportRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<AdvisorCompletionReportSnapshot> {
  return advisorCompletionReportSnapshotSchema.parse(
    await invokeFunction(ADVISOR_COMPLETION_REPORT_COMMAND, {
      request: advisorCompletionReportRequestSchema.parse(request),
    }),
  );
}

export async function readRepositoryState(
  request: RepositoryStateReadRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<RepositoryStateReadSnapshot> {
  const {
    repositoryStateReadRequestSchema,
    repositoryStateReadSnapshotSchema,
  } = await import("./repositoryState");
  const reviewedRequest = repositoryStateReadRequestSchema.parse(request);
  const payload = await invokeFunction(REPOSITORY_STATE_READ_COMMAND, {
    request: reviewedRequest,
  });
  return repositoryStateReadSnapshotSchema.parse(payload);
}

export function pickProjectDirectory(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ProjectWorkspaceSnapshot> {
  return invokeProjectWorkspace(
    PROJECT_PICK_DIRECTORY_COMMAND,
    undefined,
    invokeFunction,
  );
}

export function pickProjectRelink(
  projectId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ProjectWorkspaceSnapshot> {
  return invokeProjectWorkspace(
    PROJECT_PICK_RELINK_COMMAND,
    { projectId },
    invokeFunction,
  );
}

export function confirmProjectAttachment(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ProjectWorkspaceSnapshot> {
  return invokeProjectWorkspace(
    PROJECT_CONFIRM_ATTACHMENT_COMMAND,
    undefined,
    invokeFunction,
  );
}

export function cancelProjectAttachment(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ProjectWorkspaceSnapshot> {
  return invokeProjectWorkspace(
    PROJECT_CANCEL_ATTACHMENT_COMMAND,
    undefined,
    invokeFunction,
  );
}

export function detachProject(
  projectId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ProjectWorkspaceSnapshot> {
  return invokeProjectWorkspace(
    PROJECT_DETACH_COMMAND,
    { projectId },
    invokeFunction,
  );
}

export function archiveProject(
  projectId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ProjectWorkspaceSnapshot> {
  return invokeProjectWorkspace(
    PROJECT_ARCHIVE_COMMAND,
    { projectId },
    invokeFunction,
  );
}

export async function preflightProject(
  projectId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ProjectPreflightSnapshot> {
  const payload = await invokeFunction(PROJECT_PREFLIGHT_COMMAND, {
    projectId,
  });
  return projectPreflightSchema.parse(payload);
}

export async function pickFilePreview(
  projectId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<FilePreviewSnapshot> {
  const reviewedProjectId = z
    .string()
    .regex(
      /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/u,
    )
    .parse(projectId);
  const payload = await invokeFunction(FILE_PREVIEW_PICK_COMMAND, {
    projectId: reviewedProjectId,
  });
  return filePreviewSchema.parse(payload);
}

export async function openFilePreview(
  request: FilePreviewHandoffRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<void> {
  const reviewedRequest = filePreviewHandoffRequestSchema.parse(request);
  await invokeFunction(FILE_PREVIEW_OPEN_COMMAND, { request: reviewedRequest });
}

export async function cancelFilePreview(
  request: FilePreviewHandoffRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<boolean> {
  const reviewedRequest = filePreviewHandoffRequestSchema.parse(request);
  const payload = await invokeFunction(FILE_PREVIEW_CANCEL_COMMAND, {
    request: reviewedRequest,
  });
  return z.boolean().parse(payload);
}

export async function loadConversationAttachments(
  projectId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationAttachmentSnapshot> {
  const reviewedProjectId =
    conversationStartRequestSchema.shape.projectId.parse(projectId);
  const payload = await invokeFunction(CONVERSATION_ATTACHMENT_STATUS_COMMAND, {
    projectId: reviewedProjectId,
  });
  return conversationAttachmentSnapshotSchema.parse(payload);
}

export async function pickConversationAttachments(
  projectId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationAttachmentSnapshot> {
  const reviewedProjectId =
    conversationStartRequestSchema.shape.projectId.parse(projectId);
  const payload = await invokeFunction(CONVERSATION_ATTACHMENT_PICK_COMMAND, {
    projectId: reviewedProjectId,
  });
  return conversationAttachmentSnapshotSchema.parse(payload);
}

export async function stageDroppedConversationAttachments(
  request: ConversationAttachmentDropRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationAttachmentSnapshot> {
  if (request.files.length === 0) {
    const projectId = conversationStartRequestSchema.shape.projectId.parse(
      request.projectId,
    );
    const payload = await invokeFunction(
      CONVERSATION_ATTACHMENT_STAGE_NATIVE_DROP_COMMAND,
      { projectId },
    );
    return conversationAttachmentSnapshotSchema.parse(payload);
  }
  const reviewedRequest =
    conversationAttachmentDropRequestSchema.parse(request);
  const payload = await invokeFunction(
    CONVERSATION_ATTACHMENT_STAGE_DROP_COMMAND,
    { request: reviewedRequest },
  );
  return conversationAttachmentSnapshotSchema.parse(payload);
}

export async function cancelConversationAttachments(
  request: ConversationAttachmentCancelRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationAttachmentSnapshot> {
  const reviewedRequest =
    conversationAttachmentCancelRequestSchema.parse(request);
  const payload = await invokeFunction(CONVERSATION_ATTACHMENT_CANCEL_COMMAND, {
    request: reviewedRequest,
  });
  return conversationAttachmentSnapshotSchema.parse(payload);
}

export async function loadWorktreeStatus(
  projectId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<WorktreeWorkspaceSnapshot> {
  const reviewedProjectId =
    worktreeCreatePreviewRequestSchema.shape.projectId.parse(projectId);
  const payload = await invokeFunction(WORKTREE_STATUS_COMMAND, {
    projectId: reviewedProjectId,
  });
  return worktreeWorkspaceSchema.parse(payload);
}

export async function previewWorktreeCreate(
  request: WorktreeCreatePreviewRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<WorktreePreviewSnapshot> {
  const reviewedRequest = worktreeCreatePreviewRequestSchema.parse(request);
  const payload = await invokeFunction(WORKTREE_CREATE_PREVIEW_COMMAND, {
    request: reviewedRequest,
  });
  return worktreePreviewSchema.parse(payload);
}

export async function previewWorktreeRecover(
  request: WorktreeRecoverPreviewRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<WorktreePreviewSnapshot> {
  const reviewedRequest = worktreeRecoverPreviewRequestSchema.parse(request);
  const payload = await invokeFunction(WORKTREE_RECOVER_PREVIEW_COMMAND, {
    request: reviewedRequest,
  });
  return worktreePreviewSchema.parse(payload);
}

export async function previewWorktreeRemove(
  request: WorktreeRemovePreviewRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<WorktreePreviewSnapshot> {
  const reviewedRequest = worktreeRemovePreviewRequestSchema.parse(request);
  const payload = await invokeFunction(WORKTREE_REMOVE_PREVIEW_COMMAND, {
    request: reviewedRequest,
  });
  return worktreePreviewSchema.parse(payload);
}

export async function pickWorktreeAttach(
  projectId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<WorktreePreviewSnapshot> {
  const reviewedProjectId =
    worktreeCreatePreviewRequestSchema.shape.projectId.parse(projectId);
  const payload = await invokeFunction(WORKTREE_PICK_ATTACH_COMMAND, {
    projectId: reviewedProjectId,
  });
  return worktreePreviewSchema.parse(payload);
}

export async function confirmWorktree(
  request: WorktreeConfirmationRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<WorktreeResultSnapshot> {
  const reviewedRequest = worktreeConfirmationRequestSchema.parse(request);
  const payload = await invokeFunction(WORKTREE_CONFIRM_COMMAND, {
    request: reviewedRequest,
  });
  return worktreeResultSchema.parse(payload);
}

export async function cancelWorktree(
  request: WorktreeConfirmationRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<boolean> {
  const reviewedRequest = worktreeConfirmationRequestSchema.parse(request);
  const payload = await invokeFunction(WORKTREE_CANCEL_COMMAND, {
    request: reviewedRequest,
  });
  return z.boolean().parse(payload);
}

export async function loadGitStatus(
  projectId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<GitWorkspaceSnapshot> {
  const reviewedId = gitDiffRequestSchema.shape.projectId.parse(projectId);
  const payload = await invokeFunction(GIT_STATUS_COMMAND, {
    projectId: reviewedId,
  });
  return gitWorkspaceSchema.parse(payload);
}

export async function loadGitDiff(
  request: GitDiffRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<GitDiffSnapshot> {
  const reviewedRequest = gitDiffRequestSchema.parse(request);
  const payload = await invokeFunction(GIT_DIFF_COMMAND, {
    request: reviewedRequest,
  });
  return gitDiffSchema.parse(payload);
}

export async function openGitFile(
  request: GitOpenFileRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<void> {
  const reviewedRequest = gitOpenFileRequestSchema.parse(request);
  await invokeFunction(GIT_OPEN_FILE_COMMAND, { request: reviewedRequest });
}

export async function previewGitMutation(
  request: GitMutationPreviewRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<GitMutationPreviewSnapshot> {
  const reviewedRequest = gitMutationPreviewRequestSchema.parse(request);
  const payload = await invokeFunction(GIT_MUTATION_PREVIEW_COMMAND, {
    request: reviewedRequest,
  });
  return gitMutationPreviewSchema.parse(payload);
}

export async function confirmGitMutation(
  request: GitMutationConfirmRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<GitMutationResultSnapshot> {
  const reviewedRequest = gitMutationConfirmRequestSchema.parse(request);
  const payload = await invokeFunction(GIT_MUTATION_CONFIRM_COMMAND, {
    request: reviewedRequest,
  });
  return gitMutationResultSchema.parse(payload);
}

export async function recoverGitMutation(
  request: GitRecoveryRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<GitMutationResultSnapshot> {
  const reviewedRequest = gitRecoveryRequestSchema.parse(request);
  const payload = await invokeFunction(GIT_MUTATION_RECOVER_COMMAND, {
    request: reviewedRequest,
  });
  return gitMutationResultSchema.parse(payload);
}

export async function loadConversationStatus(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationSnapshot> {
  const payload = await invokeFunction(CONVERSATION_STATUS_COMMAND);
  return conversationSnapshotSchema.parse(payload);
}

export async function notifyConversation(
  conversationId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<DesktopNotificationResult> {
  const reviewedId = conversationIdSchema.parse(conversationId);
  const payload = await invokeFunction(CONVERSATION_NOTIFY_COMMAND, {
    request: { conversationId: reviewedId },
  });
  return desktopNotificationResultSchema.parse(payload);
}

export async function loadActiveConversations(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationRegistrySnapshot> {
  const payload = await invokeFunction(CONVERSATION_ACTIVE_COMMAND);
  return conversationRegistrySchema.parse(payload);
}

export async function startConversation(
  request: ConversationStartRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationSnapshot> {
  const reviewedRequest = conversationStartRequestSchema.safeParse(request);
  if (!reviewedRequest.success) {
    throw new ConversationActionFailure("request-invalid");
  }

  let payload: unknown;
  try {
    payload = await invokeFunction(CONVERSATION_START_COMMAND, {
      request: reviewedRequest.data,
    });
  } catch {
    throw new ConversationActionFailure("native-command-failed");
  }

  const snapshot = conversationSnapshotSchema.safeParse(payload);
  if (!snapshot.success) {
    throw new ConversationActionFailure("native-response-invalid");
  }
  return snapshot.data;
}

export async function pollConversation(
  conversationId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationSnapshot> {
  const reviewedId = conversationIdSchema.parse(conversationId);
  const payload = await invokeFunction(CONVERSATION_POLL_COMMAND, {
    conversationId: reviewedId,
  });
  return conversationSnapshotSchema.parse(payload);
}

export async function interruptConversation(
  conversationId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationSnapshot> {
  const reviewedId = conversationIdSchema.parse(conversationId);
  const payload = await invokeFunction(CONVERSATION_INTERRUPT_COMMAND, {
    conversationId: reviewedId,
  });
  return conversationSnapshotSchema.parse(payload);
}

export async function decideConversationApproval(
  request: ConversationApprovalDecisionRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationSnapshot> {
  const reviewedRequest =
    conversationApprovalDecisionRequestSchema.parse(request);
  const payload = await invokeFunction(CONVERSATION_APPROVAL_DECIDE_COMMAND, {
    request: reviewedRequest,
  });
  return conversationSnapshotSchema.parse(payload);
}

export async function updateModelSelection(
  request: ModelSelectionUpdateRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ModelSelectionSnapshot> {
  const reviewedRequest = modelSelectionUpdateRequestSchema.parse(request);
  const payload = await invokeFunction(MODEL_SELECTION_UPDATE_COMMAND, {
    request: reviewedRequest,
  });
  return modelSelectionSnapshotSchema.parse(payload);
}

export async function loadConversationSessions(
  request: SessionListRequest = { projectId: null, searchTerm: null },
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<SessionLifecycleSnapshot> {
  const reviewedRequest = sessionListRequestSchema.parse(request);
  const payload = await invokeFunction(CONVERSATION_SESSIONS_COMMAND, {
    request: reviewedRequest,
  });
  return sessionLifecycleSchema.parse(payload);
}

async function continueConversation(
  command: string,
  request: ConversationContinueRequest,
  invokeFunction: InvokeFunction,
): Promise<ConversationSnapshot> {
  const reviewedRequest = conversationContinueRequestSchema.parse(request);
  const payload = await invokeFunction(command, { request: reviewedRequest });
  return conversationSnapshotSchema.parse(payload);
}

export function resumeConversation(
  request: ConversationContinueRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationSnapshot> {
  return continueConversation(
    CONVERSATION_RESUME_COMMAND,
    request,
    invokeFunction,
  );
}

export function forkConversation(
  request: ConversationContinueRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationSnapshot> {
  return continueConversation(
    CONVERSATION_FORK_COMMAND,
    request,
    invokeFunction,
  );
}

async function setConversationArchived(
  command: string,
  conversationId: string,
  invokeFunction: InvokeFunction,
): Promise<SessionLifecycleSnapshot> {
  const reviewedId = conversationIdSchema.parse(conversationId);
  const payload = await invokeFunction(command, { conversationId: reviewedId });
  return sessionLifecycleSchema.parse(payload);
}

export function archiveConversation(
  conversationId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<SessionLifecycleSnapshot> {
  return setConversationArchived(
    CONVERSATION_ARCHIVE_COMMAND,
    conversationId,
    invokeFunction,
  );
}

export function restoreConversation(
  conversationId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<SessionLifecycleSnapshot> {
  return setConversationArchived(
    CONVERSATION_RESTORE_COMMAND,
    conversationId,
    invokeFunction,
  );
}

export async function loadTerminalStatus(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<TerminalRegistrySnapshot> {
  const payload = await invokeFunction(TERMINAL_STATUS_COMMAND);
  return terminalRegistrySchema.parse(payload);
}

export async function startTerminal(
  request: TerminalStartRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<TerminalSnapshot> {
  const reviewedRequest = terminalStartRequestSchema.parse(request);
  const payload = await invokeFunction(TERMINAL_START_COMMAND, {
    request: reviewedRequest,
  });
  return terminalSnapshotSchema.parse(payload);
}

export async function pollTerminal(
  request: TerminalPollRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<TerminalSnapshot> {
  const reviewedRequest = terminalPollRequestSchema.parse(request);
  const payload = await invokeFunction(TERMINAL_POLL_COMMAND, {
    request: reviewedRequest,
  });
  return terminalSnapshotSchema.parse(payload);
}

export async function writeTerminal(
  request: TerminalWriteRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<TerminalSnapshot> {
  const reviewedRequest = terminalWriteRequestSchema.parse(request);
  const payload = await invokeFunction(TERMINAL_WRITE_COMMAND, {
    request: reviewedRequest,
  });
  return terminalSnapshotSchema.parse(payload);
}

export async function resizeTerminal(
  request: TerminalResizeRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<TerminalSnapshot> {
  const reviewedRequest = terminalResizeRequestSchema.parse(request);
  const payload = await invokeFunction(TERMINAL_RESIZE_COMMAND, {
    request: reviewedRequest,
  });
  return terminalSnapshotSchema.parse(payload);
}

export async function closeTerminal(
  request: TerminalCloseRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<TerminalRegistrySnapshot> {
  const reviewedRequest = terminalCloseRequestSchema.parse(request);
  const payload = await invokeFunction(TERMINAL_CLOSE_COMMAND, {
    request: reviewedRequest,
  });
  return terminalRegistrySchema.parse(payload);
}
