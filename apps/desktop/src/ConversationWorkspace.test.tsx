import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ConversationWorkspace } from "./ConversationWorkspace";
import { scaffoldConversationAttachments } from "./lib/attachment";
import { scaffoldCodexRuntime } from "./lib/codex";
import {
  conversationSnapshotSchema,
  scaffoldConversation,
} from "./lib/conversation";
import { scaffoldIntegrationCatalog } from "./lib/integration";
import { projectWorkspaceSchema } from "./lib/project";

const projectId = "018f0000-0000-7000-8000-000000000001";
const conversationId = "018f0000-0000-7000-8000-000000000010";
const modelSelection = {
  schemaVersion: 1 as const,
  availability: "ready" as const,
  effective: {
    modelId: "gpt-5.6-sol",
    reasoningEffort: "high",
  },
  pending: null,
  policy: {
    ownership: "manual" as const,
    userLocked: false,
    allowedModelIds: [],
    reasoningCeiling: null,
  },
  diagnosticCode: null,
};
const project = projectWorkspaceSchema.parse({
  schemaVersion: 1,
  state: "ready",
  projects: [
    {
      id: projectId,
      displayName: "QuireForge",
      archived: false,
      directory: {
        associationId: "018f0000-0000-7000-8000-000000000002",
        displayPath: "~/work/quireforge",
        resolvedDisplayPath: "/mnt/work/quireforge",
        state: "connected-accessible",
        expectedAccess: "read-write",
        isPrimary: true,
        git: { isRepository: true, isLinkedWorktree: false },
        hasAgentsGuidance: true,
        hasCodexConfig: false,
      },
    },
  ],
  pendingAttachment: null,
  diagnosticCode: null,
}).projects[0];

const runningConversation = conversationSnapshotSchema.parse({
  schemaVersion: 3,
  state: "running",
  conversationId,
  projectId,
  modelId: "gpt-5.6-sol",
  reasoningEffort: "high",
  modelSelection,
  sandboxMode: "workspace-write",
  approvalPolicy: "on-request",
  pendingApproval: null,
  events: [{ type: "lifecycle", sequence: 1, phase: "running" }],
  diagnosticCode: null,
});

function renderWorkspace(
  overrides: Partial<React.ComponentProps<typeof ConversationWorkspace>> = {},
) {
  const onStart = vi.fn().mockResolvedValue(runningConversation);
  const onRetryPoll = vi.fn().mockResolvedValue(runningConversation);
  const onInterrupt = vi.fn().mockResolvedValue({
    ...runningConversation,
    state: "interrupted",
    events: [{ type: "lifecycle", sequence: 2, phase: "interrupted" }],
  });
  const onDecideApproval = vi.fn().mockResolvedValue(runningConversation);
  const onUpdateModelSelection = vi.fn().mockResolvedValue(modelSelection);
  const props: React.ComponentProps<typeof ConversationWorkspace> = {
    availability: "native",
    snapshot: scaffoldConversation,
    events: [],
    runtime: scaffoldCodexRuntime,
    integrations: scaffoldIntegrationCatalog,
    project,
    attachments: scaffoldConversationAttachments,
    busy: false,
    attachmentBusy: false,
    actionError: null,
    attachmentActionError: false,
    onStart,
    onRetryPoll,
    onInterrupt,
    onDecideApproval,
    onUpdateModelSelection,
    onAttachmentPick: vi.fn().mockResolvedValue(undefined),
    onAttachmentDrop: vi.fn().mockResolvedValue(undefined),
    onAttachmentCancel: vi.fn().mockResolvedValue(undefined),
    ...overrides,
  };
  render(<ConversationWorkspace {...props} />);
  return { onStart, onRetryPoll, onInterrupt, onDecideApproval };
}

describe("ConversationWorkspace", () => {
  it("submits bounded runtime-derived controls for a verified project", async () => {
    const { onStart } = renderWorkspace();
    const start = screen.getByRole("button", { name: "Send" });
    expect(start).toBeDisabled();

    fireEvent.change(screen.getByLabelText("Message"), {
      target: { value: "Review the conversation UI." },
    });
    fireEvent.change(screen.getByLabelText("Reasoning"), {
      target: { value: "high" },
    });
    fireEvent.click(start);

    await waitFor(() =>
      expect(onStart).toHaveBeenCalledWith({
        projectId,
        prompt: "Review the conversation UI.",
        attachmentIds: [],
        modelId: "gpt-5.6-sol",
        reasoningEffort: "high",
        selectionPolicy: {
          ownership: "manual",
          userLocked: false,
          allowedModelIds: [],
          reasoningCeiling: null,
        },
        sandboxMode: "workspace-write",
        approvalPolicy: "on-request",
        interactionProfile: "direct",
        integrationEntryIds: [],
      }),
    );
    expect(screen.getByLabelText("Message")).toHaveValue("");
  });

  it("submits a multiline task once as one unchanged prompt", async () => {
    let resolveStart:
      ((snapshot: typeof runningConversation) => void) | undefined;
    const onStart = vi.fn(
      () =>
        new Promise<typeof runningConversation>((resolve) => {
          resolveStart = resolve;
        }),
    );
    renderWorkspace({ onStart });
    const task =
      "Inspect the native action.\n\n- Preserve this list.\n- Run one task.";
    const textarea = screen.getByLabelText("Message");
    const form = textarea.closest("form");
    expect(form).not.toBeNull();

    fireEvent.change(textarea, { target: { value: task } });
    fireEvent.submit(form!);
    fireEvent.submit(form!);

    expect(onStart).toHaveBeenCalledTimes(1);
    expect(onStart).toHaveBeenCalledWith(
      expect.objectContaining({ prompt: task }),
    );
    expect(textarea).toHaveValue(task);

    resolveStart?.(runningConversation);
    await waitFor(() => expect(textarea).toHaveValue(""));
  });

  it("pins the chosen conversation style into the start request", async () => {
    const { onStart } = renderWorkspace();
    fireEvent.click(screen.getByRole("radio", { name: "Conversational" }));
    fireEvent.change(screen.getByLabelText("Message"), {
      target: { value: "Explain the design." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Send" }));

    await waitFor(() =>
      expect(onStart).toHaveBeenCalledWith(
        expect.objectContaining({ interactionProfile: "conversational" }),
      ),
    );
  });

  it("keeps controls collapsed and places the submitted message in the chat transcript", async () => {
    renderWorkspace();
    const settings = screen.getByText("Controls").closest("details");
    expect(settings).not.toBeNull();
    expect(settings).not.toHaveAttribute("open");

    fireEvent.change(screen.getByLabelText("Message"), {
      target: { value: "Make this workspace feel like chat." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Send" }));

    await screen.findByText("Make this workspace feel like chat.");
    const message = document.querySelector(".conversation-event--user-message");
    expect(message).toHaveTextContent("Make this workspace feel like chat.");
  });

  it("keeps lifecycle frames out of the transcript while the live header reports status", () => {
    renderWorkspace({ snapshot: runningConversation });

    expect(screen.getByRole("status")).toHaveTextContent("Codex is working");
    expect(document.querySelector(".conversation-event__lifecycle")).toBeNull();
  });

  it("limits conversation style to assistant prose", () => {
    renderWorkspace();
    fireEvent.click(screen.getByText("Controls"));

    expect(
      screen.getByText(
        /This changes assistant prose only\. Action Cards, authority and disclosure copy, lock labels, and failure messages stay exactly the same\./u,
      ),
    ).toBeInTheDocument();
  });

  it("closes controls from its explicit action, outside click, or Escape", () => {
    renderWorkspace();
    const controls = screen.getByText("Controls").closest("details");
    expect(controls).not.toHaveAttribute("open");

    fireEvent.click(screen.getByText("Controls"));
    expect(controls).toHaveAttribute("open");
    fireEvent.click(screen.getByRole("button", { name: "Close controls" }));
    expect(controls).not.toHaveAttribute("open");

    fireEvent.click(screen.getByText("Controls"));
    fireEvent.pointerDown(document.body);
    expect(controls).not.toHaveAttribute("open");

    fireEvent.click(screen.getByText("Controls"));
    fireEvent.keyDown(document, { key: "Escape" });
    expect(controls).not.toHaveAttribute("open");
  });

  it("preserves the displayed response and offers a bounded retry for an invalid native snapshot", () => {
    const { onRetryPoll } = renderWorkspace({
      actionError: "native-response-invalid",
      snapshot: runningConversation,
    });

    const alert = screen.getByRole("alert");
    expect(alert).toHaveTextContent(/could not finish reading this response/iu);
    expect(alert).toHaveTextContent(
      /response already shown is still available/iu,
    );
    fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    expect(onRetryPoll).toHaveBeenCalledWith(conversationId);
  });

  it("shows only the content-free validation classification", () => {
    renderWorkspace({
      actionError: "native-response-invalid",
      actionErrorDetail: "event.2.consequential:invalid_type:events.2.phase",
    });

    expect(screen.getByRole("alert")).toHaveTextContent(
      "Validation detail: event.2.consequential:invalid_type:events.2.phase",
    );
  });

  it("submits a selected healthy connector by normalized catalog ID", async () => {
    const { onStart } = renderWorkspace();
    fireEvent.click(
      screen.getByRole("checkbox", { name: "Fixture calendar connector" }),
    );
    fireEvent.change(screen.getByLabelText("Message"), {
      target: { value: "Check my calendar." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Send" }));

    await waitFor(() =>
      expect(onStart).toHaveBeenCalledWith(
        expect.objectContaining({
          prompt: "Check my calendar.",
          integrationEntryIds: ["connector:fixture-calendar"],
        }),
      ),
    );
  });

  it("does not offer a no-approval project conversation mode", () => {
    const { onStart } = renderWorkspace();
    const approvalPolicy = screen.getByLabelText("Approval policy");

    expect(within(approvalPolicy).getAllByRole("option")).toHaveLength(2);
    expect(
      screen.queryByRole("option", { name: /never ask/u }),
    ).not.toBeInTheDocument();
    expect(onStart).not.toHaveBeenCalled();
  });

  it("renders normalized events and interrupts only the app conversation ID", () => {
    const { onInterrupt } = renderWorkspace({
      snapshot: runningConversation,
      events: [
        ...runningConversation.events,
        {
          type: "agent-message-delta",
          sequence: 2,
          delta: "The UI is ready for review.",
        },
        {
          type: "activity",
          sequence: 3,
          activityId: "018f0000-0000-7000-8000-000000000011",
          kind: "command-execution",
          status: "completed",
          title: "Run command",
          detail: "pnpm check",
          exitCode: 0,
        },
        {
          type: "activity-output-delta",
          sequence: 4,
          activityId: "018f0000-0000-7000-8000-000000000011",
          delta: "Checks passed.",
        },
      ],
    });

    expect(screen.getByText("The UI is ready for review.")).toBeInTheDocument();
    const activity = screen.getByRole("button", { name: /Run command/u });
    expect(activity).toHaveAttribute("aria-expanded", "false");
    expect(screen.queryByText("pnpm check")).not.toBeInTheDocument();
    fireEvent.click(activity);
    expect(activity).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByText("pnpm check")).toBeInTheDocument();
    expect(screen.getByText("Checks passed.")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Stop task" }));
    expect(onInterrupt).toHaveBeenCalledWith(conversationId);
  });

  it("renders streamed assistant text as one message instead of one card per delta", () => {
    renderWorkspace({
      snapshot: runningConversation,
      events: [
        ...runningConversation.events,
        { type: "agent-message-delta", sequence: 2, delta: "Three" },
        { type: "agent-message-delta", sequence: 3, delta: "-part" },
        { type: "agent-message-delta", sequence: 4, delta: " completion" },
      ],
    });

    expect(screen.getByText("Three-part completion")).toBeInTheDocument();
    expect(screen.queryByText("Three")).not.toBeInTheDocument();
    expect(screen.queryByText("-part")).not.toBeInTheDocument();
  });

  it("submits only the exact pending approval decision", async () => {
    const approvalId = "018f0000-0000-7000-8000-000000000011";
    const activityId = "018f0000-0000-7000-8000-000000000012";
    const waiting = conversationSnapshotSchema.parse({
      ...runningConversation,
      state: "waiting-for-approval",
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
          sequence: 2,
          approvalId,
          activityId,
          kind: "command-execution",
        },
      ],
    });
    const { onInterrupt, onDecideApproval } = renderWorkspace({
      snapshot: waiting,
      events: waiting.events,
    });

    expect(
      screen.getByText("Codex is waiting for approval"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Approval requested for command execution."),
    ).toBeInTheDocument();
    expect(screen.getByText("The check needs permission.")).toBeInTheDocument();
    expect(screen.getByText("pnpm check")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Approve once" }));
    await waitFor(() =>
      expect(onDecideApproval).toHaveBeenCalledWith({
        conversationId,
        approvalId,
        decision: "approve",
      }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Stop task" }));
    expect(onInterrupt).toHaveBeenCalledWith(conversationId);
  });

  it("renders only advertised decisions and prevents duplicate submission", async () => {
    const approvalId = "018f0000-0000-7000-8000-000000000021";
    const activityId = "018f0000-0000-7000-8000-000000000022";
    const waiting = conversationSnapshotSchema.parse({
      ...runningConversation,
      state: "waiting-for-approval",
      pendingApproval: {
        approvalId,
        activityId,
        kind: "command-execution",
        title: "Allow this command?",
        reason: null,
        details: [],
        decisions: ["decline"],
      },
      events: [],
    });
    let resolveDecision:
      ((value: typeof runningConversation) => void) | undefined;
    const onDecideApproval = vi.fn(
      () =>
        new Promise<typeof runningConversation>((resolve) => {
          resolveDecision = resolve;
        }),
    );
    renderWorkspace({
      snapshot: waiting,
      onDecideApproval,
    });

    expect(screen.queryByRole("button", { name: "Approve once" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Cancel task" })).toBeNull();
    const decline = screen.getByRole("button", { name: "Decline" });
    fireEvent.click(decline);
    fireEvent.click(decline);
    expect(onDecideApproval).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("button", { name: "Decline…" })).toBeDisabled();
    resolveDecision?.(runningConversation);
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Decline" })).toBeEnabled(),
    );
  });

  it("keeps browser preview honest and non-interactive", () => {
    renderWorkspace({ availability: "preview", project: undefined });
    expect(
      screen.getByText(
        "Browser preview cannot start or simulate a Codex task.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Send" })).toBeDisabled();
  });

  it("requires the advertised native conversation capability", () => {
    renderWorkspace({
      runtime: {
        ...scaffoldCodexRuntime,
        capabilities: scaffoldCodexRuntime.capabilities.filter(
          ({ id }) => id !== "conversation-runtime",
        ),
      },
    });
    fireEvent.change(screen.getByLabelText("Message"), {
      target: { value: "Review the task." },
    });

    expect(
      screen.getByText(
        "A ready Codex conversation capability and model catalog are required.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Send" })).toBeDisabled();
  });
});
