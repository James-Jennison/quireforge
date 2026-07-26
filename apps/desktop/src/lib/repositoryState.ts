import { z } from "zod";

import { projectStateContractSchema } from "./projectState";

export const REPOSITORY_STATE_READER_SCHEMA_VERSION = 1 as const;

export const repositoryStateReadRequestSchema = z
  .object({
    projectId: z.string().uuid(),
    remoteMode: z.enum(["local-only", "existing-tracking", "fetch-authorized"]),
    artifactVerification: z
      .enum(["metadata-only", "verify-local-artifacts"])
      .default("metadata-only"),
  })
  .strict();

export const repositoryStateReadSnapshotSchema = z
  .object({
    schemaVersion: z.literal(REPOSITORY_STATE_READER_SCHEMA_VERSION),
    state: projectStateContractSchema,
    git: z
      .object({
        upstream: z.string().min(1).max(4096).nullable(),
        detached: z.boolean(),
        stagedCount: z.number().int().nonnegative(),
        unstagedCount: z.number().int().nonnegative(),
        untrackedCount: z.number().int().nonnegative(),
        mergeInProgress: z.boolean(),
        rebaseInProgress: z.boolean(),
        cherryPickInProgress: z.boolean(),
        bisectInProgress: z.boolean(),
        shallow: z.boolean().nullable(),
      })
      .strict(),
    evidence: z
      .object({
        packages: z.array(
          z
            .object({
              kind: z.enum(["deb", "app-image"]),
              sourceCommit: z.string().nullable(),
              artifactPath: z.string().nullable(),
              checksum: z.string().nullable(),
              localVerified: z.boolean(),
              freshness: z.enum([
                "current",
                "stale",
                "unknown",
                "conflicting",
                "not-applicable",
              ]),
            })
            .strict(),
        ),
        validations: z.array(
          z
            .object({
              id: z.string(),
              status: z.enum(["passed", "failed", "skipped", "unavailable"]),
              sourceCommit: z.string().nullable(),
              evidencePath: z.string(),
              freshness: z.enum([
                "current",
                "stale",
                "unknown",
                "conflicting",
                "not-applicable",
              ]),
            })
            .strict(),
        ),
        handoff: z
          .object({
            status: z.string(),
            phrase: z.string(),
            sourceCommit: z.string().nullable(),
            freshness: z.enum([
              "current",
              "stale",
              "unknown",
              "conflicting",
              "not-applicable",
            ]),
          })
          .strict()
          .nullable(),
      })
      .strict(),
    diagnostics: z
      .array(
        z
          .object({
            id: z.string().min(1).max(96),
            severity: z.enum(["info", "warning", "error"]),
            affectedField: z.string().min(1).max(4096),
            sourceRef: z.string().min(1).max(4096).nullable(),
            explanation: z.string().min(1).max(4096),
            approvalRequired: z.boolean(),
            recommendedAction: z.string().min(1).max(4096),
          })
          .strict(),
      )
      .max(256),
  })
  .strict();

export type RepositoryStateReadRequest = z.infer<
  typeof repositoryStateReadRequestSchema
>;
export type RepositoryStateReadSnapshot = z.infer<
  typeof repositoryStateReadSnapshotSchema
>;
