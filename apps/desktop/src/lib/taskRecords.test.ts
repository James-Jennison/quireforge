import { describe, expect, it } from "vitest";
import {
  planEditRequestSchema,
  taskCatalogRequestSchema,
  taskCatalogSchema,
  taskTitleRequestSchema,
  sharedTaskCatalogFixture,
} from "./taskRecords";

const taskId = "018f0000-0000-7000-8000-000000000001";
const planId = "018f0000-0000-7000-8000-000000000002";

function emptyCatalog(overrides: Record<string, unknown> = {}) {
  return {
    schemaVersion: 1,
    state: "empty",
    tasks: [],
    selectedTask: null,
    plans: [],
    taskCount: 0,
    payloadBytes: 0,
    warning: false,
    diagnosticCode: null,
    ...overrides,
  };
}

describe("task record contracts", () => {
  it("rejects unknown, path-bearing, and non-UUIDv7 request data", () => {
    expect(() =>
      taskCatalogRequestSchema.parse({
        query: null,
        includeArchived: false,
        selectedTaskId: null,
        path: "/tmp",
      }),
    ).toThrow();
    expect(() =>
      planEditRequestSchema.parse({
        taskId: "00000000-0000-4000-8000-000000000001",
        planId,
        label: "Plan",
        body: "",
      }),
    ).toThrow();
    expect(() =>
      planEditRequestSchema.parse({
        taskId,
        planId,
        label: "Plan",
        body: "",
        token: "x",
      }),
    ).toThrow();
  });

  it("normalizes title whitespace before enforcing character and byte limits", () => {
    expect(
      taskTitleRequestSchema.parse({
        taskId,
        title: "  Release\u00a0\u00a0plan\r\n—\tlocal  ",
      }).title,
    ).toBe("Release plan — local");
    expect(() =>
      taskTitleRequestSchema.parse({
        taskId,
        title: `${"界".repeat(120)}界`,
      }),
    ).toThrow();
    expect(() =>
      taskTitleRequestSchema.parse({
        taskId,
        title: `safe\u202etext`,
      }),
    ).toThrow();
  });

  it("enforces label and visible plan-body bounds without banning punctuation", () => {
    expect(
      planEditRequestSchema.parse({
        taskId,
        planId,
        label: "  Option\u2003A  ",
        body: "Inspect — then save.\n\tVisible only.",
      }),
    ).toMatchObject({
      label: "Option A",
      body: "Inspect — then save.\n\tVisible only.",
    });
    expect(() =>
      planEditRequestSchema.parse({
        taskId,
        planId,
        label: "Plan",
        body: "x".repeat(8_193),
      }),
    ).toThrow();
    expect(() =>
      planEditRequestSchema.parse({
        taskId,
        planId,
        label: "Plan",
        body: "\u0000",
      }),
    ).toThrow();
  });

  it("bounds and cross-validates catalogue responses", () => {
    expect(() =>
      taskCatalogSchema.parse(
        emptyCatalog({
          state: "ready",
          taskCount: 201,
        }),
      ),
    ).toThrow();
    expect(() =>
      taskCatalogSchema.parse(
        emptyCatalog({
          warning: true,
        }),
      ),
    ).toThrow();
    expect(() =>
      taskCatalogSchema.parse(
        emptyCatalog({
          state: "unavailable",
          diagnosticCode: null,
        }),
      ),
    ).toThrow();
    expect(() =>
      taskCatalogSchema.parse(
        emptyCatalog({
          state: "ready",
          taskCount: 1,
        }),
      ),
    ).toThrow();
  });

  it("requires selected tasks and plans to form one closed projection", () => {
    const task = {
      id: taskId,
      title: "Local task",
      status: "active",
      archived: false,
      selectedPlanId: planId,
      planCount: 1,
      updatedAtMs: 1,
      cleanupEligible: false,
    };
    expect(
      taskCatalogSchema.parse(
        emptyCatalog({
          state: "ready",
          tasks: [task],
          selectedTask: task,
          plans: [
            {
              id: planId,
              label: "Primary",
              position: 0,
              body: "",
            },
          ],
          taskCount: 1,
        }),
      ),
    ).toMatchObject({ selectedTask: { id: taskId } });
    expect(() =>
      taskCatalogSchema.parse(
        emptyCatalog({
          state: "ready",
          tasks: [task],
          selectedTask: task,
          plans: [],
          taskCount: 1,
        }),
      ),
    ).toThrow();
  });

  it("keeps the sanitized shared fixture within the closed projection", () => {
    expect(sharedTaskCatalogFixture).toMatchObject({
      state: "ready",
      taskCount: 1,
      selectedTask: { title: "Review local task records" },
    });
    expect(JSON.stringify(sharedTaskCatalogFixture)).not.toMatch(
      /path|conversation|approval|dispatch|execution|terminal|credential/iu,
    );
  });
});
