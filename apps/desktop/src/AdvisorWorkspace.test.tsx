import { fireEvent, render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import {
  parseAdvisorSelectedProjectStateSnapshot,
  advisorWorkspaceSnapshotSchema,
} from "./lib/advisorWorkspace";
import { AdvisorWorkspace } from "./AdvisorWorkspace";

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

const selectionProps = {
  selectedProjectState: null,
  selectionState: "idle" as const,
  canSelectProjectState: true,
  onRequestProjectState: vi.fn(),
  onConfirmProjectState: vi.fn(),
  onCancelProjectState: vi.fn(),
  onRemoveProjectState: vi.fn(),
};

describe("AdvisorWorkspace", () => {
  it("renders a reference-only empty state without executable controls", () => {
    render(
      <AdvisorWorkspace
        availability="native"
        snapshot={{
          ...advisorWorkspaceFixture,
          conversationCount: 0,
          contextReferenceCount: 0,
          proposalCount: 0,
          contextSummaries: [],
          proposalSummaries: [],
        }}
        {...selectionProps}
      />,
    );

    expect(
      screen.getByRole("heading", {
        name: "Reference-only planning, without execution.",
      }),
    ).toBeInTheDocument();
    expect(screen.getByText("No Advisor metadata yet.")).toBeInTheDocument();
    expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", {
        name: "Select current Project State snapshot",
      }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /approve|dispatch|terminal/u }),
    ).not.toBeInTheDocument();
  });

  it("shows only safe reference summaries for valid metadata", () => {
    render(
      <AdvisorWorkspace
        availability="native"
        snapshot={advisorWorkspaceFixture}
        {...selectionProps}
      />,
    );

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

  it("announces native read failures without inventing Advisor state", () => {
    render(
      <AdvisorWorkspace
        availability="error"
        snapshot={null}
        {...selectionProps}
      />,
    );
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Advisor metadata could not be read",
    );
  });

  it("requires explicit confirmation before asking for a selected snapshot", () => {
    const onRequestProjectState = vi.fn();
    const onConfirmProjectState = vi.fn();
    render(
      <AdvisorWorkspace
        availability="native"
        snapshot={advisorWorkspaceFixture}
        {...selectionProps}
        onRequestProjectState={onRequestProjectState}
        onConfirmProjectState={onConfirmProjectState}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", {
        name: "Select current Project State snapshot",
      }),
    );
    expect(onRequestProjectState).toHaveBeenCalledOnce();
    expect(onConfirmProjectState).not.toHaveBeenCalled();
  });

  it("renders only the safe selected-state summary", () => {
    render(
      <AdvisorWorkspace
        availability="native"
        snapshot={advisorWorkspaceFixture}
        {...selectionProps}
        selectedProjectState={selectedProjectStateFixture}
      />,
    );
    expect(
      screen.getByText(/Temporary safe summary: current, clean\./u),
    ).toBeInTheDocument();
    expect(
      screen.queryByText(/attached project|main|\.git/u),
    ).not.toBeInTheDocument();
  });
});
