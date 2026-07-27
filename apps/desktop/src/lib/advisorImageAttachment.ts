import { z } from "zod";

const safeDisplayName = z
  .string()
  .min(1)
  .max(255)
  .refine(
    (value) =>
      !value.includes("/") &&
      !value.includes("\\") &&
      ![...value].some((character) => {
        const point = character.codePointAt(0) ?? 0;
        return point <= 0x1f || (point >= 0x202a && point <= 0x202e);
      }),
  );

export const advisorImageAttachmentSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum(["empty", "ready", "unavailable"]),
    attachment: z
      .object({
        attachmentId: z.string().uuid(),
        displayName: safeDisplayName,
        contentCategory: z.literal("image"),
        mediaType: z.enum(["png", "jpeg"]),
        byteSize: z
          .number()
          .int()
          .positive()
          .max(4 * 1024 * 1024),
        width: z.number().int().positive().max(16_384),
        height: z.number().int().positive().max(16_384),
        sha256: z.string().regex(/^[a-f0-9]{64}$/iu),
        projection: z
          .object({
            kind: z.literal("local-image"),
            width: z.number().int().positive().max(16_384),
            height: z.number().int().positive().max(16_384),
          })
          .strict(),
        disposal: z.literal("transient-memory-one-send"),
      })
      .strict()
      .nullable(),
    previewDataUrl: z
      .string()
      .regex(/^data:image\/(png|jpeg);base64,[A-Za-z0-9+/=]+$/u)
      .max(6 * 1024 * 1024)
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
        value.previewDataUrl !== null &&
        value.confirmationState === "confirmation-required" &&
        value.diagnosticCode === null) ||
      (value.state === "empty" &&
        value.attachment === null &&
        value.previewDataUrl === null &&
        value.confirmationState === null &&
        value.diagnosticCode === null) ||
      (value.state === "unavailable" &&
        value.attachment === null &&
        value.previewDataUrl === null &&
        value.confirmationState === null &&
        value.diagnosticCode !== null);
    if (!valid)
      context.addIssue({
        code: "custom",
        message: "Inconsistent Advisor image attachment state",
      });
    if (
      value.attachment &&
      (value.attachment.width !== value.attachment.projection.width ||
        value.attachment.height !== value.attachment.projection.height)
    )
      context.addIssue({
        code: "custom",
        message: "Inconsistent Advisor image dimensions",
      });
  });

export type AdvisorImageAttachmentSnapshot = z.infer<
  typeof advisorImageAttachmentSnapshotSchema
>;
export const scaffoldAdvisorImageAttachment: AdvisorImageAttachmentSnapshot = {
  schemaVersion: 1,
  state: "empty",
  attachment: null,
  previewDataUrl: null,
  confirmationState: null,
  diagnosticCode: null,
};
