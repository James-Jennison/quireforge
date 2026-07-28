import { z } from "zod";

const bounded = (maximum: number) =>
  z
    .string()
    .trim()
    .min(1)
    .max(maximum)
    .refine((value) => !value.includes("\0"));

export const taskHandoffDirectionSchema = z.enum([
  "advisor-to-quireforge",
  "quireforge-to-advisor",
]);
export const taskHandoffReceiptStatusSchema = z.enum([
  "completed",
  "blocked",
  "cancelled",
  "failed",
]);
export const taskHandoffSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum(["empty", "pending", "accepted", "unavailable"]),
    taskId: z.string().uuid().nullable(),
    direction: taskHandoffDirectionSchema.nullable(),
    title: z.string().max(120).nullable(),
    originalRequest: z
      .string()
      .max(8 * 1024)
      .nullable(),
    brief: z
      .string()
      .max(12 * 1024)
      .nullable(),
    receiptStatus: taskHandoffReceiptStatusSchema.nullable(),
    expiresAtMs: z.number().int().nonnegative().nullable(),
    diagnosticCode: z
      .enum(["invalid-request", "not-found", "expired", "direction-mismatch"])
      .nullable(),
  })
  .strict();
export const taskHandoffCreateRequestSchema = z
  .object({
    title: bounded(120),
    originalRequest: bounded(8 * 1024),
    brief: bounded(12 * 1024),
  })
  .strict();
export const taskHandoffReceiptRequestSchema = z
  .object({
    taskId: z.string().uuid(),
    title: bounded(120),
    originalRequest: bounded(8 * 1024),
    summary: bounded(4 * 1024),
    status: taskHandoffReceiptStatusSchema,
  })
  .strict();
export type TaskHandoffSnapshot = z.infer<typeof taskHandoffSnapshotSchema>;
export type TaskHandoffCreateRequest = z.infer<
  typeof taskHandoffCreateRequestSchema
>;
export type TaskHandoffReceiptRequest = z.infer<
  typeof taskHandoffReceiptRequestSchema
>;
