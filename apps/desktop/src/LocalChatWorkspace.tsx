import { useState } from "react";

import type { LocalChatSnapshot } from "./lib/localChat";

interface LocalChatWorkspaceProps {
  onRun: (request: { message: string }) => Promise<LocalChatSnapshot>;
  onCancel: () => Promise<boolean>;
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
  const [busy, setBusy] = useState(false);

  async function run() {
    if (busy || !message.trim()) return;
    setBusy(true);
    setSnapshot(null);
    try {
      const result = await onRun({ message });
      setSnapshot(result);
      if (result.state === "completed") setMessage("");
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
      {snapshot?.output && (
        <div className="conversation-events" aria-live="polite">
          <p className="conversation-event__message">{snapshot.output}</p>
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
          disabled={busy}
          placeholder="Ask QuireForge anything…"
          rows={4}
        />
        <div className="conversation-composer__actions">
          <span>
            {busy ? "Local runtime is responding" : "No project context"}
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
