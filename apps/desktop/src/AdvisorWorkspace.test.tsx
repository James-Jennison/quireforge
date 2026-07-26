import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

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
    expect(screen.getByRole("status")).toHaveTextContent(/managed ChatGPT/i);
  });
});
