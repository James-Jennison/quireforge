import { describe, expect, it } from "vitest";

import {
  dynamicAnalysisSnapshotSchema,
  scaffoldDynamicAnalysis,
} from "./dynamicAnalysis";

describe("dynamic-analysis contract", () => {
  it("accepts only a bounded static-runtime manifest", () => {
    expect(
      dynamicAnalysisSnapshotSchema.parse({
        schemaVersion: 1,
        state: "ready",
        result: null,
        diagnosticCode: null,
        manifest: {
          runId: "018f2c2d-6a14-7e4f-8d67-16b3c71a9988",
          displayName: "sample",
          byteSize: 64,
          sha256: "a".repeat(64),
          elfType: "executable",
          staticRuntime: true,
          maxMemoryBytes: 512 * 1024 * 1024,
          maxWallTimeMs: 30_000,
        },
      }).manifest?.displayName,
    ).toBe("sample");
  });

  it("does not permit paths, byte payloads, or terminal output", () => {
    expect(() =>
      dynamicAnalysisSnapshotSchema.parse({
        ...scaffoldDynamicAnalysis,
        path: "/tmp/sample",
      }),
    ).toThrow();
  });
});
