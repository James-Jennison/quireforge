import type { ConversationEvent } from "./conversation";

const MAX_CONVERSATION_EVENTS = 256;

export function mergeConversationEvents(
  current: ConversationEvent[],
  incoming: ConversationEvent[],
): ConversationEvent[] {
  const bySequence = new Map(
    [...current, ...incoming].map((event) => [event.sequence, event]),
  );
  return [...bySequence.values()]
    .sort((left, right) => left.sequence - right.sequence)
    .slice(-MAX_CONVERSATION_EVENTS);
}
