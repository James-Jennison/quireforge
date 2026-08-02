import { describe, expect, it } from "vitest";
import {
  contextAssemblyPrepareRequestSchema,
  contextAssemblySnapshotSchema,
} from "./contextAssembly";

describe("M60 context assembly bridge", () => {
  it("accepts only bounded explicit selections", () => {
    expect(() =>
      contextAssemblyPrepareRequestSchema.parse({
        projectId: "019fbee6-476f-71b0-853c-f067657aa69c",
        taskId: null,
        userInstruction: "review this",
        durableSourceIds: [],
      }),
    ).not.toThrow();
    expect(() =>
      contextAssemblyPrepareRequestSchema.parse({
        projectId: "bad",
        taskId: null,
        userInstruction: "review this",
        durableSourceIds: [],
        extra: true,
      }),
    ).toThrow();
  });
  it("requires the fictional local-only closed snapshot", () => {
    expect(() =>
      contextAssemblySnapshotSchema.parse({
        schemaVersion: 1,
        fictionalLocalOnly: true,
        sink: "fictional-local-context-sink-v1",
        state: "prepared",
        projectId: null,
        taskId: null,
        bundleId: null,
        authorizationId: null,
        bundleDigest: null,
        expiresAtMs: null,
        items: [],
        totalBytes: 0,
        estimatedTokens: 0,
        exclusions: [],
        auditState: "none",
        diagnostic: null,
      }),
    ).not.toThrow();
  });
});
