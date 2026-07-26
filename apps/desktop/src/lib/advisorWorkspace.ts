import { z } from "zod";

const trust = z.enum(["verified", "reported", "inferred", "unknown"]);
const freshness = z.enum([
  "current",
  "stale",
  "unknown",
  "conflicting",
  "not-applicable",
]);

/** Safe projection of the validated Advisor v1 contract for presentation. */
export const advisorWorkspaceSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    conversationCount: z.number().int().min(0).max(256),
    contextReferenceCount: z.number().int().min(0).max(1024),
    proposalCount: z.number().int().min(0).max(256),
    contextSummaries: z
      .array(
        z
          .object({
            kind: z.enum([
              "project-state",
              "roadmap",
              "current-state",
              "execution-report",
            ]),
            trust,
            freshness,
          })
          .strict(),
      )
      .max(1024),
    proposalSummaries: z
      .array(
        z
          .object({
            state: z.enum(["draft", "approved", "rejected"]),
            requiresExplicitApproval: z.literal(true),
          })
          .strict(),
      )
      .max(256),
  })
  .strict()
  .superRefine((value, context) => {
    if (value.contextSummaries.length !== value.contextReferenceCount) {
      context.addIssue({
        code: "custom",
        message: "Context count is inconsistent",
      });
    }
    if (value.proposalSummaries.length !== value.proposalCount) {
      context.addIssue({
        code: "custom",
        message: "Proposal count is inconsistent",
      });
    }
  });

export type AdvisorWorkspaceSnapshot = z.infer<
  typeof advisorWorkspaceSnapshotSchema
>;

/**
 * The only project-derived content rendered by Advisor in M28. It contains no
 * project identity, path, branch, commit, document, artifact, or diagnostic
 * text from the underlying repository-state snapshot.
 */
export interface AdvisorProjectStateReadRequest {
  projectId: string;
}

export const advisorSelectedProjectStateSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    sourceKind: z.literal("project-state"),
    selectedAtMs: z.number().int().nonnegative(),
    trust,
    freshness,
    provenanceSource: z.literal("project-state-snapshot"),
    worktree: z.enum(["clean", "dirty", "unknown"]),
    diagnosticCount: z.number().int().nonnegative(),
  })
  .strict();

export type AdvisorSelectedProjectStateSnapshot = z.infer<
  typeof advisorSelectedProjectStateSnapshotSchema
>;

const projectIdPattern =
  /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/u;
export function parseAdvisorProjectStateReadRequest(
  value: AdvisorProjectStateReadRequest,
): AdvisorProjectStateReadRequest {
  if (
    !projectIdPattern.test(value.projectId) ||
    Object.keys(value).length !== 1
  ) {
    throw new Error("Invalid Advisor Project State request");
  }
  return value;
}

export function parseAdvisorSelectedProjectStateSnapshot(
  value: unknown,
): AdvisorSelectedProjectStateSnapshot {
  try {
    return advisorSelectedProjectStateSnapshotSchema.parse(value);
  } catch {
    throw new Error("Invalid Advisor snapshot");
  }
}
