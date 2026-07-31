import { describe, expect, it, vi } from "vitest";

import {
  archiveTaskTemplate,
  cancelTaskTemplateApplication,
  confirmTaskTemplateApplication,
  createTaskTemplate,
  deleteTaskTemplate,
  duplicateTaskTemplate,
  editTaskTemplate,
  inspectTaskTemplate,
  loadTaskTemplateCatalog,
  previewTaskTemplateApplication,
  restoreTaskTemplate,
  TASK_TEMPLATE_ARCHIVE_COMMAND,
  TASK_TEMPLATE_CANCEL_COMMAND,
  TASK_TEMPLATE_CATALOG_COMMAND,
  TASK_TEMPLATE_CONFIRM_COMMAND,
  TASK_TEMPLATE_CREATE_COMMAND,
  TASK_TEMPLATE_DELETE_COMMAND,
  TASK_TEMPLATE_DUPLICATE_COMMAND,
  TASK_TEMPLATE_EDIT_COMMAND,
  TASK_TEMPLATE_INSPECT_COMMAND,
  TASK_TEMPLATE_PREVIEW_COMMAND,
  TASK_TEMPLATE_RESTORE_COMMAND,
} from "./bridge";
import {
  taskTemplateApplicationOutcomeSchema,
  taskTemplateCatalogSchema,
  taskTemplateInspectionSchema,
  taskTemplatePreviewSchema,
} from "./taskTemplates";

const id = "01980a10-0000-7000-8000-000000000001";
const handle = "01980a10-0000-7000-8000-000000000002";
const reservation = "01980a10-0000-7000-8000-000000000003";
const content = {
  title: "Template",
  purpose: "A bounded purpose",
  instructions: "Use evidence.",
};
const catalog = {
  schemaVersion: 1,
  state: "ready",
  templates: [
    {
      id,
      title: "Template",
      purpose: "A bounded purpose",
      origin: "local",
      state: "active",
    },
  ],
  capacity: {
    recordCount: 1,
    canonicalBytes: 20,
    warning: false,
    countLimit: 64,
    canonicalByteLimit: 2 * 1024 * 1024,
  },
  diagnosticCode: null,
};
const inspection = {
  schemaVersion: 1,
  state: "ready",
  template: {
    ...catalog.templates[0],
    instructions: "Use evidence.",
    version: 1,
    sha256: "a".repeat(64),
  },
  mutationHandle: handle,
  diagnosticCode: null,
};
const preview = {
  schemaVersion: 1,
  state: "ready",
  reservationId: reservation,
  bindingSha256: "a".repeat(64),
  expiresAtMs: 1,
  checklist: {
    templateActive: true,
    taskPlanAvailable: true,
    exactDraftRequired: true,
    confirmationRequired: true,
  },
  diagnosticCode: null,
};
const outcome = {
  schemaVersion: 1,
  state: "ready",
  applied: true,
  cancelled: false,
  diagnosticCode: null,
};

describe("task-template bridge", () => {
  it("sends exact closed payloads for every command", async () => {
    const invoke = vi.fn((command: string) =>
      Promise.resolve(
        command === TASK_TEMPLATE_CATALOG_COMMAND
          ? catalog
          : command === TASK_TEMPLATE_PREVIEW_COMMAND
            ? preview
            : [
                  TASK_TEMPLATE_CONFIRM_COMMAND,
                  TASK_TEMPLATE_DELETE_COMMAND,
                  TASK_TEMPLATE_CANCEL_COMMAND,
                ].includes(command)
              ? outcome
              : inspection,
      ),
    );
    await loadTaskTemplateCatalog(invoke);
    await inspectTaskTemplate({ templateId: id }, invoke);
    await createTaskTemplate(content, invoke);
    await editTaskTemplate({ mutationHandle: handle, ...content }, invoke);
    await duplicateTaskTemplate({ mutationHandle: handle }, invoke);
    await archiveTaskTemplate({ mutationHandle: handle }, invoke);
    await restoreTaskTemplate({ mutationHandle: handle }, invoke);
    await deleteTaskTemplate(
      { mutationHandle: handle, confirmation: "confirmed" },
      invoke,
    );
    await previewTaskTemplateApplication(
      {
        templateId: id,
        taskId: handle,
        planId: reservation,
        title: "Draft",
        planText: "Plan",
      },
      invoke,
    );
    await confirmTaskTemplateApplication(
      { reservationId: reservation, title: "Draft", planText: "Plan" },
      invoke,
    );
    await cancelTaskTemplateApplication({ reservationId: reservation }, invoke);
    expect(invoke.mock.calls).toEqual([
      [TASK_TEMPLATE_CATALOG_COMMAND],
      [TASK_TEMPLATE_INSPECT_COMMAND, { request: { templateId: id } }],
      [TASK_TEMPLATE_CREATE_COMMAND, { request: content }],
      [
        TASK_TEMPLATE_EDIT_COMMAND,
        { request: { mutationHandle: handle, ...content } },
      ],
      [
        TASK_TEMPLATE_DUPLICATE_COMMAND,
        { request: { mutationHandle: handle } },
      ],
      [TASK_TEMPLATE_ARCHIVE_COMMAND, { request: { mutationHandle: handle } }],
      [TASK_TEMPLATE_RESTORE_COMMAND, { request: { mutationHandle: handle } }],
      [
        TASK_TEMPLATE_DELETE_COMMAND,
        { request: { mutationHandle: handle, confirmation: "confirmed" } },
      ],
      [
        TASK_TEMPLATE_PREVIEW_COMMAND,
        {
          request: {
            templateId: id,
            taskId: handle,
            planId: reservation,
            title: "Draft",
            planText: "Plan",
          },
        },
      ],
      [
        TASK_TEMPLATE_CONFIRM_COMMAND,
        {
          request: {
            reservationId: reservation,
            title: "Draft",
            planText: "Plan",
          },
        },
      ],
      [
        TASK_TEMPLATE_CANCEL_COMMAND,
        { request: { reservationId: reservation } },
      ],
    ]);
  });

  it("rejects native-owned request fields, malformed authority, and unknown response fields", async () => {
    await expect(
      createTaskTemplate({ ...content, version: 1 }, vi.fn()),
    ).rejects.toThrow();
    await expect(
      editTaskTemplate({ mutationHandle: "bad", ...content }, vi.fn()),
    ).rejects.toThrow();
    await expect(
      previewTaskTemplateApplication(
        {
          templateId: id,
          taskId: handle,
          planId: reservation,
          title: "Draft",
          planText: "Plan",
          bindingSha256: "a".repeat(64),
        },
        vi.fn(),
      ),
    ).rejects.toThrow();
    await expect(
      confirmTaskTemplateApplication(
        {
          reservationId: reservation,
          title: "Draft",
          planText: "Plan",
          expiresAtMs: 1,
        },
        vi.fn(),
      ),
    ).rejects.toThrow();
    expect(() =>
      taskTemplateCatalogSchema.parse({
        ...catalog,
        capacity: { ...catalog.capacity, version: 1 },
      }),
    ).toThrow();
    expect(() =>
      taskTemplateInspectionSchema.parse({
        ...inspection,
        mutationHandle: "bad",
      }),
    ).toThrow();
    expect(() =>
      taskTemplatePreviewSchema.parse({ ...preview, expiresAtMs: -1 }),
    ).toThrow();
    expect(() =>
      taskTemplateApplicationOutcomeSchema.parse({
        ...outcome,
        sqlite: "diagnostic",
      }),
    ).toThrow();
  });
});
