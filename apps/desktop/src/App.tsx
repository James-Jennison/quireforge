import {
  lazy,
  Suspense,
  useEffect,
  useId,
  useRef,
  useState,
  type CSSProperties,
  type KeyboardEvent as ReactKeyboardEvent,
  type PointerEvent as ReactPointerEvent,
  type ReactNode,
} from "react";

import brandMark from "../../../assets/brand/quireforge-app-icon.svg";
import {
  applyAppearanceTheme,
  appearanceThemes,
  appearanceThemeStorageKey,
  nextAppearanceTheme,
  storedAppearanceTheme,
  type ThemeId,
} from "./appearanceThemes";
import { AuthGate } from "./AuthGate";
import { AdvisorWorkspace } from "./AdvisorWorkspace";
import { ConversationWorkspace } from "./ConversationWorkspace";
import { FilePreviewWorkspace } from "./FilePreviewWorkspace";
import { DynamicAnalysisWorkspace } from "./DynamicAnalysisWorkspace";
import { GitWorkspace } from "./GitWorkspace";
import { HomeDashboard } from "./HomeDashboard";
import { IntegrationCenter } from "./IntegrationCenter";
import { ProjectWorkspace } from "./ProjectWorkspace";
import { ProjectStateWorkspace } from "./ProjectStateWorkspace";
import { ScheduledWorkspace } from "./ScheduledWorkspace";
import { SessionWorkspace } from "./SessionWorkspace";
import { SettingsWorkspace } from "./SettingsWorkspace";
import { UsagePanel } from "./UsagePanel";
import { TaskCatalog } from "./TaskCatalog";
import {
  clampLayoutDimension,
  layoutPreferenceStorageKey,
  restoreWorkbenchLayoutPreferences,
  terminalDockHeightMaximum,
  terminalDockHeightMinimum,
} from "./layoutPreferences";
import {
  WorktreeWorkspace,
  type WorktreeExecutionView,
} from "./WorktreeWorkspace";
import {
  archiveConversation,
  archiveProject,
  cancelFilePreview,
  cancelConversationAttachments,
  cancelCodexAuth,
  acceptTaskHandoff,
  cancelProjectAttachment,
  confirmProjectAttachment,
  confirmGitMutation,
  confirmIntegrationMutation,
  confirmIntegrationControl,
  decideConversationApproval,
  dispatchAdvisorOnce,
  detachProject,
  interruptConversation,
  interruptAdvisorConversation,
  loadAdvisorGeneratedArtifacts,
  loadAdvisorConversation,
  loadActiveConversations,
  loadCodexAuth,
  loadConversationStatus,
  loadConversationSessions,
  loadCodexRuntime,
  loadCodexUsage,
  loadDesktopBootstrap,
  loadGitDiff,
  loadGitStatus,
  loadIntegrationCatalog,
  loadAdvisorSnapshot,
  readAdvisorProjectStateSnapshot,
  notifyConversation,
  openFilePreview,
  openIntegrationControlBrowser,
  loadProjectWorkspace,
  readRepositoryState,
  logoutCodexAuth,
  openGitFile,
  pickConversationAttachments,
  pickFilePreview,
  openCodexAuthBrowser,
  pickProjectDirectory,
  pickProjectRelink,
  preflightProject,
  previewAdvisorGeneratedArtifact,
  previewGitMutation,
  previewIntegrationMutation,
  previewIntegrationControl,
  pollIntegrationControl,
  refreshIntegrationCatalog as refreshIntegrationCatalogNative,
  pollConversation,
  pollAdvisorConversation,
  refreshCodexAuth,
  refreshCodexUsage,
  recoverGitMutation,
  restoreConversation,
  resumeConversation,
  forkConversation,
  startConversation,
  startAdvisorConversation,
  startCodexAuth,
  stageDroppedConversationAttachments,
  updateModelSelection,
  cancelWorktree,
  closeTerminal,
  confirmWorktree,
  loadTerminalStatus,
  loadWorktreeStatus,
  pickWorktreeAttach,
  pollTerminal,
  previewWorktreeCreate,
  previewWorktreeRecover,
  previewWorktreeRemove,
  resizeTerminal,
  startTerminal,
  prepareAdvisorTaskHandoff as prepareAdvisorTaskHandoffNative,
  prepareTaskCompletionReceipt,
  writeTerminal,
  archiveTaskRecord,
  createTaskPlan,
  createTaskRecord,
  deleteTaskPlan,
  deleteTaskRecord,
  editTaskPlan,
  loadTaskCatalog,
  renameTaskRecord,
  restoreTaskRecord,
  selectTaskPlan,
  setTaskRecordStatus,
} from "./lib/bridge";
import type {
  TaskHandoffCreateRequest,
  TaskHandoffSnapshot,
} from "./lib/taskHandoff";
import {
  scaffoldTaskCatalog,
  type TaskCatalogSnapshot,
} from "./lib/taskRecords";
import type {
  AdvisorProjectStateReadRequest,
  AdvisorSelectedProjectStateSnapshot,
  AdvisorWorkspaceSnapshot,
} from "./lib/advisorWorkspace";
import {
  scaffoldConversationAttachments,
  type ConversationAttachmentCancelRequest,
  type ConversationAttachmentDropRequest,
  type ConversationAttachmentSnapshot,
} from "./lib/attachment";
import {
  scaffoldCodexAuth,
  type AuthLoginMethod,
  type CodexAuthSnapshot,
} from "./lib/auth";
import { scaffoldCodexRuntime, type CodexRuntimeSnapshot } from "./lib/codex";
import { unavailableCodexUsage, type CodexUsageSnapshot } from "./lib/usage";
import { scaffoldBootstrap, type DesktopBootstrap } from "./lib/contract";
import {
  scaffoldFilePreview,
  type FilePreviewHandoffRequest,
  type FilePreviewSnapshot,
} from "./lib/filePreview";
import type { DesktopNotificationResult } from "./lib/desktopIntegration";
import {
  conversationActionFailureCode,
  scaffoldConversation,
  type ConversationActionFailureCode,
  type ConversationApprovalDecisionRequest,
  type ConversationEvent,
  type ConversationRegistrySnapshot,
  type ConversationSnapshot,
  type ConversationStartRequest,
} from "./lib/conversation";
import {
  mergeAdvisorConversationSnapshot,
  scaffoldAdvisorConversation,
  type AdvisorConversationSnapshot,
  type AdvisorConversationStartRequest,
} from "./lib/advisorConversation";
import { type ConversationMode } from "./lib/conversationMode";
import { mergeConversationEvents } from "./lib/conversationView";
import {
  scaffoldGitWorkspace,
  type GitDiffRequest,
  type GitDiffSnapshot,
  type GitMutationConfirmRequest,
  type GitMutationPreviewRequest,
  type GitMutationPreviewSnapshot,
  type GitMutationResultSnapshot,
  type GitOpenFileRequest,
  type GitRecoveryRequest,
  type GitWorkspaceSnapshot,
} from "./lib/git";
import {
  scaffoldProjectWorkspace,
  type ProjectPreflightSnapshot,
  type ProjectWorkspaceSnapshot,
} from "./lib/project";
import type {
  RepositoryStateReadRequest,
  RepositoryStateReadSnapshot,
} from "./lib/repositoryState";
import {
  scaffoldIntegrationCatalog,
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
} from "./lib/integration";
import type {
  ModelSelectionSnapshot,
  ModelSelectionUpdateRequest,
} from "./lib/modelSelection";
import {
  scaffoldSessionLifecycle,
  type ConversationContinueRequest,
  type SessionLifecycleSnapshot,
  type SessionListRequest,
} from "./lib/session";
import {
  scaffoldTerminalRegistry,
  type TerminalCloseRequest,
  type TerminalPollRequest,
  type TerminalRegistrySnapshot,
  type TerminalResizeRequest,
  type TerminalSnapshot,
  type TerminalStartRequest,
  type TerminalWriteRequest,
} from "./lib/terminal";
import {
  scaffoldWorktreeWorkspace,
  type WorktreeConfirmationRequest,
  type WorktreeCreatePreviewRequest,
  type WorktreePreviewSnapshot,
  type WorktreeRecoverPreviewRequest,
  type WorktreeRemovePreviewRequest,
  type WorktreeResultSnapshot,
  type WorktreeWorkspaceSnapshot,
} from "./lib/worktree";
import {
  defaultWorkspaceLocation,
  parseWorkspaceHash,
  workspaceLocationFor,
  workspaceLocationHash,
  workspaceNavigation,
  workspaceNavigationItem,
  type SettingsSection,
  type WorkspaceLocation,
  type WorkspaceRoute,
} from "./workspaceNavigation";

import "./styles.css";

/* eslint-disable jsx-a11y/no-noninteractive-element-interactions, jsx-a11y/no-noninteractive-tabindex -- WAI-ARIA's vertical separator pattern requires a focusable, keyboard-operable separator element. */

const TerminalWorkspace = lazy(() =>
  import("./TerminalWorkspace").then(({ TerminalWorkspace: workspace }) => ({
    default: workspace,
  })),
);

const ReviewPanes = lazy(() =>
  import("./ReviewPanes").then(({ ReviewPanes: workspace }) => ({
    default: workspace,
  })),
);
const TaskTemplateWorkbench = lazy(() =>
  import("./TaskTemplateWorkbench").then(
    ({ TaskTemplateWorkbench: workspace }) => ({
      default: workspace,
    }),
  ),
);
const MockInferenceWorkbench = lazy(() =>
  import("./MockInferenceWorkbench").then(
    ({ MockInferenceWorkbench: workspace }) => ({
      default: workspace,
    }),
  ),
);
const ConnectorGovernanceWorkbench = lazy(() =>
  import("./ConnectorGovernanceWorkbench").then(
    ({ ConnectorGovernanceWorkbench: workspace }) => ({ default: workspace }),
  ),
);
const ControlledBrowserVerificationWorkbench = lazy(() =>
  import("./ControlledBrowserVerificationWorkbench").then(
    ({ ControlledBrowserVerificationWorkbench: workspace }) => ({
      default: workspace,
    }),
  ),
);
const ContextAssemblyWorkbench = lazy(() =>
  import("./ContextAssemblyWorkbench").then(
    ({ ContextAssemblyWorkbench: workspace }) => ({ default: workspace }),
  ),
);
const DurableSourcesWorkbench = lazy(() =>
  import("./DurableSourcesWorkbench").then(
    ({ DurableSourcesWorkbench: workspace }) => ({
      default: workspace,
    }),
  ),
);

type BridgeState = "connecting" | "native" | "preview";
type RuntimeState =
  "checking" | "ready" | "degraded" | "unavailable" | "preview";
type AuthViewState = CodexAuthSnapshot["state"] | "checking" | "preview";
type ProjectViewState = "checking" | "native" | "preview";
type ProjectStateViewState =
  "idle" | "checking" | "native" | "preview" | "error";
type AdvisorViewState = "checking" | "native" | "preview" | "error";
type GitViewState = "checking" | "native" | "preview";
type WorktreeViewState = "checking" | "native" | "preview";
type ConversationViewState = "checking" | "native" | "preview";
type SessionViewState = "checking" | "native" | "preview";
type TerminalViewState = "checking" | "native" | "preview";
type IntegrationViewState = "checking" | "native" | "preview";
type UsageViewState = "checking" | "native" | "preview" | "unavailable";
const workspaceStorageKey = "quireforge-workspace-location";
const inspectorWidthStorageKey = "quireforge-inspector-width";
const sidebarCompactStorageKey = "quireforge-sidebar-compact";
const conversationModeStorageKey = "quireforge-conversation-mode";
const workspaceBoundaryAcknowledgmentStorageKey =
  "quireforge-workspace-boundary-acknowledgment";
const workspaceBoundaryPolicyVersion = "advisor-quireforge-boundary-v1";
const inspectorWidthMinimum = 280;
const inspectorWidthMaximum = 520;

function initialWorkbenchLayoutPreferences() {
  return restoreWorkbenchLayoutPreferences(
    window.localStorage.getItem(layoutPreferenceStorageKey),
  );
}

interface AppProps {
  loadBootstrap?: () => Promise<DesktopBootstrap>;
  loadRuntime?: () => Promise<CodexRuntimeSnapshot>;
  loadAuth?: () => Promise<CodexAuthSnapshot>;
  refreshAuth?: () => Promise<CodexAuthSnapshot>;
  startAuth?: (method: AuthLoginMethod) => Promise<CodexAuthSnapshot>;
  cancelAuth?: () => Promise<CodexAuthSnapshot>;
  logoutAuth?: () => Promise<CodexAuthSnapshot>;
  openAuthBrowser?: () => Promise<void>;
  loadUsage?: () => Promise<CodexUsageSnapshot>;
  refreshUsage?: () => Promise<CodexUsageSnapshot>;
  loadProjects?: () => Promise<ProjectWorkspaceSnapshot>;
  loadTaskCatalogTask?: (request: {
    projectId: string;
    query: string | null;
    includeArchived: boolean;
    selectedTaskId: string | null;
  }) => Promise<TaskCatalogSnapshot>;
  selectTaskPlanTask?: (request: {
    taskId: string;
    planId: string;
  }) => Promise<TaskCatalogSnapshot>;
  loadRepositoryStateTask?: (
    request: RepositoryStateReadRequest,
  ) => Promise<RepositoryStateReadSnapshot>;
  loadAdvisorSnapshotTask?: () => Promise<AdvisorWorkspaceSnapshot>;
  readAdvisorProjectStateSnapshotTask?: (
    request: AdvisorProjectStateReadRequest,
  ) => Promise<AdvisorSelectedProjectStateSnapshot>;
  pickProject?: () => Promise<ProjectWorkspaceSnapshot>;
  pickRelink?: (projectId: string) => Promise<ProjectWorkspaceSnapshot>;
  confirmProject?: () => Promise<ProjectWorkspaceSnapshot>;
  cancelProject?: () => Promise<ProjectWorkspaceSnapshot>;
  detachProjectDirectory?: (
    projectId: string,
  ) => Promise<ProjectWorkspaceSnapshot>;
  archiveProjectMetadata?: (
    projectId: string,
  ) => Promise<ProjectWorkspaceSnapshot>;
  preflightProjectDirectory?: (
    projectId: string,
  ) => Promise<ProjectPreflightSnapshot>;
  pickFilePreviewTask?: (projectId: string) => Promise<FilePreviewSnapshot>;
  openFilePreviewTask?: (request: FilePreviewHandoffRequest) => Promise<void>;
  cancelFilePreviewTask?: (
    request: FilePreviewHandoffRequest,
  ) => Promise<boolean>;
  pickConversationAttachmentsTask?: (
    projectId: string,
  ) => Promise<ConversationAttachmentSnapshot>;
  stageDroppedConversationAttachmentsTask?: (
    request: ConversationAttachmentDropRequest,
  ) => Promise<ConversationAttachmentSnapshot>;
  cancelConversationAttachmentsTask?: (
    request: ConversationAttachmentCancelRequest,
  ) => Promise<ConversationAttachmentSnapshot>;
  loadWorktreesTask?: (projectId: string) => Promise<WorktreeWorkspaceSnapshot>;
  previewWorktreeCreateTask?: (
    request: WorktreeCreatePreviewRequest,
  ) => Promise<WorktreePreviewSnapshot>;
  previewWorktreeRecoverTask?: (
    request: WorktreeRecoverPreviewRequest,
  ) => Promise<WorktreePreviewSnapshot>;
  previewWorktreeRemoveTask?: (
    request: WorktreeRemovePreviewRequest,
  ) => Promise<WorktreePreviewSnapshot>;
  pickWorktreeAttachTask?: (
    projectId: string,
  ) => Promise<WorktreePreviewSnapshot>;
  confirmWorktreeTask?: (
    request: WorktreeConfirmationRequest,
  ) => Promise<WorktreeResultSnapshot>;
  cancelWorktreeTask?: (
    request: WorktreeConfirmationRequest,
  ) => Promise<boolean>;
  loadGitStatusTask?: (projectId: string) => Promise<GitWorkspaceSnapshot>;
  loadGitDiffTask?: (request: GitDiffRequest) => Promise<GitDiffSnapshot>;
  openGitFileTask?: (request: GitOpenFileRequest) => Promise<void>;
  previewGitMutationTask?: (
    request: GitMutationPreviewRequest,
  ) => Promise<GitMutationPreviewSnapshot>;
  confirmGitMutationTask?: (
    request: GitMutationConfirmRequest,
  ) => Promise<GitMutationResultSnapshot>;
  recoverGitMutationTask?: (
    request: GitRecoveryRequest,
  ) => Promise<GitMutationResultSnapshot>;
  loadConversation?: () => Promise<ConversationSnapshot>;
  loadActiveConversationTasks?: () => Promise<ConversationRegistrySnapshot>;
  startConversationTask?: (
    request: ConversationStartRequest,
  ) => Promise<ConversationSnapshot>;
  dispatchAdvisorOnceTask?: typeof dispatchAdvisorOnce;
  pollConversationTask?: (
    conversationId: string,
  ) => Promise<ConversationSnapshot>;
  notifyConversationTask?: (
    conversationId: string,
  ) => Promise<DesktopNotificationResult>;
  interruptConversationTask?: (
    conversationId: string,
  ) => Promise<ConversationSnapshot>;
  loadAdvisorConversationTask?: () => Promise<AdvisorConversationSnapshot>;
  startAdvisorConversationTask?: (
    request: AdvisorConversationStartRequest,
  ) => Promise<AdvisorConversationSnapshot>;
  pollAdvisorConversationTask?: (
    conversationId: string,
  ) => Promise<AdvisorConversationSnapshot>;
  interruptAdvisorConversationTask?: (
    conversationId: string,
  ) => Promise<AdvisorConversationSnapshot>;
  decideConversationApprovalTask?: (
    request: ConversationApprovalDecisionRequest,
  ) => Promise<ConversationSnapshot>;
  updateModelSelectionTask?: (
    request: ModelSelectionUpdateRequest,
  ) => Promise<ModelSelectionSnapshot>;
  loadSessions?: (
    request?: SessionListRequest,
  ) => Promise<SessionLifecycleSnapshot>;
  resumeConversationTask?: (
    request: ConversationContinueRequest,
  ) => Promise<ConversationSnapshot>;
  forkConversationTask?: (
    request: ConversationContinueRequest,
  ) => Promise<ConversationSnapshot>;
  archiveConversationTask?: (
    conversationId: string,
  ) => Promise<SessionLifecycleSnapshot>;
  restoreConversationTask?: (
    conversationId: string,
  ) => Promise<SessionLifecycleSnapshot>;
  loadTerminalsTask?: () => Promise<TerminalRegistrySnapshot>;
  startTerminalTask?: (
    request: TerminalStartRequest,
  ) => Promise<TerminalSnapshot>;
  pollTerminalTask?: (
    request: TerminalPollRequest,
  ) => Promise<TerminalSnapshot>;
  writeTerminalTask?: (
    request: TerminalWriteRequest,
  ) => Promise<TerminalSnapshot>;
  resizeTerminalTask?: (
    request: TerminalResizeRequest,
  ) => Promise<TerminalSnapshot>;
  closeTerminalTask?: (
    request: TerminalCloseRequest,
  ) => Promise<TerminalRegistrySnapshot>;
  loadIntegrationCatalogTask?: () => Promise<IntegrationCatalogSnapshot>;
  refreshIntegrationCatalogTask?: () => Promise<IntegrationCatalogSnapshot>;
  previewIntegrationMutationTask?: (
    request: IntegrationMutationPreviewRequest,
  ) => Promise<IntegrationMutationPreviewSnapshot>;
  confirmIntegrationMutationTask?: (
    request: IntegrationMutationConfirmRequest,
  ) => Promise<IntegrationMutationResultSnapshot>;
  previewIntegrationControlTask?: (
    request: IntegrationControlPreviewRequest,
  ) => Promise<IntegrationControlPreviewSnapshot>;
  confirmIntegrationControlTask?: (
    request: IntegrationControlConfirmationRequest,
  ) => Promise<IntegrationControlResultSnapshot>;
  openIntegrationControlTask?: (
    request: IntegrationControlActionRequest,
  ) => Promise<IntegrationControlResultSnapshot>;
  pollIntegrationControlTask?: (
    request: IntegrationControlActionRequest,
  ) => Promise<IntegrationControlResultSnapshot>;
}

interface TrackedConversation {
  snapshot: ConversationSnapshot;
  events: ConversationEvent[];
}

function initialTheme(): ThemeId {
  return storedAppearanceTheme();
}

function initialWorkspaceLocation(): WorkspaceLocation {
  const fromHash = parseWorkspaceHash(window.location.hash);
  if (fromHash) return fromHash;
  if (window.location.hash) return defaultWorkspaceLocation;
  const fromStorage = parseWorkspaceHash(
    window.localStorage.getItem(workspaceStorageKey) ?? "",
  );
  return fromStorage ?? defaultWorkspaceLocation;
}

function initialInspectorWidth(): number {
  const parsed = Number.parseInt(
    window.localStorage.getItem(inspectorWidthStorageKey) ?? "",
    10,
  );
  if (!Number.isFinite(parsed)) return 330;
  return Math.min(
    inspectorWidthMaximum,
    Math.max(inspectorWidthMinimum, parsed),
  );
}

function initialSidebarCompact(): boolean {
  return window.localStorage.getItem(sidebarCompactStorageKey) === "true";
}

function initialConversationMode(): ConversationMode {
  const stored = window.localStorage.getItem(conversationModeStorageKey);
  return stored === "chat" || stored === "codex" ? stored : "codex";
}

function hasCurrentWorkspaceBoundaryAcknowledgment(): boolean {
  const raw = window.localStorage.getItem(
    workspaceBoundaryAcknowledgmentStorageKey,
  );
  if (!raw) return false;
  try {
    const parsed: unknown = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      return false;
    }
    const record = parsed as Record<string, unknown>;
    return (
      Object.keys(record).length === 3 &&
      record.schemaVersion === 1 &&
      record.boundaryPolicyVersion === workspaceBoundaryPolicyVersion &&
      record.acknowledged === true
    );
  } catch {
    return false;
  }
}

function storeWorkspaceBoundaryAcknowledgment() {
  window.localStorage.setItem(
    workspaceBoundaryAcknowledgmentStorageKey,
    JSON.stringify({
      schemaVersion: 1,
      boundaryPolicyVersion: workspaceBoundaryPolicyVersion,
      acknowledged: true,
    }),
  );
}

type WorkspaceConversationMode = "advisor" | "quireforge";

function workspaceConversationMode(
  mode: ConversationMode,
): WorkspaceConversationMode {
  return mode === "chat" ? "advisor" : "quireforge";
}

function WorkspaceSelector({
  mode,
  onRequestChange,
}: {
  mode: WorkspaceConversationMode;
  onRequestChange: (mode: WorkspaceConversationMode) => void;
}) {
  const menuId = useId();
  const [open, setOpen] = useState(false);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const optionRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const workspaces: Array<{
    id: WorkspaceConversationMode;
    title: string;
    subtitle: string;
  }> = [
    {
      id: "advisor",
      title: "Chat",
      subtitle: "Advisor · read-only planning",
    },
    {
      id: "quireforge",
      title: "Code",
      subtitle: "Codex task · build, debug, and ship",
    },
  ];

  function closeAndRestoreFocus() {
    setOpen(false);
    triggerRef.current?.focus();
  }

  function selectWorkspace(next: WorkspaceConversationMode) {
    setOpen(false);
    onRequestChange(next);
  }

  function handleMenuKeyDown(event: ReactKeyboardEvent<HTMLDivElement>) {
    const index = optionRefs.current.findIndex(
      (option) => option === document.activeElement,
    );
    if (event.key === "Escape") {
      event.preventDefault();
      closeAndRestoreFocus();
      return;
    }
    if (["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) {
      event.preventDefault();
      const nextIndex =
        event.key === "Home"
          ? 0
          : event.key === "End"
            ? workspaces.length - 1
            : (index +
                (event.key === "ArrowDown" ? 1 : -1) +
                workspaces.length) %
              workspaces.length;
      optionRefs.current[nextIndex]?.focus();
    }
  }

  return (
    <div className="workspace-selector">
      <button
        ref={triggerRef}
        className="workspace-selector__trigger"
        type="button"
        aria-haspopup="menu"
        aria-expanded={open}
        aria-controls={menuId}
        onClick={() => setOpen((current) => !current)}
        onKeyDown={(event) => {
          if (["ArrowDown", "ArrowUp"].includes(event.key)) {
            event.preventDefault();
            setOpen(true);
            window.requestAnimationFrame(() =>
              optionRefs.current[event.key === "ArrowDown" ? 0 : 1]?.focus(),
            );
          }
        }}
      >
        <span>{mode === "advisor" ? "Chat" : "Code"}</span>
        <span aria-hidden="true">⌄</span>
      </button>
      {open && (
        <div
          id={menuId}
          className="workspace-selector__menu"
          role="menu"
          tabIndex={-1}
          aria-label="Choose workspace"
          onKeyDown={handleMenuKeyDown}
        >
          {workspaces.map((workspace, index) => (
            <button
              ref={(element) => {
                optionRefs.current[index] = element;
              }}
              className="workspace-selector__option"
              type="button"
              role="menuitemradio"
              aria-checked={mode === workspace.id}
              key={workspace.id}
              onClick={() => selectWorkspace(workspace.id)}
            >
              <span>
                <strong>{workspace.title}</strong>
                <small>{workspace.subtitle}</small>
              </span>
              <span aria-hidden="true">{mode === workspace.id ? "✓" : ""}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

function initialInspectorOpen(): boolean {
  // The workbench context drawer is intentionally opt-in. It remains an
  // independent, local presentation surface rather than ambient task state.
  return false;
}

type WorkbenchDrawerTab = "diff" | "git" | "problems";

function WorkbenchActionPalette({
  open,
  onClose,
  onNavigate,
  onToggleDrawer,
  onToggleTerminal,
}: {
  open: boolean;
  onClose: () => void;
  onNavigate: (route: WorkspaceRoute) => void;
  onToggleDrawer: () => void;
  onToggleTerminal: () => void;
}) {
  const firstActionRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (open) firstActionRef.current?.focus();
  }, [open]);

  if (!open) return null;
  return (
    <div
      className="workbench-actions"
      role="dialog"
      aria-modal="true"
      aria-label="Command palette"
    >
      <button
        className="workbench-actions__backdrop"
        type="button"
        aria-label="Close command palette"
        onClick={onClose}
      />
      <div
        className="workbench-actions__panel"
        role="menu"
        aria-label="QuireForge actions"
      >
        <div>
          <p className="eyebrow">QuireForge</p>
          <h2>Actions</h2>
        </div>
        <button
          ref={firstActionRef}
          type="button"
          role="menuitem"
          onClick={() => {
            onNavigate("conversation");
            onClose();
          }}
        >
          Open task conversation
        </button>
        <button
          type="button"
          role="menuitem"
          onClick={() => {
            onNavigate("files");
            onClose();
          }}
        >
          Open files
        </button>
        <button
          type="button"
          role="menuitem"
          onClick={() => {
            onNavigate("changes");
            onClose();
          }}
        >
          Open changes
        </button>
        <button
          type="button"
          role="menuitem"
          onClick={() => {
            onToggleDrawer();
            onClose();
          }}
        >
          Toggle workbench context
        </button>
        <button
          type="button"
          role="menuitem"
          onClick={() => {
            onToggleTerminal();
            onClose();
          }}
        >
          Toggle terminal dock
        </button>
        <button type="button" role="menuitem" onClick={onClose}>
          Close
        </button>
      </div>
    </div>
  );
}

function WorkspaceView({
  route,
  active,
  children,
}: {
  route: WorkspaceRoute;
  active: boolean;
  children: ReactNode;
}) {
  return (
    <div
      className="workspace-view"
      data-workspace-view={route}
      hidden={!active}
      aria-hidden={!active}
      tabIndex={-1}
    >
      {children}
    </div>
  );
}

function Glyph({ name }: { name: string }) {
  const paths: Record<string, ReactNode> = {
    grid: (
      <>
        <rect x="3" y="3" width="7" height="7" rx="2" />
        <rect x="14" y="3" width="7" height="7" rx="2" />
        <rect x="3" y="14" width="7" height="7" rx="2" />
        <rect x="14" y="14" width="7" height="7" rx="2" />
      </>
    ),
    thread: (
      <>
        <path d="M6 7.5h12M6 12h8M6 16.5h5" />
        <path d="M4 3.5h16a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2h-8l-5 3v-3H4a2 2 0 0 1-2-2v-11a2 2 0 0 1 2-2Z" />
      </>
    ),
    blocks: (
      <>
        <path d="m8 3 4 2.3v4.6L8 12.2 4 9.9V5.3L8 3ZM16 11.8l4 2.3v4.6L16 21l-4-2.3v-4.6l4-2.3Z" />
        <path d="m16 3 4 2.3v4.6l-4 2.3-4-2.3V5.3L16 3ZM8 11.8l4 2.3v4.6L8 21l-4-2.3v-4.6l4-2.3Z" />
      </>
    ),
    clock: (
      <>
        <circle cx="12" cy="12" r="9" />
        <path d="M12 7v5l3.5 2" />
      </>
    ),
    git: (
      <>
        <circle cx="6" cy="5" r="2.5" />
        <circle cx="18" cy="19" r="2.5" />
        <circle cx="18" cy="7" r="2.5" />
        <path d="M8.5 5h2.2A3.3 3.3 0 0 1 14 8.3v7.4a3.3 3.3 0 0 0 3.3 3.3h-1.8M8.5 5A3.5 3.5 0 0 1 12 8.5v2A3.5 3.5 0 0 0 15.5 14H18" />
      </>
    ),
    plus: <path d="M12 5v14M5 12h14" />,
    folder: (
      <path d="M3 6.5A2.5 2.5 0 0 1 5.5 4H10l2 2h6.5A2.5 2.5 0 0 1 21 8.5v8A2.5 2.5 0 0 1 18.5 19h-13A2.5 2.5 0 0 1 3 16.5v-10Z" />
    ),
    terminal: (
      <>
        <path d="m5 7 4 5-4 5M11 17h8" />
        <rect x="2.5" y="3.5" width="19" height="17" rx="3" />
      </>
    ),
    shield: (
      <>
        <path d="M12 2.8 20 6v5.7c0 4.5-3.1 8-8 9.5-4.9-1.5-8-5-8-9.5V6l8-3.2Z" />
        <path d="m8.5 12 2.2 2.2 4.8-5" />
      </>
    ),
    external: (
      <>
        <path d="M14 4h6v6M20 4l-9 9" />
        <path d="M18 13v5a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h5" />
      </>
    ),
    refresh: (
      <>
        <path d="M20 7v5h-5" />
        <path d="M4 17v-5h5" />
        <path d="M6.1 9A7 7 0 0 1 18.5 7L20 12M4 12l1.5 5A7 7 0 0 0 17.9 15" />
      </>
    ),
    check: <path d="m5 12 4.2 4.2L19 6.5" />,
    chevron: <path d="m9 18 6-6-6-6" />,
    sidebar: (
      <>
        <rect x="3" y="4" width="18" height="16" rx="2.5" />
        <path d="M9 4v16" />
      </>
    ),
    gear: (
      <>
        <circle cx="12" cy="12" r="3" />
        <path d="M19 13.5v-3l-2-.7a7 7 0 0 0-.7-1.6l.9-1.9-2.1-2.1-1.9.9a7 7 0 0 0-1.6-.7L10.5 3h-3l-.7 2a7 7 0 0 0-1.6.7l-1.9-.9-2.1 2.1.9 1.9a7 7 0 0 0-.7 1.6l-2 .7v3l2 .7a7 7 0 0 0 .7 1.6l-.9 1.9 2.1 2.1 1.9-.9a7 7 0 0 0 1.6.7l.7 2h3l.7-2a7 7 0 0 0 1.6-.7l1.9.9 2.1-2.1-.9-1.9a7 7 0 0 0 .7-1.6l2-.7Z" />
      </>
    ),
  };

  return (
    <svg
      aria-hidden="true"
      className="glyph"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth="1.7"
    >
      {paths[name]}
    </svg>
  );
}

function StatusDot({ state }: { state: BridgeState }) {
  return (
    <span className={`status-dot status-dot--${state}`} aria-hidden="true" />
  );
}

export default function App({
  loadBootstrap = loadDesktopBootstrap,
  loadRuntime = loadCodexRuntime,
  loadAuth = loadCodexAuth,
  refreshAuth = refreshCodexAuth,
  startAuth = startCodexAuth,
  cancelAuth = cancelCodexAuth,
  logoutAuth = logoutCodexAuth,
  openAuthBrowser = openCodexAuthBrowser,
  loadUsage = loadCodexUsage,
  refreshUsage = refreshCodexUsage,
  loadProjects = loadProjectWorkspace,
  loadTaskCatalogTask = loadTaskCatalog,
  selectTaskPlanTask = selectTaskPlan,
  loadRepositoryStateTask = readRepositoryState,
  loadAdvisorSnapshotTask = loadAdvisorSnapshot,
  readAdvisorProjectStateSnapshotTask = readAdvisorProjectStateSnapshot,
  pickProject = pickProjectDirectory,
  pickRelink = pickProjectRelink,
  confirmProject = confirmProjectAttachment,
  cancelProject = cancelProjectAttachment,
  detachProjectDirectory = detachProject,
  archiveProjectMetadata = archiveProject,
  preflightProjectDirectory = preflightProject,
  pickFilePreviewTask = pickFilePreview,
  openFilePreviewTask = openFilePreview,
  cancelFilePreviewTask = cancelFilePreview,
  pickConversationAttachmentsTask = pickConversationAttachments,
  stageDroppedConversationAttachmentsTask = stageDroppedConversationAttachments,
  cancelConversationAttachmentsTask = cancelConversationAttachments,
  loadWorktreesTask = loadWorktreeStatus,
  previewWorktreeCreateTask = previewWorktreeCreate,
  previewWorktreeRecoverTask = previewWorktreeRecover,
  previewWorktreeRemoveTask = previewWorktreeRemove,
  pickWorktreeAttachTask = pickWorktreeAttach,
  confirmWorktreeTask = confirmWorktree,
  cancelWorktreeTask = cancelWorktree,
  loadGitStatusTask = loadGitStatus,
  loadGitDiffTask = loadGitDiff,
  openGitFileTask = openGitFile,
  previewGitMutationTask = previewGitMutation,
  confirmGitMutationTask = confirmGitMutation,
  recoverGitMutationTask = recoverGitMutation,
  loadConversation = loadConversationStatus,
  loadActiveConversationTasks = loadActiveConversations,
  startConversationTask = startConversation,
  dispatchAdvisorOnceTask = dispatchAdvisorOnce,
  pollConversationTask = pollConversation,
  notifyConversationTask = notifyConversation,
  interruptConversationTask = interruptConversation,
  loadAdvisorConversationTask = loadAdvisorConversation,
  startAdvisorConversationTask = startAdvisorConversation,
  pollAdvisorConversationTask = pollAdvisorConversation,
  interruptAdvisorConversationTask = interruptAdvisorConversation,
  decideConversationApprovalTask = decideConversationApproval,
  updateModelSelectionTask = updateModelSelection,
  loadSessions = loadConversationSessions,
  resumeConversationTask = resumeConversation,
  forkConversationTask = forkConversation,
  archiveConversationTask = archiveConversation,
  restoreConversationTask = restoreConversation,
  loadTerminalsTask = loadTerminalStatus,
  startTerminalTask = startTerminal,
  pollTerminalTask = pollTerminal,
  writeTerminalTask = writeTerminal,
  resizeTerminalTask = resizeTerminal,
  closeTerminalTask = closeTerminal,
  loadIntegrationCatalogTask = loadIntegrationCatalog,
  refreshIntegrationCatalogTask = refreshIntegrationCatalogNative,
  previewIntegrationMutationTask = previewIntegrationMutation,
  confirmIntegrationMutationTask = confirmIntegrationMutation,
  previewIntegrationControlTask = previewIntegrationControl,
  confirmIntegrationControlTask = confirmIntegrationControl,
  openIntegrationControlTask = openIntegrationControlBrowser,
  pollIntegrationControlTask = pollIntegrationControl,
}: AppProps) {
  const [bootstrap, setBootstrap] =
    useState<DesktopBootstrap>(scaffoldBootstrap);
  const [bridgeState, setBridgeState] = useState<BridgeState>("connecting");
  const [runtime, setRuntime] =
    useState<CodexRuntimeSnapshot>(scaffoldCodexRuntime);
  const [runtimeState, setRuntimeState] = useState<RuntimeState>("checking");
  const [auth, setAuth] = useState<CodexAuthSnapshot>(scaffoldCodexAuth);
  const [authState, setAuthState] = useState<AuthViewState>("checking");
  const [authBusy, setAuthBusy] = useState(false);
  const [authActionError, setAuthActionError] = useState(false);
  const [confirmLogout, setConfirmLogout] = useState(false);
  const [usage, setUsage] = useState<CodexUsageSnapshot>(unavailableCodexUsage);
  const [usageState, setUsageState] = useState<UsageViewState>("checking");
  const [usageBusy, setUsageBusy] = useState(false);
  const [projects, setProjects] = useState<ProjectWorkspaceSnapshot>(
    scaffoldProjectWorkspace,
  );
  const [projectState, setProjectState] =
    useState<ProjectViewState>("checking");
  const [projectBusy, setProjectBusy] = useState(false);
  const [projectActionError, setProjectActionError] = useState(false);
  const [projectPreflights, setProjectPreflights] = useState<
    Record<string, ProjectPreflightSnapshot>
  >({});
  const [repositoryStateSnapshot, setRepositoryStateSnapshot] =
    useState<RepositoryStateReadSnapshot | null>(null);
  const [repositoryStateViewState, setRepositoryStateViewState] =
    useState<ProjectStateViewState>("idle");
  const [repositoryStateRefresh, setRepositoryStateRefresh] = useState(0);
  const [advisorSnapshot, setAdvisorSnapshot] =
    useState<AdvisorWorkspaceSnapshot | null>(null);
  const [advisorViewState, setAdvisorViewState] =
    useState<AdvisorViewState>("checking");
  const [advisorProjectStateSnapshot, setAdvisorProjectStateSnapshot] =
    useState<AdvisorSelectedProjectStateSnapshot | null>(null);
  const [advisorProjectStateProjectId, setAdvisorProjectStateProjectId] =
    useState<string | null>(null);
  const [advisorProjectStateSelection, setAdvisorProjectStateSelection] =
    useState<"idle" | "confirming" | "reading" | "error">("idle");
  const [filePreview, setFilePreview] =
    useState<FilePreviewSnapshot>(scaffoldFilePreview);
  const [filePreviewBusy, setFilePreviewBusy] = useState(false);
  const [filePreviewActionError, setFilePreviewActionError] = useState(false);
  const [conversationAttachments, setConversationAttachments] =
    useState<ConversationAttachmentSnapshot>(scaffoldConversationAttachments);
  const [conversationAttachmentBusy, setConversationAttachmentBusy] =
    useState(false);
  const [
    conversationAttachmentActionError,
    setConversationAttachmentActionError,
  ] = useState(false);
  const [worktrees, setWorktrees] = useState<WorktreeWorkspaceSnapshot>(
    scaffoldWorktreeWorkspace,
  );
  const [worktreePreview, setWorktreePreview] =
    useState<WorktreePreviewSnapshot | null>(null);
  const [worktreeResult, setWorktreeResult] =
    useState<WorktreeResultSnapshot | null>(null);
  const [worktreeState, setWorktreeState] =
    useState<WorktreeViewState>("checking");
  const [worktreeBusy, setWorktreeBusy] = useState(false);
  const [worktreeActionError, setWorktreeActionError] = useState(false);
  const [gitSnapshot, setGitSnapshot] =
    useState<GitWorkspaceSnapshot>(scaffoldGitWorkspace);
  const [gitDiff, setGitDiff] = useState<GitDiffSnapshot | null>(null);
  const [gitSelectedRequest, setGitSelectedRequest] =
    useState<GitDiffRequest | null>(null);
  const [gitState, setGitState] = useState<GitViewState>("checking");
  const [gitBusy, setGitBusy] = useState(false);
  const [gitActionError, setGitActionError] = useState(false);
  const [gitMutationPreview, setGitMutationPreview] =
    useState<GitMutationPreviewSnapshot | null>(null);
  const [gitMutationResult, setGitMutationResult] =
    useState<GitMutationResultSnapshot | null>(null);
  const [taskGitSnapshots, setTaskGitSnapshots] = useState<
    Record<string, GitWorkspaceSnapshot>
  >({});
  const [conversation, setConversation] =
    useState<ConversationSnapshot>(scaffoldConversation);
  const [conversationEvents, setConversationEvents] = useState<
    ConversationEvent[]
  >([]);
  const [trackedConversations, setTrackedConversations] = useState<
    Record<string, TrackedConversation>
  >({});
  const [conversationState, setConversationState] =
    useState<ConversationViewState>("checking");
  const [conversationBusy, setConversationBusy] = useState(false);
  const [conversationActionError, setConversationActionError] =
    useState<ConversationActionFailureCode | null>(null);
  const [conversationMode, setConversationMode] = useState<ConversationMode>(
    initialConversationMode,
  );
  const [acceptedTaskHandoff, setAcceptedTaskHandoff] =
    useState<TaskHandoffSnapshot | null>(null);
  const [pendingConversationMode, setPendingConversationMode] =
    useState<ConversationMode | null>(null);
  const [pendingTaskHandoffOpen, setPendingTaskHandoffOpen] = useState(false);
  const [advisorConversation, setAdvisorConversation] =
    useState<AdvisorConversationSnapshot>(scaffoldAdvisorConversation);
  const [advisorConversationBusy, setAdvisorConversationBusy] = useState(false);
  const [advisorResetToken, setAdvisorResetToken] = useState(0);
  const conversationActionGenerations = useRef<Record<string, number>>({});
  const observedConversationStates = useRef<
    Record<string, ConversationSnapshot["state"]>
  >({});
  const [sessions, setSessions] = useState<SessionLifecycleSnapshot>(
    scaffoldSessionLifecycle,
  );
  const [sessionState, setSessionState] =
    useState<SessionViewState>("checking");
  const [sessionBusy, setSessionBusy] = useState(false);
  const [sessionActionError, setSessionActionError] = useState(false);
  const [sessionSearchTerm, setSessionSearchTerm] = useState<string | null>(
    null,
  );
  const [terminals, setTerminals] = useState<TerminalRegistrySnapshot>(
    scaffoldTerminalRegistry,
  );
  const [terminalState, setTerminalState] =
    useState<TerminalViewState>("checking");
  const [terminalBusy, setTerminalBusy] = useState(false);
  const [terminalActionError, setTerminalActionError] = useState(false);
  const [integrationCatalog, setIntegrationCatalog] =
    useState<IntegrationCatalogSnapshot>(scaffoldIntegrationCatalog);
  const [integrationPreview, setIntegrationPreview] =
    useState<IntegrationMutationPreviewSnapshot | null>(null);
  const [integrationResult, setIntegrationResult] =
    useState<IntegrationMutationResultSnapshot | null>(null);
  const [integrationControlPreview, setIntegrationControlPreview] =
    useState<IntegrationControlPreviewSnapshot | null>(null);
  const [integrationControlResult, setIntegrationControlResult] =
    useState<IntegrationControlResultSnapshot | null>(null);
  const [integrationState, setIntegrationState] =
    useState<IntegrationViewState>("checking");
  const [integrationBusy, setIntegrationBusy] = useState(false);
  const [integrationActionError, setIntegrationActionError] = useState(false);
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(
    null,
  );
  const [theme, setTheme] = useState<ThemeId>(initialTheme);
  const [themePreview, setThemePreview] = useState<ThemeId | null>(null);
  const [workspaceLocation, setWorkspaceLocation] = useState<WorkspaceLocation>(
    initialWorkspaceLocation,
  );
  const [inspectorWidth, setInspectorWidth] = useState(initialInspectorWidth);
  const [inspectorOpen, setInspectorOpen] = useState(initialInspectorOpen);
  const [workbenchDrawerTab, setWorkbenchDrawerTab] =
    useState<WorkbenchDrawerTab>("diff");
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);
  const [terminalDockOpen, setTerminalDockOpen] = useState(false);
  const [reviewPanesOpen, setReviewPanesOpen] = useState(false);
  const [taskCatalog, setTaskCatalog] =
    useState<TaskCatalogSnapshot>(scaffoldTaskCatalog);
  const [taskCatalogBusy, setTaskCatalogBusy] = useState(false);
  const [taskCatalogOpen, setTaskCatalogOpen] = useState(false);
  const [taskTemplateWorkbenchOpen, setTaskTemplateWorkbenchOpen] =
    useState(false);
  const [mockInferenceWorkbenchOpen, setMockInferenceWorkbenchOpen] =
    useState(false);
  const [
    connectorGovernanceWorkbenchOpen,
    setConnectorGovernanceWorkbenchOpen,
  ] = useState(false);
  const [
    controlledBrowserVerificationOpen,
    setControlledBrowserVerificationOpen,
  ] = useState(false);
  const [durableSourcesWorkbenchOpen, setDurableSourcesWorkbenchOpen] =
    useState(false);
  const [contextAssemblyWorkbenchOpen, setContextAssemblyWorkbenchOpen] =
    useState(false);
  const mockInferenceLauncherRef = useRef<HTMLButtonElement>(null);
  const [workbenchLayout, setWorkbenchLayout] = useState(
    initialWorkbenchLayoutPreferences,
  );
  const terminalResizeCleanup = useRef<(() => void) | null>(null);
  const [sidebarCompact, setSidebarCompact] = useState(initialSidebarCompact);
  const [mobileNavigationOpen, setMobileNavigationOpen] = useState(false);
  const workspaceMainRef = useRef<HTMLElement>(null);
  const commandPaletteTriggerRef = useRef<HTMLButtonElement>(null);
  const focusWorkspaceAfterNavigation = useRef(false);
  const accessGranted =
    authState === "authenticated" || authState === "not-required";
  const currentProject =
    projects.projects.find(
      (project) => project.id === selectedProjectId && !project.archived,
    ) ??
    projects.projects.find((project) => !project.archived) ??
    projects.projects[0];

  useEffect(() => {
    let active = true;
    void loadBootstrap()
      .then((result) => {
        if (!active) return;
        setBootstrap(result);
        setBridgeState("native");
      })
      .catch(() => {
        if (active) setBridgeState("preview");
      });

    return () => {
      active = false;
    };
  }, [loadBootstrap]);

  useEffect(() => {
    function handleCommandPalette(event: KeyboardEvent) {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setCommandPaletteOpen((current) => !current);
      }
      if (event.key === "Escape" && commandPaletteOpen) {
        setCommandPaletteOpen(false);
        window.requestAnimationFrame(() =>
          commandPaletteTriggerRef.current?.focus(),
        );
      }
    }
    window.addEventListener("keydown", handleCommandPalette);
    return () => window.removeEventListener("keydown", handleCommandPalette);
  }, [commandPaletteOpen]);

  useEffect(() => {
    let active = true;
    void loadRuntime()
      .then((result) => {
        if (!active) return;
        setRuntime(result);
        setRuntimeState(result.availability);
      })
      .catch(() => {
        if (active) setRuntimeState("preview");
      });

    return () => {
      active = false;
    };
  }, [loadRuntime]);

  useEffect(() => {
    let active = true;
    void loadAuth()
      .then((result) => {
        if (!active) return;
        setAuth(result);
        setAuthState(result.state);
      })
      .catch(() => {
        if (active) setAuthState("preview");
      });

    return () => {
      active = false;
    };
  }, [loadAuth]);

  useEffect(() => {
    if (!accessGranted) return;
    let active = true;
    void loadAdvisorConversationTask()
      .then((result) => {
        if (active) setAdvisorConversation(result);
      })
      .catch(() => {
        if (active) setAdvisorConversation(scaffoldAdvisorConversation);
      });
    return () => {
      active = false;
    };
  }, [accessGranted, loadAdvisorConversationTask]);

  useEffect(() => {
    if (!accessGranted) return;
    let active = true;
    void loadUsage()
      .then((result) => {
        if (!active) return;
        setUsage(result);
        setUsageState("native");
      })
      .catch(() => {
        if (active) {
          setUsage(unavailableCodexUsage);
          setUsageState("preview");
        }
      });

    return () => {
      active = false;
    };
  }, [accessGranted, loadUsage]);

  useEffect(() => {
    if (!accessGranted) return;
    let active = true;
    void loadProjects()
      .then((result) => {
        if (!active) return;
        setProjects(result);
        setProjectState("native");
      })
      .catch(() => {
        if (active) setProjectState("preview");
      });

    return () => {
      active = false;
    };
  }, [accessGranted, loadProjects]);

  useEffect(() => {
    if (!accessGranted || !currentProject) return;
    const projectId = currentProject.id;
    let active = true;
    void loadTaskCatalogTask({
      projectId,
      query: null,
      includeArchived: false,
      selectedTaskId: null,
    })
      .then((snapshot) => {
        if (active) setTaskCatalog(snapshot);
      })
      .catch(() => {
        if (active) setTaskCatalog(scaffoldTaskCatalog);
      });
    return () => {
      active = false;
    };
  }, [accessGranted, currentProject, loadTaskCatalogTask]);

  useEffect(() => {
    if (!accessGranted) return;
    let active = true;
    void loadConversation()
      .then((result) => {
        if (!active) return;
        setConversation(result);
        setConversationEvents(result.events);
        if (result.projectId && result.conversationId) {
          setTrackedConversations((current) => ({
            ...current,
            [result.projectId!]: { snapshot: result, events: result.events },
          }));
        }
        setConversationState("native");
      })
      .catch(() => {
        if (active) setConversationState("preview");
      });

    return () => {
      active = false;
    };
  }, [accessGranted, loadConversation]);

  useEffect(() => {
    if (!accessGranted) return;
    let active = true;
    void loadActiveConversationTasks()
      .then((registry) => {
        if (!active) return;
        setTrackedConversations(
          Object.fromEntries(
            registry.conversations.flatMap((snapshot) =>
              snapshot.projectId
                ? [[snapshot.projectId, { snapshot, events: snapshot.events }]]
                : [],
            ),
          ),
        );
      })
      .catch(() => {
        // Older/preview bridges have no active native process registry.
      });

    return () => {
      active = false;
    };
  }, [accessGranted, loadActiveConversationTasks]);

  useEffect(() => {
    if (!accessGranted) return;
    if (projectState === "checking") return;
    let active = true;
    const resetReview = (state: GitViewState) => {
      void Promise.resolve().then(() => {
        if (!active) return;
        setGitState(state);
        setGitSnapshot(scaffoldGitWorkspace);
        setGitDiff(null);
        setGitSelectedRequest(null);
        setGitActionError(false);
        setGitMutationPreview(null);
        setGitMutationResult(null);
      });
    };
    if (projectState === "preview") {
      resetReview("preview");
      return () => {
        active = false;
      };
    }
    const project =
      projects.projects.find(
        (candidate) =>
          candidate.id === selectedProjectId && !candidate.archived,
      ) ??
      projects.projects.find((candidate) => !candidate.archived) ??
      projects.projects[0];
    if (!project) {
      resetReview("native");
      return () => {
        active = false;
      };
    }

    void Promise.resolve().then(() => {
      if (!active) return;
      setGitState("checking");
      setGitDiff(null);
      setGitSelectedRequest(null);
      setGitActionError(false);
      setGitMutationPreview(null);
      setGitMutationResult(null);
    });
    void loadGitStatusTask(project.id)
      .then((result) => {
        if (!active) return;
        setGitSnapshot(result);
        setGitState("native");
      })
      .catch(() => {
        if (!active) return;
        setGitSnapshot(scaffoldGitWorkspace);
        setGitState("preview");
      });
    return () => {
      active = false;
    };
  }, [
    accessGranted,
    loadGitStatusTask,
    projectState,
    projects,
    selectedProjectId,
  ]);

  useEffect(() => {
    if (!accessGranted || workspaceLocation.route !== "project-state") return;
    let active = true;
    const resetRepositoryState = (
      state: ProjectStateViewState,
      snapshot: RepositoryStateReadSnapshot | null = null,
    ) => {
      void Promise.resolve().then(() => {
        if (!active) return;
        setRepositoryStateSnapshot(snapshot);
        setRepositoryStateViewState(state);
      });
    };
    if (projectState === "checking") {
      resetRepositoryState("checking");
      return () => {
        active = false;
      };
    }
    if (projectState === "preview") {
      resetRepositoryState("preview");
      return () => {
        active = false;
      };
    }
    if (!currentProject) {
      resetRepositoryState("idle");
      return () => {
        active = false;
      };
    }

    resetRepositoryState("checking");
    void loadRepositoryStateTask({
      projectId: currentProject.id,
      remoteMode: "local-only",
      artifactVerification: "metadata-only",
    })
      .then((result) => {
        if (!active) return;
        resetRepositoryState("native", result);
      })
      .catch(() => {
        if (!active) return;
        resetRepositoryState("error");
      });
    return () => {
      active = false;
    };
  }, [
    accessGranted,
    currentProject,
    loadRepositoryStateTask,
    projectState,
    repositoryStateRefresh,
    workspaceLocation.route,
  ]);

  useEffect(() => {
    if (!accessGranted || workspaceLocation.route !== "advisor") return;
    let active = true;
    const resetAdvisor = (
      state: AdvisorViewState,
      snapshot: AdvisorWorkspaceSnapshot | null = null,
    ) => {
      void Promise.resolve().then(() => {
        if (!active) return;
        setAdvisorSnapshot(snapshot);
        setAdvisorViewState(state);
      });
    };
    if (bridgeState === "preview") {
      resetAdvisor("preview");
      return () => {
        active = false;
      };
    }
    resetAdvisor("checking");
    void loadAdvisorSnapshotTask()
      .then((snapshot) => {
        if (!active) return;
        resetAdvisor("native", snapshot);
      })
      .catch(() => {
        if (!active) return;
        resetAdvisor("error");
      });
    return () => {
      active = false;
    };
  }, [
    accessGranted,
    bridgeState,
    loadAdvisorSnapshotTask,
    workspaceLocation.route,
  ]);

  useEffect(() => {
    if (!accessGranted) return;
    if (projectState === "checking") return;
    let active = true;
    const resetWorktrees = (state: WorktreeViewState) => {
      void Promise.resolve().then(() => {
        if (!active) return;
        setWorktreeState(state);
        setWorktrees(scaffoldWorktreeWorkspace);
        setWorktreePreview(null);
        setWorktreeResult(null);
        setWorktreeActionError(false);
      });
    };
    if (projectState === "preview") {
      resetWorktrees("preview");
      return () => {
        active = false;
      };
    }
    const project =
      projects.projects.find(
        (candidate) =>
          candidate.id === selectedProjectId && !candidate.archived,
      ) ??
      projects.projects.find((candidate) => !candidate.archived) ??
      projects.projects[0];
    if (!project) {
      resetWorktrees("native");
      return () => {
        active = false;
      };
    }
    void Promise.resolve().then(() => {
      if (!active) return;
      setWorktreeState("checking");
      setWorktreePreview(null);
      setWorktreeResult(null);
      setWorktreeActionError(false);
    });
    void loadWorktreesTask(project.id)
      .then((result) => {
        if (!active) return;
        setWorktrees(result);
        setWorktreeState("native");
      })
      .catch(() => {
        if (!active) return;
        setWorktrees(scaffoldWorktreeWorkspace);
        setWorktreeState("preview");
      });
    return () => {
      active = false;
    };
  }, [
    accessGranted,
    loadWorktreesTask,
    projectState,
    projects,
    selectedProjectId,
  ]);

  useEffect(() => {
    if (!accessGranted) return;
    let active = true;
    void loadSessions({ projectId: null, searchTerm: null })
      .then((result) => {
        if (!active) return;
        setSessions(result);
        setSessionState("native");
      })
      .catch(() => {
        if (active) setSessionState("preview");
      });

    return () => {
      active = false;
    };
  }, [accessGranted, loadSessions]);

  useEffect(() => {
    if (!accessGranted) return;
    let active = true;
    void loadTerminalsTask()
      .then((result) => {
        if (!active) return;
        setTerminals(result);
        setTerminalState("native");
      })
      .catch(() => {
        if (active) setTerminalState("preview");
      });

    return () => {
      active = false;
    };
  }, [accessGranted, loadTerminalsTask]);

  useEffect(() => {
    if (!accessGranted) return;
    let active = true;
    void loadIntegrationCatalogTask()
      .then((result) => {
        if (!active) return;
        setIntegrationCatalog(result);
        setIntegrationState("native");
      })
      .catch(() => {
        if (active) setIntegrationState("preview");
      });

    return () => {
      active = false;
    };
  }, [accessGranted, loadIntegrationCatalogTask]);

  useEffect(() => {
    if (authState !== "login-pending") return;
    let active = true;
    const poll = window.setInterval(() => {
      void loadAuth()
        .then((result) => {
          if (!active) return;
          setAuth(result);
          setAuthState(result.state);
        })
        .catch(() => {
          if (active) setAuthState("unavailable");
        });
    }, 750);

    return () => {
      active = false;
      window.clearInterval(poll);
    };
  }, [authState, loadAuth]);

  const activeConversationIds = Object.values(trackedConversations)
    .map((tracked) => tracked.snapshot)
    .filter((snapshot) =>
      ["running", "waiting-for-approval", "stopping"].includes(snapshot.state),
    )
    .flatMap((snapshot) =>
      snapshot.conversationId ? [snapshot.conversationId] : [],
    )
    .sort();
  const activeConversationKey = activeConversationIds.join(",");
  const activeTaskProjectIds = Object.values(trackedConversations)
    .map((tracked) => tracked.snapshot)
    .filter((snapshot) =>
      ["running", "waiting-for-approval", "stopping"].includes(snapshot.state),
    )
    .flatMap((snapshot) => (snapshot.projectId ? [snapshot.projectId] : []))
    .sort();
  const activeTaskProjectKey = activeTaskProjectIds.join(",");

  useEffect(() => {
    if (!activeConversationKey) return;

    let active = true;
    let timer: number | undefined;
    const ids = activeConversationKey.split(",");
    const observed = observedConversationStates.current;
    observedConversationStates.current = Object.fromEntries(
      ids.flatMap((conversationId) =>
        observed[conversationId]
          ? [[conversationId, observed[conversationId]]]
          : [],
      ),
    );

    async function poll() {
      const pollGenerations = Object.fromEntries(
        ids.map((conversationId) => [
          conversationId,
          conversationActionGenerations.current[conversationId] ?? 0,
        ]),
      );
      const settled = await Promise.allSettled(
        ids.map((conversationId) => pollConversationTask(conversationId)),
      );
      if (!active) return;
      const results = settled.flatMap((result, index) =>
        result.status === "fulfilled" &&
        pollGenerations[ids[index]!] ===
          (conversationActionGenerations.current[ids[index]!] ?? 0)
          ? [result.value]
          : [],
      );
      if (settled.some((result) => result.status === "rejected"))
        setConversationActionError("native-command-failed");

      for (const result of results) {
        if (!result.conversationId) continue;
        const previous =
          observedConversationStates.current[result.conversationId];
        observedConversationStates.current[result.conversationId] =
          result.state;
        if (
          previous !== result.state &&
          ["waiting-for-approval", "completed", "blocked", "failed"].includes(
            result.state,
          )
        ) {
          void notifyConversationTask(result.conversationId).catch(() => {
            // Notification delivery is best-effort and never changes task state.
          });
        }
      }

      setTrackedConversations((current) => {
        const next = { ...current };
        for (const result of results) {
          if (!result.projectId || !result.conversationId) continue;
          const previous = current[result.projectId];
          if (
            previous &&
            previous.snapshot.conversationId !== result.conversationId
          )
            continue;
          next[result.projectId] = {
            snapshot: result,
            events: mergeConversationEvents(
              previous?.events ?? [],
              result.events,
            ),
          };
        }
        return next;
      });

      const displayed = results.find(
        (result) => result.conversationId === conversation.conversationId,
      );
      if (displayed) {
        setConversation(displayed);
        setConversationEvents((current) =>
          mergeConversationEvents(current, displayed.events),
        );
      }
      if (
        results.some(
          (result) =>
            !["running", "waiting-for-approval", "stopping"].includes(
              result.state,
            ),
        )
      ) {
        void loadSessions({ projectId: null, searchTerm: sessionSearchTerm })
          .then((sessionResult) => setSessions(sessionResult))
          .catch(() => setSessionActionError(true));
        for (const result of results) {
          if (
            result.projectId &&
            !["running", "waiting-for-approval", "stopping"].includes(
              result.state,
            )
          ) {
            void loadGitStatusTask(result.projectId)
              .then((gitResult) => {
                if (gitResult.projectId) {
                  setTaskGitSnapshots((current) => ({
                    ...current,
                    [gitResult.projectId!]: gitResult,
                  }));
                }
              })
              .catch(() => setGitActionError(true));
          }
        }
      }
      timer = window.setTimeout(() => void poll(), 250);
    }

    timer = window.setTimeout(() => void poll(), 250);
    return () => {
      active = false;
      if (timer !== undefined) window.clearTimeout(timer);
    };
  }, [
    activeConversationKey,
    conversation.conversationId,
    loadGitStatusTask,
    loadSessions,
    notifyConversationTask,
    pollConversationTask,
    sessionSearchTerm,
  ]);

  useEffect(() => {
    if (!activeTaskProjectKey) return;
    let active = true;
    const projectIds = activeTaskProjectKey.split(",");

    async function refreshTaskGit() {
      const settled = await Promise.allSettled(
        projectIds.map((projectId) => loadGitStatusTask(projectId)),
      );
      if (!active) return;
      setTaskGitSnapshots((current) => {
        const next = { ...current };
        for (const [index, result] of settled.entries()) {
          if (result.status === "fulfilled" && result.value.projectId) {
            next[result.value.projectId] = result.value;
          } else if (result.status === "rejected") {
            delete next[projectIds[index]!];
          }
        }
        return next;
      });
    }

    void refreshTaskGit();
    const interval = window.setInterval(() => void refreshTaskGit(), 2000);
    return () => {
      active = false;
      window.clearInterval(interval);
    };
  }, [activeTaskProjectKey, loadGitStatusTask]);

  useEffect(() => {
    applyAppearanceTheme(themePreview ?? theme);
  }, [theme, themePreview]);

  useEffect(() => {
    window.localStorage.setItem(appearanceThemeStorageKey, theme);
  }, [theme]);

  useEffect(() => {
    window.localStorage.setItem(conversationModeStorageKey, conversationMode);
  }, [conversationMode]);

  useEffect(() => {
    const synchronizeLocation = () => {
      const next =
        parseWorkspaceHash(window.location.hash) ?? defaultWorkspaceLocation;
      setWorkspaceLocation(next);
      setMobileNavigationOpen(false);
    };
    const initialHash = parseWorkspaceHash(window.location.hash);
    if (!initialHash) {
      window.history.replaceState(
        null,
        "",
        workspaceLocationHash(initialWorkspaceLocation()),
      );
    }
    window.addEventListener("hashchange", synchronizeLocation);
    window.addEventListener("popstate", synchronizeLocation);
    return () => {
      window.removeEventListener("hashchange", synchronizeLocation);
      window.removeEventListener("popstate", synchronizeLocation);
    };
  }, []);

  useEffect(() => {
    const hash = workspaceLocationHash(workspaceLocation);
    window.localStorage.setItem(workspaceStorageKey, hash);
    if (!focusWorkspaceAfterNavigation.current) return;
    focusWorkspaceAfterNavigation.current = false;
    const frame = window.requestAnimationFrame(() => {
      const view = document.querySelector<HTMLElement>(
        `[data-workspace-view="${workspaceLocation.route}"]`,
      );
      const target =
        view?.querySelector<HTMLElement>("[data-workspace-heading]") ?? view;
      target?.focus();
    });
    return () => window.cancelAnimationFrame(frame);
  }, [workspaceLocation]);

  useEffect(() => {
    window.localStorage.setItem(
      inspectorWidthStorageKey,
      String(inspectorWidth),
    );
  }, [inspectorWidth]);

  useEffect(() => {
    window.localStorage.setItem(
      sidebarCompactStorageKey,
      String(sidebarCompact),
    );
  }, [sidebarCompact]);

  useEffect(() => {
    window.localStorage.setItem(
      layoutPreferenceStorageKey,
      JSON.stringify(workbenchLayout),
    );
  }, [workbenchLayout]);

  useEffect(() => () => terminalResizeCleanup.current?.(), []);

  useEffect(() => {
    if (!mobileNavigationOpen) return;
    const previous = document.activeElement as HTMLElement | null;
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      setMobileNavigationOpen(false);
    };
    document.addEventListener("keydown", handleKeyDown);
    const frame = window.requestAnimationFrame(() => {
      document
        .querySelector<HTMLElement>(".sidebar .nav-item--active")
        ?.focus();
    });
    return () => {
      window.cancelAnimationFrame(frame);
      document.removeEventListener("keydown", handleKeyDown);
      if (previous?.isConnected) previous.focus();
    };
  }, [mobileNavigationOpen]);

  const bridgeLabel = {
    connecting: "Checking native bridge",
    native: "Native IPC verified",
    preview: "Browser preview",
  }[bridgeState];

  const runtimeLabel = {
    checking: "Checking Codex adapter",
    ready: "Codex adapter ready",
    degraded: "CLI fallback only",
    unavailable: "Codex unavailable",
    preview: "Native probe unavailable",
  }[runtimeState];

  function navigateWorkspace(
    route: WorkspaceRoute,
    settingsSection: SettingsSection = "general",
  ) {
    const next = workspaceLocationFor(route, settingsSection);
    const nextHash = workspaceLocationHash(next);
    focusWorkspaceAfterNavigation.current = true;
    if (window.location.hash === nextHash) {
      setWorkspaceLocation(next);
    } else {
      window.history.pushState(null, "", nextHash);
      setWorkspaceLocation(next);
    }
    setMobileNavigationOpen(false);
    setInspectorOpen(false);
    setCommandPaletteOpen(false);
    setTerminalDockOpen(false);
    setReviewPanesOpen(false);
  }

  function closeCommandPalette() {
    setCommandPaletteOpen(false);
    window.requestAnimationFrame(() =>
      commandPaletteTriggerRef.current?.focus(),
    );
  }

  function beginTerminalDockResize(event: ReactPointerEvent<HTMLDivElement>) {
    event.preventDefault();
    const resize = (pointerEvent: PointerEvent) =>
      setWorkbenchLayout((current) => ({
        ...current,
        terminalDockHeight: clampLayoutDimension(
          window.innerHeight - pointerEvent.clientY,
          terminalDockHeightMinimum,
          terminalDockHeightMaximum,
        ),
      }));
    const stop = () => {
      document.removeEventListener("pointermove", resize);
      document.removeEventListener("pointerup", stop);
      document.removeEventListener("pointercancel", stop);
      terminalResizeCleanup.current = null;
    };
    terminalResizeCleanup.current?.();
    terminalResizeCleanup.current = stop;
    document.addEventListener("pointermove", resize);
    document.addEventListener("pointerup", stop, { once: true });
    document.addEventListener("pointercancel", stop, { once: true });
  }

  function resizeTerminalDockFromKeyboard(
    event: ReactKeyboardEvent<HTMLDivElement>,
  ) {
    const delta =
      event.key === "ArrowUp" ? 20 : event.key === "ArrowDown" ? -20 : 0;
    if (!delta) return;
    event.preventDefault();
    setWorkbenchLayout((current) => ({
      ...current,
      terminalDockHeight: clampLayoutDimension(
        current.terminalDockHeight + delta,
        terminalDockHeightMinimum,
        terminalDockHeightMaximum,
      ),
    }));
  }

  async function refreshTaskCatalog(request: {
    query: string | null;
    includeArchived: boolean;
    selectedTaskId: string | null;
  }) {
    setTaskCatalogBusy(true);
    if (!currentProject) {
      setTaskCatalog(scaffoldTaskCatalog);
      return;
    }
    try {
      const snapshot = await loadTaskCatalogTask({
        ...request,
        projectId: currentProject.id,
      });
      setTaskCatalog(snapshot);
    } finally {
      setTaskCatalogBusy(false);
    }
  }

  async function applyTaskCatalogMutation(
    mutate: () => Promise<TaskCatalogSnapshot>,
  ) {
    setTaskCatalogBusy(true);
    try {
      const snapshot = await mutate();
      setTaskCatalog(snapshot);
      return snapshot;
    } finally {
      setTaskCatalogBusy(false);
    }
  }

  async function selectDurableTaskPlan(taskId: string, planId: string) {
    if (
      conversationAttachments.state === "ready" &&
      conversationAttachments.projectId
    ) {
      const cleared = await cancelConversationAttachmentsTask({
        projectId: conversationAttachments.projectId,
        attachmentIds: conversationAttachments.attachments.map(
          (attachment) => attachment.attachmentId,
        ),
      });
      setConversationAttachments(cleared);
      setConversationAttachmentActionError(false);
    }
    return applyTaskCatalogMutation(() =>
      selectTaskPlanTask({ taskId, planId }),
    );
  }

  function clearAdvisorTransientState() {
    setAdvisorConversation(scaffoldAdvisorConversation);
    setAdvisorConversationBusy(false);
    setAdvisorProjectStateSnapshot(null);
    setAdvisorProjectStateProjectId(null);
    setAdvisorProjectStateSelection("idle");
    setAdvisorResetToken((current) => current + 1);
    setAcceptedTaskHandoff(null);
  }

  async function prepareAdvisorTaskHandoff(request: TaskHandoffCreateRequest) {
    return prepareAdvisorTaskHandoffNative(request);
  }

  async function openTaskHandoffInQuireforge() {
    if (!hasCurrentWorkspaceBoundaryAcknowledgment()) {
      setPendingTaskHandoffOpen(true);
      setPendingConversationMode("codex");
      return;
    }
    const accepted = await acceptTaskHandoff("advisor-to-quireforge");
    if (accepted.state !== "accepted" || !accepted.brief) return;
    applyConversationModeChange("codex");
    setAcceptedTaskHandoff(accepted);
  }

  async function returnTaskHandoffToAdvisor(summary: string) {
    const handoff = acceptedTaskHandoff;
    if (!handoff?.taskId || !handoff.title || !handoff.originalRequest) return;
    const receipt = await prepareTaskCompletionReceipt({
      taskId: handoff.taskId,
      title: handoff.title,
      originalRequest: handoff.originalRequest,
      summary,
      status: "completed",
    });
    if (receipt.state !== "pending") return;
    const accepted = await acceptTaskHandoff("quireforge-to-advisor");
    if (accepted.state !== "accepted") return;
    setAcceptedTaskHandoff(null);
    applyConversationModeChange("chat");
    // The receipt is intentionally shown only as a transient Advisor draft.
    setAcceptedTaskHandoff(accepted);
  }

  function requestWorkspaceSelection(next: WorkspaceConversationMode) {
    const requested = next === "advisor" ? "chat" : "codex";
    if (requested !== conversationMode) {
      if (hasCurrentWorkspaceBoundaryAcknowledgment()) {
        applyConversationModeChange(requested);
      } else {
        setPendingConversationMode(requested);
      }
      return;
    }
    navigateWorkspace(next === "advisor" ? "advisor" : "conversation");
  }

  function requestConversationWorkspace(route: "advisor" | "conversation") {
    const requested = route === "advisor" ? "chat" : "codex";
    if (requested !== conversationMode) {
      if (hasCurrentWorkspaceBoundaryAcknowledgment()) {
        applyConversationModeChange(requested);
      } else {
        setPendingConversationMode(requested);
      }
      return;
    }
    navigateWorkspace(route);
  }

  function applyConversationModeChange(next: ConversationMode) {
    clearAdvisorTransientState();
    setConversationMode(next);
    setPendingConversationMode(null);
    navigateWorkspace(next === "chat" ? "advisor" : "conversation");
  }

  async function confirmConversationModeChange() {
    if (!pendingConversationMode) return;
    storeWorkspaceBoundaryAcknowledgment();
    if (pendingTaskHandoffOpen && pendingConversationMode === "codex") {
      setPendingTaskHandoffOpen(false);
      const accepted = await acceptTaskHandoff("advisor-to-quireforge");
      if (accepted.state !== "accepted" || !accepted.brief) return;
      applyConversationModeChange("codex");
      setAcceptedTaskHandoff(accepted);
      return;
    }
    applyConversationModeChange(pendingConversationMode);
  }

  function cancelConversationModeChange() {
    setPendingConversationMode(null);
    setPendingTaskHandoffOpen(false);
    window.requestAnimationFrame(() =>
      document
        .querySelector<HTMLButtonElement>(".workspace-selector__trigger")
        ?.focus(),
    );
  }

  function requestAdvisorProjectState() {
    if (!currentProject) return;
    setAdvisorProjectStateSelection("confirming");
  }

  function cancelAdvisorProjectState() {
    setAdvisorProjectStateSelection("idle");
  }

  function confirmAdvisorProjectState() {
    const projectId = currentProject?.id;
    if (!projectId) {
      setAdvisorProjectStateSelection("error");
      return;
    }
    setAdvisorProjectStateSelection("reading");
    void readAdvisorProjectStateSnapshotTask({ projectId })
      .then((snapshot) => {
        setAdvisorProjectStateSnapshot(snapshot);
        setAdvisorProjectStateProjectId(projectId);
        setAdvisorProjectStateSelection("idle");
      })
      .catch(() => {
        setAdvisorProjectStateSnapshot(null);
        setAdvisorProjectStateProjectId(null);
        setAdvisorProjectStateSelection("error");
      });
  }

  function removeAdvisorProjectState() {
    setAdvisorProjectStateSnapshot(null);
    setAdvisorProjectStateProjectId(null);
    setAdvisorProjectStateSelection("idle");
  }

  function beginInspectorResize(event: ReactPointerEvent<HTMLDivElement>) {
    event.preventDefault();
    const resize = (pointerEvent: PointerEvent) => {
      const nextWidth = window.innerWidth - pointerEvent.clientX;
      setInspectorWidth(
        Math.min(
          inspectorWidthMaximum,
          Math.max(inspectorWidthMinimum, nextWidth),
        ),
      );
    };
    const stop = () => {
      document.removeEventListener("pointermove", resize);
      document.removeEventListener("pointerup", stop);
    };
    document.addEventListener("pointermove", resize);
    document.addEventListener("pointerup", stop);
  }

  function resizeInspectorFromKeyboard(
    event: ReactKeyboardEvent<HTMLDivElement>,
  ) {
    const delta =
      event.key === "ArrowLeft" ? 20 : event.key === "ArrowRight" ? -20 : 0;
    if (!delta) return;
    event.preventDefault();
    setInspectorWidth((current) =>
      Math.min(
        inspectorWidthMaximum,
        Math.max(inspectorWidthMinimum, current + delta),
      ),
    );
  }

  async function applyAuthAction(
    action: () => Promise<CodexAuthSnapshot>,
    openBrowser = false,
  ) {
    setAuthBusy(true);
    setAuthActionError(false);
    try {
      const result = await action();
      setAuth(result);
      setAuthState(result.state);
      if (openBrowser && result.state === "login-pending") {
        await openAuthBrowser();
      }
    } catch {
      setAuthActionError(true);
    } finally {
      setAuthBusy(false);
    }
  }

  function beginLogin(method: AuthLoginMethod) {
    void applyAuthAction(() => startAuth(method), true);
  }

  async function refreshUsageStatus() {
    setUsageBusy(true);
    setUsage(unavailableCodexUsage);
    setUsageState("checking");
    try {
      const result = await refreshUsage();
      setUsage(result);
      setUsageState("native");
    } catch {
      setUsage(unavailableCodexUsage);
      setUsageState("unavailable");
    } finally {
      setUsageBusy(false);
    }
  }

  function discardConversationAttachmentDraft() {
    if (
      conversationAttachments.state === "ready" &&
      conversationAttachments.projectId
    ) {
      void cancelConversationAttachmentsTask({
        projectId: conversationAttachments.projectId,
        attachmentIds: conversationAttachments.attachments.map(
          (attachment) => attachment.attachmentId,
        ),
      }).catch(() => {
        // Native expiry/startup cleanup remains the fail-closed fallback.
      });
    }
    setConversationAttachments(scaffoldConversationAttachments);
    setConversationAttachmentActionError(false);
  }

  function discardFilePreview() {
    if (filePreview.state === "ready" && filePreview.openActionId) {
      void cancelFilePreviewTask({
        openActionId: filePreview.openActionId,
      }).catch(() => {
        // Native expiry and bounded eviction remain the fail-closed fallback.
      });
    }
    setFilePreview(scaffoldFilePreview);
    setFilePreviewActionError(false);
  }

  async function applyProjectAction(
    action: () => Promise<ProjectWorkspaceSnapshot>,
  ) {
    setProjectBusy(true);
    setProjectActionError(false);
    try {
      const result = await action();
      setProjects(result);
      setProjectPreflights({});
      removeAdvisorProjectState();
      if (
        conversationAttachments.state === "ready" &&
        !result.projects.some(
          (project) =>
            project.id === conversationAttachments.projectId &&
            !project.archived &&
            project.directory?.state === "connected-accessible",
        )
      ) {
        discardConversationAttachmentDraft();
      }
      if (
        filePreview.state === "ready" &&
        filePreview.projectId &&
        !result.projects.some(
          (project) =>
            project.id === filePreview.projectId &&
            !project.archived &&
            ["connected-accessible", "connected-read-only"].includes(
              project.directory?.state ?? "",
            ),
        )
      ) {
        discardFilePreview();
      }
    } catch {
      setProjectActionError(true);
    } finally {
      setProjectBusy(false);
    }
  }

  async function verifyProject(projectId: string) {
    setProjectBusy(true);
    setProjectActionError(false);
    try {
      const result = await preflightProjectDirectory(projectId);
      setProjectPreflights((current) => ({ ...current, [projectId]: result }));
    } catch {
      setProjectActionError(true);
    } finally {
      setProjectBusy(false);
    }
  }

  async function refreshWorktrees() {
    if (!currentProject) return;
    setWorktreeBusy(true);
    setWorktreeActionError(false);
    try {
      const result = await loadWorktreesTask(currentProject.id);
      setWorktrees(result);
      setWorktreePreview(null);
      setWorktreeResult(null);
      setWorktreeState("native");
    } catch {
      setWorktreeActionError(true);
    } finally {
      setWorktreeBusy(false);
    }
  }

  async function beginWorktreeCreate(branchName: string) {
    if (!currentProject) return;
    setWorktreeBusy(true);
    setWorktreeActionError(false);
    setWorktreeResult(null);
    try {
      setWorktreePreview(
        await previewWorktreeCreateTask({
          projectId: currentProject.id,
          branchName,
        }),
      );
    } catch {
      setWorktreePreview(null);
      setWorktreeActionError(true);
    } finally {
      setWorktreeBusy(false);
    }
  }

  async function beginWorktreeAttach() {
    if (!currentProject) return;
    setWorktreeBusy(true);
    setWorktreeActionError(false);
    setWorktreeResult(null);
    try {
      const preview = await pickWorktreeAttachTask(currentProject.id);
      setWorktreePreview(preview.state === "cancelled" ? null : preview);
    } catch {
      setWorktreePreview(null);
      setWorktreeActionError(true);
    } finally {
      setWorktreeBusy(false);
    }
  }

  async function beginWorktreeRecover(recoveryId: string) {
    if (!currentProject) return;
    setWorktreeBusy(true);
    setWorktreeActionError(false);
    setWorktreeResult(null);
    try {
      setWorktreePreview(
        await previewWorktreeRecoverTask({
          projectId: currentProject.id,
          recoveryId,
        }),
      );
    } catch {
      setWorktreePreview(null);
      setWorktreeActionError(true);
    } finally {
      setWorktreeBusy(false);
    }
  }

  async function beginWorktreeRemove(worktreeProjectId: string) {
    if (!currentProject) return;
    setWorktreeBusy(true);
    setWorktreeActionError(false);
    setWorktreeResult(null);
    try {
      setWorktreePreview(
        await previewWorktreeRemoveTask({
          projectId: currentProject.id,
          worktreeProjectId,
        }),
      );
    } catch {
      setWorktreePreview(null);
      setWorktreeActionError(true);
    } finally {
      setWorktreeBusy(false);
    }
  }

  async function applyWorktree(confirmationId: string) {
    setWorktreeBusy(true);
    setWorktreeActionError(false);
    try {
      const result = await confirmWorktreeTask({ confirmationId });
      setWorktreePreview(null);
      setWorktreeResult(result);
      if (result.workspace) {
        setWorktrees(result.workspace);
        setWorktreeState("native");
      }
      if (result.state === "applied") {
        const projectResult = await loadProjects();
        setProjects(projectResult);
        if (result.projectId) selectProject(result.projectId);
      }
    } catch {
      setWorktreeActionError(true);
    } finally {
      setWorktreeBusy(false);
    }
  }

  async function cancelWorktreePreview(confirmationId: string) {
    setWorktreeBusy(true);
    setWorktreeActionError(false);
    try {
      await cancelWorktreeTask({ confirmationId });
      setWorktreePreview(null);
    } catch {
      setWorktreeActionError(true);
    } finally {
      setWorktreeBusy(false);
    }
  }

  async function refreshGitReview() {
    if (!currentProject) return;
    setGitBusy(true);
    setGitActionError(false);
    try {
      const result = await loadGitStatusTask(currentProject.id);
      setGitSnapshot(result);
      setGitDiff(null);
      setGitSelectedRequest(null);
      setGitState("native");
      setGitMutationPreview(null);
      setGitMutationResult(null);
    } catch {
      setGitActionError(true);
    } finally {
      setGitBusy(false);
    }
  }

  async function reviewGitDiff(request: GitDiffRequest) {
    setGitBusy(true);
    setGitActionError(false);
    setGitSelectedRequest(request);
    setGitDiff(null);
    try {
      setGitDiff(await loadGitDiffTask(request));
    } catch {
      setGitActionError(true);
    } finally {
      setGitBusy(false);
    }
  }

  async function openReviewedGitFile(projectId: string, path: string) {
    setGitBusy(true);
    setGitActionError(false);
    try {
      await openGitFileTask({ projectId, path });
    } catch {
      setGitActionError(true);
    } finally {
      setGitBusy(false);
    }
  }

  async function beginGitMutation(request: GitMutationPreviewRequest) {
    setGitBusy(true);
    setGitActionError(false);
    setGitMutationResult(null);
    try {
      setGitMutationPreview(await previewGitMutationTask(request));
    } catch {
      setGitMutationPreview(null);
      setGitActionError(true);
    } finally {
      setGitBusy(false);
    }
  }

  async function applyGitMutation(confirmationId: string) {
    setGitBusy(true);
    setGitActionError(false);
    try {
      const result = await confirmGitMutationTask({ confirmationId });
      setGitMutationPreview(null);
      setGitMutationResult(result);
      if (result.workspace) {
        setGitSnapshot(result.workspace);
        setGitDiff(null);
        setGitSelectedRequest(null);
        setGitState("native");
      }
    } catch {
      setGitActionError(true);
    } finally {
      setGitBusy(false);
    }
  }

  async function recoverGitRevert(recoveryId: string) {
    setGitBusy(true);
    setGitActionError(false);
    try {
      const result = await recoverGitMutationTask({ recoveryId });
      setGitMutationResult(result);
      if (result.workspace) {
        setGitSnapshot(result.workspace);
        setGitDiff(null);
        setGitSelectedRequest(null);
      }
    } catch {
      setGitActionError(true);
    } finally {
      setGitBusy(false);
    }
  }

  function trackConversation(
    snapshot: ConversationSnapshot,
    replaceEvents: boolean,
  ) {
    if (!snapshot.projectId || !snapshot.conversationId) return;
    const projectId = snapshot.projectId;
    setTrackedConversations((current) => {
      const previous = current[projectId];
      return {
        ...current,
        [projectId]: {
          snapshot,
          events: replaceEvents
            ? snapshot.events
            : mergeConversationEvents(previous?.events ?? [], snapshot.events),
        },
      };
    });
  }

  function selectProject(projectId: string) {
    if (
      conversationAttachments.state === "ready" &&
      conversationAttachments.projectId &&
      conversationAttachments.projectId !== projectId
    ) {
      discardConversationAttachmentDraft();
    }
    setSelectedProjectId(projectId);
    removeAdvisorProjectState();
    discardFilePreview();
    const tracked = trackedConversations[projectId];
    setConversation(tracked?.snapshot ?? scaffoldConversation);
    setConversationEvents(tracked?.events ?? []);
    setConversationActionError(null);
  }

  async function chooseFilePreview(projectId: string) {
    setFilePreviewBusy(true);
    setFilePreviewActionError(false);
    try {
      setFilePreview(await pickFilePreviewTask(projectId));
    } catch {
      setFilePreview(scaffoldFilePreview);
      setFilePreviewActionError(true);
    } finally {
      setFilePreviewBusy(false);
    }
  }

  async function openSelectedFilePreview(request: FilePreviewHandoffRequest) {
    setFilePreviewBusy(true);
    setFilePreviewActionError(false);
    try {
      await openFilePreviewTask(request);
    } catch (error) {
      try {
        await cancelFilePreviewTask(request);
      } catch {
        // Native expiry and bounded eviction remain the fail-closed fallback.
      }
      setFilePreview(scaffoldFilePreview);
      setFilePreviewActionError(true);
      throw error;
    } finally {
      setFilePreviewBusy(false);
    }
  }

  async function chooseConversationAttachments(projectId: string) {
    setConversationAttachmentBusy(true);
    setConversationAttachmentActionError(false);
    try {
      const result = await pickConversationAttachmentsTask(projectId);
      if (
        result.state === "unavailable" &&
        conversationAttachments.state === "ready" &&
        conversationAttachments.projectId === projectId
      ) {
        setConversationAttachmentActionError(true);
      } else {
        setConversationAttachments(result);
      }
    } catch (error) {
      setConversationAttachmentActionError(true);
      throw error;
    } finally {
      setConversationAttachmentBusy(false);
    }
  }

  async function stageConversationAttachmentDrop(
    request: ConversationAttachmentDropRequest,
  ) {
    setConversationAttachmentBusy(true);
    setConversationAttachmentActionError(false);
    try {
      const result = await stageDroppedConversationAttachmentsTask(request);
      if (
        result.state === "unavailable" &&
        conversationAttachments.state === "ready" &&
        conversationAttachments.projectId === request.projectId
      ) {
        setConversationAttachmentActionError(true);
      } else {
        setConversationAttachments(result);
      }
    } catch (error) {
      setConversationAttachmentActionError(true);
      throw error;
    } finally {
      setConversationAttachmentBusy(false);
    }
  }

  async function removeConversationAttachment(
    projectId: string,
    attachmentId: string,
  ) {
    setConversationAttachmentBusy(true);
    setConversationAttachmentActionError(false);
    try {
      const result = await cancelConversationAttachmentsTask({
        projectId,
        attachmentIds: [attachmentId],
      });
      if (
        result.state === "unavailable" &&
        conversationAttachments.state === "ready" &&
        conversationAttachments.projectId === projectId
      ) {
        setConversationAttachmentActionError(true);
      } else {
        setConversationAttachments(result);
      }
    } catch (error) {
      setConversationAttachmentActionError(true);
      throw error;
    } finally {
      setConversationAttachmentBusy(false);
    }
  }

  async function beginConversation(
    request: ConversationStartRequest,
  ): Promise<ConversationSnapshot> {
    setConversationBusy(true);
    setConversationActionError(null);
    try {
      const result = await startConversationTask(request);
      setConversationAttachments(scaffoldConversationAttachments);
      setConversationAttachmentActionError(false);
      setConversation(result);
      setConversationEvents(result.events);
      trackConversation(result, true);
      return result;
    } catch (error) {
      setConversationActionError(conversationActionFailureCode(error));
      throw error;
    } finally {
      setConversationBusy(false);
    }
  }

  async function dispatchApprovedAdvisorRequest(
    request: Parameters<typeof dispatchAdvisorOnce>[0],
  ) {
    const result = await dispatchAdvisorOnceTask(request);
    if (result.state === "started") {
      const snapshot = await loadConversation();
      setConversation(snapshot);
      setConversationEvents(snapshot.events);
      trackConversation(snapshot, true);
      navigateWorkspace("conversation");
    }
    return result;
  }

  async function stopConversation(
    conversationId: string,
  ): Promise<ConversationSnapshot> {
    conversationActionGenerations.current[conversationId] =
      (conversationActionGenerations.current[conversationId] ?? 0) + 1;
    setConversationBusy(true);
    setConversationActionError(null);
    try {
      const result = await interruptConversationTask(conversationId);
      setConversation(result);
      setConversationEvents((current) =>
        mergeConversationEvents(current, result.events),
      );
      trackConversation(result, false);
      return result;
    } catch (error) {
      setConversationActionError(conversationActionFailureCode(error));
      throw error;
    } finally {
      setConversationBusy(false);
    }
  }

  async function beginAdvisorConversation(
    request: AdvisorConversationStartRequest,
  ): Promise<AdvisorConversationSnapshot> {
    setAdvisorConversationBusy(true);
    try {
      const result = await startAdvisorConversationTask(request);
      setAdvisorConversation(result);
      return result;
    } finally {
      setAdvisorConversationBusy(false);
    }
  }

  async function pollAdvisorConversationById(
    conversationId: string,
  ): Promise<AdvisorConversationSnapshot> {
    const result = await pollAdvisorConversationTask(conversationId);
    setAdvisorConversation((current) =>
      mergeAdvisorConversationSnapshot(current, result),
    );
    return result;
  }

  async function stopAdvisorConversation(
    conversationId: string,
  ): Promise<AdvisorConversationSnapshot> {
    setAdvisorConversationBusy(true);
    try {
      const result = await interruptAdvisorConversationTask(conversationId);
      setAdvisorConversation(result);
      return result;
    } finally {
      setAdvisorConversationBusy(false);
    }
  }

  async function applyConversationApproval(
    request: ConversationApprovalDecisionRequest,
  ): Promise<ConversationSnapshot> {
    conversationActionGenerations.current[request.conversationId] =
      (conversationActionGenerations.current[request.conversationId] ?? 0) + 1;
    setConversationBusy(true);
    setConversationActionError(null);
    try {
      const result = await decideConversationApprovalTask(request);
      setConversation(result);
      setConversationEvents((current) =>
        mergeConversationEvents(current, result.events),
      );
      trackConversation(result, false);
      return result;
    } catch (error) {
      setConversationActionError(conversationActionFailureCode(error));
      throw error;
    } finally {
      setConversationBusy(false);
    }
  }

  async function applyModelSelection(
    request: ModelSelectionUpdateRequest,
  ): Promise<ModelSelectionSnapshot> {
    setConversationBusy(true);
    setSessionBusy(true);
    setConversationActionError(null);
    setSessionActionError(false);
    try {
      const result = await updateModelSelectionTask(request);
      setConversation((current) =>
        current.conversationId === request.conversationId
          ? {
              ...current,
              modelId: result.effective.modelId,
              reasoningEffort: result.effective.reasoningEffort,
              modelSelection: result,
            }
          : current,
      );
      setSessions((current) => ({
        ...current,
        sessions: current.sessions.map((session) =>
          session.conversationId === request.conversationId
            ? {
                ...session,
                modelId: result.effective.modelId,
                reasoningEffort: result.effective.reasoningEffort,
                modelSelection: result,
              }
            : session,
        ),
      }));
      return result;
    } catch (error) {
      setConversationActionError(conversationActionFailureCode(error));
      setSessionActionError(true);
      throw error;
    } finally {
      setConversationBusy(false);
      setSessionBusy(false);
    }
  }

  async function refreshSessions(
    request: SessionListRequest = {
      projectId: null,
      searchTerm: sessionSearchTerm,
    },
  ) {
    setSessionBusy(true);
    setSessionActionError(false);
    try {
      const result = await loadSessions(request);
      setSessions(result);
      setSessionSearchTerm(request.searchTerm);
      setSessionState("native");
    } catch (error) {
      setSessionActionError(true);
      throw error;
    } finally {
      setSessionBusy(false);
    }
  }

  async function continueHistoricalConversation(
    action: (
      request: ConversationContinueRequest,
    ) => Promise<ConversationSnapshot>,
    request: ConversationContinueRequest,
  ): Promise<ConversationSnapshot> {
    setConversationBusy(true);
    setSessionBusy(true);
    setConversationActionError(null);
    setSessionActionError(false);
    try {
      const source = sessions.sessions.find(
        (session) => session.conversationId === request.conversationId,
      );
      if (source) selectProject(source.projectId);
      const result = await action(request);
      setConversationAttachments(scaffoldConversationAttachments);
      setConversationAttachmentActionError(false);
      setConversation(result);
      setConversationEvents(result.events);
      trackConversation(result, true);
      if (result.state === "unavailable") setSessionActionError(true);
      return result;
    } catch (error) {
      setConversationActionError(conversationActionFailureCode(error));
      setSessionActionError(true);
      throw error;
    } finally {
      setConversationBusy(false);
      setSessionBusy(false);
    }
  }

  async function mutateSession(
    action: () => Promise<SessionLifecycleSnapshot>,
  ) {
    setSessionBusy(true);
    setSessionActionError(false);
    try {
      const mutation = await action();
      if (mutation.state === "unavailable") {
        setSessionActionError(true);
        return;
      }
      const result = await loadSessions({
        projectId: null,
        searchTerm: sessionSearchTerm,
      });
      setSessions(result);
    } catch (error) {
      setSessionActionError(true);
      throw error;
    } finally {
      setSessionBusy(false);
    }
  }

  function trackTerminal(snapshot: TerminalSnapshot) {
    if (!snapshot.terminalId) return;
    setTerminals((current) => {
      const reviewed = { ...snapshot, output: [] };
      const index = current.terminals.findIndex(
        (terminal) => terminal.terminalId === snapshot.terminalId,
      );
      const next = [...current.terminals];
      if (index === -1) next.push(reviewed);
      else next[index] = reviewed;
      return { ...current, terminals: next, diagnosticCode: null };
    });
  }

  async function beginTerminal(
    request: TerminalStartRequest,
  ): Promise<TerminalSnapshot> {
    setTerminalBusy(true);
    setTerminalActionError(false);
    try {
      const result = await startTerminalTask(request);
      if (result.state === "unavailable") setTerminalActionError(true);
      else trackTerminal(result);
      return result;
    } catch (error) {
      setTerminalActionError(true);
      throw error;
    } finally {
      setTerminalBusy(false);
    }
  }

  async function pollActiveTerminal(
    request: TerminalPollRequest,
  ): Promise<TerminalSnapshot> {
    try {
      return await pollTerminalTask(request);
    } catch (error) {
      setTerminalActionError(true);
      throw error;
    }
  }

  async function writeActiveTerminal(
    request: TerminalWriteRequest,
  ): Promise<TerminalSnapshot> {
    try {
      return await writeTerminalTask(request);
    } catch (error) {
      setTerminalActionError(true);
      throw error;
    }
  }

  async function resizeActiveTerminal(
    request: TerminalResizeRequest,
  ): Promise<TerminalSnapshot> {
    try {
      return await resizeTerminalTask(request);
    } catch (error) {
      setTerminalActionError(true);
      throw error;
    }
  }

  async function endTerminal(
    request: TerminalCloseRequest,
  ): Promise<TerminalRegistrySnapshot> {
    setTerminalBusy(true);
    setTerminalActionError(false);
    try {
      const result = await closeTerminalTask(request);
      setTerminals(result);
      if (result.diagnosticCode) setTerminalActionError(true);
      return result;
    } catch (error) {
      setTerminalActionError(true);
      throw error;
    } finally {
      setTerminalBusy(false);
    }
  }

  async function refreshIntegrationCatalog() {
    if (
      integrationControlResult?.state === "handoff-ready" ||
      integrationControlResult?.state === "pending"
    )
      return;
    setIntegrationBusy(true);
    setIntegrationActionError(false);
    setIntegrationPreview(null);
    setIntegrationControlPreview(null);
    try {
      const result = await refreshIntegrationCatalogTask();
      setIntegrationCatalog(result);
      setIntegrationResult(null);
      setIntegrationControlResult(null);
      setIntegrationState("native");
    } catch (error) {
      setIntegrationActionError(true);
      throw error;
    } finally {
      setIntegrationBusy(false);
    }
  }

  async function beginIntegrationMutation(
    request: IntegrationMutationPreviewRequest,
  ) {
    if (
      integrationControlResult?.state === "handoff-ready" ||
      integrationControlResult?.state === "pending"
    )
      return;
    setIntegrationBusy(true);
    setIntegrationActionError(false);
    setIntegrationResult(null);
    setIntegrationControlPreview(null);
    setIntegrationControlResult(null);
    try {
      setIntegrationPreview(await previewIntegrationMutationTask(request));
    } catch (error) {
      setIntegrationPreview(null);
      setIntegrationActionError(true);
      throw error;
    } finally {
      setIntegrationBusy(false);
    }
  }

  async function applyIntegrationMutation(confirmationId: string) {
    setIntegrationBusy(true);
    setIntegrationActionError(false);
    try {
      const result = await confirmIntegrationMutationTask({ confirmationId });
      setIntegrationPreview(null);
      if (result.state === "applied" && result.catalogRefreshRequired) {
        setIntegrationCatalog(await loadIntegrationCatalogTask());
        setIntegrationState("native");
      }
      setIntegrationResult(result);
    } catch (error) {
      setIntegrationActionError(true);
      throw error;
    } finally {
      setIntegrationBusy(false);
    }
  }

  async function beginIntegrationControl(
    request: IntegrationControlPreviewRequest,
  ) {
    if (
      integrationControlResult?.state === "handoff-ready" ||
      integrationControlResult?.state === "pending"
    )
      return;
    setIntegrationBusy(true);
    setIntegrationActionError(false);
    setIntegrationPreview(null);
    setIntegrationResult(null);
    setIntegrationControlResult(null);
    try {
      setIntegrationControlPreview(
        await previewIntegrationControlTask(request),
      );
    } catch (error) {
      setIntegrationControlPreview(null);
      setIntegrationActionError(true);
      throw error;
    } finally {
      setIntegrationBusy(false);
    }
  }

  async function applyIntegrationControl(confirmationId: string) {
    setIntegrationBusy(true);
    setIntegrationActionError(false);
    try {
      const result = await confirmIntegrationControlTask({ confirmationId });
      setIntegrationControlPreview(null);
      if (result.catalogRefreshRequired) {
        setIntegrationCatalog(await loadIntegrationCatalogTask());
        setIntegrationState("native");
      }
      setIntegrationControlResult(result);
    } catch (error) {
      setIntegrationActionError(true);
      throw error;
    } finally {
      setIntegrationBusy(false);
    }
  }

  async function openIntegrationControl(actionId: string) {
    setIntegrationBusy(true);
    setIntegrationActionError(false);
    try {
      setIntegrationControlResult(
        await openIntegrationControlTask({ actionId }),
      );
    } catch (error) {
      setIntegrationActionError(true);
      throw error;
    } finally {
      setIntegrationBusy(false);
    }
  }

  async function checkIntegrationControl(actionId: string) {
    setIntegrationBusy(true);
    setIntegrationActionError(false);
    try {
      const result = await pollIntegrationControlTask({ actionId });
      if (result.catalogRefreshRequired) {
        setIntegrationCatalog(await loadIntegrationCatalogTask());
        setIntegrationState("native");
      }
      setIntegrationControlResult(result);
    } catch (error) {
      setIntegrationActionError(true);
      throw error;
    } finally {
      setIntegrationBusy(false);
    }
  }

  if (!accessGranted) {
    return (
      <AuthGate
        state={authState}
        snapshot={auth}
        busy={authBusy}
        actionError={authActionError}
        cliVersion={runtime.cliVersion}
        nextThemeLabel={
          appearanceThemes.find(({ id }) => id === nextAppearanceTheme(theme))
            ?.label ?? "Forge"
        }
        onThemeChange={() => setTheme(nextAppearanceTheme(theme))}
        onStart={beginLogin}
        onOpenBrowser={() => {
          setAuthActionError(false);
          void openAuthBrowser().catch(() => setAuthActionError(true));
        }}
        onCancel={() => void applyAuthAction(cancelAuth)}
        onRefresh={() => void applyAuthAction(refreshAuth)}
      />
    );
  }

  const conversationActive = [
    "running",
    "waiting-for-approval",
    "stopping",
  ].includes(conversation.state);
  const visibleWorktreeProjects = new Set(
    worktrees.worktrees.flatMap((worktree) =>
      worktree.projectId ? [worktree.projectId] : [],
    ),
  );
  const worktreeExecutions = Object.values(trackedConversations)
    .flatMap(({ snapshot }) => {
      if (
        !snapshot.projectId ||
        !snapshot.conversationId ||
        !visibleWorktreeProjects.has(snapshot.projectId) ||
        snapshot.state === "empty" ||
        snapshot.state === "unavailable"
      )
        return [];
      const project = projects.projects.find(
        (candidate) => candidate.id === snapshot.projectId,
      );
      const git = taskGitSnapshots[snapshot.projectId];
      const gitReady = git && git.state !== "unavailable";
      return [
        {
          projectId: snapshot.projectId,
          projectName: project?.displayName ?? "Attached worktree",
          conversationId: snapshot.conversationId,
          state: snapshot.state,
          changeCount: gitReady ? git.changes.length : null,
          conflictCount: gitReady
            ? git.changes.filter((change) => change.conflict).length
            : null,
        } satisfies WorktreeExecutionView,
      ];
    })
    .sort((left, right) => left.projectName.localeCompare(right.projectName));

  const activeNavigationItem = workspaceNavigationItem(workspaceLocation.route);
  const activeWorkspaceTitle =
    workspaceLocation.route === "settings"
      ? "Settings"
      : (activeNavigationItem?.label ?? "Home");
  const activeWorkspaceDescription =
    workspaceLocation.route === "settings"
      ? "Local preferences and supported connections"
      : (activeNavigationItem?.description ??
        "Dashboard and starting workspace");
  const recentSessions = [...sessions.sessions]
    .sort((left, right) => right.updatedAtMs - left.updatedAtMs)
    .slice(0, 5);
  const activeProjectPath = currentProject?.directory?.displayPath ?? null;
  const activeProjectState =
    currentProject?.directory?.state ?? "No directory attached";
  const conflictCount = gitSnapshot.changes.filter(
    (change) => change.conflict,
  ).length;
  const runningTerminalCount = terminals.terminals.filter(
    (terminal) => terminal.live,
  ).length;
  const inspectorContent: ReactNode = (() => {
    switch (workspaceLocation.route) {
      case "home":
        return (
          <>
            <section
              className="context-section"
              aria-labelledby="context-recent-title"
            >
              <div className="context-section__heading">
                <div>
                  <span>History</span>
                  <h2 id="context-recent-title">Recent threads</h2>
                </div>
                <button
                  type="button"
                  onClick={() => navigateWorkspace("sessions")}
                >
                  View all
                </button>
              </div>
              {recentSessions.length ? (
                <ul className="context-list">
                  {recentSessions.map((session) => (
                    <li key={session.conversationId}>
                      <button
                        type="button"
                        onClick={() => navigateWorkspace("sessions")}
                      >
                        <strong>
                          {session.title ?? "Untitled Codex task"}
                        </strong>
                        <span>
                          {new Intl.DateTimeFormat(undefined, {
                            month: "short",
                            day: "numeric",
                            hour: "numeric",
                            minute: "2-digit",
                          }).format(session.updatedAtMs)}
                        </span>
                      </button>
                    </li>
                  ))}
                </ul>
              ) : (
                <p className="context-empty">
                  Completed and resumable threads will appear here.
                </p>
              )}
            </section>
            <UsagePanel
              snapshot={usage}
              state={usageState}
              busy={usageBusy}
              onRefresh={() => void refreshUsageStatus()}
            />
          </>
        );
      case "conversation":
        return (
          <>
            <div
              className="workbench-context-tabs"
              role="tablist"
              aria-label="Workbench context"
            >
              {(["diff", "git", "problems"] as const).map((tab) => (
                <button
                  type="button"
                  role="tab"
                  key={tab}
                  aria-selected={workbenchDrawerTab === tab}
                  onClick={() => setWorkbenchDrawerTab(tab)}
                >
                  {tab === "diff" ? "Diff" : tab === "git" ? "Git" : "Problems"}
                </button>
              ))}
            </div>
            {workbenchDrawerTab === "diff" ? (
              <section className="context-section" role="tabpanel">
                <p className="eyebrow">Selected diff</p>
                <h2>{gitSelectedRequest?.path ?? "No diff selected"}</h2>
                <p className="context-note">
                  {gitSelectedRequest
                    ? "Open Changes for the bounded reviewed diff."
                    : "Select a changed file in Changes to review its diff."}
                </p>
              </section>
            ) : workbenchDrawerTab === "git" ? (
              <section className="context-section" role="tabpanel">
                <p className="eyebrow">Git status</p>
                <h2>{currentProject?.displayName ?? "No project selected"}</h2>
                <dl className="context-facts">
                  <div>
                    <dt>Branch</dt>
                    <dd>{gitSnapshot.branch?.head ?? "Unavailable"}</dd>
                  </div>
                  <div>
                    <dt>Changes</dt>
                    <dd>
                      {gitSnapshot.state === "unavailable"
                        ? "Unavailable"
                        : gitSnapshot.changes.length}
                    </dd>
                  </div>
                  <div>
                    <dt>Conflicts</dt>
                    <dd>{conflictCount}</dd>
                  </div>
                </dl>
              </section>
            ) : (
              <section className="context-section" role="tabpanel">
                <p className="eyebrow">Problems</p>
                <h2>No problem feed available</h2>
                <p className="context-note">
                  QuireForge does not invent diagnostics. Use the available
                  review and approval surfaces for current task evidence.
                </p>
              </section>
            )}
          </>
        );
      case "projects":
        if (!currentProject) return null;
        return (
          <section className="context-section">
            <p className="eyebrow">Selected project</p>
            <h2>{currentProject.displayName}</h2>
            <p className="context-path">
              {activeProjectPath ?? "No directory attached"}
            </p>
            <dl className="context-facts">
              <div>
                <dt>Access</dt>
                <dd>{activeProjectState}</dd>
              </div>
              <div>
                <dt>Git</dt>
                <dd>
                  {currentProject.directory?.git.isRepository
                    ? "Repository"
                    : "Not detected"}
                </dd>
              </div>
              <div>
                <dt>Guidance</dt>
                <dd>
                  {currentProject.directory?.hasAgentsGuidance
                    ? "AGENTS.md detected"
                    : "No AGENTS.md detected"}
                </dd>
              </div>
            </dl>
          </section>
        );
      case "sessions":
        if (sessions.sessions.length === 0) return null;
        return (
          <section className="context-section">
            <p className="eyebrow">Thread library</p>
            <h2>{sessions.sessions.length} app-owned threads</h2>
            <dl className="context-facts">
              <div>
                <dt>Running</dt>
                <dd>
                  {
                    sessions.sessions.filter(
                      (session) => session.state === "running",
                    ).length
                  }
                </dd>
              </div>
              <div>
                <dt>Archived</dt>
                <dd>
                  {
                    sessions.sessions.filter(
                      (session) => session.state === "archived",
                    ).length
                  }
                </dd>
              </div>
              <div>
                <dt>Authority</dt>
                <dd>Codex threads</dd>
              </div>
            </dl>
          </section>
        );
      case "integrations":
        if (integrationCatalog.entries.length === 0) return null;
        return (
          <section className="context-section">
            <p className="eyebrow">Integration catalog</p>
            <h2>{integrationCatalog.entries.length} discovered entries</h2>
            <dl className="context-facts">
              <div>
                <dt>Catalog</dt>
                <dd>{integrationCatalog.catalogState}</dd>
              </div>
              <div>
                <dt>Codex CLI</dt>
                <dd>{integrationCatalog.cliVersion ?? "Not reported"}</dd>
              </div>
              <div>
                <dt>Control boundary</dt>
                <dd>Preview and confirmation</dd>
              </div>
            </dl>
          </section>
        );
      case "files":
        if (!currentProject) return null;
        return (
          <section className="context-section">
            <p className="eyebrow">File context</p>
            <h2>
              {filePreview.displayPath ??
                (filePreview.state === "empty"
                  ? "No file selected"
                  : "Preview unavailable")}
            </h2>
            <dl className="context-facts">
              <div>
                <dt>Project</dt>
                <dd>{currentProject.displayName}</dd>
              </div>
              <div>
                <dt>Preview</dt>
                <dd>{filePreview.state}</dd>
              </div>
              <div>
                <dt>Rendering</dt>
                <dd>{filePreview.rendering ?? "Not selected"}</dd>
              </div>
            </dl>
          </section>
        );
      case "changes":
        if (!currentProject || gitSnapshot.state === "unavailable") return null;
        return (
          <section className="context-section">
            <p className="eyebrow">Source control</p>
            <h2>{currentProject.displayName}</h2>
            <dl className="context-facts">
              <div>
                <dt>Branch</dt>
                <dd>
                  {gitSnapshot.branch?.head ??
                    (gitSnapshot.branch?.detached
                      ? "Detached HEAD"
                      : "Unknown")}
                </dd>
              </div>
              <div>
                <dt>Changes</dt>
                <dd>{gitSnapshot.changes.length}</dd>
              </div>
              <div>
                <dt>Conflicts</dt>
                <dd>{conflictCount}</dd>
              </div>
              {gitSelectedRequest && (
                <div>
                  <dt>Reviewing</dt>
                  <dd>{gitSelectedRequest.path}</dd>
                </div>
              )}
            </dl>
          </section>
        );
      case "project-state":
        if (!repositoryStateSnapshot) return null;
        return (
          <section className="context-section">
            <p className="eyebrow">Snapshot provenance</p>
            <h2>{repositoryStateSnapshot.state.project.displayName}</h2>
            <dl className="context-facts">
              <div>
                <dt>Trust</dt>
                <dd>{repositoryStateSnapshot.state.provenance.trust}</dd>
              </div>
              <div>
                <dt>Source</dt>
                <dd>{repositoryStateSnapshot.state.provenance.sourceType}</dd>
              </div>
              <div>
                <dt>Diagnostics</dt>
                <dd>{repositoryStateSnapshot.diagnostics.length}</dd>
              </div>
            </dl>
            <p className="context-note">
              This view displays normalized evidence and cannot approve or
              change project state.
            </p>
          </section>
        );
      case "worktrees":
        if (worktrees.state === "unavailable") return null;
        return (
          <section className="context-section">
            <p className="eyebrow">Worktree activity</p>
            <h2>{worktrees.worktrees.length} discovered worktrees</h2>
            <dl className="context-facts">
              <div>
                <dt>Managed</dt>
                <dd>
                  {
                    worktrees.worktrees.filter(
                      (worktree) => worktree.ownership === "managed",
                    ).length
                  }
                </dd>
              </div>
              <div>
                <dt>Active tasks</dt>
                <dd>{worktreeExecutions.length}</dd>
              </div>
              <div>
                <dt>Conflict reports</dt>
                <dd>
                  {worktreeExecutions.reduce(
                    (total, execution) =>
                      total + (execution.conflictCount ?? 0),
                    0,
                  )}
                </dd>
              </div>
            </dl>
          </section>
        );
      case "terminal":
        if (!currentProject && terminals.terminals.length === 0) return null;
        return (
          <section className="context-section">
            <p className="eyebrow">Terminal processes</p>
            <h2>
              {terminals.terminals.length
                ? `${terminals.terminals.length} open terminal${
                    terminals.terminals.length === 1 ? "" : "s"
                  }`
                : "No terminal open"}
            </h2>
            <dl className="context-facts">
              <div>
                <dt>Running</dt>
                <dd>{runningTerminalCount}</dd>
              </div>
              <div>
                <dt>Capacity</dt>
                <dd>
                  {terminals.terminals.length} of {terminals.capacity}
                </dd>
              </div>
              <div>
                <dt>Default project</dt>
                <dd>{currentProject?.displayName ?? "None"}</dd>
              </div>
            </dl>
            <p className="context-note">
              Terminal input uses your Linux account and remains separate from
              Codex approvals.
            </p>
          </section>
        );
      default:
        return null;
    }
  })();
  const inspectorAvailable = inspectorContent !== null;
  const terminalWorkspace = (
    <Suspense
      fallback={
        <section
          className="workspace-loading"
          id="terminal"
          aria-labelledby="terminal-loading-title"
          aria-busy="true"
        >
          <p className="eyebrow">Integrated terminal</p>
          <h2 id="terminal-loading-title">Preparing the terminal view.</h2>
          <p role="status" aria-live="polite">
            Loading the local terminal renderer.
          </p>
        </section>
      }
    >
      <TerminalWorkspace
        theme={themePreview ?? theme}
        availability={terminalState}
        registry={terminals}
        projects={projects}
        busy={terminalBusy}
        actionError={terminalActionError}
        onStart={beginTerminal}
        onPoll={pollActiveTerminal}
        onWrite={writeActiveTerminal}
        onResize={resizeActiveTerminal}
        onClose={endTerminal}
        onSnapshot={trackTerminal}
      />
    </Suspense>
  );

  return (
    <div
      className={[
        "app-shell",
        sidebarCompact ? "app-shell--sidebar-compact" : "",
        mobileNavigationOpen ? "app-shell--mobile-navigation-open" : "",
      ]
        .filter(Boolean)
        .join(" ")}
      style={
        {
          "--inspector-width": `${inspectorWidth}px`,
        } as CSSProperties
      }
    >
      <a
        className="skip-link"
        href="#workspace-main"
        onClick={(event) => {
          event.preventDefault();
          workspaceMainRef.current?.focus();
        }}
      >
        Skip to workspace
      </a>
      <button
        className="sidebar-scrim"
        type="button"
        aria-label="Close navigation"
        tabIndex={mobileNavigationOpen ? 0 : -1}
        onClick={() => setMobileNavigationOpen(false)}
      />
      <aside className="sidebar" aria-label="QuireForge navigation">
        <div className="brand-lockup">
          <img src={brandMark} alt="" className="brand-mark" />
          <div>
            <strong>{bootstrap.product.name}</strong>
            <span>Build boldly. Work locally.</span>
          </div>
          <button
            className="sidebar-close"
            type="button"
            aria-label="Close navigation"
            onClick={() => setMobileNavigationOpen(false)}
          >
            ×
          </button>
        </div>

        <WorkspaceSelector
          mode={workspaceConversationMode(conversationMode)}
          onRequestChange={requestWorkspaceSelection}
        />

        <nav className="primary-nav" aria-label="Workspace navigation">
          {(["chat", "work", "code"] as const).map((lane, laneIndex) => (
            <div className="navigation-lane" key={lane}>
              <p
                className={
                  laneIndex === 0
                    ? "nav-label"
                    : "nav-label nav-label--secondary"
                }
              >
                {lane}
              </p>
              {workspaceNavigation
                .filter((item) => item.lane === lane)
                .map((item) =>
                  item.route === "dynamic-analysis" ? (
                    <button
                      className={
                        workspaceLocation.route === item.route
                          ? "nav-item nav-item--active"
                          : "nav-item"
                      }
                      type="button"
                      key={item.route}
                      aria-current={
                        workspaceLocation.route === item.route
                          ? "page"
                          : undefined
                      }
                      aria-label={item.label}
                      title={item.description}
                      onClick={() => navigateWorkspace(item.route)}
                    >
                      <Glyph name={item.icon} />
                      <span>{item.label}</span>
                    </button>
                  ) : (
                    <a
                      className={
                        workspaceLocation.route === item.route
                          ? "nav-item nav-item--active"
                          : "nav-item"
                      }
                      href={workspaceLocationHash(
                        workspaceLocationFor(item.route),
                      )}
                      key={item.route}
                      aria-current={
                        workspaceLocation.route === item.route
                          ? "page"
                          : undefined
                      }
                      aria-label={item.label}
                      title={item.description}
                      onClick={(event) => {
                        event.preventDefault();
                        if (
                          item.route === "advisor" ||
                          item.route === "conversation"
                        ) {
                          requestConversationWorkspace(item.route);
                        } else {
                          navigateWorkspace(item.route);
                        }
                      }}
                    >
                      <Glyph name={item.icon} />
                      <span>{item.label}</span>
                    </a>
                  ),
                )}
            </div>
          ))}
        </nav>

        <button
          className="project-panel"
          type="button"
          aria-label={
            currentProject
              ? `Open projects. Current project: ${currentProject.displayName}`
              : "Open projects"
          }
          onClick={() => navigateWorkspace("projects")}
        >
          <div className="project-icon">
            <Glyph name="folder" />
          </div>
          <div>
            <strong>
              {currentProject?.displayName ?? "No project attached"}
            </strong>
            <span>
              {currentProject?.directory?.displayPath ??
                (projectState === "preview"
                  ? "Native project access unavailable in browser preview."
                  : "Attach an original local directory in place.")}
            </span>
          </div>
        </button>

        <UsagePanel
          snapshot={usage}
          state={usageState}
          busy={usageBusy}
          compact
          onRefresh={() => void refreshUsageStatus()}
        />

        <button
          className={
            workspaceLocation.route === "settings" &&
            workspaceLocation.settingsSection === "general"
              ? "account-summary account-summary--active"
              : "account-summary"
          }
          type="button"
          aria-label="Open Codex account and connection settings"
          aria-current={
            workspaceLocation.route === "settings" &&
            workspaceLocation.settingsSection === "general"
              ? "page"
              : undefined
          }
          onClick={() => {
            navigateWorkspace("settings", "general");
          }}
        >
          <span aria-hidden="true">Q</span>
          <div>
            <strong>Codex connected</strong>
            <small>
              {auth.accountKind === "chatgpt"
                ? "ChatGPT account"
                : auth.accountKind === "api-key"
                  ? "API key"
                  : "Managed account"}
            </small>
          </div>
          <Glyph name="chevron" />
        </button>

        <div className="sidebar-footer">
          <div className="bridge-status" role="status" aria-live="polite">
            <StatusDot state={bridgeState} />
            <span>{bridgeLabel}</span>
          </div>
          <span className="version">v{bootstrap.product.version}</span>
        </div>
      </aside>

      <main
        className="workspace"
        id="workspace-main"
        tabIndex={-1}
        ref={workspaceMainRef}
      >
        <header className="topbar" data-visual-region="topbar">
          <div className="topbar-location">
            <button
              className="mobile-navigation-toggle"
              type="button"
              aria-label="Open navigation"
              aria-expanded={mobileNavigationOpen}
              onClick={() => setMobileNavigationOpen(true)}
            >
              <Glyph name="grid" />
            </button>
            <button
              className="sidebar-compact-toggle"
              type="button"
              aria-label={
                sidebarCompact ? "Expand navigation" : "Compact navigation"
              }
              aria-pressed={sidebarCompact}
              onClick={() => setSidebarCompact((current) => !current)}
            >
              <Glyph name="sidebar" />
            </button>
            <div className="breadcrumb" aria-label="Current location">
              <Glyph name={activeNavigationItem?.icon ?? "gear"} />
              <div>
                <strong>{activeWorkspaceTitle}</strong>
                <span>{activeWorkspaceDescription}</span>
              </div>
            </div>
          </div>
          <div className="topbar-actions">
            {conversationMode === "codex" && (
              <button
                ref={commandPaletteTriggerRef}
                className="topbar-button topbar-button--workbench-actions"
                type="button"
                aria-haspopup="dialog"
                aria-expanded={commandPaletteOpen}
                onClick={() => setCommandPaletteOpen((current) => !current)}
              >
                ⌘ Actions
              </button>
            )}
            {conversationMode === "codex" &&
              workspaceLocation.route === "conversation" && (
                <button
                  className="topbar-button topbar-button--review-panes"
                  type="button"
                  aria-expanded={reviewPanesOpen}
                  aria-controls="review-panes-title"
                  onClick={() => setReviewPanesOpen((current) => !current)}
                >
                  Review panes
                </button>
              )}
            {workspaceLocation.route === "home" && (
              <button
                className="topbar-button topbar-button--primary"
                type="button"
                onClick={() => requestConversationWorkspace("conversation")}
              >
                <Glyph name="plus" />
                New task
              </button>
            )}
            {workspaceLocation.route === "projects" && (
              <button
                className="topbar-button"
                type="button"
                disabled={projectBusy || projectState !== "native"}
                onClick={() => void applyProjectAction(pickProject)}
              >
                <Glyph name="plus" />
                Attach project
              </button>
            )}
            {workspaceLocation.route === "changes" && (
              <button
                className="topbar-button"
                type="button"
                disabled={gitBusy || !currentProject}
                onClick={() => void refreshGitReview()}
              >
                <Glyph name="refresh" />
                Refresh
              </button>
            )}
            {inspectorAvailable && (
              <button
                className="topbar-icon-button"
                type="button"
                aria-label={
                  inspectorOpen
                    ? "Hide workbench context"
                    : "Show workbench context"
                }
                aria-expanded={inspectorOpen}
                onClick={() => setInspectorOpen((current) => !current)}
              >
                <Glyph name="sidebar" />
              </button>
            )}
            <span className="foundation-badge">Native Linux</span>
            <button
              className="theme-shortcut"
              type="button"
              aria-label="Open appearance settings"
              onClick={() => navigateWorkspace("settings", "appearance")}
            >
              Appearance
            </button>
          </div>
        </header>

        <WorkbenchActionPalette
          open={commandPaletteOpen}
          onClose={closeCommandPalette}
          onNavigate={navigateWorkspace}
          onToggleDrawer={() => setInspectorOpen((current) => !current)}
          onToggleTerminal={() => setTerminalDockOpen((current) => !current)}
        />

        <div
          className={
            inspectorAvailable && inspectorOpen
              ? "workspace-body workspace-body--inspector-open"
              : "workspace-body"
          }
        >
          <div className="workspace-stage">
            {pendingConversationMode && (
              <div
                className="conversation-boundary-note"
                role="dialog"
                aria-modal="true"
                aria-label="Confirm conversation mode change"
              >
                <strong>Confirm mode change</strong>
                <p>
                  {pendingConversationMode === "chat"
                    ? "Advisor is read-only and has no project, terminal, Git, worktree, integration, native-action, approval, or dispatch capability."
                    : "QuireForge requires an attached project and restores its visible execution and approval boundaries."}
                  No project, attachment, integration, approval, dispatch,
                  completion report, or transient transcript transfers
                  automatically.
                </p>
                <button
                  type="button"
                  onClick={() => void confirmConversationModeChange()}
                >
                  Confirm{" "}
                  {pendingConversationMode === "chat"
                    ? "Advisor"
                    : "QuireForge"}
                </button>
                <button type="button" onClick={cancelConversationModeChange}>
                  Cancel
                </button>
              </div>
            )}
            <WorkspaceView
              route="home"
              active={workspaceLocation.route === "home"}
            >
              <HomeDashboard
                projects={projects}
                currentProject={currentProject ?? null}
                onNewTask={() => navigateWorkspace("conversation")}
                onAttachProject={() => void applyProjectAction(pickProject)}
                onOpenProjects={() => navigateWorkspace("projects")}
                onOpenSessions={() => navigateWorkspace("sessions")}
                onOpenIntegrations={() => navigateWorkspace("integrations")}
                onOpenTerminal={() => navigateWorkspace("terminal")}
              />
            </WorkspaceView>

            <WorkspaceView
              route="projects"
              active={workspaceLocation.route === "projects"}
            >
              <ProjectWorkspace
                availability={projectState}
                snapshot={projects}
                busy={projectBusy}
                actionError={projectActionError}
                preflights={projectPreflights}
                onPick={() => applyProjectAction(pickProject)}
                onPickRelink={(projectId) =>
                  applyProjectAction(() => pickRelink(projectId))
                }
                onConfirm={() => applyProjectAction(confirmProject)}
                onCancel={() => applyProjectAction(cancelProject)}
                onDetach={(projectId) =>
                  applyProjectAction(() => detachProjectDirectory(projectId))
                }
                onArchive={(projectId) =>
                  applyProjectAction(() => archiveProjectMetadata(projectId))
                }
                onPreflight={verifyProject}
              />
            </WorkspaceView>

            <WorkspaceView
              route="project-state"
              active={workspaceLocation.route === "project-state"}
            >
              <ProjectStateWorkspace
                availability={repositoryStateViewState}
                projectName={currentProject?.displayName ?? null}
                snapshot={repositoryStateSnapshot}
                busy={repositoryStateViewState === "checking"}
                onRefresh={() =>
                  setRepositoryStateRefresh((current) => current + 1)
                }
              />
            </WorkspaceView>

            <WorkspaceView
              route="advisor"
              active={workspaceLocation.route === "advisor"}
            >
              <AdvisorWorkspace
                resetToken={advisorResetToken}
                availability={advisorViewState}
                snapshot={advisorSnapshot}
                selectedProjectState={advisorProjectStateSnapshot}
                selectionState={advisorProjectStateSelection}
                canSelectProjectState={
                  bridgeState === "native" && currentProject !== undefined
                }
                onRequestProjectState={requestAdvisorProjectState}
                onConfirmProjectState={confirmAdvisorProjectState}
                onCancelProjectState={cancelAdvisorProjectState}
                onRemoveProjectState={removeAdvisorProjectState}
                auth={auth}
                runtime={runtime}
                conversation={advisorConversation}
                conversationBusy={advisorConversationBusy}
                selectedProjectId={advisorProjectStateProjectId}
                targetProjectId={currentProject?.id ?? null}
                onConversationStart={beginAdvisorConversation}
                onConversationPoll={pollAdvisorConversationById}
                onConversationInterrupt={stopAdvisorConversation}
                onDispatch={dispatchApprovedAdvisorRequest}
                onOpenExecution={() => navigateWorkspace("conversation")}
                onPrepareTaskHandoff={prepareAdvisorTaskHandoff}
                onOpenTaskHandoff={openTaskHandoffInQuireforge}
                returnedTaskReceipt={
                  acceptedTaskHandoff?.direction === "quireforge-to-advisor"
                    ? acceptedTaskHandoff.brief
                    : null
                }
              />
            </WorkspaceView>

            <WorkspaceView
              route="dynamic-analysis"
              active={workspaceLocation.route === "dynamic-analysis"}
            >
              <DynamicAnalysisWorkspace />
            </WorkspaceView>

            <WorkspaceView
              route="files"
              active={workspaceLocation.route === "files"}
            >
              <FilePreviewWorkspace
                availability={projectState}
                project={currentProject}
                snapshot={filePreview}
                busy={filePreviewBusy}
                actionError={filePreviewActionError}
                onPick={chooseFilePreview}
                onOpen={openSelectedFilePreview}
                onClear={discardFilePreview}
              />
            </WorkspaceView>

            <WorkspaceView
              route="worktrees"
              active={workspaceLocation.route === "worktrees"}
            >
              <WorktreeWorkspace
                availability={worktreeState}
                projectName={currentProject?.displayName ?? null}
                snapshot={worktrees}
                preview={worktreePreview}
                result={worktreeResult}
                busy={worktreeBusy || conversationActive || gitBusy}
                selectionBusy={worktreeBusy}
                actionError={worktreeActionError}
                executions={worktreeExecutions}
                onRefresh={refreshWorktrees}
                onCreate={beginWorktreeCreate}
                onPickAttach={beginWorktreeAttach}
                onRecover={beginWorktreeRecover}
                onRemove={beginWorktreeRemove}
                onConfirm={applyWorktree}
                onCancel={cancelWorktreePreview}
                onSelectProject={selectProject}
                onOpenExecution={(projectId) => {
                  selectProject(projectId);
                  window.setTimeout(() => navigateWorkspace("conversation"), 0);
                }}
              />
            </WorkspaceView>

            <WorkspaceView
              route="terminal"
              active={workspaceLocation.route === "terminal"}
            >
              {terminalDockOpen && workspaceLocation.route === "conversation"
                ? null
                : terminalWorkspace}
            </WorkspaceView>

            <WorkspaceView
              route="changes"
              active={workspaceLocation.route === "changes"}
            >
              <GitWorkspace
                availability={gitState}
                projectName={currentProject?.displayName ?? null}
                snapshot={gitSnapshot}
                diff={gitDiff}
                selectedRequest={gitSelectedRequest}
                mutationPreview={gitMutationPreview}
                mutationResult={gitMutationResult}
                busy={gitBusy || conversationActive}
                actionError={gitActionError}
                onRefresh={refreshGitReview}
                onReview={reviewGitDiff}
                onOpen={openReviewedGitFile}
                onPreviewMutation={beginGitMutation}
                onConfirmMutation={applyGitMutation}
                onCancelMutation={() => setGitMutationPreview(null)}
                onRecoverMutation={recoverGitRevert}
              />
            </WorkspaceView>

            <WorkspaceView
              route="sessions"
              active={workspaceLocation.route === "sessions"}
            >
              <SessionWorkspace
                availability={sessionState}
                snapshot={sessions}
                runtime={runtime}
                projects={projects.projects}
                activeConversationId={conversation.conversationId}
                attachments={conversationAttachments}
                busy={sessionBusy || conversationBusy || conversationActive}
                attachmentBusy={conversationAttachmentBusy}
                actionError={sessionActionError}
                attachmentActionError={conversationAttachmentActionError}
                searchTerm={sessionSearchTerm}
                onSearch={refreshSessions}
                onRefresh={() => refreshSessions()}
                onSelect={(session) => selectProject(session.projectId)}
                onResume={(request) =>
                  continueHistoricalConversation(
                    resumeConversationTask,
                    request,
                  )
                }
                onFork={(request) =>
                  continueHistoricalConversation(forkConversationTask, request)
                }
                onArchive={(conversationId) =>
                  mutateSession(() => archiveConversationTask(conversationId))
                }
                onRestore={(conversationId) =>
                  mutateSession(() => restoreConversationTask(conversationId))
                }
                onUpdateModelSelection={applyModelSelection}
                onAttachmentPick={chooseConversationAttachments}
                onAttachmentDrop={stageConversationAttachmentDrop}
                onAttachmentCancel={removeConversationAttachment}
              />
            </WorkspaceView>

            <WorkspaceView
              route="integrations"
              active={workspaceLocation.route === "integrations"}
            >
              <IntegrationCenter
                availability={integrationState}
                snapshot={integrationCatalog}
                preview={integrationPreview}
                result={integrationResult}
                controlPreview={integrationControlPreview}
                controlResult={integrationControlResult}
                busy={integrationBusy}
                actionError={integrationActionError}
                onRefresh={refreshIntegrationCatalog}
                onPreview={beginIntegrationMutation}
                onConfirm={applyIntegrationMutation}
                onControlPreview={beginIntegrationControl}
                onControlConfirm={applyIntegrationControl}
                onControlOpen={openIntegrationControl}
                onControlPoll={checkIntegrationControl}
                onCancel={() => {
                  setIntegrationPreview(null);
                  setIntegrationControlPreview(null);
                }}
              />
            </WorkspaceView>

            <WorkspaceView
              route="scheduled"
              active={workspaceLocation.route === "scheduled"}
            >
              <ScheduledWorkspace
                availability={integrationState}
                snapshot={integrationCatalog}
              />
            </WorkspaceView>

            <WorkspaceView
              route="conversation"
              active={workspaceLocation.route === "conversation"}
            >
              <section className="conversation-mode-workspace">
                {conversationMode === "chat" ? (
                  <p className="conversation-boundary-note" role="status">
                    Advisor is selected. Use the Advisor workspace to create,
                    learn, and explore without execution authority.
                  </p>
                ) : (
                  <>
                    <section
                      className="mock-inference-launcher"
                      aria-label="Durable task catalog"
                    >
                      <button
                        type="button"
                        aria-expanded={taskCatalogOpen}
                        onClick={() =>
                          setTaskCatalogOpen((current) => !current)
                        }
                      >
                        {taskCatalogOpen ? "Hide Task Catalog" : "Task Catalog"}
                      </button>
                      <button
                        type="button"
                        onClick={() => setDurableSourcesWorkbenchOpen(true)}
                      >
                        Durable Sources
                      </button>
                    </section>
                    {taskCatalogOpen && (
                      <TaskCatalog
                        snapshot={taskCatalog}
                        busy={taskCatalogBusy}
                        projectId={currentProject?.id ?? null}
                        onLoad={refreshTaskCatalog}
                        onCreate={(title) =>
                          applyTaskCatalogMutation(() =>
                            createTaskRecord({
                              projectId: currentProject?.id ?? "",
                              title,
                            }),
                          )
                        }
                        onRename={(taskId, title) => () =>
                          applyTaskCatalogMutation(() =>
                            renameTaskRecord({ taskId, title }),
                          )
                        }
                        onStatus={(taskId, status) => () =>
                          applyTaskCatalogMutation(() =>
                            setTaskRecordStatus({ taskId, status }),
                          )
                        }
                        onArchive={(taskId) => () =>
                          applyTaskCatalogMutation(() =>
                            archiveTaskRecord({ taskId }),
                          )
                        }
                        onRestore={(taskId) => () =>
                          applyTaskCatalogMutation(() =>
                            restoreTaskRecord({ taskId }),
                          )
                        }
                        onDelete={(taskId) => () =>
                          applyTaskCatalogMutation(() =>
                            deleteTaskRecord({ taskId }),
                          )
                        }
                        onPlanCreate={(taskId, copyPrimaryBody) => () =>
                          applyTaskCatalogMutation(() =>
                            createTaskPlan({ taskId, copyPrimaryBody }),
                          )
                        }
                        onPlanSelect={(taskId, planId) => () =>
                          selectDurableTaskPlan(taskId, planId)
                        }
                        onPlanEdit={(taskId, planId, label, body) => () =>
                          applyTaskCatalogMutation(() =>
                            editTaskPlan({ taskId, planId, label, body }),
                          )
                        }
                        onPlanDelete={(taskId, planId) => () =>
                          applyTaskCatalogMutation(() =>
                            deleteTaskPlan({ taskId, planId }),
                          )
                        }
                        onOpenTemplates={() =>
                          setTaskTemplateWorkbenchOpen(true)
                        }
                        onOpenMockInference={() =>
                          setMockInferenceWorkbenchOpen(true)
                        }
                      />
                    )}
                    {taskTemplateWorkbenchOpen && (
                      <Suspense
                        fallback={
                          <section
                            className="task-template-workbench"
                            aria-label="Task Templates"
                          >
                            <p role="status">Loading task templates…</p>
                          </section>
                        }
                      >
                        <TaskTemplateWorkbench
                          projectId={currentProject?.id ?? null}
                          onClose={() => setTaskTemplateWorkbenchOpen(false)}
                        />
                      </Suspense>
                    )}
                    {mockInferenceWorkbenchOpen && (
                      <Suspense
                        fallback={
                          <section
                            className="mock-inference-workbench"
                            aria-label="Fictional mock inference"
                          >
                            <p role="status">Loading local mock inference…</p>
                          </section>
                        }
                      >
                        <MockInferenceWorkbench
                          projectId={currentProject?.id ?? null}
                          onClose={() => {
                            setMockInferenceWorkbenchOpen(false);
                            window.requestAnimationFrame(() =>
                              mockInferenceLauncherRef.current?.focus(),
                            );
                          }}
                        />
                      </Suspense>
                    )}
                    {connectorGovernanceWorkbenchOpen && (
                      <Suspense
                        fallback={
                          <section
                            className="mock-inference-workbench"
                            aria-label="Fictional connector governance"
                          >
                            <p role="status">
                              Loading fictional connector governance…
                            </p>
                          </section>
                        }
                      >
                        <ConnectorGovernanceWorkbench
                          projectId={currentProject?.id ?? null}
                          onClose={() =>
                            setConnectorGovernanceWorkbenchOpen(false)
                          }
                        />
                      </Suspense>
                    )}
                    {controlledBrowserVerificationOpen && (
                      <Suspense
                        fallback={
                          <section
                            className="mock-inference-workbench"
                            aria-label="Fictional controlled browser verification"
                          >
                            <p role="status">Loading local verification…</p>
                          </section>
                        }
                      >
                        <ControlledBrowserVerificationWorkbench
                          projectId={currentProject?.id ?? null}
                          onClose={() =>
                            setControlledBrowserVerificationOpen(false)
                          }
                        />
                      </Suspense>
                    )}
                    {contextAssemblyWorkbenchOpen && (
                      <Suspense
                        fallback={
                          <section className="mock-inference-workbench">
                            <p role="status">
                              Loading governed context review…
                            </p>
                          </section>
                        }
                      >
                        <ContextAssemblyWorkbench
                          key={currentProject?.id ?? "no-project"}
                          projectId={currentProject?.id ?? null}
                          projectLabel={currentProject?.displayName ?? null}
                          onClose={() => setContextAssemblyWorkbenchOpen(false)}
                        />
                      </Suspense>
                    )}
                    {durableSourcesWorkbenchOpen && (
                      <Suspense
                        fallback={
                          <section
                            className="task-template-workbench"
                            aria-label="Durable Sources"
                          >
                            <p role="status">Loading durable sources…</p>
                          </section>
                        }
                      >
                        <DurableSourcesWorkbench
                          projectId={currentProject?.id ?? null}
                          onClose={() => setDurableSourcesWorkbenchOpen(false)}
                        />
                      </Suspense>
                    )}
                    <ConversationWorkspace
                      key={acceptedTaskHandoff?.taskId ?? "ordinary-task"}
                      availability={conversationState}
                      snapshot={conversation}
                      events={conversationEvents}
                      runtime={runtime}
                      project={currentProject}
                      integrations={integrationCatalog}
                      attachments={conversationAttachments}
                      busy={conversationBusy}
                      attachmentBusy={conversationAttachmentBusy}
                      actionError={conversationActionError}
                      attachmentActionError={conversationAttachmentActionError}
                      onStart={beginConversation}
                      onInterrupt={stopConversation}
                      onDecideApproval={applyConversationApproval}
                      onUpdateModelSelection={applyModelSelection}
                      onAttachmentPick={chooseConversationAttachments}
                      onAttachmentDrop={stageConversationAttachmentDrop}
                      onAttachmentCancel={removeConversationAttachment}
                      handoffBrief={acceptedTaskHandoff?.brief ?? null}
                      onReturnTaskReceipt={returnTaskHandoffToAdvisor}
                    />
                    <section
                      className="mock-inference-launcher"
                      aria-label="Fictional local mock workflow"
                    >
                      <button
                        ref={mockInferenceLauncherRef}
                        type="button"
                        onClick={() => setMockInferenceWorkbenchOpen(true)}
                      >
                        Fictional mock inference
                      </button>
                      <button
                        type="button"
                        onClick={() =>
                          setConnectorGovernanceWorkbenchOpen(true)
                        }
                      >
                        Fictional connector governance
                      </button>
                      <button
                        type="button"
                        onClick={() =>
                          setControlledBrowserVerificationOpen(true)
                        }
                      >
                        Fictional browser verification
                      </button>
                      <button
                        type="button"
                        onClick={() => setContextAssemblyWorkbenchOpen(true)}
                      >
                        Governed context review
                      </button>
                    </section>
                  </>
                )}
                {conversationMode === "codex" && (
                  <section
                    className="workbench-terminal-dock"
                    aria-label="Terminal dock"
                    style={
                      {
                        "--terminal-dock-height": `${workbenchLayout.terminalDockHeight}px`,
                      } as CSSProperties
                    }
                  >
                    <div className="workbench-terminal-dock__header">
                      <div>
                        <p className="eyebrow">Managed terminal</p>
                        <strong>Terminal dock</strong>
                      </div>
                      <button
                        type="button"
                        aria-expanded={terminalDockOpen}
                        aria-controls="terminal-dock-content"
                        onClick={() =>
                          setTerminalDockOpen((current) => !current)
                        }
                      >
                        {terminalDockOpen
                          ? "Collapse dock"
                          : "Open terminal dock"}
                      </button>
                    </div>
                    {terminalDockOpen &&
                    workspaceLocation.route === "conversation" ? (
                      <>
                        <div
                          className="workbench-terminal-dock__resize"
                          role="separator"
                          aria-label="Resize terminal dock"
                          aria-orientation="horizontal"
                          aria-valuemin={terminalDockHeightMinimum}
                          aria-valuemax={terminalDockHeightMaximum}
                          aria-valuenow={workbenchLayout.terminalDockHeight}
                          tabIndex={0}
                          onPointerDown={beginTerminalDockResize}
                          onKeyDown={resizeTerminalDockFromKeyboard}
                        />
                        <div
                          id="terminal-dock-content"
                          className="workbench-terminal-dock__content"
                        >
                          {terminalWorkspace}
                        </div>
                      </>
                    ) : null}
                  </section>
                )}
              </section>
            </WorkspaceView>

            {reviewPanesOpen && workspaceLocation.route === "conversation" && (
              <Suspense
                fallback={
                  <aside className="review-panes" aria-label="Review panes">
                    <p role="status">Loading review panes…</p>
                  </aside>
                }
              >
                <ReviewPanes
                  projectId={currentProject?.id ?? null}
                  projectName={currentProject?.displayName ?? null}
                  filePreview={filePreview}
                  conversation={conversation}
                  conversationEvents={conversationEvents}
                  taskCatalog={taskCatalog}
                  loadGitStatus={loadGitStatusTask}
                  loadGitDiff={loadGitDiffTask}
                  loadArtifacts={loadAdvisorGeneratedArtifacts}
                  previewArtifact={previewAdvisorGeneratedArtifact}
                  width={workbenchLayout.reviewPaneWidth}
                  selectedPane={workbenchLayout.selectedReviewPane}
                  onWidthChange={(reviewPaneWidth) =>
                    setWorkbenchLayout((current) => ({
                      ...current,
                      reviewPaneWidth,
                    }))
                  }
                  onSelectedPaneChange={(selectedReviewPane) =>
                    setWorkbenchLayout((current) => ({
                      ...current,
                      selectedReviewPane,
                    }))
                  }
                  onClose={() => setReviewPanesOpen(false)}
                />
              </Suspense>
            )}

            <WorkspaceView
              route="settings"
              active={workspaceLocation.route === "settings"}
            >
              <SettingsWorkspace
                section={workspaceLocation.settingsSection ?? "general"}
                auth={auth}
                authState={authState}
                authBusy={authBusy}
                authActionError={authActionError}
                confirmLogout={confirmLogout}
                usage={usage}
                usageState={usageState}
                usageBusy={usageBusy}
                theme={theme}
                productName={bootstrap.product.name}
                productVersion={bootstrap.product.version}
                bridgeLabel={bridgeLabel}
                runtimeLabel={runtimeLabel}
                cliVersion={runtime.cliVersion}
                onSectionChange={(section) =>
                  navigateWorkspace("settings", section)
                }
                onRefreshAuth={() => void applyAuthAction(refreshAuth)}
                onRefreshUsage={() => void refreshUsageStatus()}
                onRequestLogout={() => setConfirmLogout(true)}
                onConfirmLogout={() => {
                  setConfirmLogout(false);
                  void applyAuthAction(logoutAuth);
                }}
                onCancelLogout={() => setConfirmLogout(false)}
                onThemeChange={(nextTheme) => {
                  setThemePreview(null);
                  setTheme(nextTheme);
                }}
                onThemePreview={setThemePreview}
                onThemePreviewEnd={() => setThemePreview(null)}
              />
            </WorkspaceView>
          </div>

          {inspectorAvailable && inspectorOpen && (
            <>
              <div
                className="pane-divider"
                role="separator"
                aria-label="Resize workbench context"
                aria-orientation="vertical"
                aria-valuemin={inspectorWidthMinimum}
                aria-valuemax={inspectorWidthMaximum}
                aria-valuenow={inspectorWidth}
                tabIndex={0}
                onPointerDown={beginInspectorResize}
                onKeyDown={resizeInspectorFromKeyboard}
              />
              <aside
                className="context-inspector"
                aria-labelledby="context-inspector-title"
              >
                <header className="context-inspector__header">
                  <div>
                    <span>Optional workspace detail</span>
                    <h2 id="context-inspector-title">Workbench context</h2>
                  </div>
                  <button
                    type="button"
                    aria-label="Close workbench context"
                    onClick={() => setInspectorOpen(false)}
                  >
                    ×
                  </button>
                </header>
                <div className="context-inspector__scroll">
                  {inspectorContent}
                </div>
              </aside>
            </>
          )}
        </div>

        <footer className="workspace-statusbar" aria-label="Workspace status">
          <div>
            <StatusDot state={bridgeState} />
            <span>{bridgeLabel}</span>
          </div>
          <span>{currentProject?.displayName ?? "No project attached"}</span>
          <span>{activeWorkspaceTitle}</span>
          <span>v{bootstrap.product.version}</span>
        </footer>
      </main>
    </div>
  );
}
