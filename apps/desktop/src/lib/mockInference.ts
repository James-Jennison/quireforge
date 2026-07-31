import { z } from "zod";

const uuid = z.string().uuid();
const digest = z.string().regex(/^[0-9a-f]{64}$/u);
const boundedInput = z
  .string()
  .transform((value) => value.trim())
  .pipe(z.string().min(1).max(2_000));

export const mockInferencePrepareRequestSchema = z
  .object({
    taskId: uuid,
    profileId: z.string().min(1).max(64),
    input: boundedInput,
  })
  .strict();
export const mockInferenceAuthorizationRequestSchema = z
  .object({ taskId: uuid, attemptId: uuid, authorizationId: uuid })
  .strict();
export const mockInferenceAttemptRequestSchema = z
  .object({ taskId: uuid, attemptId: uuid })
  .strict();

const scenario = z.enum([
  "streamed-text",
  "structured",
  "refusal",
  "failure",
  "timeout",
  "interrupted",
  "ambiguous",
]);
const attemptState = z.enum([
  "draft",
  "ready",
  "authorized",
  "submitted",
  "streaming",
  "cancelling",
  "cancelled",
  "completed",
  "refused",
  "failed",
  "interrupted",
  "ambiguous",
  "invalidated",
]);
const diagnostic = z.enum([
  "invalid-request",
  "task-unavailable",
  "attempt-unavailable",
  "authorization-required",
  "authorization-replayed",
  "authorization-invalid",
  "lease-unavailable",
  "manifest-invalidated",
  "terminal-attempt",
  "cross-task-rejected",
]);

export const mockInferenceCatalogSchema = z
  .object({
    schemaVersion: z.literal(1),
    profiles: z
      .array(
        z
          .object({
            id: z.string().min(1).max(64),
            providerLabel: z.string().min(1).max(120),
            endpointLabel: z.string().min(1).max(120),
            modelLabel: z.string().min(1).max(120),
            adapterLabel: z.string().min(1).max(120),
            scenario,
            descriptorSha256: digest,
          })
          .strict(),
      )
      .min(1)
      .max(8),
  })
  .strict();

const destination = z
  .object({
    providerId: uuid,
    endpointId: uuid,
    modelId: uuid,
    adapterId: uuid,
    descriptorSha256: digest,
    capabilityProfileSha256: digest,
  })
  .strict();
const manifest = z
  .object({
    id: uuid,
    sha256: digest,
    inputSha256: digest,
    itemCount: z.literal(1),
    inputCharCount: z.number().int().min(1).max(2_000),
    exclusions: z.array(z.string().min(1).max(80)).min(1).max(8),
    retention: z.literal("transient-local-mock"),
    expiresAtTick: z.number().int().nonnegative(),
    state: z.enum(["ready", "invalidated"]),
  })
  .strict();
const lease = z
  .object({
    credentialReferenceId: uuid,
    leaseId: uuid,
    accountReference: z.literal("fictional-account-reference"),
    scopes: z.array(z.literal("mock-inference-submit")).length(1),
    state: z.enum([
      "issued",
      "expired",
      "revoked",
      "quarantined",
      "invalidated",
    ]),
    expiresAtTick: z.number().int().nonnegative(),
  })
  .strict();
const authorization = z
  .object({
    id: uuid,
    bindingSha256: digest,
    state: z.enum(["pending", "authorized", "consumed"]),
    expiresAtTick: z.number().int().nonnegative(),
  })
  .strict();
const event = z
  .object({
    id: uuid,
    sequence: z.number().int().positive().max(128),
    kind: z.string().min(1).max(64),
    text: z.string().max(512).nullable(),
    structuredState: z
      .enum(["partial", "complete-valid", "complete-invalid", "nonconforming"])
      .nullable(),
    sha256: digest,
  })
  .strict();

export const mockInferenceSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    mockOnly: z.literal(true),
    attemptId: uuid.nullable(),
    state: attemptState,
    diagnostic: diagnostic.nullable(),
    destination: destination.nullable(),
    manifest: manifest.nullable(),
    lease: lease.nullable(),
    authorization: authorization.nullable(),
    events: z.array(event).max(128),
    usage: z
      .object({
        basis: z.literal("fictional-reported"),
        units: z
          .array(
            z
              .object({
                unit: z.string().min(1).max(80),
                quantity: z.number().int().nonnegative(),
              })
              .strict(),
          )
          .max(8),
      })
      .strict()
      .nullable(),
    evidence: z
      .array(
        z
          .object({
            kind: z.string().min(1).max(80),
            sha256: digest,
            detail: z.string().min(1).max(240),
          })
          .strict(),
      )
      .max(8),
  })
  .strict()
  .superRefine((value, context) => {
    if (value.attemptId === null && value.destination !== null) {
      context.addIssue({
        code: "custom",
        message: "Attempt details require an attempt identity",
      });
    }
    if (value.events.some((item, index) => item.sequence !== index + 1)) {
      context.addIssue({
        code: "custom",
        message: "Mock events must be ordered",
      });
    }
  });

export type MockInferenceCatalog = z.infer<typeof mockInferenceCatalogSchema>;
export type MockInferenceSnapshot = z.infer<typeof mockInferenceSnapshotSchema>;
