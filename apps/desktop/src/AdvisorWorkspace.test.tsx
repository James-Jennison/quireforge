import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { advisorWorkspaceSnapshotSchema } from "./lib/advisorWorkspace";
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
      />,
    );

    expect(
      screen.getByRole("heading", {
        name: "Reference-only planning, without execution.",
      }),
    ).toBeInTheDocument();
    expect(screen.getByText("No Advisor metadata yet.")).toBeInTheDocument();
    expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  it("shows only safe reference summaries for valid metadata", () => {
    render(
      <AdvisorWorkspace
        availability="native"
        snapshot={advisorWorkspaceFixture}
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
    render(<AdvisorWorkspace availability="error" snapshot={null} />);
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Advisor metadata could not be read",
    );
  });
});
