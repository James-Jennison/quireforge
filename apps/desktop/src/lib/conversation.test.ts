import fixture from "../../fixtures/conversation.json";
import { describe, expect, it } from "vitest";

import {
  conversationRegistrySchema,
  parseConversationSnapshotResponse,
  conversationSnapshotSchema,
  conversationStartRequestSchema,
  conversationApprovalDecisionRequestSchema,
  scaffoldConversationRegistry,
  scaffoldConversation,
} from "./conversation";
import { mergeConversationEvents } from "./conversationView";

const conversationId = "018f0000-0000-7000-8000-000000000010";
const projectId = "018f0000-0000-7000-8000-000000000001";
const selectionPolicy = {
  ownership: "manual" as const,
  userLocked: false,
  allowedModelIds: [],
  reasoningCeiling: null,
};
const modelSelection = {
  schemaVersion: 1 as const,
  availability: "ready" as const,
  effective: { modelId: "gpt-5.6-sol", reasoningEffort: "high" },
  pending: null,
  policy: selectionPolicy,
  diagnosticCode: null,
};

describe("conversation contract", () => {
  it("parses the shared empty fixture", () => {
    expect(conversationSnapshotSchema.parse(fixture)).toEqual(
      scaffoldConversation,
    );
  });

  it("parses the shared bounded registry fixture", () => {
    expect(scaffoldConversationRegistry).toEqual({
      schemaVersion: 1,
      capacity: 4,
      conversations: [],
    });
  });

  it("accepts only bounded explicit start controls", () => {
    expect(
      conversationStartRequestSchema.parse({
        projectId,
        prompt: "Inspect this project without changing files.",
        attachmentIds: [],
        modelId: "gpt-5.6-sol",
        reasoningEffort: "high",
        selectionPolicy,
        sandboxMode: "read-only",
        approvalPolicy: "untrusted",
        integrationEntryIds: [],
      }),
    ).toMatchObject({ projectId, sandboxMode: "read-only" });
  });

  it("rejects paths, protocol IDs, unknown fields, and no-approval project modes", () => {
    const valid = {
      projectId,
      prompt: "Review the project.",
      attachmentIds: [],
      modelId: "gpt-5.6-sol",
      reasoningEffort: "high",
      selectionPolicy,
      sandboxMode: "workspace-write" as const,
      approvalPolicy: "on-request" as const,
      integrationEntryIds: [],
    };

    expect(() =>
      conversationStartRequestSchema.parse({
        ...valid,
        cwd: "/private/raw/path",
      }),
    ).toThrow();
    expect(() =>
      conversationStartRequestSchema.parse({ ...valid, threadId: projectId }),
    ).toThrow();
    expect(() =>
      conversationStartRequestSchema.parse({
        ...valid,
        approvalPolicy: "never",
      }),
    ).toThrow();
  });

  it("accepts normalized ordered events without Codex protocol identity", () => {
    const snapshot = conversationSnapshotSchema.parse({
      schemaVersion: 3,
      state: "running",
      conversationId,
      projectId,
      modelId: "gpt-5.6-sol",
      reasoningEffort: "high",
      modelSelection,
      sandboxMode: "read-only",
      approvalPolicy: "untrusted",
      pendingApproval: null,
      events: [
        { type: "lifecycle", sequence: 1, phase: "starting" },
        {
          type: "agent-message-delta",
          sequence: 2,
          delta: "Reviewing the project.",
        },
      ],
      diagnosticCode: null,
    });

    expect(snapshot.events).toHaveLength(2);
    expect(() =>
      conversationSnapshotSchema.parse({
        ...snapshot,
        threadId: "018f0000-0000-7000-8000-000000000020",
      }),
    ).toThrow();
    expect(() =>
      conversationSnapshotSchema.parse({
        ...snapshot,
        events: snapshot.events.toReversed(),
      }),
    ).toThrow();
  });

  it("preserves a consumed completed reply when passive evidence is unpresentable", () => {
    const payload = {
      schemaVersion: 3,
      state: "completed",
      conversationId,
      projectId,
      modelId: "gpt-5.6-sol",
      reasoningEffort: "high",
      modelSelection,
      sandboxMode: "read-only",
      approvalPolicy: "untrusted",
      pendingApproval: null,
      events: [
        { type: "lifecycle", sequence: 1, phase: "starting" },
        {
          type: "activity",
          sequence: 2,
          activityId: "unpresentable-passive-id",
          kind: "command-execution",
          status: "completed",
          title: "Run command",
          detail: null,
          exitCode: 0,
        },
        {
          type: "agent-message-completed",
          sequence: 3,
          itemId: "item_status_reply",
          text: "Here is the complete project status.",
        },
        { type: "lifecycle", sequence: 4, phase: "completed" },
      ],
      diagnosticCode: null,
    };

    expect(conversationSnapshotSchema.safeParse(payload).success).toBe(false);
    expect(parseConversationSnapshotResponse(payload)?.events).toEqual([
      payload.events[0],
      payload.events[2],
      payload.events[3],
    ]);
  });

  it("never recovers an invalid actionable event", () => {
    const payload = {
      schemaVersion: 3,
      state: "completed",
      conversationId,
      projectId,
      modelId: "gpt-5.6-sol",
      reasoningEffort: "high",
      modelSelection,
      sandboxMode: "read-only",
      approvalPolicy: "untrusted",
      pendingApproval: null,
      events: [
        {
          type: "approval-requested",
          sequence: 1,
          approvalId: "invalid",
          activityId: "invalid",
          kind: "command-execution",
        },
      ],
      diagnosticCode: null,
    };

    expect(parseConversationSnapshotResponse(payload)).toBeNull();
  });

  it("preserves assistant delta order around an unpresentable passive event", () => {
    const payload = {
      schemaVersion: 3,
      state: "running",
      conversationId,
      projectId,
      modelId: "gpt-5.6-sol",
      reasoningEffort: "high",
      modelSelection,
      sandboxMode: "read-only",
      approvalPolicy: "untrusted",
      pendingApproval: null,
      events: [
        {
          type: "agent-message-delta",
          sequence: 1,
          itemId: "item_interleaved_reply",
          delta: "Opening ",
        },
        {
          type: "activity",
          sequence: 2,
          activityId: "unpresentable-passive-id",
          kind: "command-execution",
          status: "completed",
          title: "Run command",
          detail: null,
          exitCode: 0,
        },
        {
          type: "agent-message-delta",
          sequence: 3,
          itemId: "item_interleaved_reply",
          delta: "continues.",
        },
      ],
      diagnosticCode: null,
    };

    const recovered = parseConversationSnapshotResponse(payload);
    expect(recovered).not.toBeNull();
    expect(mergeConversationEvents([], recovered?.events ?? [])).toEqual([
      {
        type: "agent-message-delta",
        sequence: 3,
        itemId: "item_interleaved_reply",
        delta: "Opening continues.",
      },
    ]);
  });

  it("bounds the active registry and requires unique app-owned projects", () => {
    const active = conversationSnapshotSchema.parse({
      schemaVersion: 3,
      state: "running",
      conversationId,
      projectId,
      modelId: "gpt-5.6-sol",
      reasoningEffort: "high",
      modelSelection,
      sandboxMode: "read-only",
      approvalPolicy: "untrusted",
      pendingApproval: null,
      events: [],
      diagnosticCode: null,
    });
    expect(
      conversationRegistrySchema.parse({
        schemaVersion: 1,
        capacity: 4,
        conversations: [active],
      }).conversations,
    ).toHaveLength(1);
    expect(() =>
      conversationRegistrySchema.parse({
        schemaVersion: 1,
        capacity: 4,
        conversations: [active, active],
      }),
    ).toThrow();
    expect(() =>
      conversationRegistrySchema.parse({
        schemaVersion: 1,
        capacity: 8,
        conversations: [],
      }),
    ).toThrow();
    expect(() =>
      conversationRegistrySchema.parse({
        schemaVersion: 1,
        capacity: 4,
        conversations: [
          {
            ...active,
            events: [{ type: "lifecycle", sequence: 1, phase: "running" }],
          },
        ],
      }),
    ).toThrow();
  });

  it("accepts only app-owned approval decisions and bounded activity detail", () => {
    const approvalId = "018f0000-0000-7000-8000-000000000011";
    const activityId = "018f0000-0000-7000-8000-000000000012";
    const request = conversationApprovalDecisionRequestSchema.parse({
      conversationId,
      approvalId,
      decision: "decline",
    });
    expect(request).toEqual({
      conversationId,
      approvalId,
      decision: "decline",
    });

    const waiting = conversationSnapshotSchema.parse({
      schemaVersion: 3,
      state: "waiting-for-approval",
      conversationId,
      projectId,
      modelId: "gpt-5.6-sol",
      reasoningEffort: "high",
      modelSelection,
      sandboxMode: "workspace-write",
      approvalPolicy: "on-request",
      pendingApproval: {
        approvalId,
        activityId,
        kind: "command-execution",
        title: "Run this command?",
        reason: "The check needs permission.",
        details: [{ label: "Command", value: "pnpm check" }],
        decisions: ["approve", "decline", "cancel"],
      },
      events: [
        {
          type: "approval-requested",
          sequence: 1,
          approvalId,
          activityId,
          kind: "command-execution",
        },
      ],
      diagnosticCode: null,
    });
    expect(waiting.pendingApproval?.approvalId).toBe(approvalId);

    expect(() =>
      conversationSnapshotSchema.parse({
        ...waiting,
        pendingApproval: {
          ...waiting.pendingApproval,
          rawArguments: { token: "private" },
        },
      }),
    ).toThrow();
  });
});
