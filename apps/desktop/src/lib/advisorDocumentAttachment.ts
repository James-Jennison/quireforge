import { z } from "zod";

export const advisorDocumentAttachmentSnapshotSchema = z
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
        contentCategory: z.literal("document"),
        mediaType: z.literal("pdf"),
        byteSize: z
          .number()
          .int()
          .positive()
          .max(8 * 1024 * 1024),
        sha256: z.string().regex(/^[a-f0-9]{64}$/iu),
        projection: z
          .object({
            kind: z.literal("pdf-plain-text-v1"),
            schemaVersion: z.literal(1),
            pageCount: z.number().int().nonnegative().max(200),
            processedPageCount: z.number().int().nonnegative().max(200),
            projectedByteSize: z
              .number()
              .int()
              .nonnegative()
              .max(256 * 1024),
            outlineEntryCount: z.number().int().nonnegative().max(128),
            truncated: z.boolean(),
            warnings: z.array(z.string().max(64)).max(8),
          })
          .strict(),
        disposal: z.literal("transient-memory-one-send"),
      })
      .strict()
      .nullable(),
    confirmationState: z
      .enum(["confirmation-required", "confirmed-for-single-send"])
      .nullable(),
    diagnosticCode: z
      .enum([
        "invalid-request",
        "unsupported-type",
        "file-too-large",
        "invalid-content",
        "unsafe-name",
        "read-failed",
        "encrypted",
        "active-content",
        "page-limit-exceeded",
        "parse-budget-exceeded",
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
        value.attachment &&
        value.confirmationState === "confirmation-required" &&
        value.diagnosticCode === null) ||
      (value.state === "empty" &&
        !value.attachment &&
        value.confirmationState === null &&
        value.diagnosticCode === null) ||
      (value.state === "unavailable" &&
        !value.attachment &&
        value.confirmationState === null &&
        value.diagnosticCode !== null);
    if (!valid)
      context.addIssue({
        code: "custom",
        message: "Inconsistent Advisor document attachment state",
      });
  });
export type AdvisorDocumentAttachmentSnapshot = z.infer<
  typeof advisorDocumentAttachmentSnapshotSchema
>;
export const scaffoldAdvisorDocumentAttachment: AdvisorDocumentAttachmentSnapshot =
  {
    schemaVersion: 1,
    state: "empty",
    attachment: null,
    confirmationState: null,
    diagnosticCode: null,
  };
