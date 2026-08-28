import type { ConversationEvent } from "./conversation";

const MAX_CONVERSATION_EVENTS = 256;
const MAX_ACTIVITY_VIEWS = 64;
const MAX_ACTIVITY_OUTPUT_CHARACTERS = 32 * 1024;
const MAX_MESSAGE_CHARACTERS = 32 * 1024;
const OMITTED_OUTPUT_MARKER = "… earlier output omitted …\n";

type ActivityEvent = Extract<ConversationEvent, { type: "activity" }>;

export interface ConversationActivityView {
  activityId: string;
  kind: ActivityEvent["kind"];
  title: string;
  detail: string | null;
  status: ActivityEvent["status"];
  exitCode: number | null;
  output: string;
  firstSequence: number;
  lastSequence: number;
}

export function mergeConversationEvents(
  current: ConversationEvent[],
  incoming: ConversationEvent[],
): ConversationEvent[] {
  const bySequence = new Map(
    [...current, ...incoming].map((event) => [event.sequence, event]),
  );
  // Retain complete assistant messages rather than a fixed number of their
  // transport fragments. A long token-by-token response could otherwise lose
  // its opening text before the view had a chance to join the deltas.
  return coalesceConversationMessageDeltas(
    [...bySequence.values()].sort(
      (left, right) => left.sequence - right.sequence,
    ),
  ).slice(-MAX_CONVERSATION_EVENTS);
}

/**
 * The app-server sends assistant text as small streaming deltas. Keep those
 * deltas intact in state so polling can still deduplicate by sequence, but
 * present contiguous assistant text as one bounded message to the user.
 */
export function coalesceConversationMessageDeltas(
  events: ConversationEvent[],
): ConversationEvent[] {
  const coalesced: ConversationEvent[] = [];

  for (const event of events) {
    const previous = coalesced.at(-1);
    if (
      event.type === "agent-message-delta" &&
      previous?.type === "agent-message-delta"
    ) {
      previous.delta = appendBoundedOutput(previous.delta, event.delta);
      previous.sequence = event.sequence;
      continue;
    }
    coalesced.push({ ...event });
  }

  return coalesced;
}

function appendBoundedOutput(current: string, delta: string): string {
  const combined = `${current}${delta}`;
  if (combined.length <= MAX_MESSAGE_CHARACTERS) return combined;
  const retainedLength = MAX_MESSAGE_CHARACTERS - OMITTED_OUTPUT_MARKER.length;
  return `${OMITTED_OUTPUT_MARKER}${combined.slice(-retainedLength)}`;
}

export function buildConversationActivityViews(
  events: ConversationEvent[],
): ConversationActivityView[] {
  const activities = new Map<string, ConversationActivityView>();

  for (const event of [...events].sort(
    (left, right) => left.sequence - right.sequence,
  )) {
    if (event.type === "activity") {
      const current = activities.get(event.activityId);
      activities.set(event.activityId, {
        activityId: event.activityId,
        kind: event.kind,
        title: event.title || current?.title || "Activity progress",
        detail: event.detail ?? current?.detail ?? null,
        status: event.status,
        exitCode: event.exitCode,
        output: current?.output ?? "",
        firstSequence: current?.firstSequence ?? event.sequence,
        lastSequence: event.sequence,
      });
      continue;
    }

    if (event.type === "activity-output-delta") {
      const current = activities.get(event.activityId);
      activities.set(event.activityId, {
        activityId: event.activityId,
        kind: current?.kind ?? "other",
        title: current?.title ?? "Activity progress",
        detail: current?.detail ?? null,
        status: current?.status ?? "started",
        exitCode: current?.exitCode ?? null,
        output: appendBoundedActivityOutput(current?.output ?? "", event.delta),
        firstSequence: current?.firstSequence ?? event.sequence,
        lastSequence: event.sequence,
      });
    }
  }

  return [...activities.values()]
    .sort((left, right) => left.firstSequence - right.firstSequence)
    .slice(-MAX_ACTIVITY_VIEWS);
}

function appendBoundedActivityOutput(current: string, delta: string): string {
  const combined = `${current}${delta}`;
  if (combined.length <= MAX_ACTIVITY_OUTPUT_CHARACTERS) return combined;
  const retainedLength =
    MAX_ACTIVITY_OUTPUT_CHARACTERS - OMITTED_OUTPUT_MARKER.length;
  return `${OMITTED_OUTPUT_MARKER}${combined.slice(-retainedLength)}`;
}
