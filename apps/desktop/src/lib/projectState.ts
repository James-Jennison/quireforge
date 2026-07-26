import fixtures from "../../fixtures/project-state.json";
import { z } from "zod";

export const PROJECT_STATE_SCHEMA_VERSION = 1 as const;

const identifierSchema = z
  .string()
  .min(1)
  .max(96)
  .regex(/^[a-z0-9._/-]+$/u);
const commitSchema = z.string().regex(/^[0-9a-f]{40}$/u);
const nullableText = z.string().min(1).max(4096).nullable();
const optionalTimestamp = z.string().datetime().nullable();

export const provenanceSchema = z
  .object({
    trust: z.enum(["verified", "reported", "inferred", "unknown"]),
    sourceType: z.enum([
      "git",
      "repository-document",
      "validation-report",
      "package-manifest",
      "application-storage",
      "agent-session",
      "user-approval",
      "unknown",
    ]),
    sourceRef: nullableText,
    sourceCommit: commitSchema.nullable(),
    observedAt: optionalTimestamp,
    verifiedAt: optionalTimestamp,
    note: nullableText,
  })
  .strict();

const approvalSchema = z
  .object({
    decision: z.enum([
      "required",
      "approved",
      "rejected",
      "superseded",
      "unknown",
    ]),
    authority: nullableText,
    approvedAt: optionalTimestamp,
    scope: nullableText,
    supersededAt: optionalTimestamp,
    provenance: provenanceSchema,
  })
  .strict()
  .superRefine((approval, context) => {
    if (
      ["approved", "rejected", "superseded"].includes(approval.decision) &&
      approval.authority === null
    ) {
      context.addIssue({
        code: "custom",
        message: "A decided approval requires an authority",
      });
    }
  });

const checkpointStatusSchema = z.enum(["pushed", "paused", "finished"]);
const validationResultSchema = z.enum([
  "passed",
  "failed",
  "blocked",
  "not-run",
]);

export const checkpointStateSchema = z
  .object({
    status: checkpointStatusSchema,
    commit: commitSchema.nullable(),
    branch: nullableText,
    pushed: z.boolean(),
    validationsCurrent: z.boolean(),
    documentationCurrent: z.boolean(),
    completionClaimed: z.boolean(),
    timestamp: optionalTimestamp,
    remainingWork: z.array(z.string().min(1).max(4096)).max(256),
    provenance: provenanceSchema,
  })
  .strict()
  .superRefine((checkpoint, context) => {
    const valid =
      (checkpoint.status === "pushed" &&
        checkpoint.pushed &&
        checkpoint.commit !== null &&
        !checkpoint.completionClaimed) ||
      (checkpoint.status === "paused" && !checkpoint.completionClaimed) ||
      (checkpoint.status === "finished" &&
        checkpoint.pushed &&
        checkpoint.commit !== null &&
        checkpoint.completionClaimed &&
        checkpoint.remainingWork.length === 0);
    if (!valid) {
      context.addIssue({
        code: "custom",
        message: "Checkpoint fields are inconsistent",
      });
    }
  });

const workSessionSchema = z
  .object({
    id: identifierSchema,
    actor: z.string().min(1).max(128),
    executionContext: z.enum(["local", "remote", "unknown"]),
    status: z.enum(["active", "paused", "complete", "unknown"]),
    target: z.string().min(1).max(4096),
    startedAt: optionalTimestamp,
    endedAt: optionalTimestamp,
    lastActivityAt: optionalTimestamp,
    pauseReason: nullableText,
    uncommittedWorkMayExist: z.boolean(),
    provenance: provenanceSchema,
  })
  .strict();

const validationStateSchema = z
  .object({
    category: identifierSchema,
    checkId: identifierSchema,
    command: nullableText,
    result: validationResultSchema,
    scope: z.string().min(1).max(4096),
    timestamp: optionalTimestamp,
    evidenceRef: nullableText,
    commitTested: commitSchema.nullable(),
    current: z.boolean(),
    blockerId: identifierSchema.nullable(),
    provenance: provenanceSchema,
  })
  .strict();

const packageRequirementsSchema = z
  .object({
    required: z.boolean(),
    evidence: z
      .array(
        z
          .object({
            artifactType: identifierSchema,
            path: nullableText,
            filename: nullableText,
            sourceCommit: commitSchema.nullable(),
            sizeBytes: z.number().int().nonnegative().safe().nullable(),
            sha256: z
              .string()
              .regex(/^[0-9a-f]{64}$/u)
              .nullable(),
            manifestRef: nullableText,
            platformBaseline: nullableText,
            installResult: validationResultSchema.nullable(),
            launchResult: validationResultSchema.nullable(),
            desktopIntegrationResult: validationResultSchema.nullable(),
            smokeTestResult: validationResultSchema.nullable(),
            provenance: provenanceSchema,
          })
          .strict(),
      )
      .max(32),
    provenance: provenanceSchema,
  })
  .strict();

const boundariesSchema = z
  .object({
    approvedActions: z.array(z.string().min(1).max(4096)).max(256),
    prohibitedActions: z.array(z.string().min(1).max(4096)).max(256),
    confirmationRequiredActions: z.array(z.string().min(1).max(4096)).max(256),
    approvals: z.array(approvalSchema).max(256),
    provenance: provenanceSchema,
  })
  .strict();

const blockerSchema = z
  .object({
    id: identifierSchema,
    description: z.string().min(1).max(4096),
    severity: z.enum(["info", "warning", "error"]),
    affectedRequirement: z.string().min(1).max(4096),
    external: z.boolean(),
    preExisting: z.boolean(),
    milestoneCaused: z.boolean(),
    recommendedAction: z.string().min(1).max(4096),
    approvalRequired: z.boolean(),
    provenance: provenanceSchema,
  })
  .strict();

const contradictionSchema = z
  .object({
    id: identifierSchema,
    kind: identifierSchema,
    description: z.string().min(1).max(4096),
    affectedRequirement: nullableText,
    provenance: provenanceSchema,
  })
  .strict();

export const projectStateContractSchema = z
  .object({
    schemaVersion: z.literal(PROJECT_STATE_SCHEMA_VERSION),
    project: z
      .object({
        id: identifierSchema,
        displayName: z.string().min(1).max(120),
        repository: z.string().regex(/^[a-z0-9._-]+\/[a-z0-9._-]+$/u),
        localWorkspaceId: identifierSchema.nullable(),
        primaryPlatform: identifierSchema,
        activeUiPlatform: identifierSchema,
        productDirectionRef: z.string().min(1).max(4096),
        lifecyclePhase: identifierSchema,
        provenance: provenanceSchema,
      })
      .strict(),
    roadmapRef: z.string().min(1).max(4096),
    repository: z
      .object({
        currentBranch: nullableText,
        baseBranch: nullableText,
        localHead: commitSchema.nullable(),
        remoteHead: commitSchema.nullable(),
        ahead: z.number().int().nonnegative().safe().nullable(),
        behind: z.number().int().nonnegative().safe().nullable(),
        worktree: z.enum(["clean", "dirty", "unknown"]),
        lastVerifiedCheckpoint: commitSchema.nullable(),
        mergeAuthorization: approvalSchema,
        releaseAuthorization: approvalSchema,
        provenance: provenanceSchema,
      })
      .strict()
      .superRefine((repository, context) => {
        if (
          (repository.ahead === null) !== (repository.behind === null) ||
          (repository.localHead === null) !== (repository.remoteHead === null)
        ) {
          context.addIssue({
            code: "custom",
            message: "Repository pairs are inconsistent",
          });
        }
      }),
    milestone: z
      .object({
        id: identifierSchema,
        title: z.string().min(1).max(256),
        status: z.enum(["planned", "active", "paused", "complete", "blocked"]),
        objective: z.string().min(1).max(4096),
        approvedScope: z.array(z.string().min(1).max(4096)).max(256),
        exclusions: z.array(z.string().min(1).max(4096)).max(256),
        completionRequirements: z.array(z.string().min(1).max(4096)).max(256),
        predecessorId: identifierSchema.nullable(),
        successorId: identifierSchema.nullable(),
        ownerApproval: approvalSchema,
        provenance: provenanceSchema,
      })
      .strict(),
    workSessions: z.array(workSessionSchema).max(256),
    checkpoints: z.array(checkpointStateSchema).max(256),
    validations: z.array(validationStateSchema).max(1024),
    packages: packageRequirementsSchema,
    boundaries: boundariesSchema,
    blockers: z.array(blockerSchema).max(256),
    contradictions: z.array(contradictionSchema).max(256),
    nextAction: z
      .object({
        action: z.string().min(1).max(4096),
        why: z.string().min(1).max(4096),
        approvalRequired: z.boolean(),
        targetMilestone: identifierSchema.nullable(),
        requiredStartingCommit: commitSchema.nullable(),
        requiredBranch: nullableText,
        provenance: provenanceSchema,
      })
      .strict()
      .nullable(),
    handoff: z
      .object({
        status: checkpointStatusSchema,
        phrase: z.string().min(1).max(256),
        generatedAt: optionalTimestamp,
        sourceCheckpoint: commitSchema.nullable(),
        provenance: provenanceSchema,
      })
      .strict()
      .superRefine((handoff, context) => {
        const expected = {
          pushed: "Codex checkpoint pushed. Continue.",
          paused: "Codex paused. Continue.",
          finished: "Codex finished. Continue.",
        }[handoff.status];
        if (handoff.phrase !== expected) {
          context.addIssue({
            code: "custom",
            message: "Handoff phrase is inconsistent",
          });
        }
      }),
    provenance: provenanceSchema,
  })
  .strict();

export type ProjectStateContract = z.infer<typeof projectStateContractSchema>;
export const projectStateFixturesSchema = z
  .object({
    minimalValid: projectStateContractSchema,
    activeMilestone: projectStateContractSchema,
    pushedCheckpoint: checkpointStateSchema,
    pausedSession: workSessionSchema,
    completedMilestone: checkpointStateSchema,
    missingEvidence: packageRequirementsSchema,
    missingValidationEvidence: validationStateSchema,
    contradictoryEvidence: z.array(contradictionSchema).max(256),
  })
  .strict();
export const projectStateFixtures = projectStateFixturesSchema.parse(fixtures);
export const scaffoldProjectState = projectStateContractSchema.parse(
  projectStateFixtures.activeMilestone,
);
