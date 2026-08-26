import { useState } from "react";

import type { LocalChatSnapshot } from "./lib/localChat";

interface LocalChatWorkspaceProps {
  onRun: (request: { message: string }) => Promise<LocalChatSnapshot>;
  onCancel: () => Promise<boolean>;
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

export function LocalChatWorkspace({
  onRun,
  onCancel,
}: LocalChatWorkspaceProps) {
  const [message, setMessage] = useState("");
  const [snapshot, setSnapshot] = useState<LocalChatSnapshot | null>(null);
  const [turns, setTurns] = useState<LocalChatTurn[]>([]);
  const [busy, setBusy] = useState(false);

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

  return (
    <section
      className="conversation-workspace"
      aria-labelledby="local-chat-title"
    >
      <header className="conversation-workspace__header">
        <div>
          <p className="eyebrow">Local Chat</p>
          <h1 id="local-chat-title">Start a conversation.</h1>
          <p>Local runtime · No project · Ephemeral</p>
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
            <button
              type="button"
              disabled={!message.trim()}
              onClick={() => void run()}
            >
              Send
            </button>
          )}
        </div>
      </div>
    </section>
  );
}
