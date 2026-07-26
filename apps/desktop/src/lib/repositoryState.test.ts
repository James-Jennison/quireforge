import { describe, expect, it } from "vitest";

import { projectStateFixtures } from "./projectState";
import {
  repositoryStateReadRequestSchema,
  repositoryStateReadSnapshotSchema,
} from "./repositoryState";

describe("repository-state reader bridge contract", () => {
  it("accepts a local-only attached-project request", () => {
    expect(
      repositoryStateReadRequestSchema.parse({
        projectId: "018f0000-0000-7000-8000-000000000001",
        remoteMode: "local-only",
      }),
    ).toEqual({
      projectId: "018f0000-0000-7000-8000-000000000001",
      remoteMode: "local-only",
    });
  });

  it("preserves a local head when remote evidence is unavailable", () => {
    const state = structuredClone(projectStateFixtures.activeMilestone);
    state.repository.localHead = "0123456789abcdef0123456789abcdef01234567";
    state.repository.remoteHead = null;
    state.repository.ahead = null;
    state.repository.behind = null;
    expect(
      repositoryStateReadSnapshotSchema.parse({
        schemaVersion: 1,
        state,
        git: {
          upstream: null,
          detached: false,
          stagedCount: 0,
          unstagedCount: 0,
          untrackedCount: 0,
          mergeInProgress: false,
          rebaseInProgress: false,
          cherryPickInProgress: false,
          bisectInProgress: false,
          shallow: false,
        },
        evidence: {
          packages: [],
          validations: [],
          handoff: null,
        },
        diagnostics: [],
      }).state.repository,
    ).toMatchObject({
      localHead: "0123456789abcdef0123456789abcdef01234567",
      remoteHead: null,
    });
  });

  it("fails closed for arbitrary paths and unknown diagnostics", () => {
    expect(() =>
      repositoryStateReadRequestSchema.parse({
        projectId: "not-a-project-id",
        remoteMode: "local-only",
      }),
    ).toThrow();
    expect(() =>
      repositoryStateReadSnapshotSchema.parse({
        schemaVersion: 1,
        state: projectStateFixtures.minimalValid,
        git: {},
        diagnostics: [{ id: "unexpected", severity: "fatal" }],
      }),
    ).toThrow();
  });
});
