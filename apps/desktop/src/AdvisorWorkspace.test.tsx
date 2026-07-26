import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

const advisorDraftBridge = vi.hoisted(() => ({
  createAdvisorDraft: vi.fn(),
  decideAdvisorDraft: vi.fn(),
}));

vi.mock("./lib/bridge", () => advisorDraftBridge);

import { AdvisorWorkspace } from "./AdvisorWorkspace";
import { scaffoldAdvisorConversation } from "./lib/advisorConversation";
import { scaffoldCodexAuth } from "./lib/auth";
import {
  parseAdvisorSelectedProjectStateSnapshot,
  advisorWorkspaceSnapshotSchema,
} from "./lib/advisorWorkspace";

const advisorWorkspaceFixture = advisorWorkspaceSnapshotSchema.parse({
  schemaVersion: 1,
  conversationCount: 1,
  contextReferenceCount: 1,
  proposalCount: 1,
  contextSummaries: [
    { kind: "project-state", trust: "verified", freshness: "current" },
  ],
  proposalSummaries: [{ state: "draft", requiresExplicitApproval: true }],
});

const selectedProjectStateFixture = parseAdvisorSelectedProjectStateSnapshot({
  schemaVersion: 1,
  sourceKind: "project-state",
  selectedAtMs: 1,
  trust: "verified",
  freshness: "current",
  provenanceSource: "project-state-snapshot",
  worktree: "clean",
  diagnosticCount: 0,
});

const props = {
  availability: "native" as const,
  snapshot: advisorWorkspaceFixture,
  selectedProjectState: null,
  selectionState: "idle" as const,
  canSelectProjectState: true,
  onRequestProjectState: vi.fn(),
  onConfirmProjectState: vi.fn(),
  onCancelProjectState: vi.fn(),
  onRemoveProjectState: vi.fn(),
  auth: {
    ...scaffoldCodexAuth,
    state: "authenticated" as const,
    accountKind: "chatgpt" as const,
  },
  conversation: scaffoldAdvisorConversation,
  conversationBusy: false,
  selectedProjectId: null,
  onConversationStart: vi.fn().mockResolvedValue(scaffoldAdvisorConversation),
  onConversationPoll: vi.fn(),
  onConversationInterrupt: vi.fn(),
};

describe("AdvisorWorkspace", () => {
  it("renders a managed read-only composer without executable controls", () => {
    render(
      <AdvisorWorkspace
        {...props}
        snapshot={{
          ...advisorWorkspaceFixture,
          conversationCount: 0,
          contextReferenceCount: 0,
          proposalCount: 0,
          contextSummaries: [],
          proposalSummaries: [],
        }}
      />,
    );

    expect(
      screen.getByRole("heading", {
        name: "Read-only planning, without execution.",
      }),
    ).toBeInTheDocument();
    expect(screen.getByText("No Advisor metadata yet.")).toBeInTheDocument();
    expect(
      screen.getByRole("textbox", { name: "Advisor message" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Send to Advisor" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Advisor is read-only: no commands/u),
    ).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent(
      "Enter a message to send.",
    );
    expect(
      screen.queryByRole("button", { name: /approve|dispatch|terminal/u }),
    ).not.toBeInTheDocument();
  });

  it("shows only safe reference summaries for valid metadata", () => {
    render(<AdvisorWorkspace {...props} />);
    const summaries = within(
      screen.getByRole("list", { name: "Advisor metadata summaries" }),
    );
    const [contextSummary, proposalSummary] =
      summaries.getAllByRole("listitem");
    expect(contextSummary).toHaveTextContent(
      "project-state: verified, current",
    );
    expect(proposalSummary).toHaveTextContent(
      "Proposal digest: draft, explicit approval required.",
    );
    expect(
      screen.queryByText("advisor-thread-fixture-01"),
    ).not.toBeInTheDocument();
    expect(screen.queryByText(/gpt-5.6-terra/i)).not.toBeInTheDocument();
  });

  it("requires explicit confirmation before selecting a snapshot", () => {
    const onRequestProjectState = vi.fn();
    render(
      <AdvisorWorkspace
        {...props}
        onRequestProjectState={onRequestProjectState}
      />,
    );
    fireEvent.click(
      screen.getByRole("button", {
        name: "Select current Project State snapshot",
      }),
    );
    expect(onRequestProjectState).toHaveBeenCalledOnce();
  });

  it("requires a second per-send confirmation before including selected context", () => {
    const onConversationStart = vi
      .fn()
      .mockResolvedValue(scaffoldAdvisorConversation);
    render(
      <AdvisorWorkspace
        {...props}
        selectedProjectState={selectedProjectStateFixture}
        selectedProjectId="018f0000-0000-7000-8000-000000000001"
        onConversationStart={onConversationStart}
      />,
    );
    expect(
      screen.getByText(/Temporary safe summary: current, clean/u),
    ).toBeInTheDocument();
    fireEvent.change(screen.getByRole("textbox", { name: "Advisor message" }), {
      target: { value: "Prepare a safe milestone plan." },
    });
    fireEvent.click(
      screen.getByRole("checkbox", {
        name: /include the selected temporary project state summary/i,
      }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Send to Advisor" }));
    expect(onConversationStart).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Confirm inclusion" }));
    expect(onConversationStart).toHaveBeenCalledWith({
      prompt: "Prepare a safe milestone plan.",
      projectId: "018f0000-0000-7000-8000-000000000001",
    });
  });

  it("keeps an API-key account unavailable", () => {
    render(
      <AdvisorWorkspace
        {...props}
        auth={{ ...props.auth, accountKind: "api-key" }}
      />,
    );
    expect(
      screen.getByRole("button", { name: "Send to Advisor" }),
    ).toBeDisabled();
    expect(
      screen.getByText(
        "Advisor is unavailable until managed ChatGPT browser sign-in is complete.",
      ),
    ).toBeInTheDocument();
  });

  it("keeps the capability notice separate when a sendable message is entered", () => {
    render(<AdvisorWorkspace {...props} />);
    fireEvent.change(screen.getByRole("textbox", { name: "Advisor message" }), {
      target: { value: "Review the milestone boundary." },
    });

    expect(
      screen.getByRole("button", { name: "Send to Advisor" }),
    ).toBeEnabled();
    expect(
      screen.getByText(/Project State is optional and requires confirmation/u),
    ).toBeInTheDocument();
    expect(
      screen.queryByText("Enter a message to send."),
    ).not.toBeInTheDocument();
  });

  it("labels Phase A draft approval as non-executable", () => {
    render(
      <AdvisorWorkspace
        {...props}
        targetProjectId="018f0000-0000-7000-8000-000000000002"
        conversation={{
          ...scaffoldAdvisorConversation,
          state: "completed",
          conversationId: "018f0000-0000-7000-8000-000000000003",
        }}
      />,
    );
    expect(
      screen.getByRole("textbox", { name: "Editable draft" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Create approval draft" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Copy draft" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        /cannot start Codex, run a command, or change a project/u,
      ),
    ).toBeInTheDocument();
  });

  it("binds the selected safe projection and invalidates approval when its target changes", async () => {
    advisorDraftBridge.createAdvisorDraft.mockResolvedValue({
      proposalId: "018f0000-0000-7000-8000-000000000004",
      state: "draft",
      expiresAtMs: 1771235467000,
      dispatchAvailable: false,
    });
    const conversation = {
      ...scaffoldAdvisorConversation,
      state: "completed" as const,
      conversationId: "018f0000-0000-7000-8000-000000000003",
    };
    const { rerender } = render(
      <AdvisorWorkspace
        {...props}
        conversation={conversation}
        targetProjectId="018f0000-0000-7000-8000-000000000002"
        selectedProjectId="018f0000-0000-7000-8000-000000000002"
        selectedProjectState={selectedProjectStateFixture}
      />,
    );
    fireEvent.click(
      screen.getByRole("checkbox", {
        name: /include the selected temporary project state summary/i,
      }),
    );
    fireEvent.change(screen.getByRole("textbox", { name: "Editable draft" }), {
      target: { value: "Prepare a bounded implementation plan." },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Create approval draft" }),
    );

    await waitFor(() =>
      expect(advisorDraftBridge.createAdvisorDraft).toHaveBeenCalledWith(
        expect.objectContaining({
          selectedProjectState: selectedProjectStateFixture,
        }),
      ),
    );
    expect(await screen.findByText(/Draft is draft/u)).toBeInTheDocument();

    rerender(
      <AdvisorWorkspace
        {...props}
        conversation={conversation}
        targetProjectId="018f0000-0000-7000-8000-000000000005"
        selectedProjectId="018f0000-0000-7000-8000-000000000005"
        selectedProjectState={selectedProjectStateFixture}
      />,
    );
    await waitFor(() =>
      expect(screen.queryByText(/Draft is draft/u)).not.toBeInTheDocument(),
    );
  });
});
