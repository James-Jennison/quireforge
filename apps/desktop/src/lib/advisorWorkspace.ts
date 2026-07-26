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
