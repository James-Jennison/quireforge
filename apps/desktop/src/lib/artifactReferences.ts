import { z } from "zod";

const id = z
  .string()
  .uuid()
  .regex(/^[0-9a-f-]+$/u);
const sha256 = z.string().regex(/^[a-f0-9]{64}$/u);
const diagnostic = z.enum([
  "project-unavailable",
  "task-unavailable",
  "project-task-mismatch",
  "artifact-unavailable",
  "artifact-mismatch",
  "preparation-expired",
  "preparation-missing",
  "confirmation-mismatch",
  "reference-unavailable",
  "reference-already-deleted",
  "private-storage-failure",
]);
export const artifactReferencePrepareRequestSchema = z
  .object({
    projectId: id,
    taskId: id.nullable().optional(),
    artifactId: id,
    artifactSha256: sha256,
  })
  .strict();
export const artifactReferenceConfirmRequestSchema = z
  .object({ preparationId: id, nonce: id, artifactSha256: sha256 })
  .strict();
export const artifactReferenceProjectRequestSchema = z
  .object({ projectId: id })
  .strict();
export const artifactReferenceDeletePrepareRequestSchema = z
  .object({ referenceId: id })
  .strict();
export const artifactReferenceDeleteConfirmRequestSchema = z
  .object({ preparationId: id, nonce: id, referenceId: id })
  .strict();
export const artifactReferencePreparationSchema = z
  .object({
    schemaVersion: z.literal(1),
    preparationId: id.or(z.literal("")),
    nonce: id.or(z.literal("")),
    expiresAtMs: z.number().int().nonnegative(),
    referenceId: id.nullable(),
    projectId: id.or(z.literal("")),
    taskId: id.nullable(),
    artifactId: id.or(z.literal("")),
    artifactSha256: sha256.or(z.literal("")),
    artifactClass: z
      .enum(["text", "markdown", "json", "csv", "python"])
      .or(z.literal("")),
    displayLabel: z.string().max(120),
    diagnosticCode: diagnostic.nullable(),
  })
  .strict();
export const artifactReferenceSummarySchema = z
  .object({
    referenceId: id,
    projectId: id,
    taskId: id.nullable(),
    artifactId: id,
    artifactSha256: sha256,
    artifactClass: z.enum(["text", "markdown", "json", "csv", "python"]),
    displayLabel: z.string().min(1).max(120),
    state: z.enum(["active", "deleted"]),
    availability: z.enum(["live", "unavailable"]),
    createdAtMs: z.number().int().nonnegative(),
  })
  .strict();
export const artifactReferenceSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    references: z.array(artifactReferenceSummarySchema).max(100),
    diagnosticCode: diagnostic.nullable(),
  })
  .strict();
export type ArtifactReferencePreparation = z.infer<
  typeof artifactReferencePreparationSchema
>;
export type ArtifactReferenceSnapshot = z.infer<
  typeof artifactReferenceSnapshotSchema
>;
