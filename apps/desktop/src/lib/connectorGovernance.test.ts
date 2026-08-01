import { describe, expect, it } from "vitest";

import {
  connectorConfirmRequestSchema,
  connectorPrepareRequestSchema,
  connectorSnapshotSchema,
} from "./connectorGovernance";

const id = "019a5900-0000-7000-8000-000000000001";
const digest = "c".repeat(64);

describe("fictional connector governance bridge contracts", () => {
  it("accepts only the closed local read or mutation request shape", () => {
    expect(() =>
      connectorPrepareRequestSchema.parse({
        taskId: id,
        operation: "read",
        target: "mock-object-read",
        url: "https://example.invalid",
      }),
    ).toThrow();
    expect(() =>
      connectorPrepareRequestSchema.parse({
        taskId: id,
        operation: "fetch",
        target: "mock-object-read",
      }),
    ).toThrow();
    expect(() =>
      connectorConfirmRequestSchema.parse({
        taskId: id,
        operationId: id,
        authorizationId: id,
        retry: true,
      }),
    ).toThrow();
  });

  it("rejects snapshots that could imply a real connector or unbounded authority", () => {
    expect(() =>
      connectorSnapshotSchema.parse({
        schemaVersion: 1,
        fictionalLocalOnly: false,
        state: "succeeded",
        projectId: id,
        taskId: id,
        operationId: id,
        authorizationId: null,
        operation: "read",
        diagnostic: null,
        bindingId: id,
        descriptorId: id,
        descriptorVersion: 1,
        descriptorSha256: digest,
        scopeDigest: digest,
        requestDigest: digest,
        expiresAtMs: 1,
        declaredCapabilities: ["read", "mutation", "read"],
        grantedAuthority: ["read", "mutation"],
        auditState: "local",
      }),
    ).toThrow();
  });
});
