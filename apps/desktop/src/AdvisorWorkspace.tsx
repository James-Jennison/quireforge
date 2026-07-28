/* eslint-disable jsx-a11y/no-noninteractive-tabindex -- The bounded log must be keyboard-focusable for scrollback review. */

import { useEffect, useRef, useState } from "react";

import type { CodexAuthSnapshot } from "./lib/auth";
import type {
  AdvisorConversationSnapshot,
  AdvisorConversationStartRequest,
} from "./lib/advisorConversation";
import { managedChatAuthenticationState } from "./lib/conversationMode";
import type {
  AdvisorSelectedProjectStateSnapshot,
  AdvisorWorkspaceSnapshot,
} from "./lib/advisorWorkspace";
import {
  createAdvisorDraft,
  decideAdvisorDraft,
  dispatchAdvisorOnce,
  loadAdvisorCompletionReport,
  cancelAdvisorTextAttachment,
  loadAdvisorTextAttachment,
  pickAdvisorTextAttachment,
  saveAdvisorTextExport,
  cancelAdvisorImageAttachment,
  loadAdvisorImageAttachment,
  pickAdvisorImageAttachment,
  cancelAdvisorDocumentAttachment,
  loadAdvisorDocumentAttachment,
  pickAdvisorDocumentAttachment,
  cancelAdvisorArchiveAttachment,
  loadAdvisorArchiveAttachment,
  pickAdvisorArchiveAttachment,
  cancelAdvisorBinaryAttachment,
  loadAdvisorBinaryAttachment,
  pickAdvisorBinaryAttachment,
} from "./lib/bridge";
import {
  scaffoldAdvisorTextAttachment,
  advisorTextExportCandidates,
  type AdvisorTextAttachmentSnapshot,
} from "./lib/advisorAttachment";
import {
  scaffoldAdvisorImageAttachment,
  type AdvisorImageAttachmentSnapshot,
} from "./lib/advisorImageAttachment";
import {
  scaffoldAdvisorDocumentAttachment,
  type AdvisorDocumentAttachmentSnapshot,
} from "./lib/advisorDocumentAttachment";
import {
  scaffoldAdvisorArchiveAttachment,
  type AdvisorArchiveAttachmentSnapshot,
} from "./lib/advisorArchiveAttachment";
import {
  scaffoldAdvisorBinaryAttachment,
  type AdvisorBinaryAttachmentSnapshot,
} from "./lib/advisorBinaryAttachment";
import type {
  AdvisorApprovalSnapshot,
  AdvisorDispatchSnapshot,
  AdvisorCompletionReportSnapshot,
} from "./lib/advisorApproval";
import type { CodexRuntimeSnapshot } from "./lib/codex";
import type {
  TaskHandoffCreateRequest,
  TaskHandoffSnapshot,
} from "./lib/taskHandoff";

interface AdvisorApprovalState {
  snapshot: AdvisorApprovalSnapshot;
  bindingKey: string;
}

type AdvisorConversationEvent = AdvisorConversationSnapshot["events"][number];

function coalesceAssistantMessageDeltas(
  events: AdvisorConversationEvent[],
): AdvisorConversationEvent[] {
  return events.reduce<AdvisorConversationEvent[]>((coalesced, event) => {
    const previous = coalesced.at(-1);
    if (
      event.type === "agent-message-delta" &&
      previous?.type === "agent-message-delta"
    ) {
      previous.delta += event.delta;
      return coalesced;
    }
    coalesced.push({ ...event });
    return coalesced;
  }, []);
}

interface AdvisorWorkspaceProps {
  resetToken?: number;
  availability: "checking" | "native" | "preview" | "error";
  snapshot: AdvisorWorkspaceSnapshot | null;
  selectedProjectState: AdvisorSelectedProjectStateSnapshot | null;
  selectionState: "idle" | "confirming" | "reading" | "error";
  canSelectProjectState: boolean;
  onRequestProjectState: () => void;
  onConfirmProjectState: () => void;
  onCancelProjectState: () => void;
  onRemoveProjectState: () => void;
  auth: CodexAuthSnapshot;
  runtime: CodexRuntimeSnapshot;
  conversation: AdvisorConversationSnapshot;
  conversationBusy: boolean;
  selectedProjectId: string | null;
  targetProjectId?: string | null;
  onConversationStart: (
    request: AdvisorConversationStartRequest,
  ) => Promise<AdvisorConversationSnapshot>;
  onConversationPoll: (
    conversationId: string,
  ) => Promise<AdvisorConversationSnapshot>;
  onConversationInterrupt: (
    conversationId: string,
  ) => Promise<AdvisorConversationSnapshot>;
  onDispatch: (
    request: Parameters<typeof dispatchAdvisorOnce>[0],
  ) => Promise<AdvisorDispatchSnapshot>;
  onOpenExecution: () => void;
  onPrepareTaskHandoff?: (
    request: TaskHandoffCreateRequest,
  ) => Promise<TaskHandoffSnapshot>;
  onOpenTaskHandoff?: () => Promise<void>;
  returnedTaskReceipt?: string | null;
}

const diagnosticMessage: Partial<
  Record<NonNullable<AdvisorConversationSnapshot["diagnosticCode"]>, string>
> = {
  "authentication-required":
    "Sign in with the managed ChatGPT browser flow before starting Advisor.",
  "authentication-unavailable":
    "Advisor requires a managed ChatGPT account. API-key and external-token accounts cannot enable it.",
  "context-unavailable":
    "The selected Project State summary could not be safely refreshed for this message.",
  "capability-blocked":
    "Advisor blocked an attempted tool or permission request.",
  "metadata-unavailable":
    "Advisor could not record its bounded thread reference.",
  "protocol-invalid":
    "Advisor could not complete the managed Codex conversation safely. Try again later.",
  "runtime-unavailable":
    "The managed Codex service is unavailable. Try again later.",
  "thread-start-rejected":
    "Advisor could not start a managed conversation with its read-only settings. Try again later.",
};

export function AdvisorWorkspace({
  resetToken = 0,
  availability,
  selectedProjectState,
  selectionState,
  canSelectProjectState,
  onRequestProjectState,
  onConfirmProjectState,
  onCancelProjectState,
  onRemoveProjectState,
  auth,
  runtime,
  conversation,
  conversationBusy,
  selectedProjectId,
  targetProjectId = null,
  onConversationStart,
  onConversationPoll,
  onConversationInterrupt,
  onDispatch,
  onOpenExecution,
  onPrepareTaskHandoff,
  onOpenTaskHandoff,
  returnedTaskReceipt = null,
}: AdvisorWorkspaceProps) {
  const [prompt, setPrompt] = useState("");
  const [includeProjectState, setIncludeProjectState] = useState(false);
  const [confirmContextSend, setConfirmContextSend] = useState(false);
  const [confirmAttachmentSend, setConfirmAttachmentSend] = useState(false);
  const [textAttachment, setTextAttachment] =
    useState<AdvisorTextAttachmentSnapshot>(scaffoldAdvisorTextAttachment);
  const [attachmentBusy, setAttachmentBusy] = useState(false);
  const [imageAttachment, setImageAttachment] =
    useState<AdvisorImageAttachmentSnapshot>(scaffoldAdvisorImageAttachment);
  const [documentAttachment, setDocumentAttachment] =
    useState<AdvisorDocumentAttachmentSnapshot>(
      scaffoldAdvisorDocumentAttachment,
    );
  const [archiveAttachment, setArchiveAttachment] =
    useState<AdvisorArchiveAttachmentSnapshot>(
      scaffoldAdvisorArchiveAttachment,
    );
  const [binaryAttachment, setBinaryAttachment] =
    useState<AdvisorBinaryAttachmentSnapshot>(scaffoldAdvisorBinaryAttachment);
  const [exportCandidateIndex, setExportCandidateIndex] = useState(0);
  const [actionError, setActionError] = useState(false);
  const [draft, setDraft] = useState("");
  const defaultModel = runtime.models.find((model) => model.isDefault) ?? null;
  const [requestedModel, setRequestedModel] = useState("");
  const [requestedReasoning, setRequestedReasoning] = useState("");
  const [declaredCapability, setDeclaredCapability] = useState<
    "read-only" | "workspace-write" | "danger-full-access"
  >("workspace-write");
  const [approval, setApproval] = useState<AdvisorApprovalState | null>(null);
  const [approvalBusy, setApprovalBusy] = useState(false);
  const [dispatchResult, setDispatchResult] =
    useState<AdvisorDispatchSnapshot | null>(null);
  const draftRef = useRef<HTMLTextAreaElement>(null);
  const [completionReport, setCompletionReport] =
    useState<AdvisorCompletionReportSnapshot | null>(null);
  const [detailsOpen, setDetailsOpen] = useState(false);
  const [handoffBrief, setHandoffBrief] = useState("");
  const [handoffPending, setHandoffPending] = useState(false);
  const lastResetToken = useRef(resetToken);
  const conversationViewportRef = useRef<HTMLDivElement>(null);
  const detailsTriggerRef = useRef<HTMLButtonElement>(null);
  const [followLatest, setFollowLatest] = useState(true);
  const authentication = managedChatAuthenticationState(auth);
  const active =
    conversation.state === "running" && conversation.conversationId !== null;
  const hasConversation = conversation.events.length > 0;
  const canIncludeProjectState = Boolean(
    selectedProjectState && selectedProjectId,
  );
  const effectiveModel = requestedModel || defaultModel?.id || "";
  const selectedModel =
    runtime.models.find((model) => model.id === effectiveModel) ?? null;
  const effectiveReasoning =
    requestedReasoning || selectedModel?.defaultReasoningEffort || "";
  const approvalBindingKey = JSON.stringify({
    advisorConversationId: conversation.conversationId,
    targetProjectId,
    draft,
    declaredCapability,
    requestedModel: effectiveModel,
    requestedReasoning: effectiveReasoning,
    selectedProjectState:
      includeProjectState && canIncludeProjectState
        ? selectedProjectState
        : null,
  });
  const currentApproval =
    approval?.bindingKey === approvalBindingKey ? approval.snapshot : null;
  const dispatchSupported =
    declaredCapability !== "danger-full-access" &&
    selectedModel !== null &&
    selectedModel.supportedReasoningEfforts.includes(effectiveReasoning);
  const sendDisabledReason =
    authentication !== "ready"
      ? "Sign in with managed ChatGPT to send."
      : conversationBusy
        ? "Advisor is preparing."
        : !prompt.trim()
          ? "Enter a message to send."
          : null;

  useEffect(() => {
    void loadAdvisorTextAttachment()
      .then(setTextAttachment)
      .catch(() =>
        setTextAttachment({
          ...scaffoldAdvisorTextAttachment,
          state: "unavailable",
          diagnosticCode: "read-failed",
        }),
      );
  }, []);

  useEffect(() => {
    if (lastResetToken.current === resetToken) return;
    lastResetToken.current = resetToken;
    setPrompt("");
    setIncludeProjectState(false);
    setConfirmContextSend(false);
    setConfirmAttachmentSend(false);
    setDraft("");
    setApproval(null);
    setDispatchResult(null);
    setCompletionReport(null);
    setExportCandidateIndex(0);
    setTextAttachment(scaffoldAdvisorTextAttachment);
    setImageAttachment(scaffoldAdvisorImageAttachment);
    setDocumentAttachment(scaffoldAdvisorDocumentAttachment);
    setArchiveAttachment(scaffoldAdvisorArchiveAttachment);
    setBinaryAttachment(scaffoldAdvisorBinaryAttachment);
    void Promise.all([
      cancelAdvisorTextAttachment(),
      cancelAdvisorImageAttachment(),
      cancelAdvisorDocumentAttachment(),
      cancelAdvisorArchiveAttachment(),
      cancelAdvisorBinaryAttachment(),
    ]).catch(() => setActionError(true));
  }, [resetToken]);

  useEffect(() => {
    const viewport = conversationViewportRef.current;
    if (!viewport || !followLatest) return;
    viewport.scrollTop = viewport.scrollHeight;
  }, [conversation.events, followLatest]);

  useEffect(() => {
    if (!detailsOpen) return undefined;
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      setDetailsOpen(false);
      detailsTriggerRef.current?.focus();
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [detailsOpen]);

  function updateViewportPosition() {
    const viewport = conversationViewportRef.current;
    if (!viewport) return;
    setFollowLatest(
      viewport.scrollHeight - viewport.scrollTop - viewport.clientHeight < 12,
    );
  }

  function jumpToLatest() {
    const viewport = conversationViewportRef.current;
    if (!viewport) return;
    viewport.scrollTop = viewport.scrollHeight;
    setFollowLatest(true);
  }

  function closeDetails() {
    setDetailsOpen(false);
    detailsTriggerRef.current?.focus();
  }
  useEffect(() => {
    void loadAdvisorImageAttachment()
      .then(setImageAttachment)
      .catch(() =>
        setImageAttachment({
          ...scaffoldAdvisorImageAttachment,
          state: "unavailable",
          diagnosticCode: "read-failed",
        }),
      );
  }, []);
  useEffect(() => {
    void loadAdvisorBinaryAttachment()
      .then(setBinaryAttachment)
      .catch(() =>
        setBinaryAttachment({
          ...scaffoldAdvisorBinaryAttachment,
          state: "unavailable",
          diagnosticCode: "read-failed",
        }),
      );
  }, []);
  useEffect(() => {
    void loadAdvisorArchiveAttachment()
      .then(setArchiveAttachment)
      .catch(() =>
        setArchiveAttachment({
          ...scaffoldAdvisorArchiveAttachment,
          state: "unavailable",
          diagnosticCode: "read-failed",
        }),
      );
  }, []);
  useEffect(() => {
    void loadAdvisorDocumentAttachment()
      .then(setDocumentAttachment)
      .catch(() =>
        setDocumentAttachment({
          ...scaffoldAdvisorDocumentAttachment,
          state: "unavailable",
          diagnosticCode: "read-failed",
        }),
      );
  }, []);

  useEffect(() => {
    if (!active || !conversation.conversationId) return undefined;
    const conversationId = conversation.conversationId;
    const timer = window.setTimeout(() => {
      void onConversationPoll(conversationId).catch(() => setActionError(true));
    }, 300);
    return () => window.clearTimeout(timer);
  }, [active, conversation.conversationId, onConversationPoll]);

  function submit(projectId: string | null) {
    if (conversationBusy || active || authentication !== "ready") return;
    setActionError(false);
    setFollowLatest(true);
    const attachment =
      textAttachment.state === "ready" ? textAttachment.attachment : null;
    const image =
      imageAttachment.state === "ready" ? imageAttachment.attachment : null;
    const document =
      documentAttachment.state === "ready"
        ? documentAttachment.attachment
        : null;
    const archive =
      archiveAttachment.state === "ready" ? archiveAttachment.attachment : null;
    const binary =
      binaryAttachment.state === "ready" ? binaryAttachment.attachment : null;
    void onConversationStart({
      prompt,
      projectId,
      attachmentId: attachment?.attachmentId ?? null,
      attachmentManifestSha256: attachment?.sha256 ?? null,
      attachmentConfirmation: attachment ? "confirmed-for-single-send" : null,
      imageAttachmentId: image?.attachmentId ?? null,
      imageAttachmentManifestSha256: image?.sha256 ?? null,
      imageAttachmentConfirmation: image ? "confirmed-for-single-send" : null,
      documentAttachmentId: document?.attachmentId ?? null,
      documentAttachmentManifestSha256: document?.sha256 ?? null,
      documentAttachmentConfirmation: document
        ? "confirmed-for-single-send"
        : null,
      archiveAttachmentId: archive?.attachmentId ?? null,
      archiveAttachmentManifestSha256: archive?.sha256 ?? null,
      archiveAttachmentConfirmation: archive
        ? "confirmed-for-single-send"
        : null,
      binaryAttachmentId: binary?.attachmentId ?? null,
      binaryAttachmentManifestSha256: binary?.sha256 ?? null,
      binaryAttachmentConfirmation: binary ? "confirmed-for-single-send" : null,
    })
      .then(() => setPrompt(""))
      .then(() => setTextAttachment(scaffoldAdvisorTextAttachment))
      .then(() => setImageAttachment(scaffoldAdvisorImageAttachment))
      .then(() => setDocumentAttachment(scaffoldAdvisorDocumentAttachment))
      .then(() => setArchiveAttachment(scaffoldAdvisorArchiveAttachment))
      .then(() => setBinaryAttachment(scaffoldAdvisorBinaryAttachment))
      .catch(() => setActionError(true));
  }

  async function pickTextAttachment() {
    setAttachmentBusy(true);
    setActionError(false);
    try {
      setTextAttachment(await pickAdvisorTextAttachment());
    } catch {
      setActionError(true);
    } finally {
      setAttachmentBusy(false);
    }
  }

  async function clearTextAttachment() {
    setAttachmentBusy(true);
    try {
      setTextAttachment(await cancelAdvisorTextAttachment());
    } catch {
      setActionError(true);
    } finally {
      setAttachmentBusy(false);
    }
  }
  async function pickImageAttachment() {
    setAttachmentBusy(true);
    setActionError(false);
    try {
      setImageAttachment(await pickAdvisorImageAttachment());
    } catch {
      setActionError(true);
    } finally {
      setAttachmentBusy(false);
    }
  }
  async function clearImageAttachment() {
    setAttachmentBusy(true);
    try {
      setImageAttachment(await cancelAdvisorImageAttachment());
    } catch {
      setActionError(true);
    } finally {
      setAttachmentBusy(false);
    }
  }
  async function pickDocumentAttachment() {
    setAttachmentBusy(true);
    setActionError(false);
    try {
      setDocumentAttachment(await pickAdvisorDocumentAttachment());
    } catch {
      setActionError(true);
    } finally {
      setAttachmentBusy(false);
    }
  }
  async function clearDocumentAttachment() {
    setAttachmentBusy(true);
    try {
      setDocumentAttachment(await cancelAdvisorDocumentAttachment());
    } catch {
      setActionError(true);
    } finally {
      setAttachmentBusy(false);
    }
  }
  async function pickArchiveAttachment() {
    setAttachmentBusy(true);
    setActionError(false);
    try {
      setArchiveAttachment(await pickAdvisorArchiveAttachment());
    } catch {
      setActionError(true);
    } finally {
      setAttachmentBusy(false);
    }
  }
  async function clearArchiveAttachment() {
    setAttachmentBusy(true);
    try {
      setArchiveAttachment(await cancelAdvisorArchiveAttachment());
    } catch {
      setActionError(true);
    } finally {
      setAttachmentBusy(false);
    }
  }
  async function pickBinaryAttachment() {
    setAttachmentBusy(true);
    setActionError(false);
    try {
      setBinaryAttachment(await pickAdvisorBinaryAttachment());
    } catch {
      setActionError(true);
    } finally {
      setAttachmentBusy(false);
    }
  }
  async function clearBinaryAttachment() {
    setAttachmentBusy(true);
    try {
      setBinaryAttachment(await cancelAdvisorBinaryAttachment());
    } catch {
      setActionError(true);
    } finally {
      setAttachmentBusy(false);
    }
  }

  const latestReply = coalesceAssistantMessageDeltas(conversation.events)
    .filter((event) => event.type === "agent-message-delta")
    .map((event) => event.delta)
    .join("");
  const exportCandidates = advisorTextExportCandidates(latestReply);

  async function exportLatestReply() {
    const candidate =
      exportCandidates[exportCandidateIndex] ?? exportCandidates[0];
    if (!candidate?.content) return;
    setActionError(false);
    try {
      await saveAdvisorTextExport({
        suggestedName: candidate.suggestedName,
        contentType: candidate.contentType,
        content: candidate.content,
      });
    } catch {
      setActionError(true);
    }
  }

  async function createDraft() {
    if (!conversation.conversationId || !targetProjectId || !draft.trim())
      return;
    setApprovalBusy(true);
    setActionError(false);
    try {
      const snapshot = await createAdvisorDraft({
        advisorConversationId: conversation.conversationId,
        targetProjectId,
        prompt: draft,
        selectedProjectState:
          includeProjectState && canIncludeProjectState
            ? selectedProjectState
            : null,
        declaredCapabilities: [declaredCapability],
        requestedModel: effectiveModel,
        requestedReasoningEffort: effectiveReasoning,
      });
      setApproval({ snapshot, bindingKey: approvalBindingKey });
    } catch {
      setActionError(true);
    } finally {
      setApprovalBusy(false);
    }
  }

  async function decideDraft(decision: "approved" | "rejected") {
    if (!currentApproval) return;
    setApprovalBusy(true);
    try {
      const snapshot = await decideAdvisorDraft({
        proposalId: currentApproval.proposalId,
        decision,
        binding: {
          advisorConversationId: conversation.conversationId!,
          targetProjectId: targetProjectId!,
          prompt: draft,
          selectedProjectState:
            includeProjectState && canIncludeProjectState
              ? selectedProjectState
              : null,
          declaredCapabilities: [declaredCapability],
          requestedModel: effectiveModel,
          requestedReasoningEffort: effectiveReasoning,
        },
      });
      setApproval({ snapshot, bindingKey: approvalBindingKey });
    } catch {
      setActionError(true);
    } finally {
      setApprovalBusy(false);
    }
  }

  async function dispatchOnce() {
    if (
      !currentApproval ||
      currentApproval.state !== "approved" ||
      !dispatchSupported
    )
      return;
    setApprovalBusy(true);
    setActionError(false);
    try {
      setDispatchResult(
        await onDispatch({
          proposalId: currentApproval.proposalId,
          binding: {
            advisorConversationId: conversation.conversationId!,
            targetProjectId: targetProjectId!,
            prompt: draft,
            selectedProjectState:
              includeProjectState && canIncludeProjectState
                ? selectedProjectState
                : null,
            declaredCapabilities: [declaredCapability],
            requestedModel: effectiveModel,
            requestedReasoningEffort: effectiveReasoning,
          },
        }),
      );
    } catch {
      setActionError(true);
    } finally {
      setApprovalBusy(false);
    }
  }

  async function readCompletionReport() {
    if (!dispatchResult) return;
    setApprovalBusy(true);
    try {
      setCompletionReport(
        await loadAdvisorCompletionReport({
          proposalId: dispatchResult.proposalId,
        }),
      );
    } catch {
      setActionError(true);
    } finally {
      setApprovalBusy(false);
    }
  }

  return (
    <section
      className="project-workspace"
      id="advisor"
      aria-labelledby="advisor-title"
    >
      <header className="advisor-chat-header">
        <div>
          <p className="eyebrow">Advisor</p>
          <h1 id="advisor-title" data-workspace-heading tabIndex={-1}>
            Advisor
          </h1>
          <p>Create, Learn, Explore · Read-only</p>
        </div>
        <button
          ref={detailsTriggerRef}
          type="button"
          aria-expanded={detailsOpen}
          aria-controls="advisor-details"
          onClick={() => setDetailsOpen((open) => !open)}
        >
          Details
        </button>
      </header>
      {returnedTaskReceipt && (
        <section className="project-card" aria-label="Returned task receipt">
          <h2>QuireForge task receipt</h2>
          <p>{returnedTaskReceipt}</p>
        </section>
      )}
      {availability === "native" &&
        onPrepareTaskHandoff &&
        onOpenTaskHandoff && (
          <section
            className="project-card"
            aria-labelledby="advisor-handoff-title"
          >
            <h2 id="advisor-handoff-title">Continue this task in QuireForge</h2>
            <p>
              Review a bounded brief before opening it. No transcript, project,
              attachment, or authority is transferred.
            </p>
            <label htmlFor="advisor-handoff-brief">Reviewed task brief</label>
            <textarea
              id="advisor-handoff-brief"
              value={handoffBrief}
              onChange={(event) => {
                setHandoffBrief(event.target.value);
                setHandoffPending(false);
              }}
              rows={3}
            />
            {!handoffPending ? (
              <button
                type="button"
                disabled={!prompt.trim() || !handoffBrief.trim()}
                onClick={() => {
                  void onPrepareTaskHandoff({
                    title: "Advisor task",
                    originalRequest: prompt,
                    brief: handoffBrief,
                  }).then((result) =>
                    setHandoffPending(result.state === "pending"),
                  );
                }}
              >
                Review handoff
              </button>
            ) : (
              <>
                <p role="status">
                  Reviewed handoff ready. It expires and is used once.
                </p>
                <button type="button" onClick={() => void onOpenTaskHandoff()}>
                  Open in QuireForge
                </button>
              </>
            )}
          </section>
        )}
      {detailsOpen && (
        <aside
          id="advisor-details"
          className="advisor-details"
          aria-label="Advisor details"
        >
          <div className="advisor-details__header">
            <strong>Advisor details</strong>
            <button type="button" onClick={closeDetails}>
              Close Advisor details
            </button>
          </div>
          <p role="note">
            Advisor has no shell, terminal, Git, repository-write, or dispatch
            capability. Context and attachments are transient and require
            confirmation.
          </p>
          {selectedProjectState && (
            <p role="status">
              Selected Project State is temporary:{" "}
              {selectedProjectState.freshness}, {selectedProjectState.worktree}.
              It is not retained after restart.
            </p>
          )}
        </aside>
      )}
      {availability === "checking" && (
        <p role="status">Reading Advisor metadata.</p>
      )}
      {availability === "preview" && (
        <p>Browser preview cannot read Advisor metadata.</p>
      )}
      {availability === "error" && (
        <p className="project-message project-message--warning" role="alert">
          Advisor metadata could not be read; no state changed.
        </p>
      )}
      {availability === "native" && (
        <section
          className="project-card advisor-context-panel"
          aria-labelledby="advisor-source-title"
        >
          <h2 id="advisor-source-title">Selected Project State</h2>
          {selectedProjectState ? (
            <>
              <p>
                Temporary safe summary: {selectedProjectState.freshness},{" "}
                {selectedProjectState.worktree}. No project identity or source
                content is retained.
              </p>
              <button type="button" onClick={onRemoveProjectState}>
                Remove temporary snapshot
              </button>
            </>
          ) : (
            <>
              <p>
                Select one normalized local snapshot. Advisor does not browse
                repositories or retain it after restart.
              </p>
              <button
                type="button"
                disabled={
                  !canSelectProjectState || selectionState === "reading"
                }
                onClick={onRequestProjectState}
              >
                Select current Project State snapshot
              </button>
              {!canSelectProjectState && (
                <p className="project-message">
                  Select an attached project outside Advisor before choosing a
                  Project State source.
                </p>
              )}
            </>
          )}
          {selectionState === "confirming" && (
            <div
              className="project-confirmation"
              role="dialog"
              aria-modal="true"
              aria-label="Confirm Project State selection"
            >
              <p>
                Read one local Project State summary. No files, paths, images,
                remote refresh, or repository change is included.
              </p>
              <div className="project-actions">
                <button type="button" onClick={onConfirmProjectState}>
                  Confirm selection
                </button>
                <button type="button" onClick={onCancelProjectState}>
                  Cancel
                </button>
              </div>
            </div>
          )}
          {selectionState === "reading" && (
            <p role="status">Reading the selected local snapshot.</p>
          )}
          {selectionState === "error" && (
            <p
              className="project-message project-message--warning"
              role="alert"
            >
              The selected snapshot could not be read; no context was retained.
            </p>
          )}
        </section>
      )}
      {availability === "native" && (
        <section
          className="project-card advisor-approval-panel"
          aria-labelledby="advisor-approval-title"
        >
          <h2 id="advisor-approval-title">Approval draft</h2>
          <p role="note">
            This records a digest-only approval draft. It cannot start Codex,
            run a command, or change a project.
          </p>
          {!conversation.conversationId || !targetProjectId ? (
            <p className="project-message">
              Complete an Advisor conversation and select an attached project
              outside Advisor before preparing a draft.
            </p>
          ) : (
            <>
              <label htmlFor="advisor-dispatch-draft">Editable draft</label>
              <textarea
                id="advisor-dispatch-draft"
                ref={draftRef}
                value={draft}
                onChange={(event) => {
                  setDraft(event.target.value);
                  setApproval(null);
                }}
                rows={5}
              />
              <label htmlFor="advisor-draft-model">Requested model</label>
              <select
                id="advisor-draft-model"
                value={effectiveModel}
                onChange={(event) => {
                  setRequestedModel(event.target.value);
                  const model = runtime.models.find(
                    (entry) => entry.id === event.target.value,
                  );
                  setRequestedReasoning(model?.defaultReasoningEffort ?? "");
                  setApproval(null);
                }}
              >
                <option value="" disabled>
                  Select a live model
                </option>
                {runtime.models.map((model) => (
                  <option key={model.id} value={model.id}>
                    {model.displayName}
                  </option>
                ))}
              </select>
              <label htmlFor="advisor-draft-reasoning">
                Requested reasoning
              </label>
              <select
                id="advisor-draft-reasoning"
                value={effectiveReasoning}
                onChange={(event) => {
                  setRequestedReasoning(event.target.value);
                  setApproval(null);
                }}
              >
                <option value="" disabled>
                  Select reasoning
                </option>
                {(selectedModel?.supportedReasoningEfforts ?? []).map(
                  (effort) => (
                    <option key={effort} value={effort}>
                      {effort}
                    </option>
                  ),
                )}
              </select>
              <label htmlFor="advisor-draft-capability">
                Declared future capability
              </label>
              <select
                id="advisor-draft-capability"
                value={declaredCapability}
                onChange={(event) => {
                  setDeclaredCapability(
                    event.target.value as typeof declaredCapability,
                  );
                  setApproval(null);
                }}
              >
                <option value="read-only">Read-only</option>
                <option value="workspace-write">Workspace write</option>
                <option value="danger-full-access">Danger full access</option>
              </select>
              <p className="project-message">
                Proposed execution profile:{" "}
                {declaredCapability === "read-only"
                  ? "read-only sandbox; untrusted approvals"
                  : declaredCapability === "workspace-write"
                    ? "workspace-write sandbox; on-request approvals"
                    : "danger full access requested for a future execution; it is not granted here."}
              </p>
              <div className="project-actions">
                <button
                  type="button"
                  disabled={!currentApproval}
                  onClick={() => {
                    setApproval(null);
                    draftRef.current?.focus();
                  }}
                >
                  Edit draft
                </button>
                <button
                  type="button"
                  disabled={approvalBusy || !draft.trim() || !dispatchSupported}
                  onClick={() => void createDraft()}
                >
                  Create approval draft
                </button>
                <button
                  type="button"
                  disabled={!draft.trim()}
                  onClick={() => void navigator.clipboard?.writeText(draft)}
                >
                  Copy draft
                </button>
                {currentApproval?.state === "approved" && (
                  <button
                    type="button"
                    disabled={approvalBusy || !dispatchSupported}
                    onClick={() => void dispatchOnce()}
                  >
                    Dispatch once to execution workspace
                  </button>
                )}
              </div>
              {currentApproval && (
                <div className="project-confirmation" role="status">
                  <p>
                    Draft is {currentApproval.state}. A dispatch, if you select
                    it, is one-time only. Approval is revalidated against this
                    exact draft, context, target project, capability profile,
                    model, and reasoning choice before any future handoff. This
                    record expires at{" "}
                    {new Date(currentApproval.expiresAtMs).toLocaleTimeString()}
                    .
                  </p>
                  {currentApproval.state === "draft" && (
                    <div className="project-actions">
                      <button
                        type="button"
                        disabled={approvalBusy}
                        onClick={() => void decideDraft("approved")}
                      >
                        Approve draft
                      </button>
                      <button
                        type="button"
                        disabled={approvalBusy}
                        onClick={() => void decideDraft("rejected")}
                      >
                        Reject draft
                      </button>
                    </div>
                  )}
                </div>
              )}
              {!dispatchSupported && (
                <p
                  className="project-message project-message--warning"
                  role="status"
                >
                  Select a live model and reasoning. Danger full access is not
                  supported for B2 dispatch.
                </p>
              )}
              {dispatchResult && (
                <div className="project-confirmation" role="status">
                  <p>
                    {dispatchResult.state === "started"
                      ? "The approved request started in the execution workspace."
                      : "The dispatch did not start. Create a new approval before trying again."}
                  </p>
                  <button
                    type="button"
                    disabled={approvalBusy}
                    onClick={() => void readCompletionReport()}
                  >
                    Read bounded execution report
                  </button>
                </div>
              )}
              {completionReport && (
                <div className="project-message" role="status">
                  <p>
                    Execution report: {completionReport.status}
                    {completionReport.status === "unavailable"
                      ? ". Completion evidence is unavailable; this does not mean execution succeeded."
                      : "."}
                  </p>
                  {completionReport.status !== "unavailable" && (
                    <button type="button" onClick={onOpenExecution}>
                      Open in Execution Workspace
                    </button>
                  )}
                </div>
              )}
            </>
          )}
        </section>
      )}
      {availability === "native" && (
        <section
          className="project-card advisor-chat-panel"
          aria-labelledby="advisor-chat-title"
        >
          <h2
            id="advisor-chat-title"
            className={hasConversation ? "sr-only" : undefined}
          >
            Advisor conversation
          </h2>
          <p className={hasConversation ? "sr-only" : undefined}>
            Uses the managed ChatGPT browser sign-in through Codex. No project
            browsing, tools, or execution permissions are available. You may
            explicitly include one bounded text or data file with one message.
          </p>
          {authentication !== "ready" && (
            <p className="project-message" role="status">
              {authentication === "sign-in-pending"
                ? "Browser sign-in is pending. Finish it in the native flow, then refresh Settings → General."
                : "Advisor is unavailable until managed ChatGPT browser sign-in is complete."}
            </p>
          )}
          {conversation.diagnosticCode && (
            <p
              className="project-message project-message--warning"
              role="alert"
            >
              {diagnosticMessage[conversation.diagnosticCode] ??
                "Advisor could not complete the requested conversation action."}
            </p>
          )}
          {actionError && (
            <p
              className="project-message project-message--warning"
              role="alert"
            >
              QuireForge could not reach the native Advisor bridge.
            </p>
          )}
          <div
            className="conversation-events conversation-events--advisor"
            ref={conversationViewportRef}
            id="advisor-conversation-log"
            role="log"
            aria-label="Active Advisor conversation"
            aria-live="polite"
            aria-relevant="additions text"
            tabIndex={0}
            onScroll={updateViewportPosition}
          >
            {coalesceAssistantMessageDeltas(conversation.events).map((event) =>
              event.type === "agent-message-delta" ? (
                <p className="conversation-event__message" key={event.sequence}>
                  {event.delta}
                </p>
              ) : event.type === "reasoning-summary-delta" ? (
                <details
                  className="conversation-event__reasoning"
                  key={event.sequence}
                >
                  <summary>Reasoning summary</summary>
                  <p>{event.delta}</p>
                </details>
              ) : (
                <p
                  className="project-message project-message--warning"
                  key={event.sequence}
                >
                  Advisor reported {event.code}.
                </p>
              ),
            )}
          </div>
          {!followLatest && hasConversation && (
            <button
              type="button"
              className="advisor-jump-to-latest"
              aria-controls="advisor-conversation-log"
              onClick={jumpToLatest}
            >
              Jump to latest
            </button>
          )}
          {conversation.state === "completed" && latestReply && (
            <div className="conversation-actions">
              {exportCandidates.length > 1 && (
                <label>
                  Output to save
                  <select
                    value={exportCandidateIndex}
                    onChange={(event) =>
                      setExportCandidateIndex(Number(event.target.value))
                    }
                  >
                    {exportCandidates.map((candidate, index) => (
                      <option
                        key={`${candidate.suggestedName}-${index}`}
                        value={index}
                      >
                        {candidate.label}
                      </option>
                    ))}
                  </select>
                </label>
              )}
              <button type="button" onClick={() => void exportLatestReply()}>
                Save selected Advisor output
              </button>
            </div>
          )}
          <div className="conversation-composer">
            {selectedProjectState && (
              <div className="advisor-composer-context" role="status">
                <span>
                  Project State: {selectedProjectState.freshness},{" "}
                  {selectedProjectState.worktree}
                </span>
                <button type="button" onClick={onRemoveProjectState}>
                  Remove
                </button>
              </div>
            )}
            <label className="sr-only" htmlFor="advisor-prompt">
              Advisor message
            </label>
            <textarea
              id="advisor-prompt"
              aria-label="Advisor message"
              value={prompt}
              onChange={(event) => setPrompt(event.target.value)}
              disabled={
                conversationBusy || active || authentication !== "ready"
              }
              placeholder="Plan a milestone, review a safe Project State summary, or prepare a draft…"
              rows={4}
            />
            {canIncludeProjectState && (
              <label>
                <input
                  type="checkbox"
                  aria-label="Include the selected temporary Project State summary with this message"
                  checked={includeProjectState}
                  disabled={
                    conversationBusy || active || authentication !== "ready"
                  }
                  onChange={(event) =>
                    setIncludeProjectState(event.target.checked)
                  }
                />{" "}
                Include the selected temporary Project State summary with this
                message
              </label>
            )}
            <div
              className="advisor-text-attachment"
              aria-label="Optional text attachment"
            >
              <p className="project-message" role="note">
                Optional: choose one .txt, .md, .csv, .json, or .py file up to
                512 KiB. Its text is temporary, has no path attached, and is
                included only after you confirm this message.
              </p>
              {textAttachment.state === "ready" && textAttachment.attachment ? (
                <div>
                  <p role="status">
                    Ready: {textAttachment.attachment.displayName} (
                    {textAttachment.attachment.byteSize} bytes, SHA-256
                    verified).
                  </p>
                  <button
                    type="button"
                    disabled={attachmentBusy || active || conversationBusy}
                    onClick={() => void clearTextAttachment()}
                  >
                    Remove attached text file
                  </button>
                </div>
              ) : (
                <button
                  type="button"
                  onClick={() => void pickTextAttachment()}
                  disabled={
                    attachmentBusy ||
                    active ||
                    conversationBusy ||
                    authentication !== "ready" ||
                    imageAttachment.state === "ready" ||
                    documentAttachment.state === "ready" ||
                    archiveAttachment.state === "ready" ||
                    binaryAttachment.state === "ready"
                  }
                >
                  Attach text or data file
                </button>
              )}
              {textAttachment.state === "unavailable" && (
                <p
                  className="project-message project-message--warning"
                  role="alert"
                >
                  The text attachment was not available and was not included.
                </p>
              )}
            </div>
            <div
              className="advisor-text-attachment"
              aria-label="Optional image attachment"
            >
              <p className="project-message" role="note">
                Optional: choose one PNG or JPEG image up to 4 MiB. Its preview
                and manifest are temporary, its source path is not shared, and
                it is sent only after confirmation.
              </p>
              {imageAttachment.state === "ready" &&
              imageAttachment.attachment &&
              imageAttachment.previewDataUrl ? (
                <div>
                  <img
                    src={imageAttachment.previewDataUrl}
                    alt={`Preview of ${imageAttachment.attachment.displayName}`}
                    className="advisor-image-preview"
                  />
                  <p role="status">
                    Ready: {imageAttachment.attachment.displayName} (
                    {imageAttachment.attachment.mediaType},{" "}
                    {imageAttachment.attachment.byteSize} bytes,{" "}
                    {imageAttachment.attachment.width} ×{" "}
                    {imageAttachment.attachment.height}, SHA-256 verified).
                  </p>
                  <button
                    type="button"
                    disabled={attachmentBusy || active || conversationBusy}
                    onClick={() => void clearImageAttachment()}
                  >
                    Remove attached image
                  </button>
                </div>
              ) : (
                <button
                  type="button"
                  disabled={
                    attachmentBusy ||
                    active ||
                    conversationBusy ||
                    authentication !== "ready" ||
                    textAttachment.state === "ready" ||
                    documentAttachment.state === "ready" ||
                    archiveAttachment.state === "ready" ||
                    binaryAttachment.state === "ready"
                  }
                  onClick={() => void pickImageAttachment()}
                >
                  Attach PNG or JPEG image
                </button>
              )}
              {imageAttachment.state === "unavailable" && (
                <p
                  className="project-message project-message--warning"
                  role="alert"
                >
                  The image attachment was not available and was not included.
                </p>
              )}
            </div>
            <div
              className="advisor-text-attachment"
              aria-label="Optional PDF document attachment"
            >
              <p className="project-message" role="note">
                Optional: choose one PDF up to 8 MiB. Advisor receives only a
                temporary bounded text projection, never document bytes or a
                source path.
              </p>
              {documentAttachment.state === "ready" &&
              documentAttachment.attachment ? (
                <div>
                  <p role="status">
                    Ready: {documentAttachment.attachment.displayName} (PDF,{" "}
                    {documentAttachment.attachment.byteSize} bytes,{" "}
                    {documentAttachment.attachment.projection.includedPageCount}{" "}
                    of {documentAttachment.attachment.projection.pageCount}{" "}
                    pages projected
                    {documentAttachment.attachment.projection.partialPageCount >
                    0
                      ? "; final page truncated"
                      : ""}
                    , SHA-256 verified).
                  </p>
                  <button
                    type="button"
                    disabled={attachmentBusy || active || conversationBusy}
                    onClick={() => void clearDocumentAttachment()}
                  >
                    Remove attached PDF
                  </button>
                </div>
              ) : (
                <button
                  type="button"
                  disabled={
                    attachmentBusy ||
                    active ||
                    conversationBusy ||
                    authentication !== "ready" ||
                    textAttachment.state === "ready" ||
                    imageAttachment.state === "ready" ||
                    archiveAttachment.state === "ready" ||
                    binaryAttachment.state === "ready"
                  }
                  onClick={() => void pickDocumentAttachment()}
                >
                  Attach PDF document
                </button>
              )}
              {documentAttachment.state === "unavailable" && (
                <p
                  className="project-message project-message--warning"
                  role="alert"
                >
                  {documentAttachment.diagnosticCode ===
                  "malformed-or-unsupported-document"
                    ? "This PDF is malformed or unsupported and was not included."
                    : documentAttachment.diagnosticCode === "encrypted"
                      ? "Encrypted PDFs are not supported and were not included."
                      : documentAttachment.diagnosticCode === "embedded-content"
                        ? "PDF embedded content is not supported and was not included."
                        : documentAttachment.diagnosticCode ===
                            "external-action"
                          ? "PDF external actions are not supported and were not included."
                          : documentAttachment.diagnosticCode ===
                              "active-content"
                            ? "PDF active content is not supported and was not included."
                            : "The PDF was not available and was not included."}
                </p>
              )}
            </div>
            <div
              className="advisor-text-attachment"
              aria-label="Optional ZIP archive attachment"
            >
              <p className="project-message" role="note">
                Optional: choose one ZIP archive up to 32 MiB. Advisor receives
                only a temporary bounded entry-name manifest, never archive
                contents or a source path.
              </p>
              {archiveAttachment.state === "ready" &&
              archiveAttachment.attachment ? (
                <div>
                  <p role="status">
                    Ready: {archiveAttachment.attachment.displayName} (ZIP,{" "}
                    {archiveAttachment.attachment.byteSize} bytes,{" "}
                    {archiveAttachment.attachment.projection.includedEntryCount}{" "}
                    of{" "}
                    {
                      archiveAttachment.attachment.projection
                        .discoveredEntryCount
                    }{" "}
                    entries listed
                    {archiveAttachment.attachment.projection.truncated
                      ? "; manifest truncated"
                      : ""}
                    , SHA-256 verified).
                  </p>
                  <button
                    type="button"
                    disabled={attachmentBusy || active || conversationBusy}
                    onClick={() => void clearArchiveAttachment()}
                  >
                    Remove attached ZIP archive
                  </button>
                </div>
              ) : (
                <button
                  type="button"
                  disabled={
                    attachmentBusy ||
                    active ||
                    conversationBusy ||
                    authentication !== "ready" ||
                    textAttachment.state === "ready" ||
                    imageAttachment.state === "ready" ||
                    documentAttachment.state === "ready" ||
                    binaryAttachment.state === "ready"
                  }
                  onClick={() => void pickArchiveAttachment()}
                >
                  Attach ZIP archive
                </button>
              )}
              {archiveAttachment.state === "unavailable" && (
                <p
                  className="project-message project-message--warning"
                  role="alert"
                >
                  {archiveAttachment.diagnosticCode === "encrypted-archive"
                    ? "Encrypted ZIP archives are not supported and were not included."
                    : archiveAttachment.diagnosticCode === "unsafe-entry-path"
                      ? "This ZIP has an unsafe entry path and was not included."
                      : archiveAttachment.diagnosticCode === "duplicate-entry"
                        ? "This ZIP has ambiguous duplicate entries and was not included."
                        : "The ZIP archive was not available and was not included."}
                </p>
              )}
            </div>
            <div
              className="advisor-text-attachment"
              aria-label="Optional ELF static-binary attachment"
            >
              <p className="project-message" role="note">
                Optional: choose one ELF32 or ELF64 file up to 32 MiB. Advisor
                receives only temporary bounded static metadata, never binary
                bytes, paths, symbols, or executable content.
              </p>
              {binaryAttachment.state === "ready" &&
              binaryAttachment.attachment ? (
                <div>
                  <p role="status">
                    Ready: {binaryAttachment.attachment.displayName} (ELF,{" "}
                    {binaryAttachment.attachment.byteSize} bytes,{" "}
                    {binaryAttachment.attachment.projection.fileType}, SHA-256
                    verified).
                  </p>
                  <button
                    type="button"
                    disabled={attachmentBusy || active || conversationBusy}
                    onClick={() => void clearBinaryAttachment()}
                  >
                    Remove attached ELF file
                  </button>
                </div>
              ) : (
                <button
                  type="button"
                  disabled={
                    attachmentBusy ||
                    active ||
                    conversationBusy ||
                    authentication !== "ready" ||
                    textAttachment.state === "ready" ||
                    imageAttachment.state === "ready" ||
                    documentAttachment.state === "ready" ||
                    archiveAttachment.state === "ready"
                  }
                  onClick={() => void pickBinaryAttachment()}
                >
                  Attach ELF file
                </button>
              )}
              {binaryAttachment.state === "unavailable" && (
                <p
                  className="project-message project-message--warning"
                  role="alert"
                >
                  {binaryAttachment.diagnosticCode === "invalid-signature"
                    ? "This file is not a supported ELF file and was not included."
                    : binaryAttachment.diagnosticCode ===
                        "metadata-limit-exceeded"
                      ? "This ELF exceeds the static metadata inspection limit and was not included."
                      : "The ELF file was not available and was not included."}
                </p>
              )}
            </div>
            <p className="project-message" role="note">
              Advisor is read-only: no commands, project changes, or dispatch.
              Project State and one text/data file, one image, one PDF
              projection, one ZIP manifest, or one ELF static metadata manifest
              are optional and require confirmation.
            </p>
            {sendDisabledReason && !active && (
              <p className="project-message" role="status">
                {sendDisabledReason}
              </p>
            )}
            <div className="conversation-actions">
              {active ? (
                <button
                  type="button"
                  disabled={conversationBusy}
                  onClick={() => {
                    if (conversation.conversationId)
                      void onConversationInterrupt(conversation.conversationId);
                  }}
                >
                  Stop
                </button>
              ) : (
                <button
                  type="button"
                  disabled={
                    conversationBusy ||
                    !prompt.trim() ||
                    authentication !== "ready"
                  }
                  onClick={() => {
                    if (
                      textAttachment.state === "ready" ||
                      imageAttachment.state === "ready" ||
                      documentAttachment.state === "ready" ||
                      archiveAttachment.state === "ready" ||
                      binaryAttachment.state === "ready"
                    )
                      setConfirmAttachmentSend(true);
                    else if (includeProjectState && canIncludeProjectState)
                      setConfirmContextSend(true);
                    else submit(null);
                  }}
                >
                  Send to Advisor
                </button>
              )}
            </div>
          </div>
          {confirmContextSend && selectedProjectId && (
            <div
              className="project-confirmation"
              role="dialog"
              aria-modal="true"
              aria-label="Confirm Project State inclusion"
            >
              <p>
                Include the selected temporary Project State safe summary in
                this single Advisor message? It remains unretained by QuireForge
                and no project authority is granted.
              </p>
              <div className="project-actions">
                <button
                  type="button"
                  onClick={() => {
                    setConfirmContextSend(false);
                    submit(selectedProjectId);
                  }}
                >
                  Confirm inclusion
                </button>
                <button
                  type="button"
                  onClick={() => setConfirmContextSend(false)}
                >
                  Cancel
                </button>
              </div>
            </div>
          )}
          {confirmAttachmentSend &&
            (textAttachment.attachment ||
              imageAttachment.attachment ||
              documentAttachment.attachment ||
              archiveAttachment.attachment ||
              binaryAttachment.attachment) && (
              <div
                className="project-confirmation"
                role="dialog"
                aria-modal="true"
                aria-label="Confirm attachment inclusion"
              >
                <p>
                  Include{" "}
                  {
                    (
                      textAttachment.attachment ??
                      imageAttachment.attachment ??
                      documentAttachment.attachment ??
                      archiveAttachment.attachment ??
                      binaryAttachment.attachment
                    )?.displayName
                  }{" "}
                  as transient
                  {imageAttachment.attachment
                    ? " PNG/JPEG image data"
                    : documentAttachment.attachment
                      ? " bounded PDF text projection"
                      : archiveAttachment.attachment
                        ? " bounded ZIP entry-name manifest"
                        : binaryAttachment.attachment
                          ? " bounded ELF static metadata"
                          : " normalized text"}{" "}
                  in this one Advisor message? Its path is not shared, it is
                  consumed after this send, and it grants no project or
                  execution authority.
                </p>
                <div className="project-actions">
                  <button
                    type="button"
                    onClick={() => {
                      setConfirmAttachmentSend(false);
                      if (
                        includeProjectState &&
                        canIncludeProjectState &&
                        selectedProjectId
                      )
                        setConfirmContextSend(true);
                      else submit(null);
                    }}
                  >
                    Confirm inclusion
                  </button>
                  <button
                    type="button"
                    onClick={() => setConfirmAttachmentSend(false)}
                  >
                    Cancel
                  </button>
                </div>
              </div>
            )}
        </section>
      )}
    </section>
  );
}
