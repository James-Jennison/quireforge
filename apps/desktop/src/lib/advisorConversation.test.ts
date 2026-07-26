import { describe, expect, it } from "vitest";

import {
  advisorConversationSnapshotSchema,
  advisorConversationStartRequestSchema,
  scaffoldAdvisorConversation,
} from "./advisorConversation";

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
      }),
    ).toMatchObject({ projectId: "018f0000-0000-7000-8000-000000000001" });
    expect(() =>
      advisorConversationStartRequestSchema.parse({
        prompt: "Review this",
        projectId: "/tmp/project",
      }),
    ).toThrow();
    expect(() =>
      advisorConversationStartRequestSchema.parse({
        prompt: "\0",
        projectId: null,
      }),
    ).toThrow();
  });
});
