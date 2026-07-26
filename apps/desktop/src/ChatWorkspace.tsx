import { useEffect, useState } from "react";

import type { CodexAuthSnapshot } from "./lib/auth";
import {
  type ChatConversationSnapshot,
  type ChatConversationStartRequest,
} from "./lib/chat";
import { managedChatAuthenticationState } from "./lib/conversationMode";

interface ChatWorkspaceProps {
  auth: CodexAuthSnapshot;
  snapshot: ChatConversationSnapshot;
  busy: boolean;
  onStart: (
    request: ChatConversationStartRequest,
  ) => Promise<ChatConversationSnapshot>;
  onPoll: (conversationId: string) => Promise<ChatConversationSnapshot>;
  onInterrupt: (conversationId: string) => Promise<ChatConversationSnapshot>;
}

const diagnosticMessage: Record<
  NonNullable<ChatConversationSnapshot["diagnosticCode"]>,
  string
> = {
  "authentication-required":
    "Sign in with the managed ChatGPT browser flow before starting Chat.",
  "authentication-unavailable":
    "Chat requires a managed ChatGPT account. API-key and external-token accounts cannot enable it.",
  "conversation-not-found": "This Chat conversation is no longer available.",
  "conversation-active":
    "Finish or stop the active Chat conversation before starting another.",
  "invalid-request": "Enter a non-empty message and try again.",
  "runtime-unavailable": "The native Codex runtime is unavailable.",
  "protocol-invalid":
    "The native Chat bridge returned a response QuireForge could not safely use.",
  "capability-blocked":
    "Chat blocked an attempted native tool or permission request.",
  "metadata-unavailable":
    "QuireForge could not record bounded local Chat metadata.",
};

export function ChatWorkspace({
  auth,
  snapshot,
  busy,
  onStart,
  onPoll,
  onInterrupt,
}: ChatWorkspaceProps) {
  const [prompt, setPrompt] = useState("");
  const [actionError, setActionError] = useState(false);
  const authentication = managedChatAuthenticationState(auth);
  const active =
    snapshot.state === "running" && snapshot.conversationId !== null;

  useEffect(() => {
    if (!active || !snapshot.conversationId) return undefined;
    const conversationId = snapshot.conversationId;
    const timer = window.setTimeout(() => {
      void onPoll(conversationId).catch(() => setActionError(true));
    }, 300);
    return () => window.clearTimeout(timer);
  }, [active, onPoll, snapshot.conversationId, snapshot.state]);

  async function submit() {
    if (busy || active || authentication !== "ready") return;
    setActionError(false);
    try {
      await onStart({ prompt });
      setPrompt("");
    } catch {
      setActionError(true);
    }
  }

  return (
    <section className="conversation-workspace" aria-labelledby="chat-title">
      <header className="conversation-workspace__header">
        <div>
          <p className="eyebrow">Chat</p>
          <h1 id="chat-title">A no-project conversation.</h1>
          <p>
            Chat has no attached directory, terminal, Git, worktree,
            integration, native-action, or approval capability.
          </p>
        </div>
      </header>

      <div className="conversation-boundary-note" role="note">
        <strong>Managed ChatGPT browser sign-in only.</strong>
        <p>
          QuireForge never accepts passwords, API keys, browser cookies, or
          external tokens for Chat.
        </p>
      </div>

      {authentication !== "ready" && (
        <p className="conversation-error" role="status">
          {authentication === "sign-in-pending"
            ? "Browser sign-in is pending. Finish it in the native flow, then refresh Settings → General."
            : "Chat is unavailable until a managed ChatGPT browser sign-in is complete."}
        </p>
      )}
      {snapshot.diagnosticCode && (
        <p className="conversation-error" role="alert">
          {diagnosticMessage[snapshot.diagnosticCode]}
        </p>
      )}
      {actionError && (
        <p className="conversation-error" role="alert">
          QuireForge could not reach the native Chat bridge.
        </p>
      )}

      <div className="conversation-events" aria-live="polite">
        {snapshot.events.map((event) =>
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
            <p className="conversation-error" key={event.sequence}>
              Chat reported {event.code}.
            </p>
          ),
        )}
      </div>

      <label className="conversation-composer" htmlFor="chat-prompt">
        <span className="sr-only">Chat message</span>
        <textarea
          id="chat-prompt"
          aria-label="Chat message"
          value={prompt}
          onChange={(event) => setPrompt(event.target.value)}
          disabled={busy || active || authentication !== "ready"}
          placeholder="Ask a question, explore an idea, or create a draft…"
          rows={4}
        />
        <div className="conversation-composer__actions">
          <span>{active ? "Chat is responding" : "No project context"}</span>
          {active ? (
            <button
              type="button"
              disabled={busy}
              onClick={() => {
                if (snapshot.conversationId)
                  void onInterrupt(snapshot.conversationId);
              }}
            >
              Stop
            </button>
          ) : (
            <button
              type="button"
              disabled={busy || !prompt.trim() || authentication !== "ready"}
              onClick={() => void submit()}
            >
              Send
            </button>
          )}
        </div>
      </label>
    </section>
  );
}
