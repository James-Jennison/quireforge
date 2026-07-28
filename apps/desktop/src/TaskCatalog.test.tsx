import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useState } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { TaskCatalog } from "./TaskCatalog";
import {
  scaffoldTaskCatalog,
  taskCatalogSchema,
  type TaskCatalogSnapshot,
} from "./lib/taskRecords";

const taskId = "018f0000-0000-7000-8000-000000000001";
const primaryPlanId = "018f0000-0000-7000-8000-000000000002";
const alternatePlanId = "018f0000-0000-7000-8000-000000000003";

const readyCatalog = taskCatalogSchema.parse({
  schemaVersion: 1,
  state: "ready",
  tasks: [
    {
      id: taskId,
      title: "Ship local task records",
      status: "completed",
      archived: false,
      selectedPlanId: primaryPlanId,
      planCount: 2,
      updatedAtMs: 1,
      cleanupEligible: true,
    },
  ],
  selectedTask: {
    id: taskId,
    title: "Ship local task records",
    status: "completed",
    archived: false,
    selectedPlanId: primaryPlanId,
    planCount: 2,
    updatedAtMs: 1,
    cleanupEligible: true,
  },
  plans: [
    {
      id: primaryPlanId,
      label: "Primary",
      position: 0,
      body: "Implement the bounded local store.",
    },
    {
      id: alternatePlanId,
      label: "Alternate",
      position: 1,
      body: "Use another visible plan.",
    },
  ],
  taskCount: 1,
  payloadBytes: 512,
  warning: false,
  diagnosticCode: null,
});

function props(snapshot: TaskCatalogSnapshot = readyCatalog) {
  return {
    snapshot,
    busy: false,
    onLoad: vi.fn().mockResolvedValue(undefined),
    onCreate: vi.fn().mockResolvedValue(snapshot),
    onRename: vi.fn(() => vi.fn().mockResolvedValue(snapshot)),
    onStatus: vi.fn(() => vi.fn().mockResolvedValue(snapshot)),
    onArchive: vi.fn(() => vi.fn().mockResolvedValue(snapshot)),
    onRestore: vi.fn(() => vi.fn().mockResolvedValue(snapshot)),
    onDelete: vi.fn(() => vi.fn().mockResolvedValue(snapshot)),
    onPlanCreate: vi.fn(() => vi.fn().mockResolvedValue(snapshot)),
    onPlanSelect: vi.fn(() => vi.fn().mockResolvedValue(snapshot)),
    onPlanEdit: vi.fn(() => vi.fn().mockResolvedValue(snapshot)),
    onPlanDelete: vi.fn(() => vi.fn().mockResolvedValue(snapshot)),
  };
}

describe("durable task catalogue", () => {
  beforeEach(() => {
    vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => {
      callback(0);
      return 1;
    });
  });

  it("renders bounded local-only semantics, cleanup, and accessible plan state", () => {
    render(<TaskCatalog {...props()} />);

    expect(screen.getByRole("navigation", { name: "Task list" })).toBeVisible();
    expect(
      screen.getByRole("searchbox", { name: "Search tasks" }),
    ).toHaveAttribute("maxlength", "120");
    expect(
      screen.getByRole("button", {
        name: /Ship local task records, completed, not archived, 2 plans, eligible for cleanup/u,
      }),
    ).toHaveAttribute("aria-current", "page");
    expect(screen.getByRole("tab", { name: "Primary" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
    expect(
      screen.queryByRole("option", { name: "Paused" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByText(
        /do not contain or control conversations, files, approvals, or execution/u,
      ),
    ).toBeVisible();
  });

  it("issues bounded search and archived-filter requests", () => {
    const catalogProps = props();
    render(<TaskCatalog {...catalogProps} />);

    fireEvent.change(screen.getByRole("searchbox", { name: "Search tasks" }), {
      target: { value: "Alternate" },
    });
    fireEvent.click(
      screen.getByRole("checkbox", { name: "Include archived tasks" }),
    );

    expect(catalogProps.onLoad).toHaveBeenNthCalledWith(1, {
      query: "Alternate",
      includeArchived: false,
      selectedTaskId: null,
    });
    expect(catalogProps.onLoad).toHaveBeenNthCalledWith(2, {
      query: "Alternate",
      includeArchived: true,
      selectedTaskId: null,
    });
  });

  it("supports keyboard plan switching and announces transient-state clearing", async () => {
    const catalogProps = props();
    render(<TaskCatalog {...catalogProps} />);
    const primary = screen.getByRole("tab", { name: "Primary" });

    primary.focus();
    fireEvent.keyDown(primary, { key: "ArrowRight" });

    await waitFor(() =>
      expect(catalogProps.onPlanSelect).toHaveBeenCalledWith(
        taskId,
        alternatePlanId,
      ),
    );
    expect(screen.getByRole("tab", { name: "Alternate" })).toHaveFocus();
    expect(screen.getByRole("status")).toHaveTextContent(
      "Transient task-plan state was cleared.",
    );
  });

  it("traps destructive confirmation focus and restores the opening control", async () => {
    render(<TaskCatalog {...props()} />);
    const trigger = screen.getByRole("button", { name: "Delete task" });

    trigger.focus();
    fireEvent.click(trigger);
    const dialog = screen.getByRole("dialog", {
      name: "Delete “Ship local task records”?",
    });
    expect(screen.getByRole("button", { name: "Cancel" })).toHaveFocus();
    expect(dialog).toHaveTextContent(
      "External project files, worktrees, Git history, package evidence, repository source, and user-saved artifacts will not change.",
    );
    expect(dialog).toHaveTextContent("no application trash");

    fireEvent.keyDown(dialog, { key: "Escape" });
    await waitFor(() => expect(trigger).toHaveFocus());
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("keeps archived task plans inspectable but read-only", () => {
    const archived = taskCatalogSchema.parse({
      ...readyCatalog,
      tasks: readyCatalog.tasks.map((task) => ({ ...task, archived: true })),
      selectedTask: { ...readyCatalog.selectedTask!, archived: true },
    });
    render(<TaskCatalog {...props(archived)} />);

    expect(screen.getByRole("textbox", { name: "Task title" })).toBeDisabled();
    expect(screen.getByRole("textbox", { name: "Plan label" })).toHaveAttribute(
      "readonly",
    );
    expect(screen.getByRole("textbox", { name: "Plan text" })).toHaveAttribute(
      "readonly",
    );
    expect(screen.getByRole("button", { name: "Restore task" })).toBeVisible();
    expect(
      screen.queryByRole("button", { name: "Add empty plan" }),
    ).not.toBeInTheDocument();
  });

  it("renders honest empty and migration-unavailable states", () => {
    const { rerender } = render(
      <TaskCatalog
        {...props(
          taskCatalogSchema.parse({
            ...scaffoldTaskCatalog,
            state: "empty",
            diagnosticCode: null,
          }),
        )}
      />,
    );
    expect(
      screen.getByText(
        "Create a local task to keep a title and up to four plans.",
      ),
    ).toBeVisible();

    rerender(<TaskCatalog {...props(scaffoldTaskCatalog)} />);
    expect(
      screen.getByText(
        "Task records are unavailable. Existing project and conversation state is unchanged.",
      ),
    ).toBeVisible();
    expect(screen.getByRole("button", { name: "New task" })).toBeDisabled();
  });

  it("places focus in the created task title without importing transient state", async () => {
    function Harness() {
      const [snapshot, setSnapshot] = useState<TaskCatalogSnapshot>(
        taskCatalogSchema.parse({
          ...scaffoldTaskCatalog,
          state: "empty",
          diagnosticCode: null,
        }),
      );
      const catalogProps = props(snapshot);
      return (
        <TaskCatalog
          {...catalogProps}
          snapshot={snapshot}
          onCreate={() => {
            setSnapshot(readyCatalog);
            return Promise.resolve(readyCatalog);
          }}
        />
      );
    }
    render(<Harness />);

    fireEvent.click(screen.getByRole("button", { name: "New task" }));

    await waitFor(() =>
      expect(screen.getByRole("textbox", { name: "Task title" })).toHaveFocus(),
    );
    expect(screen.getByRole("textbox", { name: "Plan text" })).toHaveValue(
      "Implement the bounded local store.",
    );
  });
});
