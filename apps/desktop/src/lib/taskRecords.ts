import taskCatalogFixture from "../../fixtures/task-catalog.json";
import { z } from "zod";

const id = z
  .string()
  .regex(
    /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/,
    "expected a canonical UUIDv7",
  );

const utf8Length = (value: string) =>
  new TextEncoder().encode(value).byteLength;
const bidirectionalFormatControl =
  /[\u061c\u200e\u200f\u202a-\u202e\u2066-\u2069]/u;
const normalizeTaskText = (value: string) =>
  value.trim().replaceAll(/\s+/gu, " ");
const normalizedTaskText = (characters: number, bytes: number) =>
  z
    .string()
    .transform(normalizeTaskText)
    .pipe(
      z
        .string()
        .min(1)
        .refine((value) => [...value].length <= characters)
        .refine((value) => utf8Length(value) <= bytes)
        .refine(
          (value) =>
            !/[\p{Cc}]/u.test(value) && !bidirectionalFormatControl.test(value),
        ),
    );
const storedTaskText = (characters: number, bytes: number) =>
  z
    .string()
    .min(1)
    .refine((value) => [...value].length <= characters)
    .refine((value) => utf8Length(value) <= bytes)
    .refine(
      (value) =>
        !/[\p{Cc}]/u.test(value) &&
        !bidirectionalFormatControl.test(value) &&
        normalizeTaskText(value) === value,
    );
const planBody = z
  .string()
  .refine((value) => [...value].length <= 8_192)
  .refine((value) => utf8Length(value) <= 32 * 1024)
  .refine(
    (value) =>
      ![...value].some(
        (character) =>
          (/\p{Cc}/u.test(character) &&
            character !== "\n" &&
            character !== "\t") ||
          bidirectionalFormatControl.test(character),
      ),
  );

export const taskIdRequestSchema = z.object({ taskId: id }).strict();
export const taskTitleRequestSchema = z
  .object({ taskId: id, title: normalizedTaskText(120, 480) })
  .strict();
export const taskStatusRequestSchema = z
  .object({ taskId: id, status: z.enum(["active", "paused", "completed"]) })
  .strict();
export const planCreateRequestSchema = z
  .object({ taskId: id, copyPrimaryBody: z.boolean() })
  .strict();
export const planIdRequestSchema = z
  .object({ taskId: id, planId: id })
  .strict();
export const planEditRequestSchema = z
  .object({
    taskId: id,
    planId: id,
    label: normalizedTaskText(80, 320),
    body: planBody,
  })
  .strict();
export const taskCatalogRequestSchema = z
  .object({
    query: normalizedTaskText(120, 480).or(z.literal("")).nullable(),
    includeArchived: z.boolean(),
    selectedTaskId: id.nullable(),
  })
  .strict();
const plan = z
  .object({
    id,
    label: storedTaskText(80, 320),
    position: z.number().int().min(0).max(3),
    body: planBody,
  })
  .strict();
const task = z
  .object({
    id,
    title: storedTaskText(120, 480),
    status: z.enum(["active", "paused", "completed"]),
    archived: z.boolean(),
    selectedPlanId: id,
    planCount: z.number().int().min(1).max(4),
    updatedAtMs: z.number().int().nonnegative(),
    cleanupEligible: z.boolean(),
  })
  .strict();
export const taskCatalogSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum(["empty", "ready", "unavailable"]),
    tasks: z.array(task).max(50),
    selectedTask: task.nullable(),
    plans: z.array(plan).max(4),
    taskCount: z.number().int().min(0).max(200),
    payloadBytes: z
      .number()
      .int()
      .nonnegative()
      .max(8 * 1024 * 1024),
    warning: z.boolean(),
    diagnosticCode: z
      .enum([
        "metadata-unavailable",
        "invalid-request",
        "capacity-reached",
        "task-not-found",
        "task-archived",
        "plan-not-found",
        "invalid-status-transition",
        "duplicate-id",
        "invalid-stored-value",
      ])
      .nullable(),
  })
  .strict()
  .superRefine((snapshot, context) => {
    const taskIds = new Set(snapshot.tasks.map((task) => task.id));
    const planIds = new Set(snapshot.plans.map((entry) => entry.id));
    if (
      taskIds.size !== snapshot.tasks.length ||
      planIds.size !== snapshot.plans.length
    ) {
      context.addIssue({
        code: "custom",
        message: "Task catalogue identifiers must be unique",
      });
    }
    if (
      snapshot.warning !==
      (snapshot.taskCount >= 160 || snapshot.payloadBytes >= 6 * 1024 * 1024)
    ) {
      context.addIssue({
        code: "custom",
        message: "Task capacity warning is inconsistent",
      });
    }
    if (
      snapshot.state === "unavailable" &&
      (snapshot.tasks.length !== 0 ||
        snapshot.selectedTask !== null ||
        snapshot.plans.length !== 0 ||
        snapshot.taskCount !== 0 ||
        snapshot.payloadBytes !== 0 ||
        snapshot.diagnosticCode === null)
    ) {
      context.addIssue({
        code: "custom",
        message: "Unavailable task catalogue is inconsistent",
      });
    }
    if (snapshot.state === "ready" && snapshot.tasks.length === 0) {
      context.addIssue({
        code: "custom",
        message: "Ready task catalogue requires tasks",
      });
    }
    if (snapshot.state === "empty" && snapshot.tasks.length !== 0) {
      context.addIssue({
        code: "custom",
        message: "Empty task catalogue cannot expose tasks",
      });
    }
    if (snapshot.selectedTask === null) {
      if (snapshot.plans.length !== 0) {
        context.addIssue({
          code: "custom",
          message: "Plans require a selected task",
        });
      }
      return;
    }
    const selected = snapshot.selectedTask;
    if (!taskIds.has(selected.id)) {
      context.addIssue({
        code: "custom",
        message: "Selected task must be listed",
      });
    }
    if (
      snapshot.plans.length !== selected.planCount ||
      !planIds.has(selected.selectedPlanId) ||
      new Set(snapshot.plans.map((entry) => entry.position)).size !==
        snapshot.plans.length
    ) {
      context.addIssue({
        code: "custom",
        message: "Selected task plan projection is inconsistent",
      });
    }
  });
export type TaskCatalogSnapshot = z.infer<typeof taskCatalogSchema>;

export const scaffoldTaskCatalog: TaskCatalogSnapshot = {
  schemaVersion: 1,
  state: "unavailable",
  tasks: [],
  selectedTask: null,
  plans: [],
  taskCount: 0,
  payloadBytes: 0,
  warning: false,
  diagnosticCode: "metadata-unavailable",
};

export const sharedTaskCatalogFixture =
  taskCatalogSchema.parse(taskCatalogFixture);
