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
      artifactVerification: "metadata-only",
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

  it("accepts strict package and validation evidence but rejects extra fields", () => {
    const snapshot = {
      schemaVersion: 1,
      state: projectStateFixtures.minimalValid,
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
        packages: [
          {
            manifestVersion: 1,
            kind: "deb",
            sourceCommit: "0123456789abcdef0123456789abcdef01234567",
            artifactPath: "target/pkg.deb",
            checksum: "a".repeat(64),
            checksumFile: "a".repeat(64),
            localVerified: true,
            localPresent: true,
            declaredSize: 1,
            freshness: "current",
          },
        ],
        validations: [
          {
            version: 1,
            id: "rust-tests",
            family: "rust-tests",
            status: "passed",
            sourceCommit: "0123456789abcdef0123456789abcdef01234567",
            evidencePath: "target/validation-summary.json",
            operation: "cargo-test",
            timestamp: "2026-01-01T00:00:00Z",
            freshness: "current",
          },
        ],
        handoff: {
          status: "checkpoint-pushed",
          phrase: "Codex checkpoint pushed. Continue.",
          sourceCommit: "0123456789abcdef0123456789abcdef01234567",
          freshness: "current",
        },
      },
      diagnostics: [],
    };
    expect(
      repositoryStateReadSnapshotSchema.parse(snapshot).evidence.packages,
    ).toHaveLength(1);
    expect(() =>
      repositoryStateReadSnapshotSchema.parse({
        ...snapshot,
        evidence: {
          ...snapshot.evidence,
          packages: [{ ...snapshot.evidence.packages[0], unexpected: true }],
        },
      }),
    ).toThrow();
  });
});
