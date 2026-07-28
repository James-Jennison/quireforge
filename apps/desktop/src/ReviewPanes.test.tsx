import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ReviewPanes } from "./ReviewPanes";
import { scaffoldConversation } from "./lib/conversation";
import { scaffoldFilePreview } from "./lib/filePreview";
import { scaffoldGitWorkspace } from "./lib/git";

const projectId = "018f0000-0000-7000-8000-000000000001";

function renderPanes() {
  const loadGitStatus = vi.fn().mockResolvedValue(scaffoldGitWorkspace);
  const loadGitDiff = vi.fn();
  const loadArtifacts = vi.fn().mockResolvedValue({
    schemaVersion: 1 as const,
    artifacts: [],
    diagnosticCode: null,
  });
  const previewArtifact = vi.fn();
  const onClose = vi.fn();
  render(
    <ReviewPanes
      projectId={projectId}
      projectName="QuireForge"
      filePreview={scaffoldFilePreview}
      conversation={scaffoldConversation}
      conversationEvents={[]}
      loadGitStatus={loadGitStatus}
      loadGitDiff={loadGitDiff}
      loadArtifacts={loadArtifacts}
      previewArtifact={previewArtifact}
      onClose={onClose}
    />,
  );
  return {
    loadArtifacts,
    loadGitDiff,
    loadGitStatus,
    onClose,
    previewArtifact,
  };
}

describe("review panes", () => {
  it("stays closed-data until a pane explicitly needs its typed service", async () => {
    const calls = renderPanes();
    expect(calls.loadGitStatus).not.toHaveBeenCalled();
    expect(calls.loadArtifacts).not.toHaveBeenCalled();
    expect(
      await screen.findByText("No bounded file evidence is open."),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("tab", { name: "Git" }));
    await waitFor(() =>
      expect(calls.loadGitStatus).toHaveBeenCalledWith(projectId),
    );
    expect(calls.loadArtifacts).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("tab", { name: "Preview" }));
    await waitFor(() => expect(calls.loadArtifacts).toHaveBeenCalledTimes(1));
  });

  it("exposes all six labelled inspection panes and restores focus on close", () => {
    const trigger = document.createElement("button");
    document.body.append(trigger);
    trigger.focus();
    const calls = renderPanes();
    for (const name of [
      "Files",
      "Diff",
      "Git",
      "Preview",
      "Activity",
      "Approval",
    ]) {
      expect(screen.getByRole("tab", { name })).toBeVisible();
    }
    fireEvent.click(screen.getByRole("button", { name: "Close review panes" }));
    expect(calls.onClose).toHaveBeenCalledTimes(1);
    document.body.removeChild(trigger);
  });
});
