import { useEffect, useRef, useState } from "react";

import type { ActionCardAction, ActionCardSnapshot } from "./lib/actionCard";
import type { LocalChatSnapshot } from "./lib/localChat";

interface LocalChatWorkspaceProps {
  onRun: (request: { message: string }) => Promise<LocalChatSnapshot>;
  onCancel: () => Promise<boolean>;
  onPrepareActionCard: (request: {
    action: ActionCardAction;
  }) => Promise<ActionCardSnapshot>;
  onApproveActionCard: (request: {
    cardId: string;
  }) => Promise<ActionCardSnapshot>;
  onRevokeActionCard: (request: {
    cardId: string;
  }) => Promise<ActionCardSnapshot>;
  onOpenLinkedProjectChat: () => void;
  onOpenBrowserResearch?: () => void;
}

interface LocalChatTurn {
  role: "user" | "assistant";
  text: string;
}

const diagnosticMessage: Record<string, string> = {
  "invalid-request": "Enter a message to start a local chat turn.",
  "input-too-large": "That message is too large for this local runtime.",
  "model-unavailable":
    "The local runtime is unavailable. Recheck it and try again.",
  "memory-ceiling-unavailable":
    "The local runtime memory boundary is unavailable.",
  "runtime-busy": "The local runtime is already working on one turn.",
  cancelled: "The local chat turn was cancelled.",
};

const actionLabels: Record<ActionCardAction, string> = {
  "attach-project": "Attach a project",
  "use-source": "Use a source",
  "draft-artifact": "Draft an artifact",
  "work-with-code": "Work with code",
};

export function LocalChatWorkspace({
  onRun,
  onCancel,
  onPrepareActionCard,
  onApproveActionCard,
  onRevokeActionCard,
  onOpenLinkedProjectChat,
  onOpenBrowserResearch,
}: LocalChatWorkspaceProps) {
  const [message, setMessage] = useState("");
  const [snapshot, setSnapshot] = useState<LocalChatSnapshot | null>(null);
  const [turns, setTurns] = useState<LocalChatTurn[]>([]);
  const [busy, setBusy] = useState(false);
  const [actionPickerOpen, setActionPickerOpen] = useState(false);
  const [actionCard, setActionCard] = useState<ActionCardSnapshot | null>(null);
  const [actionBusy, setActionBusy] = useState(false);
  const actionPickerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!actionPickerOpen) return undefined;
    const closePicker = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      setActionPickerOpen(false);
    };
    const closeOnOutsidePointer = (event: PointerEvent) => {
      if (actionPickerRef.current?.contains(event.target as Node)) return;
      setActionPickerOpen(false);
    };
    window.addEventListener("keydown", closePicker);
    document.addEventListener("pointerdown", closeOnOutsidePointer);
    return () => {
      window.removeEventListener("keydown", closePicker);
      document.removeEventListener("pointerdown", closeOnOutsidePointer);
    };
  }, [actionPickerOpen]);

  async function run() {
    if (busy || !message.trim()) return;
    const submittedMessage = message;
    setBusy(true);
    setSnapshot(null);
    setTurns((current) => [
      ...current,
      { role: "user", text: submittedMessage },
    ]);
    setMessage("");
    try {
      const result = await onRun({ message: submittedMessage });
      setSnapshot(result);
      if (result.state === "completed" && result.output) {
        setTurns((current) => [
          ...current,
          { role: "assistant", text: result.output ?? "" },
        ]);
      }
    } catch {
      setSnapshot({
        schemaVersion: 1,
        localOnly: true,
        state: "failed",
        output: null,
        diagnostic: "model-unavailable",
        inputTokenLimit: 4096,
        outputTokenLimit: 512,
        deadlineSeconds: 60,
        memoryCeilingMib: 6144,
      });
    } finally {
      setBusy(false);
    }
  }

  async function prepareActionCard(action: ActionCardAction) {
    if (actionBusy) return;
    setActionBusy(true);
    try {
      setActionCard(await onPrepareActionCard({ action }));
      setActionPickerOpen(false);
    } finally {
      setActionBusy(false);
    }
  }

  async function decideActionCard(approve: boolean) {
    if (!actionCard || actionBusy || actionCard.state !== "prepared") return;
    setActionBusy(true);
    try {
      setActionCard(
        await (approve
          ? onApproveActionCard({ cardId: actionCard.cardId })
          : onRevokeActionCard({ cardId: actionCard.cardId })),
      );
    } finally {
      setActionBusy(false);
    }
  }

  return (
    <section
      className="conversation-workspace"
      aria-labelledby="local-chat-title"
    >
      <header className="conversation-workspace__header">
        <div>
          <p className="eyebrow">Local Chat</p>
          <h1 id="local-chat-title">Start a conversation.</h1>
          <p>Local runtime · Project context not attached · Ephemeral</p>
          <button type="button" onClick={onOpenLinkedProjectChat}>
            Continue with linked project
          </button>
          <p>
            Opens a separate managed project conversation. This Local Chat
            transcript is not transferred automatically.
          </p>
          {onOpenBrowserResearch && (
            <button type="button" onClick={onOpenBrowserResearch}>
              Research Google (read only)
            </button>
          )}
          {onOpenBrowserResearch && (
            <p>
              Opens a separate, owner-approved Google review. Local Chat does
              not receive browser access or page content.
            </p>
          )}
        </div>
      </header>
      {turns.length > 0 && (
        <div
          className="conversation-events conversation-events--local-chat"
          aria-live="polite"
          aria-relevant="additions"
        >
          {turns.map((turn, index) => (
            <article
              className={`local-chat-turn local-chat-turn--${turn.role}`}
              key={`${turn.role}-${index}`}
            >
              <p className="local-chat-turn__role">
                {turn.role === "user" ? "You" : "QuireForge"}
              </p>
              <p className="conversation-event__message">{turn.text}</p>
            </article>
          ))}
        </div>
      )}
      {snapshot?.diagnostic && (
        <p className="conversation-error" role="status">
          {diagnosticMessage[snapshot.diagnostic] ??
            "The local chat turn could not complete."}
        </p>
      )}
      {actionCard && (
        <section className="action-card" aria-label="Action Card">
          <p className="eyebrow">Action Card</p>
          <h2>{actionLabels[actionCard.action]}</h2>
          <p>
            No project, source, artifact, code, provider, or tool data is
            selected or used by this card.
          </p>
          {actionCard.state === "prepared" ? (
            <>
              <p>
                Review this proposed boundary before any later
                capability-specific step.
              </p>
              <div className="action-card__actions">
                <button
                  type="button"
                  disabled={actionBusy}
                  onClick={() => void decideActionCard(true)}
                >
                  Approve for later
                </button>
                <button
                  type="button"
                  disabled={actionBusy}
                  onClick={() => void decideActionCard(false)}
                >
                  Revoke
                </button>
              </div>
            </>
          ) : actionCard.state === "approved" ? (
            <p>
              Approved for a later capability-specific step. No action has run.
            </p>
          ) : (
            <p>This Action Card is no longer available.</p>
          )}
        </section>
      )}
      <div className="conversation-composer">
        <label className="sr-only" htmlFor="local-chat-message">
          Local chat message
        </label>
        <textarea
          id="local-chat-message"
          aria-label="Local chat message"
          value={message}
          onChange={(event) => setMessage(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter" && !event.shiftKey) {
              event.preventDefault();
              void run();
            }
          }}
          disabled={busy}
          placeholder="Ask QuireForge anything…"
          rows={4}
        />
        <div className="conversation-composer__actions">
          <span>
            {busy
              ? "QuireForge is responding"
              : "Enter to send · Shift+Enter for a new line"}
          </span>
          {busy ? (
            <button type="button" onClick={() => void onCancel()}>
              Stop
            </button>
          ) : (
            <>
              <div className="action-card-picker" ref={actionPickerRef}>
                <button
                  type="button"
                  aria-expanded={actionPickerOpen}
                  aria-haspopup="menu"
                  onClick={() => setActionPickerOpen((open) => !open)}
                >
                  Actions
                </button>
                {actionPickerOpen && (
                  <div className="action-card-picker__menu" role="menu">
                    <p>
                      Prepare a visible proposal. Nothing will run from this
                      menu.
                    </p>
                    {(Object.keys(actionLabels) as ActionCardAction[]).map(
                      (action) => (
                        <button
                          type="button"
                          role="menuitem"
                          disabled={actionBusy}
                          key={action}
                          onClick={() => void prepareActionCard(action)}
                        >
                          {actionLabels[action]}
                        </button>
                      ),
                    )}
                  </div>
                )}
              </div>
              <button
                type="button"
                disabled={!message.trim()}
                onClick={() => void run()}
              >
                Send
              </button>
            </>
          )}
        </div>
      </div>
    </section>
  );
}
