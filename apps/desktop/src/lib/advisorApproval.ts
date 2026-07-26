import { z } from "zod";

import { advisorSelectedProjectStateSnapshotSchema } from "./advisorWorkspace";

const uuidV7 = z
  .string()
  .regex(
    /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/u,
  );

export const advisorDeclaredCapabilitySchema = z.enum([
  "read-only",
  "workspace-write",
  "danger-full-access",
]);

export const advisorDraftCreateRequestSchema = z
  .object({
    advisorConversationId: uuidV7,
    targetProjectId: uuidV7,
    prompt: z
      .string()
      .trim()
      .min(1)
      .max(64 * 1024)
      .refine((value) => !value.includes("\0")),
    selectedProjectState: advisorSelectedProjectStateSnapshotSchema.nullable(),
    declaredCapabilities: z
      .array(advisorDeclaredCapabilitySchema)
      .min(1)
      .max(3),
    requestedModel: z.string().trim().min(1).max(128),
    requestedReasoningEffort: z.string().trim().min(1).max(64),
  })
  .strict();

export const advisorApprovalDecisionRequestSchema = z
  .object({
    proposalId: uuidV7,
    decision: z.enum(["approved", "rejected"]),
  })
  .strict();

export const advisorApprovalSnapshotSchema = z
  .object({
    proposalId: uuidV7,
    state: z.enum(["draft", "approved", "rejected"]),
    expiresAtMs: z.number().int().nonnegative(),
    dispatchAvailable: z.literal(false),
  })
  .strict();

export type AdvisorDraftCreateRequest = z.infer<
  typeof advisorDraftCreateRequestSchema
>;
export type AdvisorApprovalDecisionRequest = z.infer<
  typeof advisorApprovalDecisionRequestSchema
>;
export type AdvisorApprovalSnapshot = z.infer<
  typeof advisorApprovalSnapshotSchema
>;
