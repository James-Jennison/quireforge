import { z } from "zod";
const uuid = z.string().uuid();
const digest = z.string().regex(/^[a-f0-9]{64}$/u);
export const contextAssemblyPrepareRequestSchema = z
  .object({
    projectId: uuid,
    taskId: uuid.nullable(),
    userInstruction: z.string().max(8192),
    durableSourceIds: z.array(uuid).max(16),
    selectedPlanId: uuid.nullable().optional(),
    reviewEvidenceIds: z.array(uuid).max(16).optional(),
    includeScopeMetadata: z.boolean().optional(),
  })
  .strict();
export const contextAssemblyConfirmRequestSchema = z
  .object({ bundleId: uuid, authorizationId: uuid, bundleDigest: digest })
  .strict();
export const contextAssemblyAttemptRequestSchema = z
  .object({ bundleId: uuid })
  .strict();
export const contextAssemblySnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    fictionalLocalOnly: z.literal(true),
    sink: z.literal("fictional-local-context-sink-v1"),
    state: z.string().min(1).max(40),
    projectId: uuid.nullable(),
    taskId: uuid.nullable(),
    bundleId: uuid.nullable(),
    authorizationId: uuid.nullable(),
    bundleDigest: digest.nullable(),
    expiresAtMs: z.number().int().nonnegative().nullable(),
    items: z
      .array(
        z
          .object({
            ordinal: z.number().int().min(0).max(15),
            sourceClass: z.string().max(80),
            provenance: z.string().max(120),
            byteSize: z.number().int().nonnegative(),
            digest,
            redactionCount: z.number().int().nonnegative(),
            truncated: z.boolean(),
          })
          .strict(),
      )
      .max(16),
    totalBytes: z.number().int().nonnegative(),
    estimatedTokens: z.number().int().nonnegative(),
    exclusions: z.array(z.string().max(120)).max(16),
    auditState: z.string().max(240),
    diagnostic: z.string().max(120).nullable(),
  })
  .strict();
export type ContextAssemblySnapshot = z.infer<
  typeof contextAssemblySnapshotSchema
>;

export const localRuntimeSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    localOnly: z.literal(true),
    state: z.enum(["completed", "failed", "cancelled"]),
    output: z
      .string()
      .max(16 * 1024)
      .nullable(),
    diagnostic: z.string().max(120).nullable(),
    inputTokenLimit: z.literal(4096),
    outputTokenLimit: z.literal(512),
    deadlineSeconds: z.literal(60),
  })
  .strict();
export type LocalRuntimeSnapshot = z.infer<typeof localRuntimeSnapshotSchema>;
