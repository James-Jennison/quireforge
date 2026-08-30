import { useEffect, useRef, useState } from "react";

import { ConversationShell } from "./ConversationShell";
import type { CodexAuthSnapshot } from "./lib/auth";
import {
  type ChatConversationEvent,
  type ChatConversationSnapshot,
  type ChatConversationStartRequest,
  type ChatProviderId,
} from "./lib/chat";
import { managedChatAuthenticationState } from "./lib/conversationMode";
import type { InteractionProfileId } from "./interactionProfiles";

interface ChatWorkspaceProps {
  auth: CodexAuthSnapshot;
  snapshot: ChatConversationSnapshot;
  busy: boolean;
  provider: ChatProviderId | null;
  onProviderChange: (provider: ChatProviderId | null) => void;
  interactionProfile: InteractionProfileId;
  onInteractionProfileChange: (profile: InteractionProfileId) => void;
  onStart: (
    request: ChatConversationStartRequest,
  ) => Promise<ChatConversationSnapshot>;
  onPoll: (conversationId: string) => Promise<ChatConversationSnapshot>;
  onInterrupt: (conversationId: string) => Promise<ChatConversationSnapshot>;
}

interface ChatTurn {
  role: "user" | "assistant" | "reasoning" | "error";
  text: string;
}

const diagnosticMessage: Record<
  NonNullable<ChatConversationSnapshot["diagnosticCode"]>,
  string
> = {
  "authentication-required":
    "Managed Codex needs an existing Codex sign-in before it can be used.",
  "authentication-unavailable":
    "Managed Codex is unavailable for this account.",
  "conversation-not-found":
    "This Chat & Cowork conversation is no longer available.",
  "conversation-active": "Chat & Cowork is already responding.",
  "invalid-request": "Enter a non-empty message and try again.",
  "runtime-unavailable": "The managed Codex runtime is unavailable.",
  "protocol-invalid": "QuireForge could not safely read this response.",
  "capability-blocked":
    "Chat & Cowork blocked a tool, permission, or context request.",
  "metadata-unavailable": "QuireForge could not record bounded local metadata.",
};

function appendEvent(
  turns: ChatTurn[],
  event: ChatConversationEvent,
): ChatTurn[] {
  if (event.type === "agent-message-delta") {
    const last = turns.at(-1);
    if (last?.role === "assistant") {
      return [
        ...turns.slice(0, -1),
        { ...last, text: last.text + event.delta },
      ];
    }
    return [...turns, { role: "assistant", text: event.delta }];
  }
  if (event.type === "reasoning-summary-delta") {
    const last = turns.at(-1);
    if (last?.role === "reasoning") {
      return [
        ...turns.slice(0, -1),
        { ...last, text: last.text + event.delta },
      ];
    }
    return [...turns, { role: "reasoning", text: event.delta }];
  }
  return [
    ...turns,
    { role: "error", text: `Managed Codex reported ${event.code}.` },
  ];
}

export function ChatWorkspace({
  auth,
  snapshot,
  busy,
  provider,
  onProviderChange,
  interactionProfile,
  onInteractionProfileChange,
  onStart,
  onPoll,
  onInterrupt,
}: ChatWorkspaceProps) {
  const [prompt, setPrompt] = useState("");
  const [actionError, setActionError] = useState(false);
  const [turns, setTurns] = useState<ChatTurn[]>([]);
  const seenEventSequence = useRef(new Set<number>());
  const authentication = managedChatAuthenticationState(auth);
  const active =
    snapshot.state === "running" && snapshot.conversationId !== null;
  const managedCodexReady =
    provider === "managed-codex" && authentication === "ready";

  useEffect(() => {
    const unseen = snapshot.events.filter((event) => {
      if (seenEventSequence.current.has(event.sequence)) return false;
      seenEventSequence.current.add(event.sequence);
      return true;
    });
    if (unseen.length) {
      setTurns((current) => unseen.reduce(appendEvent, current));
    }
  }, [snapshot.events]);

  useEffect(() => {
    if (!active || !snapshot.conversationId) return undefined;
    const conversationId = snapshot.conversationId;
    const timer = window.setTimeout(() => {
      void onPoll(conversationId).catch(() => setActionError(true));
    }, 300);
    return () => window.clearTimeout(timer);
  }, [active, onPoll, snapshot.conversationId]);

  async function submit() {
    if (busy || active || !managedCodexReady || !prompt.trim()) return;
    const submittedPrompt = prompt;
    setActionError(false);
    setTurns((current) => [
      ...current,
      { role: "user", text: submittedPrompt },
    ]);
    setPrompt("");
    try {
      await onStart({ prompt: submittedPrompt, interactionProfile });
    } catch {
      setActionError(true);
      setPrompt(submittedPrompt);
    }
  }

  return (
    <ConversationShell
      mode="chat"
      id="advisor"
      titleId="chat-cowork-title"
      eyebrow="Chat & Cowork"
      title="Start a conversation."
      boundary={
        <details className="conversation-boundary-disclosure">
          <summary>About Chat & Cowork</summary>
          <p>
            No project, Code tools, browser content, filesystem, attachments, or
            automatic context transfer are available here. A provider is used
            only after you explicitly select it.
          </p>
        </details>
      }
      shelf={
        <div className="conversation-mode-shelf">
          {provider === null ? (
            <button
              type="button"
              onClick={() => onProviderChange("managed-codex")}
            >
              Use managed Codex
            </button>
          ) : (
            <span role="status">Provider: Managed Codex</span>
          )}
        </div>
      }
    >
      <section className="conversation-transcript" aria-live="polite">
        {turns.map((turn, index) =>
          turn.role === "reasoning" ? (
            <details
              className="conversation-event__reasoning"
              key={`${turn.role}-${index}`}
            >
              <summary>Reasoning summary</summary>
              <p>{turn.text}</p>
            </details>
          ) : (
            <p
              className={
                turn.role === "user"
                  ? "conversation-turn conversation-turn--user"
                  : turn.role === "error"
                    ? "conversation-error"
                    : "conversation-turn conversation-turn--assistant"
              }
              key={`${turn.role}-${index}`}
            >
              {turn.text}
            </p>
          ),
        )}
      </section>

      {provider === null && (
        <p className="conversation-boundary-note" role="status">
          No provider connected. Choose a provider before sending; your draft
          stays here until you do.
        </p>
      )}
      {provider === "managed-codex" && authentication !== "ready" && (
        <p className="conversation-error" role="status">
          {
            diagnosticMessage[
              authentication === "unavailable"
                ? "authentication-unavailable"
                : "authentication-required"
            ]
          }
        </p>
      )}
      {snapshot.diagnosticCode && (
        <p className="conversation-error" role="alert">
          {diagnosticMessage[snapshot.diagnosticCode]}
        </p>
      )}
      {actionError && (
        <p className="conversation-error" role="alert">
          QuireForge could not reach the managed Codex bridge.
        </p>
      )}

      <label className="conversation-composer" htmlFor="chat-prompt">
        <span className="sr-only">Chat message</span>
        <textarea
          id="chat-prompt"
          aria-label="Chat message"
          value={prompt}
          onChange={(event) => setPrompt(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter" && !event.shiftKey) {
              event.preventDefault();
              void submit();
            }
          }}
          disabled={busy || active}
          placeholder="Ask QuireForge anything…"
          rows={2}
        />
        <div className="conversation-composer__actions">
          <span>
            {active ? "Managed Codex is responding" : "No project context"}
          </span>
          <fieldset aria-label="Conversation style" disabled={busy || active}>
            <label>
              <input
                type="radio"
                name="chat-interaction-profile"
                checked={interactionProfile === "direct"}
                onChange={() => onInteractionProfileChange("direct")}
              />
              Direct
            </label>
            <label>
              <input
                type="radio"
                name="chat-interaction-profile"
                checked={interactionProfile === "conversational"}
                onChange={() => onInteractionProfileChange("conversational")}
              />
              Conversational
            </label>
          </fieldset>
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
              disabled={busy || !prompt.trim() || !managedCodexReady}
              onClick={() => void submit()}
            >
              Send
            </button>
          )}
        </div>
      </label>
    </ConversationShell>
  );
}
