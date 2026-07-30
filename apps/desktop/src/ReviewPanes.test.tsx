import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

import { ReviewPanes } from "./ReviewPanes";
import type { ReviewPaneId } from "./review-panes/types";
import { scaffoldConversation } from "./lib/conversation";
import { scaffoldFilePreview } from "./lib/filePreview";
import { scaffoldGitWorkspace } from "./lib/git";
import { scaffoldTaskCatalog } from "./lib/taskRecords";

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
  const onWidthChange = vi.fn();
  const onSelectedPaneChange = vi.fn();
  function Harness() {
    const [selectedPane, setSelectedPane] = useState<ReviewPaneId>("files");
    return (
      <ReviewPanes
        projectId={projectId}
        projectName="QuireForge"
        filePreview={scaffoldFilePreview}
        conversation={scaffoldConversation}
        conversationEvents={[]}
        taskCatalog={scaffoldTaskCatalog}
        loadGitStatus={loadGitStatus}
        loadGitDiff={loadGitDiff}
        loadArtifacts={loadArtifacts}
        previewArtifact={previewArtifact}
        width={480}
        selectedPane={selectedPane}
        onWidthChange={onWidthChange}
        onSelectedPaneChange={(pane) => {
          onSelectedPaneChange(pane);
          setSelectedPane(pane);
        }}
        onClose={onClose}
      />
    );
  }
  render(<Harness />);
  return {
    loadArtifacts,
    loadGitDiff,
    loadGitStatus,
    onClose,
    onSelectedPaneChange,
    onWidthChange,
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

  it("exposes all seven labelled inspection panes and restores focus on close", () => {
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
      "Review",
    ]) {
      expect(screen.getByRole("tab", { name })).toBeVisible();
    }
    fireEvent.click(
      screen.getByRole("button", { name: "Close task evidence" }),
    );
    expect(calls.onClose).toHaveBeenCalledTimes(1);
    document.body.removeChild(trigger);
  });

  it("bounds keyboard and pointer resizing and removes drag listeners", () => {
    const remove = vi.spyOn(document, "removeEventListener");
    const calls = renderPanes();
    const separator = screen.getByRole("separator", {
      name: "Resize task evidence",
    });
    expect(separator).toHaveAttribute("aria-valuemin", "360");
    expect(separator).toHaveAttribute("aria-valuemax", "560");
    fireEvent.keyDown(separator, { key: "ArrowLeft" });
    expect(calls.onWidthChange).toHaveBeenLastCalledWith(500);
    fireEvent.pointerDown(separator, { clientX: 10 });
    fireEvent.pointerMove(document, { clientX: -100 });
    expect(calls.onWidthChange).toHaveBeenLastCalledWith(560);
    fireEvent.pointerUp(document);
    expect(remove).toHaveBeenCalledWith("pointermove", expect.any(Function));
    remove.mockRestore();
  });
});
