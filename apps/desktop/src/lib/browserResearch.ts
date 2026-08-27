import { z } from "zod";

const uuid = z.string().uuid();
const target = z.literal("https://google.com/");
const origin = z.literal("https://google.com");
export const browserResearchPrepareRequestSchema = z
  .object({
    projectId: uuid,
    taskId: uuid.nullable(),
    target,
    origin,
    observationLimit: z.number().int().min(1).max(2048),
  })
  .strict();
export const browserResearchConfirmRequestSchema = z
  .object({ attemptId: uuid, authorizationId: uuid })
  .strict();
export const browserResearchAttemptRequestSchema = z
  .object({ attemptId: uuid })
  .strict();
export const browserResearchSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    isolated: z.literal(true),
    readOnly: z.literal(true),
    adapter: z.literal("ephemeral-webkitgtk-research"),
    state: z.string().min(1).max(40),
    projectId: uuid.nullable(),
    taskId: uuid.nullable(),
    attemptId: uuid.nullable(),
    authorizationId: uuid.nullable(),
    target: target.nullable(),
    origin: origin.nullable(),
    requestDigest: z
      .string()
      .regex(/^[a-f0-9]{64}$/)
      .nullable(),
    expiresAtMs: z.number().int().nonnegative().nullable(),
    observationLimit: z.number().int().min(1).max(2048).nullable(),
    observedAtMs: z.number().int().nonnegative().nullable(),
    contentDigest: z
      .string()
      .regex(/^[a-f0-9]{64}$/)
      .nullable(),
    observedBytes: z.number().int().min(0).max(2048).nullable(),
    diagnostic: z.string().max(120).nullable(),
    auditState: z.string().max(240),
  })
  .strict();
export type BrowserResearchSnapshot = z.infer<
  typeof browserResearchSnapshotSchema
>;
