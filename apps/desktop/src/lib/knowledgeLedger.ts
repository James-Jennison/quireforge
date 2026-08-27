import { z } from "zod";

const id = z
  .string()
  .uuid()
  .regex(/^[0-9a-f-]+$/u);
const kind = z.enum([
  "owner-decision",
  "constraint",
  "observed-fact",
  "verified-implementation",
  "agent-claim",
  "assumption",
  "recommendation",
  "rejected-approach",
  "unresolved-question",
]);
const status = z.enum([
  "proposed",
  "pending-owner-binding",
  "recorded",
  "active",
  "validated",
  "disproven",
  "resolved",
  "superseded",
  "retired",
]);
const provenance = z.enum(["owner", "agent", "system"]);
const evidenceKind = z.enum([
  "m48-artifact-reference",
  "task-evidence",
  "package-validation",
  "owner-trial",
]);
const ownerTrialKind = z.enum(["functional", "visual", "device"]);
const ownerTrialResult = z.enum(["passed", "failed", "inconclusive"]);
const evidenceConclusion = z.enum(["supports", "contradicts", "inconclusive"]);
export const knowledgeLedgerProjectRequestSchema = z
  .object({ projectId: id })
  .strict();
export const knowledgeLedgerCreateRequestSchema = z
  .object({
    projectId: id,
    taskId: id.nullable().optional(),
    kind,
    title: z.string().min(1).max(240),
    body: z.string().min(1).max(8192),
    supersedesId: id.nullable().optional(),
  })
  .strict();
export const knowledgeLedgerBindingRequestSchema = z
  .object({ recordId: id })
  .strict();
export const knowledgeLedgerTransitionRequestSchema = z
  .object({ recordId: id, status })
  .strict();
export const knowledgeEvidenceLinkCreateRequestSchema = z
  .object({
    recordId: id,
    kind: evidenceKind,
    sourceId: id.nullable().optional(),
    ownerTrialKind: ownerTrialKind.nullable().optional(),
    ownerTrialResult: ownerTrialResult.nullable().optional(),
  })
  .strict()
  .superRefine((value, context) => {
    const ownerTrial = value.kind === "owner-trial";
    if (ownerTrial !== (value.sourceId == null))
      context.addIssue({
        code: z.ZodIssueCode.custom,
        message: "sourceId must be absent only for owner trials",
      });
    if (
      ownerTrial !==
      (value.ownerTrialKind != null && value.ownerTrialResult != null)
    )
      context.addIssue({
        code: z.ZodIssueCode.custom,
        message: "owner trial receipts require both bounded fields",
      });
  });
export const knowledgeEvidenceConclusionRequestSchema = z
  .object({ linkId: id, conclusion: evidenceConclusion })
  .strict();
export const knowledgeLedgerSnapshotSchema = z
  .object({
    schemaVersion: z.literal(2),
    records: z
      .array(
        z
          .object({
            id,
            projectId: id,
            taskId: id.nullable(),
            kind,
            provenance,
            status,
            title: z.string().max(240),
            body: z.string().max(8192),
            supersedesId: id.nullable(),
            createdAtMs: z.number().int().nonnegative(),
            updatedAtMs: z.number().int().nonnegative(),
          })
          .strict(),
      )
      .max(128),
    evidenceLinks: z
      .array(
        z
          .object({
            id,
            recordId: id,
            kind: evidenceKind,
            sourceClass: z.string().min(1).max(80),
            sourceId: id,
            sourceDigest: z.string().regex(/^[0-9a-f]{64}$/u),
            ownerTrialKind: ownerTrialKind.nullable(),
            ownerTrialResult: ownerTrialResult.nullable(),
            createdAtMs: z.number().int().nonnegative(),
          })
          .strict(),
      )
      .max(256),
    evidenceConclusions: z
      .array(
        z
          .object({
            id,
            linkId: id,
            conclusion: evidenceConclusion,
            createdAtMs: z.number().int().nonnegative(),
          })
          .strict(),
      )
      .max(256),
    diagnosticCode: z.string().max(120).nullable(),
  })
  .strict();
export type KnowledgeLedgerSnapshot = z.infer<
  typeof knowledgeLedgerSnapshotSchema
>;
