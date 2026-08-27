import { z } from "zod";

const id = z
  .string()
  .uuid()
  .regex(/^[0-9a-f-]+$/u);
export const objectiveAuthorityLaneSchema = z.enum([
  "work-with-code",
  "browser-workspace",
  "browser-observation",
  "connector-read",
  "scheduled-work",
  "connector-mutation",
  "provider-inference",
  "computer-use",
]);
const state = z.enum(["draft", "active", "revoked", "expired"]);

export const objectiveAuthorityProjectRequestSchema = z
  .object({ projectId: id })
  .strict();
export const objectiveAuthorityCreateRequestSchema = z
  .object({
    projectId: id,
    title: z.string().trim().min(1).max(240),
    objective: z.string().trim().min(1).max(8192),
    allowedLanes: z.array(objectiveAuthorityLaneSchema).min(1).max(8),
    confirmationRequiredLanes: z.array(objectiveAuthorityLaneSchema).max(8),
    expiresInMinutes: z.number().int().min(1).max(10_080),
  })
  .strict()
  .superRefine((value, context) => {
    if (new Set(value.allowedLanes).size !== value.allowedLanes.length)
      context.addIssue({
        code: "custom",
        message: "allowed lanes must be unique",
      });
    if (
      new Set(value.confirmationRequiredLanes).size !==
      value.confirmationRequiredLanes.length
    )
      context.addIssue({
        code: "custom",
        message: "confirmation lanes must be unique",
      });
    if (
      value.confirmationRequiredLanes.some(
        (lane) => !value.allowedLanes.includes(lane),
      )
    )
      context.addIssue({
        code: "custom",
        message: "confirmation lanes must be allowed",
      });
  });
export const objectiveAuthorityDecisionRequestSchema = z
  .object({ objectiveId: id })
  .strict();
export const objectiveAuthoritySnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    objectives: z
      .array(
        z
          .object({
            id,
            projectId: id,
            title: z.string().max(240),
            objective: z.string().max(8192),
            allowedLanes: z.array(objectiveAuthorityLaneSchema).max(8),
            confirmationRequiredLanes: z
              .array(objectiveAuthorityLaneSchema)
              .max(8),
            state,
            createdAtMs: z.number().int().nonnegative(),
            activatedAtMs: z.number().int().nonnegative().nullable(),
            expiresAtMs: z.number().int().nonnegative(),
            revokedAtMs: z.number().int().nonnegative().nullable(),
          })
          .strict(),
      )
      .max(128),
    diagnosticCode: z.string().max(120).nullable(),
  })
  .strict();

export type ObjectiveAuthoritySnapshot = z.infer<
  typeof objectiveAuthoritySnapshotSchema
>;
