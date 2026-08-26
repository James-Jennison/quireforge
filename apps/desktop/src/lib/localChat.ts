import { z } from "zod";

export const localChatRequestSchema = z
  .object({
    message: z
      .string()
      .min(1)
      .max(96 * 1024),
  })
  .strict();

export const localChatSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    localOnly: z.literal(true),
    state: z.enum(["completed", "failed", "cancelled"]),
    output: z
      .string()
      .max(16 * 1024)
      .nullable(),
    diagnostic: z.string().max(120).nullable(),
    inputTokenLimit: z.literal(4096),
    outputTokenLimit: z.literal(512),
    deadlineSeconds: z.literal(60),
    memoryCeilingMib: z.literal(6144),
  })
  .strict();

export type LocalChatRequest = z.infer<typeof localChatRequestSchema>;
export type LocalChatSnapshot = z.infer<typeof localChatSnapshotSchema>;
