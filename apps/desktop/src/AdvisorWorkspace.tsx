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
} from "./lib/bridge";
import type {
  AdvisorApprovalSnapshot,
  AdvisorDispatchSnapshot,
} from "./lib/advisorApproval";
import type { CodexRuntimeSnapshot } from "./lib/codex";

interface AdvisorApprovalState {
  snapshot: AdvisorApprovalSnapshot;
  bindingKey: string;
}

interface AdvisorWorkspaceProps {
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
};

export function AdvisorWorkspace({
  availability,
  snapshot,
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
}: AdvisorWorkspaceProps) {
  const [prompt, setPrompt] = useState("");
  const [includeProjectState, setIncludeProjectState] = useState(false);
  const [confirmContextSend, setConfirmContextSend] = useState(false);
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
  const empty =
    snapshot?.conversationCount === 0 &&
    snapshot.contextReferenceCount === 0 &&
    snapshot.proposalCount === 0;
  const authentication = managedChatAuthenticationState(auth);
  const active =
    conversation.state === "running" && conversation.conversationId !== null;
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
    void onConversationStart({ prompt, projectId })
      .then(() => setPrompt(""))
      .catch(() => setActionError(true));
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

  return (
    <section
      className="project-workspace"
      id="advisor"
      aria-labelledby="advisor-title"
    >
      <p className="eyebrow">Advisor</p>
      <h1 id="advisor-title" data-workspace-heading tabIndex={-1}>
        Read-only planning, without execution.
      </h1>
      <p role="note">
        Advisor has no shell, terminal, Git, repository-write, or dispatch
        capability. Draft approval records are digest-only and cannot execute.
        QuireForge retains no prompt or transcript text.
      </p>
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
      {availability === "native" && snapshot && (
        <div className="project-list">
          <dl className="context-facts">
            <div>
              <dt>Conversations</dt>
              <dd>{snapshot.conversationCount}</dd>
            </div>
            <div>
              <dt>Contexts</dt>
              <dd>{snapshot.contextReferenceCount}</dd>
            </div>
            <div>
              <dt>Proposals</dt>
              <dd>{snapshot.proposalCount}</dd>
            </div>
          </dl>
          {empty ? (
            <p className="project-message">No Advisor metadata yet.</p>
          ) : (
            <ul
              className="project-list"
              aria-label="Advisor metadata summaries"
            >
              {snapshot.contextSummaries.map(
                ({ kind, freshness, trust }, index) => (
                  <li className="project-message" key={`${kind}-${index}`}>
                    {kind}: {trust}, {freshness}
                  </li>
                ),
              )}
              {snapshot.proposalSummaries.map(({ state }, index) => (
                <li className="project-message" key={`${state}-${index}`}>
                  Proposal digest: {state}, explicit approval required.
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
      {availability === "native" && (
        <section
          className="project-card"
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
          className="project-card"
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
                <p className="project-confirmation" role="status">
                  {dispatchResult.state === "started"
                    ? "The approved request started in the execution workspace."
                    : "The dispatch did not start. Create a new approval before trying again."}
                </p>
              )}
            </>
          )}
        </section>
      )}
      {availability === "native" && (
        <section className="project-card" aria-labelledby="advisor-chat-title">
          <h2 id="advisor-chat-title">Advisor conversation</h2>
          <p>
            Uses the managed ChatGPT browser sign-in through Codex. No project
            attachment, tools, or execution permissions are available.
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
          <div className="conversation-events" aria-live="polite">
            {conversation.events.map((event) =>
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
          <div className="conversation-composer">
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
            <p className="project-message" role="note">
              Advisor is read-only: no commands, project changes, or dispatch.
              Project State is optional and requires confirmation.
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
                    if (includeProjectState && canIncludeProjectState)
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
        </section>
      )}
    </section>
  );
}
