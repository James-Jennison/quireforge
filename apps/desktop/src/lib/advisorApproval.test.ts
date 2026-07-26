import { describe, expect, it } from "vitest";

import {
  advisorApprovalDecisionRequestSchema,
  advisorApprovalSnapshotSchema,
  advisorDraftCreateRequestSchema,
} from "./advisorApproval";

const conversationId = "019d4e3c-3b14-7a2b-8c91-3f27d4f7aa10";
const projectId = "019d4e3c-3b17-7e50-9f35-3f27d4f7aa13";

describe("Phase A Advisor approval contract", () => {
  it("accepts a bounded draft but rejects unknown fields and empty capability declarations", () => {
    const draft = {
      advisorConversationId: conversationId,
      targetProjectId: projectId,
      prompt: "Prepare a focused implementation plan.",
      selectedProjectState: {
        schemaVersion: 1,
        sourceKind: "project-state",
        selectedAtMs: 1771234567000,
        trust: "verified",
        freshness: "current",
        provenanceSource: "project-state-snapshot",
        worktree: "clean",
        diagnosticCount: 0,
      },
      declaredCapabilities: ["workspace-write"],
      requestedModel: "default",
      requestedReasoningEffort: "default",
    };
    expect(advisorDraftCreateRequestSchema.parse(draft)).toEqual(draft);
    expect(() =>
      advisorDraftCreateRequestSchema.parse({ ...draft, dispatch: true }),
    ).toThrow();
    expect(() =>
      advisorDraftCreateRequestSchema.parse({
        ...draft,
        declaredCapabilities: [],
      }),
    ).toThrow();
    expect(() =>
      advisorDraftCreateRequestSchema.parse({
        ...draft,
        selectedProjectState: {
          ...draft.selectedProjectState,
          path: "/private",
        },
      }),
    ).toThrow();
  });

  it("allows only explicit approval or rejection and never a dispatch result", () => {
    expect(
      advisorApprovalDecisionRequestSchema.parse({
        proposalId: conversationId,
        decision: "approved",
      }),
    ).toMatchObject({ decision: "approved" });
    expect(() =>
      advisorApprovalDecisionRequestSchema.parse({
        proposalId: conversationId,
        decision: "draft",
      }),
    ).toThrow();
    expect(
      advisorApprovalSnapshotSchema.parse({
        proposalId: conversationId,
        state: "approved",
        expiresAtMs: 1771235467000,
        dispatchAvailable: false,
      }),
    ).toMatchObject({ dispatchAvailable: false });
  });
});
