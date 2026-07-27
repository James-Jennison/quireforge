import { z } from "zod";

const diagnosticCodeSchema = z.enum([
  "worker-unavailable",
  "invalid-request",
  "unsupported-type",
  "invalid-signature",
  "unsupported-runtime",
  "source-too-large",
  "source-unavailable",
  "source-changed",
  "unsafe-name",
  "attachment-not-found",
  "attachment-expired",
  "manifest-mismatch",
  "worker-rejected",
  "read-failed",
]);

export const dynamicAnalysisSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum(["empty", "ready", "unavailable", "completed"]),
    manifest: z
      .object({
        runId: z.string().uuid(),
        displayName: z
          .string()
          .min(1)
          .max(255)
          .refine((name) => !/[\\/]/u.test(name)),
        byteSize: z
          .number()
          .int()
          .positive()
          .max(32 * 1024 * 1024),
        sha256: z.string().regex(/^[a-f0-9]{64}$/iu),
        elfType: z.enum(["executable", "shared-object"]),
        staticRuntime: z.literal(true),
        maxMemoryBytes: z.literal(512 * 1024 * 1024),
        maxWallTimeMs: z.literal(30_000),
      })
      .strict()
      .nullable(),
    result: z
      .object({
        kind: z.literal("dynamic-analysis-result-v1"),
        schemaVersion: z.literal(1),
        runId: z.string().uuid(),
        outcome: z.enum([
          "completed",
          "nonzero-exit",
          "signal",
          "timeout",
          "policy-denied",
          "setup-failed",
        ]),
        elapsedMs: z.number().int().nonnegative().max(30_000),
        guestStarted: z.boolean(),
        resourceLimits: z.array(z.string().max(64)).max(8),
      })
      .strict()
      .nullable(),
    diagnosticCode: diagnosticCodeSchema.nullable(),
  })
  .strict();

export type DynamicAnalysisSnapshot = z.infer<
  typeof dynamicAnalysisSnapshotSchema
>;
export type DynamicAnalysisRunRequest = {
  runId: string;
  sha256: string;
  confirmed: boolean;
};

export const scaffoldDynamicAnalysis: DynamicAnalysisSnapshot = {
  schemaVersion: 1,
  state: "empty",
  manifest: null,
  result: null,
  diagnosticCode: null,
};
