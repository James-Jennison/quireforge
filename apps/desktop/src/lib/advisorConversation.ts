import { z } from "zod";

export const advisorConversationStateSchema = z.enum([
  "empty",
  "running",
  "completed",
  "interrupted",
  "blocked",
  "failed",
  "unavailable",
]);

export const advisorConversationDiagnosticCodeSchema = z.enum([
  "authentication-required",
  "authentication-unavailable",
  "conversation-not-found",
  "conversation-active",
  "invalid-request",
  "context-unavailable",
  "runtime-unavailable",
  "protocol-invalid",
  "capability-blocked",
  "metadata-unavailable",
]);

export const advisorConversationEventSchema = z.discriminatedUnion("type", [
  z
    .object({
      type: z.literal("agent-message-delta"),
      sequence: z.number().int().nonnegative(),
      delta: z.string().max(64 * 1024),
    })
    .strict(),
  z
    .object({
      type: z.literal("reasoning-summary-delta"),
      sequence: z.number().int().nonnegative(),
      delta: z.string().max(64 * 1024),
    })
    .strict(),
  z
    .object({
      type: z.literal("error"),
      sequence: z.number().int().nonnegative(),
      code: z.string().min(1).max(128),
    })
    .strict(),
]);

export const advisorConversationSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    mode: z.literal("advisor"),
    state: advisorConversationStateSchema,
    conversationId: z.string().uuid().nullable(),
    projectStateIncluded: z.boolean(),
    events: z.array(advisorConversationEventSchema).max(32),
    diagnosticCode: advisorConversationDiagnosticCodeSchema.nullable(),
  })
  .strict();

export const advisorConversationStartRequestSchema = z
  .object({
    prompt: z
      .string()
      .trim()
      .min(1)
      .max(64 * 1024)
      .refine((value) => !value.includes("\0"), "Prompt must not contain NUL"),
    projectId: z.string().uuid().nullable(),
  })
  .strict();

export const advisorConversationIdSchema = z.string().uuid();

export type AdvisorConversationSnapshot = z.infer<
  typeof advisorConversationSnapshotSchema
>;
export type AdvisorConversationStartRequest = z.infer<
  typeof advisorConversationStartRequestSchema
>;

export const scaffoldAdvisorConversation: AdvisorConversationSnapshot = {
  schemaVersion: 1,
  mode: "advisor",
  state: "empty",
  conversationId: null,
  projectStateIncluded: false,
  events: [],
  diagnosticCode: null,
};
