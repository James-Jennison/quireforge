import { z } from "zod";

const uuid = z.string().uuid();

export const connectorPrepareRequestSchema = z
  .object({
    taskId: uuid,
    operation: z.enum(["read", "mutation"]),
    target: z.string().trim().min(1).max(80),
  })
  .strict();
export const connectorConfirmRequestSchema = z
  .object({ taskId: uuid, operationId: uuid, authorizationId: uuid })
  .strict();
export const connectorCancelRequestSchema = z
  .object({ taskId: uuid, authorizationId: uuid })
  .strict();
export const connectorOperationRequestSchema = z
  .object({ taskId: uuid, operationId: uuid })
  .strict();

export const connectorSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    fictionalLocalOnly: z.literal(true),
    state: z.string().min(1).max(40),
    projectId: uuid.nullable(),
    taskId: uuid.nullable(),
    operationId: uuid.nullable(),
    authorizationId: uuid.nullable(),
    operation: z.enum(["read", "mutation"]).nullable(),
    diagnostic: z.string().min(1).max(80).nullable(),
    bindingId: uuid.nullable(),
    descriptorId: uuid.nullable(),
    descriptorVersion: z.number().int().positive().nullable(),
    descriptorSha256: z
      .string()
      .regex(/^[a-f0-9]{64}$/)
      .nullable(),
    scopeDigest: z
      .string()
      .regex(/^[a-f0-9]{64}$/)
      .nullable(),
    requestDigest: z
      .string()
      .regex(/^[a-f0-9]{64}$/)
      .nullable(),
    expiresAtMs: z.number().int().nonnegative().nullable(),
    declaredCapabilities: z.array(z.enum(["read", "mutation"])).max(2),
    grantedAuthority: z.array(z.enum(["read", "mutation"])).max(1),
    auditState: z.string().min(1).max(240),
  })
  .strict();

export type ConnectorSnapshot = z.infer<typeof connectorSnapshotSchema>;
