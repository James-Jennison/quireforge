import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { WorktreeWorkspace } from "./WorktreeWorkspace";
import {
  scaffoldWorktreeWorkspace,
  worktreePreviewSchema,
  worktreeWorkspaceSchema,
} from "./lib/worktree";

const sourceProjectId = "018f0000-0000-7000-8000-000000000001";
const confirmationId = "018f0000-0000-7000-8000-000000000002";
const linkedProjectId = "018f0000-0000-7000-8000-000000000003";
const recoveryId = "018f0000-0000-7000-8000-000000000004";
const workspace = worktreeWorkspaceSchema.parse({
  schemaVersion: 2,
  state: "ready",
  sourceProjectId,
  worktrees: [
    {
      projectId: sourceProjectId,
      recoveryId: null,
      displayName: "QuireForge",
      displayPath: "~/work/quireforge",
      branchName: "main",
      ownership: "source",
      state: "ready",
      current: true,
    },
    {
      projectId: null,
      recoveryId: null,
      displayName: "feature/external",
      displayPath: "~/work/external",
      branchName: "feature/external",
      ownership: "external",
      state: "ready",
      current: false,
    },
    {
      projectId: linkedProjectId,
      recoveryId: null,
      displayName: "feature/managed",
      displayPath: "~/.local/share/quireforge/worktrees/managed",
      branchName: "feature/managed",
      ownership: "managed",
      state: "ready",
      current: false,
    },
    {
      projectId: null,
      recoveryId,
      displayName: "feature/recoverable",
      displayPath: "~/.local/share/quireforge/worktrees/recoverable",
      branchName: "feature/recoverable",
      ownership: "external",
      state: "ready",
      current: false,
    },
  ],
  truncated: false,
  diagnosticCode: null,
});

const handlers = {
  onRefresh: vi.fn().mockResolvedValue(undefined),
  onCreate: vi.fn().mockResolvedValue(undefined),
  onPickAttach: vi.fn().mockResolvedValue(undefined),
  onRecover: vi.fn().mockResolvedValue(undefined),
  onRemove: vi.fn().mockResolvedValue(undefined),
  onConfirm: vi.fn().mockResolvedValue(undefined),
  onCancel: vi.fn().mockResolvedValue(undefined),
  onSelectProject: vi.fn(),
  onOpenExecution: vi.fn(),
};

describe("WorktreeWorkspace", () => {
  it("accepts a bounded branch and limits cleanup to managed entries", () => {
    render(
      <WorktreeWorkspace
        availability="native"
        projectName="QuireForge"
        snapshot={workspace}
        preview={null}
        result={null}
        busy={false}
        selectionBusy={false}
        actionError={false}
        executions={[]}
        {...handlers}
      />,
    );

    const create = screen.getByRole("button", {
      name: "Preview managed worktree",
    });
    expect(create).toBeDisabled();
    fireEvent.change(screen.getByLabelText("New branch name"), {
      target: { value: "feature/native-foundation" },
    });
    expect(create).toBeEnabled();
    fireEvent.click(create);
    expect(handlers.onCreate).toHaveBeenCalledWith("feature/native-foundation");
    fireEvent.click(screen.getByRole("button", { name: "Review cleanup" }));
    fireEvent.click(screen.getByRole("button", { name: "Review recovery" }));
    expect(handlers.onRemove).toHaveBeenCalledWith(linkedProjectId);
    expect(handlers.onRecover).toHaveBeenCalledWith(recoveryId);
    expect(screen.getAllByText("external checkout")).toHaveLength(2);
  });

  it("requires confirmation and can cancel by opaque token only", () => {
    const preview = worktreePreviewSchema.parse({
      schemaVersion: 2,
      state: "ready",
      sourceProjectId,
      operation: "create",
      branchName: "feature/confirmed",
      displayPath: "~/.local/share/quireforge/worktrees/confirmed",
      ownership: "managed",
      destructive: false,
      confirmationId,
      diagnosticCode: null,
    });
    render(
      <WorktreeWorkspace
        availability="native"
        projectName="QuireForge"
        snapshot={workspace}
        preview={preview}
        result={null}
        busy={false}
        selectionBusy={false}
        actionError={false}
        executions={[]}
        {...handlers}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Confirm create" }));
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(handlers.onConfirm).toHaveBeenCalledWith(confirmationId);
    expect(handlers.onCancel).toHaveBeenCalledWith(confirmationId);
  });

  it("makes managed removal explicitly destructive and promises branch retention", () => {
    const preview = worktreePreviewSchema.parse({
      schemaVersion: 2,
      state: "ready",
      sourceProjectId,
      operation: "remove",
      branchName: "feature/managed",
      displayPath: "~/.local/share/quireforge/worktrees/managed",
      ownership: "managed",
      destructive: true,
      confirmationId,
      diagnosticCode: null,
    });
    render(
      <WorktreeWorkspace
        availability="native"
        projectName="QuireForge"
        snapshot={workspace}
        preview={preview}
        result={null}
        busy={false}
        selectionBusy={false}
        actionError={false}
        executions={[]}
        {...handlers}
      />,
    );

    expect(screen.getByText("Destructive cleanup preview")).toBeInTheDocument();
    expect(screen.getByText(/Its branch is preserved/u)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Confirm remove" }));
    expect(handlers.onConfirm).toHaveBeenCalledWith(confirmationId);
  });

  it("aggregates live task and conflict status without exposing native IDs", () => {
    render(
      <WorktreeWorkspace
        availability="native"
        projectName="QuireForge"
        snapshot={workspace}
        preview={null}
        result={null}
        busy={false}
        selectionBusy={false}
        actionError={false}
        executions={[
          {
            projectId: sourceProjectId,
            projectName: "QuireForge",
            conversationId: "018f0000-0000-7000-8000-000000000010",
            state: "running",
            changeCount: 3,
            conflictCount: 1,
          },
        ]}
        {...handlers}
      />,
    );

    expect(screen.getByText("1 of 4 active")).toBeInTheDocument();
    expect(screen.getByText("1 conflict")).toBeInTheDocument();
    expect(screen.getByText("3 changed files")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "View live activity" }));
    expect(handlers.onOpenExecution).toHaveBeenCalledWith(sourceProjectId);
    expect(
      screen.queryByText("018f0000-0000-7000-8000-000000000010"),
    ).not.toBeInTheDocument();
  });

  it("does not simulate worktrees in browser preview", () => {
    render(
      <WorktreeWorkspace
        availability="preview"
        projectName={null}
        snapshot={scaffoldWorktreeWorkspace}
        preview={null}
        result={null}
        busy={false}
        selectionBusy={false}
        actionError={false}
        executions={[]}
        {...handlers}
      />,
    );

    expect(
      screen.getByText(/Browser preview cannot inspect or create/u),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Preview managed worktree" }),
    ).toBeDisabled();
  });
});
