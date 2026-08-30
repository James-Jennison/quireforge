import { z } from "zod";

// This intentionally names only the one explicit adapter available in the
// current milestone. It is not a provider registry and must not grow into one
// before M81's consented provider-connection boundary exists.
export const chatProviderIdSchema = z.literal("managed-codex");
export type ChatProviderId = z.infer<typeof chatProviderIdSchema>;

export const chatConversationStateSchema = z.enum([
  "empty",
  "running",
  "completed",
  "interrupted",
  "blocked",
  "failed",
  "unavailable",
]);

export const chatConversationDiagnosticCodeSchema = z.enum([
  "authentication-required",
  "authentication-unavailable",
  "conversation-not-found",
  "conversation-active",
  "invalid-request",
  "runtime-unavailable",
  "protocol-invalid",
  "capability-blocked",
  "metadata-unavailable",
]);

export const chatConversationEventSchema = z.discriminatedUnion("type", [
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

export const chatConversationSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    mode: z.literal("chat"),
    state: chatConversationStateSchema,
    conversationId: z.string().uuid().nullable(),
    threadId: z.string().uuid().nullable(),
    events: z.array(chatConversationEventSchema).max(32),
    diagnosticCode: chatConversationDiagnosticCodeSchema.nullable(),
  })
  .strict();

export const chatConversationStartRequestSchema = z
  .object({
    prompt: z
      .string()
      .trim()
      .min(1)
      .max(64 * 1024)
      .refine((value) => !value.includes("\0"), "Prompt must not contain NUL"),
    interactionProfile: z.enum(["direct", "conversational"]).optional(),
  })
  .strict();

export const chatConversationIdSchema = z.string().uuid();

export type ChatConversationSnapshot = z.infer<
  typeof chatConversationSnapshotSchema
>;
export type ChatConversationEvent = z.infer<typeof chatConversationEventSchema>;
export type ChatConversationStartRequest = z.infer<
  typeof chatConversationStartRequestSchema
>;

export const scaffoldChatConversation: ChatConversationSnapshot = {
  schemaVersion: 1,
  mode: "chat",
  state: "empty",
  conversationId: null,
  threadId: null,
  events: [],
  diagnosticCode: null,
};
