import { z } from "zod";

const id = z
  .string()
  .uuid()
  .regex(/^[0-9a-f-]+$/u);
const sha256 = z.string().regex(/^[a-f0-9]{64}$/u);
const classSchema = z.enum([
  "manual-text",
  "local-text-file",
  "reviewed-artifact-text",
]);
const diagnostic = z.enum([
  "project-unavailable",
  "task-unavailable",
  "project-task-mismatch",
  "source-class-unsupported",
  "invalid-utf8",
  "file-not-regular",
  "symlink-rejected",
  "file-changed-during-intake",
  "source-oversized",
  "too-many-lines",
  "artifact-unavailable",
  "artifact-ineligible",
  "preparation-expired",
  "preparation-missing",
  "preparation-replayed",
  "confirmation-mismatch",
  "admission-ambiguous",
  "source-unavailable",
  "source-already-deleted",
  "deletion-ambiguous",
  "private-storage-failure",
  "recovery-cleanup-failure",
]);
const title = z.string().trim().min(1).max(240);

export const durableSourceManualPrepareRequestSchema = z
  .object({
    projectId: id,
    taskId: id.nullable().optional(),
    title,
    text: z.string().max(128 * 1024),
  })
  .strict();
export const durableSourceFilePrepareRequestSchema = z
  .object({ projectId: id, taskId: id.nullable().optional(), title })
  .strict();
export const durableSourceArtifactPrepareRequestSchema = z
  .object({
    projectId: id,
    taskId: id.nullable().optional(),
    title,
    artifactId: id,
    artifactSha256: sha256,
  })
  .strict();
export const durableSourceConfirmRequestSchema = z
  .object({ preparationId: id, nonce: id, sha256 })
  .strict();
export const durableSourceProjectRequestSchema = z
  .object({ projectId: id })
  .strict();
export const durableSourceReadRequestSchema = z
  .object({ sourceId: id })
  .strict();
export const durableSourceDeleteConfirmRequestSchema = z
  .object({ preparationId: id, nonce: id, sourceId: id })
  .strict();

export const durableSourcePreparationSchema = z
  .object({
    schemaVersion: z.literal(1),
    preparationId: id.or(z.literal("")),
    nonce: id.or(z.literal("")),
    expiresAtMs: z.number().int().nonnegative(),
    projectId: id.or(z.literal("")),
    taskId: id.nullable(),
    sourceClass: classSchema,
    title: z.string(),
    originDisplay: z.string().max(255).nullable(),
    sha256: sha256.or(z.literal("")),
    byteSize: z
      .number()
      .int()
      .nonnegative()
      .max(128 * 1024),
    lineCount: z.number().int().nonnegative().max(2000),
    preview: z.string().max(4096),
    diagnosticCode: diagnostic.nullable(),
  })
  .strict();
export const durableSourceSummarySchema = z
  .object({
    sourceId: id,
    projectId: id,
    taskId: id.nullable(),
    sourceClass: classSchema,
    title,
    originDisplay: z.string().max(255).nullable(),
    byteSize: z
      .number()
      .int()
      .nonnegative()
      .max(128 * 1024),
    lineCount: z.number().int().nonnegative().max(2000),
    sha256,
    state: z.enum(["active", "deleted"]),
    createdAtMs: z.number().int().nonnegative(),
  })
  .strict();
export const durableSourceSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    sources: z.array(durableSourceSummarySchema).max(100),
    diagnosticCode: diagnostic.nullable(),
  })
  .strict();

export type DurableSourceManualPrepareRequest = z.infer<
  typeof durableSourceManualPrepareRequestSchema
>;
export type DurableSourceFilePrepareRequest = z.infer<
  typeof durableSourceFilePrepareRequestSchema
>;
export type DurableSourceArtifactPrepareRequest = z.infer<
  typeof durableSourceArtifactPrepareRequestSchema
>;
export type DurableSourceConfirmRequest = z.infer<
  typeof durableSourceConfirmRequestSchema
>;
export type DurableSourceProjectRequest = z.infer<
  typeof durableSourceProjectRequestSchema
>;
export type DurableSourceReadRequest = z.infer<
  typeof durableSourceReadRequestSchema
>;
export type DurableSourceDeleteConfirmRequest = z.infer<
  typeof durableSourceDeleteConfirmRequestSchema
>;
export type DurableSourcePreparation = z.infer<
  typeof durableSourcePreparationSchema
>;
export type DurableSourceSummary = z.infer<typeof durableSourceSummarySchema>;
export type DurableSourceSnapshot = z.infer<typeof durableSourceSnapshotSchema>;
