import { describe, expect, it } from "vitest";

import {
  objectiveAuthorityCreateRequestSchema,
  objectiveAuthoritySnapshotSchema,
} from "./objectiveAuthority";

const projectId = "019fbee6-476f-71b0-853c-f067657aa69c";

describe("M72 objective authority contract", () => {
  it("allows only bounded unique lanes and confirmation subsets", () => {
    expect(
      objectiveAuthorityCreateRequestSchema.parse({
        projectId,
        title: "Review browser workspace",
        objective: "Review an owner-visible browser workspace.",
        allowedLanes: ["browser-workspace", "browser-observation"],
        confirmationRequiredLanes: ["browser-observation"],
        expiresInMinutes: 60,
      }),
    ).toMatchObject({ projectId, expiresInMinutes: 60 });

    expect(() =>
      objectiveAuthorityCreateRequestSchema.parse({
        projectId,
        title: "Bad scope",
        objective: "No implicit scope.",
        allowedLanes: ["browser-workspace", "browser-workspace"],
        confirmationRequiredLanes: [],
        expiresInMinutes: 60,
      }),
    ).toThrow();
    expect(() =>
      objectiveAuthorityCreateRequestSchema.parse({
        projectId,
        title: "Bad confirmation",
        objective: "No out-of-scope confirmation.",
        allowedLanes: ["browser-workspace"],
        confirmationRequiredLanes: ["computer-use"],
        expiresInMinutes: 60,
      }),
    ).toThrow();
  });

  it("rejects snapshots that claim executable authority", () => {
    expect(() =>
      objectiveAuthoritySnapshotSchema.parse({
        schemaVersion: 1,
        objectives: [],
        diagnosticCode: null,
        execution: "authorized",
      }),
    ).toThrow();
  });
});
