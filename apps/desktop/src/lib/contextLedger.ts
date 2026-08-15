import { z } from "zod";

const id = z
  .string()
  .uuid()
  .regex(/^[0-9a-f-]+$/u);
const digest = z.string().regex(/^[a-f0-9]{64}$/u);

export const contextLedgerSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    projectId: id,
    entries: z
      .array(
        z
          .object({
            recordKind: z.enum([
              "context-bundle",
              "durable-source",
              "artifact-reference",
              "connector-operation",
              "browser-verification",
            ]),
            recordId: id,
            projectId: id,
            taskId: id.nullable(),
            state: z.string().max(40),
            bundleDigest: digest,
            itemCount: z.number().int().min(0).max(16),
            expiresAtMs: z.number().int().nonnegative(),
            createdAtMs: z.number().int().nonnegative(),
            completedAtMs: z.number().int().nonnegative().nullable(),
            auditOutcome: z.string().max(120),
          })
          .strict(),
      )
      .max(64),
    diagnostic: z.literal("ledger-unavailable").nullable(),
  })
  .strict();
export type ContextLedgerSnapshot = z.infer<typeof contextLedgerSnapshotSchema>;
