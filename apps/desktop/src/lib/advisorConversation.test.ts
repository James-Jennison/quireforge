import { describe, expect, it } from "vitest";

import {
  advisorConversationSnapshotSchema,
  advisorConversationStartRequestSchema,
  mergeAdvisorConversationSnapshot,
  scaffoldAdvisorConversation,
} from "./advisorConversation";

const conversationId = "018f0000-0000-7000-8000-000000000001";

function snapshot(overrides: Partial<typeof scaffoldAdvisorConversation> = {}) {
  return {
    ...scaffoldAdvisorConversation,
    state: "running" as const,
    conversationId,
    ...overrides,
  };
}

describe("managed Advisor conversation contract", () => {
  it("accepts only a bounded Advisor snapshot without a Codex thread ID", () => {
    expect(
      advisorConversationSnapshotSchema.parse(scaffoldAdvisorConversation),
    ).toEqual(scaffoldAdvisorConversation);
    expect(() =>
      advisorConversationSnapshotSchema.parse({
        ...scaffoldAdvisorConversation,
        threadId: "private-thread",
      }),
    ).toThrow();
  });

  it("requires a bounded prompt and an optional app-owned project UUID", () => {
    expect(
      advisorConversationStartRequestSchema.parse({
        prompt: "Review the selected summary.",
        projectId: "018f0000-0000-7000-8000-000000000001",
        attachmentId: null,
        attachmentManifestSha256: null,
        attachmentConfirmation: null,
      }),
    ).toMatchObject({ projectId: "018f0000-0000-7000-8000-000000000001" });
    expect(() =>
      advisorConversationStartRequestSchema.parse({
        prompt: "Review this",
        projectId: "/tmp/project",
        attachmentId: null,
        attachmentManifestSha256: null,
        attachmentConfirmation: null,
      }),
    ).toThrow();
    expect(() =>
      advisorConversationStartRequestSchema.parse({
        prompt: "\0",
        projectId: null,
        attachmentId: null,
        attachmentManifestSha256: null,
        attachmentConfirmation: null,
      }),
    ).toThrow();
  });

  it("accumulates sequence-ordered stream fragments without persisting them", () => {
    const first = snapshot({
      events: [{ type: "agent-message-delta", sequence: 1, delta: "Hello" }],
    });
    const second = snapshot({
      events: [
        { type: "agent-message-delta", sequence: 2, delta: " world" },
        { type: "agent-message-delta", sequence: 3, delta: "." },
      ],
    });

    expect(mergeAdvisorConversationSnapshot(first, second)).toMatchObject({
      state: "running",
      conversationId,
      events: [
        {
          type: "agent-message-delta",
          sequence: 3,
          delta: "Hello world.",
        },
      ],
    });
  });

  it("keeps accumulated text through an empty terminal poll and ignores stale fragments", () => {
    const current = snapshot({
      events: [
        { type: "agent-message-delta", sequence: 3, delta: "Complete reply." },
      ],
    });
    const terminal = snapshot({ state: "completed", events: [] });
    const completed = mergeAdvisorConversationSnapshot(current, terminal);
    expect(completed.events).toEqual(current.events);

    const stale = snapshot({
      events: [
        { type: "agent-message-delta", sequence: 2, delta: " duplicate" },
      ],
    });
    expect(mergeAdvisorConversationSnapshot(completed, stale).events).toEqual(
      current.events,
    );
  });

  it("resets transient text when a different conversation begins", () => {
    const current = snapshot({
      events: [
        { type: "agent-message-delta", sequence: 1, delta: "Old reply." },
      ],
    });
    const next = {
      ...snapshot({
        events: [
          { type: "agent-message-delta", sequence: 1, delta: "New reply." },
        ],
      }),
      conversationId: "018f0000-0000-7000-8000-000000000002",
    };

    expect(mergeAdvisorConversationSnapshot(current, next)).toEqual(next);
  });
});
