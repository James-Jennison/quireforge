import { describe, expect, it, vi } from "vitest";
import {
  TASK_CATALOG_CREATE_COMMAND,
  TASK_CATALOG_RENAME_COMMAND,
  TASK_CATALOG_STATUS_COMMAND,
  TASK_PLAN_SELECT_COMMAND,
  createTaskRecord,
  loadTaskCatalog,
  renameTaskRecord,
  selectTaskPlan,
} from "./bridge";

const taskId = "018f0000-0000-7000-8000-000000000001";
const planId = "018f0000-0000-7000-8000-000000000002";
const emptyCatalog = {
  schemaVersion: 1,
  state: "empty",
  tasks: [],
  selectedTask: null,
  plans: [],
  taskCount: 0,
  payloadBytes: 0,
  warning: false,
  diagnosticCode: null,
};

describe("task record bridge", () => {
  it("uses fixed commands and normalized closed request envelopes", async () => {
    const invoke = vi.fn().mockResolvedValue(emptyCatalog);

    await loadTaskCatalog(
      {
        query: "  local\u00a0 task ",
        includeArchived: false,
        selectedTaskId: null,
      },
      invoke,
    );
    await createTaskRecord(invoke);
    await renameTaskRecord({ taskId, title: "  New\t title " }, invoke);
    await selectTaskPlan({ taskId, planId }, invoke);

    expect(invoke.mock.calls).toEqual([
      [
        TASK_CATALOG_STATUS_COMMAND,
        {
          request: {
            query: "local task",
            includeArchived: false,
            selectedTaskId: null,
          },
        },
      ],
      [TASK_CATALOG_CREATE_COMMAND],
      [
        TASK_CATALOG_RENAME_COMMAND,
        { request: { taskId, title: "New title" } },
      ],
      [TASK_PLAN_SELECT_COMMAND, { request: { taskId, planId } }],
    ]);
  });

  it("rejects capability-bearing input before native invocation", async () => {
    const invoke = vi.fn().mockResolvedValue(emptyCatalog);

    await expect(
      loadTaskCatalog(
        {
          query: null,
          includeArchived: false,
          selectedTaskId: null,
          projectPath: "/tmp/private",
        },
        invoke,
      ),
    ).rejects.toThrow();
    await expect(
      renameTaskRecord({ taskId, title: "Safe", execute: true }, invoke),
    ).rejects.toThrow();
    expect(invoke).not.toHaveBeenCalled();
  });

  it("rejects malformed native responses", async () => {
    const invoke = vi.fn().mockResolvedValue({
      ...emptyCatalog,
      taskCount: 1,
      warning: true,
      transcript: "must not cross the bridge",
    });

    await expect(createTaskRecord(invoke)).rejects.toThrow();
  });
});
