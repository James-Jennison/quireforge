import { z } from "zod";

export const generatedArtifactClassSchema = z.enum([
  "text",
  "markdown",
  "json",
  "csv",
  "python",
]);
export const generatedArtifactSourceKindSchema = z.enum([
  "visible-completed-reply",
  "visible-fenced-block",
  "explicit-review-promotion",
]);
export const generatedArtifactStateSchema = z.enum([
  "ready",
  "saving",
  "expired",
  "saved",
]);
const sha256Schema = z.string().regex(/^[a-f0-9]{64}$/u);
const safeName = (value: string) =>
  !value.includes("/") &&
  !value.includes("\\") &&
  [...value].every((character) => {
    const code = character.codePointAt(0) ?? 0;
    return (
      code > 0x1f &&
      !(code >= 0x202a && code <= 0x202e) &&
      !(code >= 0x2066 && code <= 0x2069)
    );
  });

export const generatedArtifactManifestSchema = z
  .object({
    schemaVersion: z.literal(1),
    artifactId: z.string().uuid(),
    class: generatedArtifactClassSchema,
    mimeType: z.string().min(1),
    sourceKind: generatedArtifactSourceKindSchema,
    displayLabel: z.string().min(1).max(120).refine(safeName),
    suggestedFilename: z.string().min(1).max(120).refine(safeName),
    byteSize: z
      .number()
      .int()
      .min(1)
      .max(512 * 1024),
    sha256: sha256Schema,
    createdAt: z.number().int().nonnegative(),
    expiresAt: z.number().int().positive(),
    state: generatedArtifactStateSchema,
    disposal: z.literal("transient-memory-one-successful-save"),
  })
  .strict();
export const generatedArtifactSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    artifacts: z.array(generatedArtifactManifestSchema).max(5),
    diagnosticCode: z
      .enum([
        "invalid-request",
        "invalid-content",
        "invalid-json",
        "invalid-csv",
        "unsafe-name",
        "artifact-not-found",
        "manifest-mismatch",
        "artifact-expired",
        "already-saving",
        "capacity-exceeded",
        "aggregate-exceeded",
        "save-cancelled",
        "save-failed",
        "file-exists",
        "cleanup-failed",
      ])
      .nullable(),
  })
  .strict();
export const generatedArtifactCreateRequestSchema = z
  .object({
    class: generatedArtifactClassSchema,
    sourceKind: generatedArtifactSourceKindSchema,
    displayLabel: z.string().min(1).max(120).refine(safeName),
    suggestedFilename: z.string().min(1).max(120).refine(safeName),
    content: z
      .string()
      .min(1)
      .max(512 * 1024),
  })
  .strict();
export const generatedArtifactClaimRequestSchema = z
  .object({ artifactId: z.string().uuid(), manifestSha256: sha256Schema })
  .strict();
export const generatedArtifactPreviewSchema = z
  .object({
    schemaVersion: z.literal(1),
    artifactId: z.string().uuid(),
    sha256: sha256Schema,
    text: z.string().max(512 * 1024),
  })
  .strict();
export const generatedArtifactReceiptSchema = z
  .object({
    schemaVersion: z.literal(1),
    artifactId: z.string().uuid(),
    class: generatedArtifactClassSchema,
    filename: z.string().min(1).max(120).refine(safeName),
    byteSize: z
      .number()
      .int()
      .min(1)
      .max(512 * 1024),
    sha256: sha256Schema,
    savedAt: z.number().int().nonnegative(),
  })
  .strict();
export type GeneratedArtifactManifest = z.infer<
  typeof generatedArtifactManifestSchema
>;
export type GeneratedArtifactSnapshot = z.infer<
  typeof generatedArtifactSnapshotSchema
>;
export type GeneratedArtifactCreateRequest = z.infer<
  typeof generatedArtifactCreateRequestSchema
>;
export type GeneratedArtifactClaimRequest = z.infer<
  typeof generatedArtifactClaimRequestSchema
>;
export type GeneratedArtifactPreview = z.infer<
  typeof generatedArtifactPreviewSchema
>;
export type GeneratedArtifactReceipt = z.infer<
  typeof generatedArtifactReceiptSchema
>;
