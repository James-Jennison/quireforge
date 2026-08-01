import { z } from "zod";

const uuid = z.string().uuid();
export const browserVerificationPrepareRequestSchema = z
  .object({
    projectId: uuid,
    taskId: uuid.nullable(),
    target: z.literal(
      "quireforge-fixture://verification/expected?assert=marker",
    ),
    assertion: z.literal("fixture-marker"),
  })
  .strict();
export const browserVerificationConfirmRequestSchema = z
  .object({
    attemptId: uuid,
    authorizationId: uuid,
  })
  .strict();
export const browserVerificationAttemptRequestSchema = z
  .object({ attemptId: uuid })
  .strict();
export const browserVerificationSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    fictionalLocalOnly: z.literal(true),
    readOnly: z.literal(true),
    adapter: z.literal("ephemeral-webkitgtk-fixture"),
    state: z.string().min(1).max(40),
    projectId: uuid.nullable(),
    taskId: uuid.nullable(),
    attemptId: uuid.nullable(),
    authorizationId: uuid.nullable(),
    target: z.string().max(160).nullable(),
    origin: z.string().max(120).nullable(),
    assertion: z.string().max(80).nullable(),
    requestDigest: z
      .string()
      .regex(/^[a-f0-9]{64}$/)
      .nullable(),
    expiresAtMs: z.number().int().nonnegative().nullable(),
    evidenceDigest: z
      .string()
      .regex(/^[a-f0-9]{64}$/)
      .nullable(),
    visibleText: z.string().max(240).nullable(),
    diagnostic: z.string().max(120).nullable(),
    auditState: z.string().max(240),
  })
  .strict();
export type BrowserVerificationSnapshot = z.infer<
  typeof browserVerificationSnapshotSchema
>;
