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
  "thread-start-rejected",
  "protocol-invalid",
  "capability-blocked",
  "metadata-unavailable",
  "attachment-unavailable",
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
    attachmentId: z.string().uuid().nullable(),
    attachmentManifestSha256: z
      .string()
      .regex(/^[a-f0-9]{64}$/iu)
      .nullable(),
    attachmentConfirmation: z.literal("confirmed-for-single-send").nullable(),
    imageAttachmentId: z.string().uuid().nullable().default(null),
    imageAttachmentManifestSha256: z
      .string()
      .regex(/^[a-f0-9]{64}$/iu)
      .nullable()
      .default(null),
    imageAttachmentConfirmation: z
      .literal("confirmed-for-single-send")
      .nullable()
      .default(null),
    documentAttachmentId: z.string().uuid().nullable().default(null),
    documentAttachmentManifestSha256: z
      .string()
      .regex(/^[a-f0-9]{64}$/iu)
      .nullable()
      .default(null),
    documentAttachmentConfirmation: z
      .literal("confirmed-for-single-send")
      .nullable()
      .default(null),
    archiveAttachmentId: z.string().uuid().nullable().default(null),
    archiveAttachmentManifestSha256: z
      .string()
      .regex(/^[a-f0-9]{64}$/iu)
      .nullable()
      .default(null),
    archiveAttachmentConfirmation: z
      .literal("confirmed-for-single-send")
      .nullable()
      .default(null),
    binaryAttachmentId: z.string().uuid().nullable().default(null),
    binaryAttachmentManifestSha256: z
      .string()
      .regex(/^[a-f0-9]{64}$/iu)
      .nullable()
      .default(null),
    binaryAttachmentConfirmation: z
      .literal("confirmed-for-single-send")
      .nullable()
      .default(null),
  })
  .strict()
  .refine(
    (value) =>
      (value.attachmentId === null) ===
      (value.attachmentManifestSha256 === null),
  )
  .refine(
    (value) =>
      (value.binaryAttachmentId === null) ===
        (value.binaryAttachmentManifestSha256 === null) &&
      (value.binaryAttachmentId === null) ===
        (value.binaryAttachmentConfirmation === null),
  )
  .refine(
    (value) =>
      (value.attachmentId === null) === (value.attachmentConfirmation === null),
  )
  .refine(
    (value) =>
      (value.imageAttachmentId === null) ===
      (value.imageAttachmentManifestSha256 === null),
  )
  .refine(
    (value) =>
      (value.imageAttachmentId === null) ===
      (value.imageAttachmentConfirmation === null),
  )
  .refine(
    (value) =>
      (value.documentAttachmentId === null) ===
        (value.documentAttachmentManifestSha256 === null) &&
      (value.documentAttachmentId === null) ===
        (value.documentAttachmentConfirmation === null),
  )
  .refine(
    (value) =>
      (value.archiveAttachmentId === null) ===
        (value.archiveAttachmentManifestSha256 === null) &&
      (value.archiveAttachmentId === null) ===
        (value.archiveAttachmentConfirmation === null),
  )
  .refine(
    (value) =>
      [
        value.attachmentId,
        value.imageAttachmentId,
        value.documentAttachmentId,
        value.archiveAttachmentId,
        value.binaryAttachmentId,
      ].filter((value) => value !== null).length <= 1,
    "Advisor accepts one content attachment per send",
  );

export const advisorConversationIdSchema = z.string().uuid();

const MAX_ADVISOR_TRANSIENT_EVENTS = 32;
const MAX_ADVISOR_MESSAGE_DELTA_CHARS = 64 * 1024;

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

/**
 * Poll responses contain only the newly received stream fragments. Merge them
 * into the live UI snapshot for the same conversation without persisting text.
 */
export function mergeAdvisorConversationSnapshot(
  current: AdvisorConversationSnapshot,
  incoming: AdvisorConversationSnapshot,
): AdvisorConversationSnapshot {
  if (
    current.conversationId === null ||
    incoming.conversationId === null ||
    current.conversationId !== incoming.conversationId
  ) {
    return incoming;
  }

  const events = current.events.map((event) => ({ ...event }));
  let newestSequence = events.at(-1)?.sequence ?? 0;
  for (const event of incoming.events) {
    if (event.sequence <= newestSequence) continue;
    const previous = events.at(-1);
    if (
      event.type === "agent-message-delta" &&
      previous?.type === "agent-message-delta" &&
      previous.delta.length + event.delta.length <=
        MAX_ADVISOR_MESSAGE_DELTA_CHARS
    ) {
      previous.delta += event.delta;
      previous.sequence = event.sequence;
    } else {
      events.push({ ...event });
    }
    newestSequence = event.sequence;
  }

  return {
    ...incoming,
    events: events.slice(-MAX_ADVISOR_TRANSIENT_EVENTS),
  };
}
