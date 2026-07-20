import { describe, expect, it } from "vitest";

import type { ConversationEvent } from "./conversation";
import { mergeConversationEvents } from "./conversationView";

describe("conversation event view", () => {
  it("deduplicates and orders batches by bounded sequence", () => {
    const current: ConversationEvent[] = [
      { type: "lifecycle", sequence: 1, phase: "starting" },
      { type: "lifecycle", sequence: 2, phase: "running" },
    ];
    const incoming: ConversationEvent[] = [
      { type: "lifecycle", sequence: 2, phase: "running" },
      { type: "agent-message-delta", sequence: 3, delta: "Ready." },
    ];

    expect(
      mergeConversationEvents(current, incoming).map(
        ({ sequence }) => sequence,
      ),
    ).toEqual([1, 2, 3]);
  });

  it("retains only the most recent 256 normalized events", () => {
    const incoming: ConversationEvent[] = Array.from(
      { length: 300 },
      (_, index) => ({
        type: "lifecycle" as const,
        sequence: index + 1,
        phase: "running" as const,
      }),
    );

    const result = mergeConversationEvents([], incoming);
    expect(result).toHaveLength(256);
    expect(result[0]?.sequence).toBe(45);
    expect(result.at(-1)?.sequence).toBe(300);
  });
});
