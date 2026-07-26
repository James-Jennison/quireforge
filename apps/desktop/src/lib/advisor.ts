import { z } from "zod";

/**
 * Reference-only boundary for the future Advisor workspace. These schemas do
 * not authorize a model call, attached-project read, native action, or Codex
 * dispatch. Prompt and transcript bodies intentionally have no representation.
 */
export const ADVISOR_FOUNDATION_SCHEMA_VERSION = 1 as const;

const uuidV7Schema = z
  .string()
  .regex(
    /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/u,
  );
const opaqueIdSchema = z
  .string()
  .min(1)
  .max(160)
  .regex(/^[A-Za-z0-9._-]+$/u);
const sourceRefSchema = z
  .string()
  .min(1)
  .max(96)
  .regex(/^[A-Za-z0-9._-]+$/u);
const commitSchema = z.string().regex(/^[0-9a-f]{40}$/u);
const sha256Schema = z.string().regex(/^[0-9a-f]{64}$/u);
const timestampSchema = z.number().int().nonnegative().safe();
const hasControlCharacter = (value: string) =>
  Array.from(value).some((character) => {
    const codePoint = character.codePointAt(0) ?? 0;
    return codePoint <= 0x1f || codePoint === 0x7f;
  });

export const advisorTrustSchema = z.enum([
  "verified",
  "reported",
  "inferred",
  "unknown",
]);
export const advisorFreshnessSchema = z.enum([
  "current",
  "stale",
  "unknown",
  "conflicting",
  "not-applicable",
]);
export const advisorContextKindSchema = z.enum([
  "project-state",
  "roadmap",
  "current-state",
  "execution-report",
]);
export const advisorProvenanceSourceSchema = z.enum([
  "git-observation",
  "project-state-snapshot",
  "repository-document",
  "execution-report",
  "user-selection",
  "unknown",
]);

export const advisorProvenanceSchema = z
  .object({
    trust: advisorTrustSchema,
    source: advisorProvenanceSourceSchema,
    sourceRef: sourceRefSchema.nullable(),
    sourceCommit: commitSchema.nullable(),
    observedAtMs: timestampSchema.nullable(),
    note: z
      .string()
      .max(512)
      .refine((value) => !hasControlCharacter(value))
      .nullable(),
  })
  .strict();

export const advisorConversationReferenceSchema = z
  .object({
    id: uuidV7Schema,
    codexThreadId: opaqueIdSchema,
    createdAtMs: timestampSchema,
    updatedAtMs: timestampSchema,
  })
  .strict()
  .superRefine((value, context) => {
    if (value.updatedAtMs < value.createdAtMs) {
      context.addIssue({
        code: "custom",
        message: "Conversation timestamps are inconsistent",
      });
    }
  });

export const advisorContextReferenceSchema = z
  .object({
    id: uuidV7Schema,
    advisorConversationId: uuidV7Schema,
    kind: advisorContextKindSchema,
    sourceRef: sourceRefSchema,
    sourceCommit: commitSchema.nullable(),
    contentSha256: sha256Schema,
    selectedAtMs: timestampSchema,
    freshness: advisorFreshnessSchema,
    provenance: advisorProvenanceSchema,
  })
  .strict();

export const advisorDispatchStateSchema = z.enum([
  "draft",
  "approved",
  "rejected",
]);

export const advisorDispatchProposalSchema = z
  .object({
    id: uuidV7Schema,
    advisorConversationId: uuidV7Schema,
    targetProjectId: uuidV7Schema,
    promptSha256: sha256Schema,
    contextManifestSha256: sha256Schema,
    capabilityManifestSha256: sha256Schema,
    state: advisorDispatchStateSchema,
    requiresExplicitApproval: z.literal(true),
    requestedModel: sourceRefSchema.max(128).nullable(),
    requestedReasoningEffort: sourceRefSchema.max(64).nullable(),
    createdAtMs: timestampSchema,
    updatedAtMs: timestampSchema,
    decidedAtMs: timestampSchema.nullable(),
    expiresAtMs: timestampSchema,
    provenance: advisorProvenanceSchema,
  })
  .strict()
  .superRefine((value, context) => {
    if (value.updatedAtMs < value.createdAtMs) {
      context.addIssue({
        code: "custom",
        message: "Dispatch timestamps are inconsistent",
      });
    }
    if (value.expiresAtMs !== 0 && value.expiresAtMs < value.createdAtMs) {
      context.addIssue({
        code: "custom",
        message: "Dispatch expiry is inconsistent",
      });
    }
  });

export const advisorFoundationSnapshotSchema = z
  .object({
    schemaVersion: z.literal(ADVISOR_FOUNDATION_SCHEMA_VERSION),
    conversations: z.array(advisorConversationReferenceSchema).max(256),
    contextReferences: z.array(advisorContextReferenceSchema).max(1024),
    dispatchProposals: z.array(advisorDispatchProposalSchema).max(256),
  })
  .strict()
  .superRefine((value, context) => {
    const conversationIds = new Set(
      value.conversations.map((conversation) => conversation.id),
    );
    if (conversationIds.size !== value.conversations.length) {
      context.addIssue({
        code: "custom",
        message: "Advisor conversation IDs must be unique",
      });
    }
    const contextIds = new Set(
      value.contextReferences.map((reference) => reference.id),
    );
    if (
      contextIds.size !== value.contextReferences.length ||
      value.contextReferences.some(
        (reference) => !conversationIds.has(reference.advisorConversationId),
      )
    ) {
      context.addIssue({
        code: "custom",
        message: "Advisor context references must be owned",
      });
    }
    const proposalIds = new Set(
      value.dispatchProposals.map((proposal) => proposal.id),
    );
    if (
      proposalIds.size !== value.dispatchProposals.length ||
      value.dispatchProposals.some(
        (proposal) => !conversationIds.has(proposal.advisorConversationId),
      )
    ) {
      context.addIssue({
        code: "custom",
        message: "Advisor dispatch proposals must be owned",
      });
    }
  });

export type AdvisorFoundationSnapshot = z.infer<
  typeof advisorFoundationSnapshotSchema
>;
