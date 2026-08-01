import { describe, expect, it } from "vitest";

import {
  browserVerificationConfirmRequestSchema,
  browserVerificationPrepareRequestSchema,
  browserVerificationSnapshotSchema,
} from "./controlledBrowserVerification";

const id = "019a5800-0000-7000-8000-000000000001";

describe("controlled browser verification bridge contracts", () => {
  it("accepts only the exact fictional local fixture preparation", () => {
    expect(() =>
      browserVerificationPrepareRequestSchema.parse({
        projectId: id,
        taskId: null,
        target: "https://example.invalid",
        assertion: "fixture-marker",
      }),
    ).toThrow();
    expect(() =>
      browserVerificationPrepareRequestSchema.parse({
        projectId: id,
        taskId: null,
        target: "quireforge-fixture://verification/expected?assert=marker",
        assertion: "other",
      }),
    ).toThrow();
    expect(() =>
      browserVerificationConfirmRequestSchema.parse({
        attemptId: id,
        authorizationId: id,
        retry: true,
      }),
    ).toThrow();
  });

  it("rejects snapshots that could imply networked, mutable, or non-fictional browsing", () => {
    expect(() =>
      browserVerificationSnapshotSchema.parse({
        schemaVersion: 1,
        fictionalLocalOnly: true,
        readOnly: false,
        adapter: "ephemeral-webkitgtk-fixture",
        state: "verified",
        projectId: id,
        taskId: null,
        attemptId: id,
        authorizationId: null,
        target: "quireforge-fixture://verification/expected?assert=marker",
        origin: "quireforge-fixture://verification",
        assertion: "fixture-marker",
        requestDigest: "a".repeat(64),
        expiresAtMs: 1,
        evidenceDigest: "b".repeat(64),
        visibleText: "fixture marker verified",
        diagnostic: null,
        auditState: "terminal",
      }),
    ).toThrow();
  });
});
