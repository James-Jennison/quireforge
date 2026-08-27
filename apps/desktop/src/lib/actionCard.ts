import { z } from "zod";

export const actionCardActionSchema = z.enum([
  "attach-project",
  "use-source",
  "draft-artifact",
  "work-with-code",
]);

export const actionCardPrepareRequestSchema = z
  .object({ action: actionCardActionSchema })
  .strict();

export const actionCardDecisionRequestSchema = z
  .object({ cardId: z.string().uuid() })
  .strict();

export const actionCardSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    cardId: z.string().uuid(),
    action: actionCardActionSchema,
    state: z.enum(["prepared", "approved", "revoked", "expired"]),
    dataScope: z.literal("none"),
    execution: z.literal("not-authorized"),
    receiptId: z.string().uuid().nullable(),
    expiresAtMs: z.number().int().positive(),
  })
  .strict();

export type ActionCardPrepareRequest = z.infer<
  typeof actionCardPrepareRequestSchema
>;
export type ActionCardDecisionRequest = z.infer<
  typeof actionCardDecisionRequestSchema
>;
export type ActionCardSnapshot = z.infer<typeof actionCardSnapshotSchema>;
