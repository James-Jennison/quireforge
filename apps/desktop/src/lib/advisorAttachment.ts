import { z } from "zod";

/** Closed registry: only text-data is active in F1. Other categories are
 * reserved names, not supported file paths or upload capabilities. */
export const advisorContentCategorySchema = z.enum([
  "text-data",
  "image",
  "document",
  "archive",
  "static-binary",
]);
export const advisorContentTypeSchema = z.enum([
  "text",
  "markdown",
  "csv",
  "json",
  "python",
]);
export const advisorContentProjectionKindSchema = z.enum([
  "normalized-utf8-text",
]);
export const advisorContentDisposalSchema = z.enum([
  "transient-memory-one-send",
]);
export const advisorContentConfirmationStateSchema = z.enum([
  "confirmation-required",
  "confirmed-for-single-send",
]);
export const advisorTextAttachmentKindSchema = advisorContentTypeSchema;
const manifestSchema = z
  .object({
    attachmentId: z.string().uuid(),
    displayName: z
      .string()
      .min(1)
      .max(255)
      .refine((value) => !hasUnsafeNameCharacter(value)),
    contentCategory: z.literal("text-data"),
    contentType: advisorContentTypeSchema,
    byteSize: z
      .number()
      .int()
      .positive()
      .max(512 * 1024),
    sha256: z.string().regex(/^[a-f0-9]{64}$/iu),
    projection: z
      .object({
        kind: advisorContentProjectionKindSchema,
        normalizedByteSize: z
          .number()
          .int()
          .positive()
          .max(512 * 1024),
      })
      .strict(),
    disposal: advisorContentDisposalSchema,
  })
  .strict();

export const advisorTextAttachmentSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum(["empty", "ready", "unavailable"]),
    attachment: manifestSchema.nullable(),
    confirmationState: advisorContentConfirmationStateSchema.nullable(),
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
        "save-cancelled",
        "save-failed",
        "file-exists",
      ])
      .nullable(),
  })
  .strict()
  .superRefine((value, context) => {
    const valid =
      (value.state === "ready" &&
        value.attachment &&
        value.confirmationState === "confirmation-required" &&
        !value.diagnosticCode) ||
      (value.state === "empty" &&
        !value.attachment &&
        value.confirmationState === null &&
        !value.diagnosticCode) ||
      (value.state === "unavailable" &&
        !value.attachment &&
        value.confirmationState === null &&
        value.diagnosticCode);
    if (!valid)
      context.addIssue({
        code: "custom",
        message: "Inconsistent Advisor attachment state",
      });
  });

export const advisorTextExportRequestSchema = z
  .object({
    suggestedName: z
      .string()
      .min(5)
      .max(255)
      .refine((value) => !hasUnsafeNameCharacter(value)),
    contentType: advisorContentTypeSchema,
    content: z
      .string()
      .min(1)
      .max(512 * 1024)
      .refine((value) => !value.includes("\0")),
  })
  .strict();

export type AdvisorTextAttachmentSnapshot = z.infer<
  typeof advisorTextAttachmentSnapshotSchema
>;
export type AdvisorTextExportRequest = z.infer<
  typeof advisorTextExportRequestSchema
>;
export const scaffoldAdvisorTextAttachment: AdvisorTextAttachmentSnapshot = {
  schemaVersion: 1,
  state: "empty",
  attachment: null,
  confirmationState: null,
  diagnosticCode: null,
};

export interface AdvisorTextExportCandidate {
  label: string;
  suggestedName: string;
  contentType: z.infer<typeof advisorContentTypeSchema>;
  content: string;
}

const languageKinds: Record<string, AdvisorTextExportCandidate["contentType"]> =
  {
    csv: "csv",
    json: "json",
    md: "markdown",
    markdown: "markdown",
    py: "python",
    python: "python",
    text: "text",
    txt: "text",
  };

/** Extract only user-visible fenced text blocks. The fallback is the complete
 * visible reply, so export never reads a transcript outside the active UI. */
export function advisorTextExportCandidates(
  reply: string,
): AdvisorTextExportCandidate[] {
  const candidates: AdvisorTextExportCandidate[] = [
    {
      label: "Complete reply (.txt)",
      suggestedName: "advisor-response.txt",
      contentType: "text",
      content: reply,
    },
  ];
  for (const match of reply.matchAll(/```([A-Za-z0-9_-]+)?\n([\s\S]*?)```/gu)) {
    const contentType =
      languageKinds[(match[1] ?? "text").toLowerCase()] ?? "text";
    const extension =
      contentType === "markdown"
        ? "md"
        : contentType === "python"
          ? "py"
          : contentType;
    const content = match[2] ?? "";
    if (content && content.length <= 512 * 1024)
      candidates.push({
        label: `Code/data block (.${extension})`,
        suggestedName: `advisor-output.${extension}`,
        contentType,
        content,
      });
  }
  return candidates;
}

function hasUnsafeNameCharacter(value: string): boolean {
  return (
    value.includes("/") ||
    value.includes("\\") ||
    [...value].some((character) => {
      const point = character.codePointAt(0) ?? 0;
      return (
        point <= 0x1f ||
        (point >= 0x202a && point <= 0x202e) ||
        (point >= 0x2066 && point <= 0x2069)
      );
    })
  );
}
