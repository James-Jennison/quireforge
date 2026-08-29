import { useEffect, useMemo, useRef, useState } from "react";

import { ConversationAttachmentTray } from "./ConversationAttachmentTray";
import {
  ModelSelectionPanel,
  ModelSelectionPolicyFields,
} from "./ModelSelectionPanel";
import type {
  ConversationAttachmentDropRequest,
  ConversationAttachmentSnapshot,
} from "./lib/attachment";
import type { CodexRuntimeSnapshot } from "./lib/codex";
import {
  conversationStartRequestSchema,
  type ConversationActionFailureCode,
  type ConversationApprovalDecisionRequest,
  type ConversationEvent,
  type ConversationSnapshot,
  type ConversationStartRequest,
} from "./lib/conversation";
import {
  buildConversationActivityViews,
  coalesceConversationMessageDeltas,
  type ConversationActivityView,
} from "./lib/conversationView";
import type { ProjectWorkspaceSnapshot } from "./lib/project";
import type { IntegrationCatalogSnapshot } from "./lib/integration";
import {
  interactionProfiles,
  type InteractionProfileId,
} from "./interactionProfiles";
import {
  defaultModelSelectionPolicy,
  type ModelSelectionSnapshot,
  type ModelSelectionUpdateRequest,
} from "./lib/modelSelection";

type ConversationAvailability = "checking" | "native" | "preview";
type Project = ProjectWorkspaceSnapshot["projects"][number];

interface ConversationWorkspaceProps {
  availability: ConversationAvailability;
  snapshot: ConversationSnapshot;
  events: ConversationEvent[];
  runtime: CodexRuntimeSnapshot;
  project: Project | undefined;
  integrations: IntegrationCatalogSnapshot;
  attachments: ConversationAttachmentSnapshot;
  busy: boolean;
  attachmentBusy: boolean;
  actionError: ConversationActionFailureCode | null;
  actionErrorDetail?: string | null;
  attachmentActionError: boolean;
  interactionProfile?: InteractionProfileId;
  onStart: (request: ConversationStartRequest) => Promise<ConversationSnapshot>;
  onRetryPoll: (conversationId: string) => Promise<ConversationSnapshot>;
  onInterrupt: (conversationId: string) => Promise<ConversationSnapshot>;
  onDecideApproval: (
    request: ConversationApprovalDecisionRequest,
  ) => Promise<ConversationSnapshot>;
  onUpdateModelSelection: (
    request: ModelSelectionUpdateRequest,
  ) => Promise<ModelSelectionSnapshot>;
  onAttachmentPick: (projectId: string) => Promise<void>;
  onAttachmentDrop: (
    request: ConversationAttachmentDropRequest,
  ) => Promise<void>;
  onAttachmentCancel: (
    projectId: string,
    attachmentId: string,
  ) => Promise<void>;
  handoffBrief?: string | null;
  onReturnTaskReceipt?: (summary: string) => Promise<void>;
}

const sandboxOptions = [
  { value: "read-only", label: "Read only" },
  { value: "workspace-write", label: "Workspace write" },
  { value: "danger-full-access", label: "Unrestricted" },
] as const;

const approvalOptions = [
  { value: "untrusted", label: "Ask for untrusted actions" },
  { value: "on-request", label: "Ask when Codex requests" },
] as const;

const stateLabels: Record<ConversationSnapshot["state"], string> = {
  empty: "Ready for a task",
  running: "Codex is working",
  "waiting-for-approval": "Codex is waiting for approval",
  stopping: "Stopping safely",
  completed: "Task completed",
  interrupted: "Task stopped",
  blocked: "Approval required",
  failed: "Task could not continue",
  unavailable: "Conversation runtime unavailable",
};

const diagnosticMessages: Record<
  NonNullable<ConversationSnapshot["diagnosticCode"]>,
  string
> = {
  "conversation-active": "Another task is already active.",
  "parallel-capacity-reached":
    "Four tasks are already active. Finish or stop one before starting another.",
  "conversation-not-found": "This task is no longer available.",
  "invalid-request": "Review the task settings and try again.",
  "project-unavailable": "The attached project is unavailable.",
  "project-identity-changed":
    "The project identity changed. Verify it before continuing.",
  "project-not-writable": "The project is not writable.",
  "project-busy": "This project is already in use by another task.",
  "runtime-unavailable": "The native Codex runtime is unavailable.",
  "model-unavailable": "The selected model is no longer available.",
  "reasoning-unavailable": "The selected reasoning level is unavailable.",
  "integration-unavailable":
    "A selected connector is no longer authorized, enabled, or callable.",
  "attachment-unavailable":
    "A staged image is no longer available. Add it again before retrying.",
  "metadata-unavailable":
    "QuireForge could not read its conversation metadata.",
  "approval-required": "Codex needs an approval before it can continue.",
  "approval-not-found": "That approval is no longer pending.",
  "approval-decision-unavailable":
    "That decision is not available for this approval.",
  "process-exited": "The Codex process exited before the task finished.",
  "transport-failed": "The connection to the Codex process was interrupted.",
  "protocol-invalid":
    "Codex returned data QuireForge could not safely display.",
  "rpc-rejected": "Codex rejected the requested operation.",
};

const actionFailureMessages: Record<ConversationActionFailureCode, string> = {
  "request-invalid":
    "The task request was rejected before native execution. Review the task text and controls, then try again.",
  "native-command-failed":
    "QuireForge could not reach the native conversation service. Verify the native bridge and Codex runtime, then try again.",
  "native-response-invalid":
    "QuireForge could not finish reading this response. The response already shown is still available.",
};

const activityLabels: Record<
  Extract<ConversationEvent, { type: "activity" }>["kind"],
  string
> = {
  "user-message": "Task submitted",
  "agent-message": "Response",
  plan: "Plan",
  reasoning: "Reasoning summary",
  "command-execution": "Command",
  "file-change": "File change",
  "tool-call": "Tool",
  "web-search": "Web search",
  image: "Image",
  other: "Activity",
};

const decisionLabels: Record<
  ConversationApprovalDecisionRequest["decision"],
  string
> = {
  approve: "Approve once",
  decline: "Decline",
  cancel: "Cancel task",
};

function ActivityCard({
  activity,
  expanded,
  onToggle,
}: {
  activity: ConversationActivityView;
  expanded: boolean;
  onToggle: () => void;
}) {
  const panelId = `conversation-activity-${activity.activityId}`;
  const label = activity.title || activityLabels[activity.kind];
  return (
    <article className="conversation-activity">
      <button
        className="conversation-activity__toggle"
        type="button"
        aria-expanded={expanded}
        aria-controls={panelId}
        onClick={onToggle}
      >
        <span
          className="conversation-activity__status"
          data-status={activity.status}
        >
          <span aria-hidden="true" />
          {activity.status}
        </span>
        <strong>{label}</strong>
        <span className="conversation-activity__chevron" aria-hidden="true">
          ›
        </span>
      </button>
      {expanded && (
        <div className="conversation-activity__panel" id={panelId}>
          <span className="conversation-activity__kind">
            {activityLabels[activity.kind]}
          </span>
          {activity.detail && (
            <div>
              <strong>Details</strong>
              <pre>{activity.detail}</pre>
            </div>
          )}
          {activity.output && (
            <div>
              <strong>Live output</strong>
              <pre aria-label={`${label} live output`}>{activity.output}</pre>
            </div>
          )}
          {activity.exitCode !== null && (
            <small>Exit code {activity.exitCode}</small>
          )}
          {!activity.detail &&
            !activity.output &&
            activity.exitCode === null && (
              <p>No additional normalized detail is available yet.</p>
            )}
        </div>
      )}
    </article>
  );
}

function EventCard({ event }: { event: ConversationEvent }) {
  if (event.type === "agent-message-completed") {
    return <p className="conversation-event__message">{event.text}</p>;
  }
  if (event.type === "agent-message-delta") {
    return <p className="conversation-event__message">{event.delta}</p>;
  }
  if (event.type === "reasoning-summary-delta") {
    return (
      <details className="conversation-event__reasoning">
        <summary>Reasoning summary</summary>
        <p>{event.delta}</p>
      </details>
    );
  }
  if (event.type === "plan-updated") {
    return (
      <div className="conversation-event__plan">
        <strong>Plan updated</strong>
        {event.explanation && <p>{event.explanation}</p>}
        <ol>
          {event.steps.map((step, index) => (
            <li data-state={step.status} key={`${event.sequence}-${index}`}>
              {step.step}
            </li>
          ))}
        </ol>
      </div>
    );
  }
  if (event.type === "activity" || event.type === "activity-output-delta")
    return null;
  if (event.type === "approval-requested") {
    return (
      <p className="conversation-event__approval">
        Approval requested for {event.kind.split("-").join(" ")}.
      </p>
    );
  }
  if (event.type === "approval-resolved") {
    return (
      <p className="conversation-event__approval">
        Approval {event.resolution.split("-").join(" ")}.
      </p>
    );
  }
  if (event.type === "model-selection-requested") {
    return (
      <p className="conversation-event__selection">
        Codex requested {event.choice.modelId} · {event.choice.reasoningEffort}{" "}
        for the next turn ({event.application}).
      </p>
    );
  }
  if (event.type === "error") {
    return (
      <p className="conversation-event__error" role="alert">
        {event.code.split("-").join(" ")}
        {event.willRetry ? " — retrying" : ""}
      </p>
    );
  }
  // The live header is the single source for passive lifecycle state. Repeating
  // individual STARTING/RUNNING frames in the transcript turns progress into
  // stacked system captions rather than a conversation.
  if (event.type === "lifecycle") return null;
  return null;
}

export function ConversationWorkspace({
  availability,
  snapshot,
  events,
  runtime,
  project,
  integrations,
  attachments,
  busy,
  attachmentBusy,
  actionError,
  actionErrorDetail = null,
  attachmentActionError,
  interactionProfile = "direct",
  onStart,
  onRetryPoll,
  onInterrupt,
  onDecideApproval,
  onUpdateModelSelection,
  onAttachmentPick,
  onAttachmentDrop,
  onAttachmentCancel,
  handoffBrief = null,
  onReturnTaskReceipt,
}: ConversationWorkspaceProps) {
  const controlsRef = useRef<HTMLDetailsElement>(null);
  const [controlsOpen, setControlsOpen] = useState(false);
  const defaultModel =
    runtime.models.find((model) => model.isDefault) ?? runtime.models[0];
  const [prompt, setPrompt] = useState(handoffBrief ?? "");
  const [submittedPrompt, setSubmittedPrompt] = useState<string | null>(null);
  const [receiptSummary, setReceiptSummary] = useState("");
  const [modelId, setModelId] = useState(defaultModel?.id ?? "");
  const [reasoningEffort, setReasoningEffort] = useState(
    defaultModel?.defaultReasoningEffort ?? "",
  );
  const [sandboxMode, setSandboxMode] =
    useState<ConversationStartRequest["sandboxMode"]>("workspace-write");
  const [approvalPolicy, setApprovalPolicy] =
    useState<ConversationStartRequest["approvalPolicy"]>("on-request");
  const [selectedInteractionProfile, setSelectedInteractionProfile] =
    useState<InteractionProfileId>(interactionProfile);
  const [selectionPolicy, setSelectionPolicy] = useState(
    defaultModelSelectionPolicy,
  );
  const [selectedConnectorIds, setSelectedConnectorIds] = useState<Set<string>>(
    new Set(),
  );
  const [expandedActivities, setExpandedActivities] = useState<Set<string>>(
    new Set(),
  );
  const [pendingDecision, setPendingDecision] = useState<
    ConversationApprovalDecisionRequest["decision"] | null
  >(null);
  const startInFlight = useRef(false);
  const decisionInFlight = useRef(false);

  useEffect(() => {
    if (!controlsOpen) return undefined;

    const closeOnOutsidePointer = (event: PointerEvent) => {
      if (!controlsRef.current?.contains(event.target as Node)) {
        setControlsOpen(false);
      }
    };
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") setControlsOpen(false);
    };

    document.addEventListener("pointerdown", closeOnOutsidePointer);
    document.addEventListener("keydown", closeOnEscape);
    return () => {
      document.removeEventListener("pointerdown", closeOnOutsidePointer);
      document.removeEventListener("keydown", closeOnEscape);
    };
  }, [controlsOpen]);

  const activities = useMemo(
    () => buildConversationActivityViews(events),
    [events],
  );
  const displayEvents = useMemo(
    () => coalesceConversationMessageDeltas(events),
    [events],
  );
  const activitiesByFirstSequence = useMemo(
    () =>
      new Map(activities.map((activity) => [activity.firstSequence, activity])),
    [activities],
  );

  const selectedModel =
    runtime.models.find((model) => model.id === modelId) ?? defaultModel;
  const effectiveModelId = selectedModel?.id ?? "";
  const effectiveReasoningEffort =
    selectedModel?.supportedReasoningEfforts.includes(reasoningEffort)
      ? reasoningEffort
      : (selectedModel?.defaultReasoningEffort ?? "");

  const projectReady =
    project !== undefined &&
    !project.archived &&
    project.directory?.state === "connected-accessible";
  const runtimeReady =
    runtime.availability === "ready" &&
    runtime.models.length > 0 &&
    runtime.capabilities.some(
      (capability) =>
        capability.id === "conversation-runtime" &&
        capability.state === "ready",
    );
  const active = ["running", "waiting-for-approval", "stopping"].includes(
    snapshot.state,
  );
  const availableConnectors = useMemo(
    () =>
      integrations.entries.filter(
        (entry) =>
          entry.kind === "connector" &&
          entry.authentication === "connected" &&
          entry.enablement === "enabled" &&
          entry.health.state === "ready",
      ),
    [integrations.entries],
  );
  const availableConnectorIds = useMemo(
    () => new Set(availableConnectors.map((entry) => entry.id)),
    [availableConnectors],
  );
  const effectiveSelectedConnectorIds = useMemo(
    () =>
      new Set(
        [...selectedConnectorIds].filter((entryId) =>
          availableConnectorIds.has(entryId),
        ),
      ),
    [availableConnectorIds, selectedConnectorIds],
  );
  const integrationEntryIds = useMemo(
    () =>
      availableConnectors
        .filter((entry) => effectiveSelectedConnectorIds.has(entry.id))
        .slice(0, 8)
        .map((entry) => entry.id),
    [availableConnectors, effectiveSelectedConnectorIds],
  );
  const attachmentIds = useMemo(
    () =>
      attachments.projectId === project?.id && attachments.state === "ready"
        ? attachments.attachments.map((attachment) => attachment.attachmentId)
        : [],
    [attachments, project?.id],
  );
  const request = useMemo(
    () => ({
      projectId: project?.id ?? "",
      prompt,
      attachmentIds,
      integrationEntryIds,
      modelId: effectiveModelId,
      reasoningEffort: effectiveReasoningEffort,
      selectionPolicy,
      sandboxMode,
      approvalPolicy,
      interactionProfile: selectedInteractionProfile,
    }),
    [
      approvalPolicy,
      effectiveModelId,
      effectiveReasoningEffort,
      project?.id,
      prompt,
      attachmentIds,
      integrationEntryIds,
      selectionPolicy,
      sandboxMode,
      selectedInteractionProfile,
    ],
  );
  const requestValid =
    conversationStartRequestSchema.safeParse(request).success;
  const canStart =
    availability === "native" &&
    projectReady &&
    runtimeReady &&
    snapshot.state !== "unavailable" &&
    !active &&
    !busy &&
    requestValid;

  async function beginTask() {
    if (!canStart || startInFlight.current) return;
    startInFlight.current = true;
    try {
      const result = await onStart(request);
      if (result.state === "running") {
        setSubmittedPrompt(prompt.trim());
        setPrompt("");
      }
    } catch {
      // The bounded action message is owned by App state.
    } finally {
      startInFlight.current = false;
    }
  }

  async function stopTask() {
    if (
      !snapshot.conversationId ||
      !["running", "waiting-for-approval"].includes(snapshot.state) ||
      busy
    )
      return;
    try {
      await onInterrupt(snapshot.conversationId);
    } catch {
      // The bounded action message is owned by App state.
    }
  }

  async function decideApproval(
    decision: ConversationApprovalDecisionRequest["decision"],
  ) {
    const approval = snapshot.pendingApproval;
    if (
      decisionInFlight.current ||
      busy ||
      !snapshot.conversationId ||
      !approval ||
      !approval.decisions.includes(decision)
    )
      return;

    decisionInFlight.current = true;
    setPendingDecision(decision);
    try {
      await onDecideApproval({
        conversationId: snapshot.conversationId,
        approvalId: approval.approvalId,
        decision,
      });
    } catch {
      // The bounded action message is owned by App state.
    } finally {
      decisionInFlight.current = false;
      setPendingDecision(null);
    }
  }

  function toggleActivity(activityId: string) {
    setExpandedActivities((current) => {
      const next = new Set(current);
      if (next.has(activityId)) next.delete(activityId);
      else next.add(activityId);
      return next;
    });
  }

  return (
    <section
      className="conversation-workspace"
      id="conversation"
      aria-labelledby="conversation-title"
    >
      <div className="conversation-workspace__intro">
        <div>
          <p className="eyebrow">Native conversation</p>
          <h1 id="conversation-title" data-workspace-heading tabIndex={-1}>
            Start a conversation with your project.
          </h1>
        </div>
        <details className="conversation-boundary-disclosure">
          <summary>About this workspace</summary>
          <p>
            Work stays scoped to the attached directory. QuireForge displays a
            normalized event stream and does not persist transcript content.
            Background approval, completion, and failure alerts use fixed text
            without project names, prompts, paths, or task output.
          </p>
        </details>
      </div>

      <div className="conversation-layout">
        {handoffBrief && onReturnTaskReceipt && (
          <section className="project-card" aria-label="Task handoff receipt">
            <p>
              Opened from a reviewed Advisor brief. Review a bounded completion
              receipt before returning to Advisor.
            </p>
            <label htmlFor="task-handoff-receipt">Completion receipt</label>
            <textarea
              id="task-handoff-receipt"
              value={receiptSummary}
              onChange={(event) => setReceiptSummary(event.target.value)}
              maxLength={4 * 1024}
            />
            <button
              type="button"
              disabled={!receiptSummary.trim()}
              onClick={() => void onReturnTaskReceipt(receiptSummary)}
            >
              Review in Advisor
            </button>
          </section>
        )}
        <form
          className="conversation-composer"
          data-visual-region="conversation-composer"
          onSubmit={(event) => {
            event.preventDefault();
            void beginTask();
          }}
        >
          <label
            className="conversation-composer__label"
            htmlFor="conversation-prompt"
          >
            Message
          </label>
          <textarea
            id="conversation-prompt"
            maxLength={64 * 1024}
            placeholder="Ask QuireForge to investigate, change, explain, or build…"
            value={prompt}
            disabled={active || busy}
            onChange={(event) => setPrompt(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter" && !event.shiftKey) {
                event.preventDefault();
                void beginTask();
              }
            }}
          />
          <ConversationAttachmentTray
            availability={availability}
            projectId={project?.id ?? null}
            snapshot={attachments}
            busy={attachmentBusy}
            disabled={active || busy || !projectReady}
            actionError={attachmentActionError}
            onPick={onAttachmentPick}
            onDrop={onAttachmentDrop}
            onCancel={onAttachmentCancel}
          />
          <details
            ref={controlsRef}
            className="conversation-options"
            open={controlsOpen}
          >
            <summary
              onClick={(event) => {
                event.preventDefault();
                setControlsOpen((current) => !current);
              }}
            >
              Controls
            </summary>
            {controlsOpen && (
              <button
                className="conversation-options__close"
                type="button"
                onClick={() => setControlsOpen(false)}
              >
                Close controls
              </button>
            )}
            <p>
              The workspace stays a chat. Adjust style, tools, model, or access
              only when this turn needs them.
            </p>
            <fieldset className="conversation-profile">
              <legend>Conversation style</legend>
              <p>
                This changes assistant prose only. Action Cards, authority and
                disclosure copy, lock labels, and failure messages stay exactly
                the same.
              </p>
              {interactionProfiles.map((profile) => (
                <label key={profile.id}>
                  <input
                    type="radio"
                    name="conversation-interaction-profile"
                    value={profile.id}
                    checked={selectedInteractionProfile === profile.id}
                    disabled={active || busy}
                    onChange={() => setSelectedInteractionProfile(profile.id)}
                  />
                  <span>{profile.label}</span>
                </label>
              ))}
            </fieldset>
            <fieldset className="conversation-integrations">
              <legend>Connected integrations</legend>
              {availableConnectors.length ? (
                availableConnectors.map((entry) => (
                  <label key={entry.id}>
                    <input
                      type="checkbox"
                      checked={effectiveSelectedConnectorIds.has(entry.id)}
                      disabled={
                        active ||
                        busy ||
                        availability !== "native" ||
                        (!effectiveSelectedConnectorIds.has(entry.id) &&
                          integrationEntryIds.length >= 8)
                      }
                      onChange={(event) => {
                        setSelectedConnectorIds((current) => {
                          const next = new Set(
                            [...current].filter((entryId) =>
                              availableConnectorIds.has(entryId),
                            ),
                          );
                          if (event.target.checked) next.add(entry.id);
                          else next.delete(entry.id);
                          return next;
                        });
                      }}
                    />
                    <span>{entry.displayName}</span>
                  </label>
                ))
              ) : (
                <p>
                  No authorized, enabled, and healthy connector is available for
                  this task.
                </p>
              )}
            </fieldset>
            <div className="conversation-controls">
              <label>
                <span>Model</span>
                <select
                  aria-label="Model"
                  value={effectiveModelId}
                  disabled={active || busy || !runtimeReady}
                  onChange={(event) => {
                    const nextModel = runtime.models.find(
                      (model) => model.id === event.target.value,
                    );
                    setModelId(event.target.value);
                    if (nextModel) {
                      setReasoningEffort(nextModel.defaultReasoningEffort);
                      setSelectionPolicy((current) =>
                        current.ownership === "automatic"
                          ? {
                              ...current,
                              allowedModelIds: [
                                ...new Set([
                                  ...current.allowedModelIds,
                                  nextModel.id,
                                ]),
                              ].slice(0, 32),
                            }
                          : current,
                      );
                    }
                  }}
                >
                  {runtime.models.map((model) => (
                    <option value={model.id} key={model.id}>
                      {model.displayName}
                      {model.isDefault ? " — default" : ""}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                <span>Reasoning</span>
                <select
                  aria-label="Reasoning"
                  value={effectiveReasoningEffort}
                  disabled={active || busy || !selectedModel}
                  onChange={(event) => setReasoningEffort(event.target.value)}
                >
                  {selectedModel?.supportedReasoningEfforts.map((effort) => (
                    <option value={effort} key={effort}>
                      {effort}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                <span>Filesystem</span>
                <select
                  aria-label="Filesystem access"
                  value={sandboxMode}
                  disabled={active || busy}
                  onChange={(event) =>
                    setSandboxMode(
                      event.target
                        .value as ConversationStartRequest["sandboxMode"],
                    )
                  }
                >
                  {sandboxOptions.map((option) => (
                    <option value={option.value} key={option.value}>
                      {option.label}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                <span>Approvals</span>
                <select
                  aria-label="Approval policy"
                  value={approvalPolicy}
                  disabled={active || busy}
                  onChange={(event) =>
                    setApprovalPolicy(
                      event.target
                        .value as ConversationStartRequest["approvalPolicy"],
                    )
                  }
                >
                  {approvalOptions.map((option) => (
                    <option value={option.value} key={option.value}>
                      {option.label}
                    </option>
                  ))}
                </select>
              </label>
            </div>

            {selectedModel && (
              <ModelSelectionPolicyFields
                idPrefix="conversation-start-selection"
                policy={selectionPolicy}
                effectiveChoice={{
                  modelId: effectiveModelId,
                  reasoningEffort: effectiveReasoningEffort,
                }}
                models={runtime.models}
                disabled={active || busy || !runtimeReady}
                onChange={setSelectionPolicy}
              />
            )}
            {snapshot.conversationId && snapshot.modelSelection && (
              <ModelSelectionPanel
                key={[
                  snapshot.conversationId,
                  snapshot.modelSelection.availability,
                  snapshot.modelSelection.effective.modelId,
                  snapshot.modelSelection.effective.reasoningEffort,
                  snapshot.modelSelection.pending?.requestedAtMs ?? "none",
                  snapshot.modelSelection.policy.ownership,
                  snapshot.modelSelection.policy.userLocked,
                  snapshot.modelSelection.policy.allowedModelIds.join(","),
                  snapshot.modelSelection.policy.reasoningCeiling ?? "none",
                ].join(":")}
                conversationId={snapshot.conversationId}
                selection={snapshot.modelSelection}
                models={runtime.models}
                disabled={busy || availability !== "native"}
                onUpdate={onUpdateModelSelection}
              />
            )}
          </details>

          <div className="conversation-prerequisite" aria-live="polite">
            {availability === "checking" &&
              "Checking the native conversation runtime…"}
            {availability === "preview" &&
              "Browser preview cannot start or simulate a Codex task."}
            {availability === "native" &&
              !projectReady &&
              "Attach and verify a writable project before starting a task."}
            {availability === "native" &&
              projectReady &&
              !runtimeReady &&
              "A ready Codex conversation capability and model catalog are required."}
          </div>

          <div className="conversation-actions">
            {!active ? (
              <button
                className="conversation-start"
                type="submit"
                disabled={!canStart}
              >
                Send
              </button>
            ) : (
              <button
                className="conversation-stop"
                type="button"
                disabled={snapshot.state === "stopping" || busy}
                onClick={() => void stopTask()}
              >
                {snapshot.state === "stopping" ? "Stopping…" : "Stop task"}
              </button>
            )}
            <span>
              {projectReady
                ? `${project.displayName} · Enter to send · Shift+Enter to insert a line break`
                : "No runnable project"}
            </span>
          </div>
        </form>

        <div
          className="conversation-stream"
          aria-labelledby="conversation-stream-title"
        >
          <div className="conversation-stream__header">
            <div>
              <span>Conversation</span>
              <strong
                id="conversation-stream-title"
                role="status"
                aria-live="polite"
              >
                {stateLabels[snapshot.state]}
              </strong>
            </div>
            {(snapshot.state === "running" ||
              snapshot.state === "waiting-for-approval" ||
              snapshot.state === "stopping") && (
              <span className="conversation-pulse" aria-hidden="true" />
            )}
          </div>

          <div
            className="conversation-events"
            aria-live="polite"
            aria-relevant="additions"
          >
            {submittedPrompt && (
              <article className="conversation-event conversation-event--user-message">
                <p className="conversation-event__message">{submittedPrompt}</p>
              </article>
            )}
            {snapshot.pendingApproval && (
              <section
                className="conversation-approval"
                aria-labelledby={`approval-${snapshot.pendingApproval.approvalId}`}
              >
                <p className="eyebrow">Action required</p>
                <h2 id={`approval-${snapshot.pendingApproval.approvalId}`}>
                  {snapshot.pendingApproval.title}
                </h2>
                <span className="conversation-approval__kind">
                  {snapshot.pendingApproval.kind.split("-").join(" ")} approval
                </span>
                {snapshot.pendingApproval.reason && (
                  <p>{snapshot.pendingApproval.reason}</p>
                )}
                {snapshot.pendingApproval.details.length > 0 && (
                  <dl>
                    {snapshot.pendingApproval.details.map((detail) => (
                      <div key={detail.label}>
                        <dt>{detail.label}</dt>
                        <dd>{detail.value}</dd>
                      </div>
                    ))}
                  </dl>
                )}
                <div className="conversation-approval__actions">
                  {snapshot.pendingApproval.decisions.map((decision) => (
                    <button
                      key={decision}
                      type="button"
                      data-decision={decision}
                      disabled={busy || pendingDecision !== null}
                      onClick={() => void decideApproval(decision)}
                    >
                      {pendingDecision === decision
                        ? `${decisionLabels[decision]}…`
                        : decisionLabels[decision]}
                    </button>
                  ))}
                </div>
                <small>
                  Approval applies only to this requested action. Declining
                  keeps broader access unchanged.
                </small>
              </section>
            )}
            {displayEvents.length === 0 ? (
              <div className="conversation-empty">
                <span aria-hidden="true">›</span>
                <p>Normalized progress and response text will appear here.</p>
              </div>
            ) : (
              displayEvents.map((event) => {
                if (
                  event.type === "activity" ||
                  event.type === "activity-output-delta"
                ) {
                  const activity = activitiesByFirstSequence.get(
                    event.sequence,
                  );
                  if (!activity) return null;
                  return (
                    <ActivityCard
                      key={activity.activityId}
                      activity={activity}
                      expanded={expandedActivities.has(activity.activityId)}
                      onToggle={() => toggleActivity(activity.activityId)}
                    />
                  );
                }
                return (
                  <article
                    className={`conversation-event conversation-event--${event.type}`}
                    key={event.sequence}
                  >
                    <EventCard event={event} />
                  </article>
                );
              })
            )}
          </div>

          {snapshot.diagnosticCode && (
            <p className="conversation-diagnostic" role="alert">
              {diagnosticMessages[snapshot.diagnosticCode]}
            </p>
          )}
          {actionError && (
            <div className="conversation-diagnostic" role="alert">
              {actionFailureMessages[actionError]}
              {actionError === "native-response-invalid" &&
                actionErrorDetail && (
                  <p>Validation detail: {actionErrorDetail}</p>
                )}
              {actionError === "native-response-invalid" &&
                snapshot.conversationId && (
                  <button
                    type="button"
                    disabled={busy}
                    onClick={() => void onRetryPoll(snapshot.conversationId!)}
                  >
                    Retry
                  </button>
                )}
            </div>
          )}
        </div>
      </div>
    </section>
  );
}
