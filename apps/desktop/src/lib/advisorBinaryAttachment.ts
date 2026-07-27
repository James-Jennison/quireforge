import { z } from "zod";

export const advisorBinaryAttachmentSnapshotSchema = z
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
        contentCategory: z.literal("static-binary"),
        mediaType: z.literal("elf"),
        byteSize: z
          .number()
          .int()
          .positive()
          .max(32 * 1024 * 1024),
        sha256: z.string().regex(/^[a-f0-9]{64}$/iu),
        projection: z
          .object({
            kind: z.literal("static-binary-manifest-v1"),
            schemaVersion: z.literal(1),
            elfClass: z.enum(["elf32", "elf64"]),
            endianness: z.enum(["little", "big"]),
            fileType: z.enum(["relocatable", "executable", "shared-object"]),
            machine: z.number().int().nonnegative(),
            osAbi: z.number().int().nonnegative(),
            programHeaderCount: z.number().int().nonnegative().max(256),
            sectionHeaderCount: z.number().int().nonnegative().max(1024),
            dynamicSectionPresent: z.boolean(),
            dynamicEntryCount: z.number().int().nonnegative().max(256),
            manifestByteSize: z
              .number()
              .int()
              .nonnegative()
              .max(8 * 1024),
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
        "invalid-signature",
        "source-too-large",
        "source-unavailable",
        "source-changed",
        "malformed-or-unsupported-elf",
        "unsupported-elf-layout",
        "metadata-limit-exceeded",
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
        value.diagnosticCode === null) ||
      (value.state === "empty" &&
        value.attachment === null &&
        value.confirmationState === null &&
        value.diagnosticCode === null) ||
      (value.state === "unavailable" &&
        value.attachment === null &&
        value.confirmationState === null &&
        value.diagnosticCode !== null);
    if (!valid)
      context.addIssue({
        code: "custom",
        message: "Inconsistent Advisor static-binary attachment state",
      });
  });

export type AdvisorBinaryAttachmentSnapshot = z.infer<
  typeof advisorBinaryAttachmentSnapshotSchema
>;
export const scaffoldAdvisorBinaryAttachment: AdvisorBinaryAttachmentSnapshot =
  {
    schemaVersion: 1,
    state: "empty",
    attachment: null,
    confirmationState: null,
    diagnosticCode: null,
  };
