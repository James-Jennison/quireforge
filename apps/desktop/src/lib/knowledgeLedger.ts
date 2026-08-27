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
export const knowledgeLedgerSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    records: z
      .array(
        z
          .object({
            id,
            projectId: id,
            taskId: id.nullable(),
            kind,
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
    diagnosticCode: z.string().max(120).nullable(),
  })
  .strict();
export type KnowledgeLedgerSnapshot = z.infer<
  typeof knowledgeLedgerSnapshotSchema
>;
