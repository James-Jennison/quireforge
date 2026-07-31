import { z } from "zod";

const uuid = z
  .string()
  .regex(
    /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/u,
  );
const utf8Length = (value: string) =>
  new TextEncoder().encode(value).byteLength;
const bidi = /[\u061c\u200e\u200f\u202a-\u202e\u2066-\u2069]/u;
const normalized = (characters: number, bytes: number) =>
  z
    .string()
    .transform((value) => value.trim().replaceAll(/\s+/gu, " "))
    .pipe(
      z
        .string()
        .min(1)
        .refine((value) => [...value].length <= characters)
        .refine((value) => utf8Length(value) <= bytes)
        .refine((value) => !/[\p{Cc}]/u.test(value) && !bidi.test(value)),
    );
const instructions = z
  .string()
  .refine((value) => utf8Length(value) <= 32 * 1024)
  .refine(
    (value) =>
      ![...value].some(
        (character) =>
          (/\p{Cc}/u.test(character) &&
            character !== "\n" &&
            character !== "\t") ||
          bidi.test(character),
      ),
  );
const planText = z
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
          bidi.test(character),
      ),
  );

export const taskTemplateIdRequestSchema = z
  .object({ templateId: uuid })
  .strict();
export const taskTemplateContentRequestSchema = z
  .object({
    title: normalized(80, 320),
    purpose: normalized(240, 960),
    instructions,
  })
  .strict();
export const taskTemplateMutationRequestSchema = z
  .object({ mutationHandle: uuid })
  .strict();
export const taskTemplateEditRequestSchema = taskTemplateContentRequestSchema
  .extend({ mutationHandle: uuid })
  .strict();
export const taskTemplateDeleteRequestSchema = z
  .object({ mutationHandle: uuid, confirmation: z.literal("confirmed") })
  .strict();
export const taskTemplatePreviewRequestSchema = z
  .object({
    templateId: uuid,
    taskId: uuid,
    planId: uuid,
    title: normalized(120, 480),
    planText,
  })
  .strict();
export const taskTemplateConfirmRequestSchema = z
  .object({ reservationId: uuid, title: normalized(120, 480), planText })
  .strict();
export const taskTemplateCancelRequestSchema = z
  .object({ reservationId: uuid })
  .strict();

const diagnostic = z.enum([
  "metadata-unavailable",
  "invalid-request",
  "not-found",
  "built-in-immutable",
  "archived-read-only",
  "active-already",
  "archived-already",
  "stale",
  "capacity-reached",
  "unavailable",
]);
const origin = z.enum(["built-in", "local"]);
const state = z.enum(["active", "archived"]);
const summary = z
  .object({
    id: uuid,
    title: z.string().min(1).max(320),
    purpose: z.string().min(1).max(960),
    origin,
    state,
  })
  .strict();
const detail = summary
  .extend({
    instructions: z.string().max(32 * 1024),
    version: z.number().int().min(1).max(4_294_967_295),
    sha256: z.string().regex(/^[0-9a-f]{64}$/u),
  })
  .strict();
const capacity = z
  .object({
    recordCount: z.number().int().min(0).max(64),
    canonicalBytes: z
      .number()
      .int()
      .min(0)
      .max(2 * 1024 * 1024),
    warning: z.boolean(),
    countLimit: z.literal(64),
    canonicalByteLimit: z.literal(2 * 1024 * 1024),
  })
  .strict();
const bridgeState = z.enum(["ready", "unavailable"]);

export const taskTemplateCatalogSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: bridgeState,
    templates: z.array(summary).max(68),
    capacity: capacity.nullable(),
    diagnosticCode: diagnostic.nullable(),
  })
  .strict()
  .superRefine((value, context) => {
    if (
      value.state === "ready" &&
      (value.capacity === null || value.diagnosticCode !== null)
    )
      context.addIssue({
        code: "custom",
        message: "Ready catalog must have native capacity and no diagnostic",
      });
    if (
      value.state === "unavailable" &&
      (value.templates.length !== 0 ||
        value.capacity !== null ||
        value.diagnosticCode === null)
    )
      context.addIssue({
        code: "custom",
        message: "Unavailable catalog must be closed",
      });
  });
export const taskTemplateInspectionSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: bridgeState,
    template: detail.nullable(),
    mutationHandle: uuid.nullable(),
    diagnosticCode: diagnostic.nullable(),
  })
  .strict()
  .superRefine((value, context) => {
    if (
      value.state === "ready" &&
      (value.template === null ||
        value.diagnosticCode !== null ||
        value.mutationHandle === null)
    )
      context.addIssue({
        code: "custom",
        message: "Inspection authority is inconsistent",
      });
    if (
      value.state === "unavailable" &&
      (value.template !== null ||
        value.mutationHandle !== null ||
        value.diagnosticCode === null)
    )
      context.addIssue({
        code: "custom",
        message: "Unavailable inspection must be closed",
      });
  });
export const taskTemplatePreviewSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: bridgeState,
    reservationId: uuid.nullable(),
    expiresAtMs: z.number().int().nonnegative().nullable(),
    checklist: z
      .object({
        templateActive: z.literal(true),
        taskPlanAvailable: z.literal(true),
        exactDraftRequired: z.literal(true),
        confirmationRequired: z.literal(true),
      })
      .strict()
      .nullable(),
    diagnosticCode: diagnostic.nullable(),
  })
  .strict()
  .superRefine((value, context) => {
    const ready = value.state === "ready";
    if (
      ready !==
        (value.reservationId !== null &&
          value.expiresAtMs !== null &&
          value.checklist !== null) ||
      (ready && value.diagnosticCode !== null) ||
      (!ready && value.diagnosticCode === null)
    )
      context.addIssue({
        code: "custom",
        message: "Preview state is inconsistent",
      });
  });
export const taskTemplateApplicationOutcomeSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: bridgeState,
    applied: z.boolean(),
    cancelled: z.boolean(),
    diagnosticCode: diagnostic.nullable(),
  })
  .strict()
  .superRefine((value, context) => {
    if (value.applied && value.cancelled)
      context.addIssue({
        code: "custom",
        message: "Outcome cannot apply and cancel",
      });
    if ((value.state === "ready") !== (value.diagnosticCode === null))
      context.addIssue({
        code: "custom",
        message: "Outcome diagnostic is inconsistent",
      });
  });

export type TaskTemplateCatalogSnapshot = z.infer<
  typeof taskTemplateCatalogSchema
>;
export type TaskTemplateInspectionSnapshot = z.infer<
  typeof taskTemplateInspectionSchema
>;
export type TaskTemplatePreviewSnapshot = z.infer<
  typeof taskTemplatePreviewSchema
>;
export type TaskTemplateApplicationOutcome = z.infer<
  typeof taskTemplateApplicationOutcomeSchema
>;
