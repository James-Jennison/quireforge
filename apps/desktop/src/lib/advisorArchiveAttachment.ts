import { z } from "zod";

const archiveEntrySchema = z
  .object({
    name: z
      .string()
      .min(1)
      .max(512)
      .refine((value) => !value.includes("/") || !value.startsWith("/")),
    kind: z.enum(["file", "directory"]),
    compressedSize: z.number().int().nonnegative(),
    declaredUncompressedSize: z.number().int().nonnegative(),
    nestedArchiveLike: z.boolean(),
  })
  .strict();

export const advisorArchiveAttachmentSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum(["empty", "ready", "unavailable"]),
    attachment: z
      .object({
        attachmentId: z.string().uuid(),
        displayName: z
          .string()
          .min(1)
          .max(255)
          .refine((value) => !value.includes("/") && !value.includes("\\")),
        contentCategory: z.literal("archive"),
        mediaType: z.literal("zip"),
        byteSize: z
          .number()
          .int()
          .positive()
          .max(32 * 1024 * 1024),
        sha256: z.string().regex(/^[a-f0-9]{64}$/iu),
        projection: z
          .object({
            kind: z.literal("archive-manifest-v1"),
            schemaVersion: z.literal(1),
            discoveredEntryCount: z.number().int().nonnegative().max(10_000),
            includedEntryCount: z.number().int().nonnegative().max(2_000),
            omittedEntryCount: z.number().int().nonnegative().max(10_000),
            declaredAggregateUncompressedBytes: z.number().int().nonnegative(),
            manifestByteSize: z
              .number()
              .int()
              .nonnegative()
              .max(256 * 1024),
            truncated: z.boolean(),
            warnings: z.array(z.literal("manifest-truncated")).max(1),
          })
          .strict(),
        disposal: z.literal("transient-memory-one-send"),
      })
      .strict()
      .nullable(),
    entries: z.array(archiveEntrySchema).max(2_000),
    confirmationState: z
      .enum(["confirmation-required", "confirmed-for-single-send"])
      .nullable(),
    diagnosticCode: z
      .enum([
        "invalid-request",
        "unsupported-type",
        "invalid-signature",
        "source-too-large",
        "source-unavailable",
        "source-changed",
        "encrypted-archive",
        "malformed-or-unsupported-archive",
        "entry-limit-exceeded",
        "manifest-size-limit-exceeded",
        "expanded-size-limit-exceeded",
        "compression-ratio-limit-exceeded",
        "unsafe-entry-path",
        "duplicate-entry",
        "symlink-entry",
        "unsupported-entry-kind",
        "unsafe-name",
        "read-failed",
        "attachment-not-found",
        "attachment-expired",
        "manifest-mismatch",
      ])
      .nullable(),
  })
  .strict()
  .superRefine((value, context) => {
    const valid =
      (value.state === "ready" &&
        value.attachment !== null &&
        value.confirmationState === "confirmation-required" &&
        value.diagnosticCode === null &&
        value.entries.length ===
          value.attachment.projection.includedEntryCount &&
        value.attachment.projection.discoveredEntryCount ===
          value.attachment.projection.includedEntryCount +
            value.attachment.projection.omittedEntryCount) ||
      (value.state === "empty" &&
        value.attachment === null &&
        value.entries.length === 0 &&
        value.confirmationState === null &&
        value.diagnosticCode === null) ||
      (value.state === "unavailable" &&
        value.attachment === null &&
        value.entries.length === 0 &&
        value.confirmationState === null &&
        value.diagnosticCode !== null);
    if (!valid)
      context.addIssue({
        code: "custom",
        message: "Inconsistent Advisor archive attachment state",
      });
    if (
      value.attachment &&
      (value.attachment.projection.truncated !==
        value.attachment.projection.warnings.includes("manifest-truncated") ||
        (!value.attachment.projection.truncated &&
          value.attachment.projection.omittedEntryCount > 0))
    )
      context.addIssue({
        code: "custom",
        message: "Inconsistent Advisor archive manifest truncation",
      });
  });

export type AdvisorArchiveAttachmentSnapshot = z.infer<
  typeof advisorArchiveAttachmentSnapshotSchema
>;

export const scaffoldAdvisorArchiveAttachment: AdvisorArchiveAttachmentSnapshot =
  {
    schemaVersion: 1,
    state: "empty",
    attachment: null,
    entries: [],
    confirmationState: null,
    diagnosticCode: null,
  };
