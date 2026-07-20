import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ConversationWorkspace } from "./ConversationWorkspace";
import { scaffoldCodexRuntime } from "./lib/codex";
import {
  conversationSnapshotSchema,
  scaffoldConversation,
} from "./lib/conversation";
import { projectWorkspaceSchema } from "./lib/project";

const projectId = "018f0000-0000-7000-8000-000000000001";
const conversationId = "018f0000-0000-7000-8000-000000000010";
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
  schemaVersion: 1,
  state: "running",
  conversationId,
  projectId,
  modelId: "gpt-5.6-sol",
  reasoningEffort: "high",
  sandboxMode: "workspace-write",
  approvalPolicy: "on-request",
  events: [{ type: "lifecycle", sequence: 1, phase: "running" }],
  diagnosticCode: null,
});

function renderWorkspace(
  overrides: Partial<React.ComponentProps<typeof ConversationWorkspace>> = {},
) {
  const onStart = vi.fn().mockResolvedValue(runningConversation);
  const onInterrupt = vi.fn().mockResolvedValue({
    ...runningConversation,
    state: "interrupted",
    events: [{ type: "lifecycle", sequence: 2, phase: "interrupted" }],
  });
  render(
    <ConversationWorkspace
      availability="native"
      snapshot={scaffoldConversation}
      events={[]}
      runtime={scaffoldCodexRuntime}
      project={project}
      busy={false}
      actionError={false}
      onStart={onStart}
      onInterrupt={onInterrupt}
      {...overrides}
    />,
  );
  return { onStart, onInterrupt };
}

describe("ConversationWorkspace", () => {
  it("submits bounded runtime-derived controls for a verified project", async () => {
    const { onStart } = renderWorkspace();
    const start = screen.getByRole("button", { name: "Start task" });
    expect(start).toBeDisabled();

    fireEvent.change(screen.getByLabelText("Task"), {
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
        modelId: "gpt-5.6-sol",
        reasoningEffort: "high",
        sandboxMode: "workspace-write",
        approvalPolicy: "on-request",
      }),
    );
    expect(screen.getByLabelText("Task")).toHaveValue("");
  });

  it("blocks an unrestricted no-approval combination before IPC", () => {
    const { onStart } = renderWorkspace();
    fireEvent.change(screen.getByLabelText("Task"), {
      target: { value: "Make a change." },
    });
    fireEvent.change(screen.getByLabelText("Filesystem access"), {
      target: { value: "danger-full-access" },
    });
    fireEvent.change(screen.getByLabelText("Approval policy"), {
      target: { value: "never" },
    });

    expect(
      screen.getByText(
        "Unrestricted execution cannot be combined with disabled approvals.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Start task" })).toBeDisabled();
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
          kind: "command-execution",
          status: "completed",
        },
      ],
    });

    expect(screen.getByText("The UI is ready for review.")).toBeInTheDocument();
    expect(screen.getByText("Command completed")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Stop task" }));
    expect(onInterrupt).toHaveBeenCalledWith(conversationId);
  });

  it("keeps browser preview honest and non-interactive", () => {
    renderWorkspace({ availability: "preview", project: undefined });
    expect(
      screen.getByText(
        "Browser preview cannot start or simulate a Codex task.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Start task" })).toBeDisabled();
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
    fireEvent.change(screen.getByLabelText("Task"), {
      target: { value: "Review the task." },
    });

    expect(
      screen.getByText(
        "A ready Codex conversation capability and model catalog are required.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Start task" })).toBeDisabled();
  });
});
