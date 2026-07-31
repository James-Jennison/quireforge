import { describe, expect, it } from "vitest";

import {
  mockInferencePrepareRequestSchema,
  mockInferenceSnapshotSchema,
} from "./mockInference";

const id = "019a5900-0000-7000-8000-000000000001";
const digest = "b".repeat(64);

describe("mock inference bridge contracts", () => {
  it("rejects unknown request fields and unbounded input", () => {
    expect(() =>
      mockInferencePrepareRequestSchema.parse({
        taskId: id,
        profileId: "lantern-stream",
        input: "visible",
        extra: true,
      }),
    ).toThrow();
    expect(() =>
      mockInferencePrepareRequestSchema.parse({
        taskId: id,
        profileId: "lantern-stream",
        input: "x".repeat(2001),
      }),
    ).toThrow();
  });

  it("requires ordered events and content-free bounded snapshots", () => {
    expect(() =>
      mockInferenceSnapshotSchema.parse({
        schemaVersion: 1,
        mockOnly: true,
        attemptId: id,
        state: "completed",
        diagnostic: null,
        destination: {
          providerId: id,
          endpointId: id,
          modelId: id,
          adapterId: id,
          descriptorSha256: digest,
          capabilityProfileSha256: digest,
        },
        manifest: null,
        lease: null,
        authorization: null,
        events: [
          {
            id,
            sequence: 2,
            kind: "text-delta",
            text: "fixture",
            structuredState: null,
            sha256: digest,
          },
        ],
        usage: null,
        evidence: [],
      }),
    ).toThrow();
  });
});
