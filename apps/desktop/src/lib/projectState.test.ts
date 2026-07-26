import { describe, expect, it } from "vitest";

import {
  checkpointStateSchema,
  projectStateContractSchema,
  projectStateFixtures,
  scaffoldProjectState,
} from "./projectState";

describe("project-state contract", () => {
  it("parses the shared active-milestone fixture", () => {
    expect(
      projectStateContractSchema.parse(projectStateFixtures.activeMilestone),
    ).toEqual(scaffoldProjectState);
  });

  it("parses the minimal valid state without inventing missing facts", () => {
    expect(
      projectStateContractSchema.parse(projectStateFixtures.minimalValid),
    ).toEqual(projectStateFixtures.minimalValid);
  });

  it("parses pushed, paused, completed, missing, and contradictory evidence", () => {
    const pushed = structuredClone(scaffoldProjectState);
    pushed.checkpoints = [projectStateFixtures.pushedCheckpoint];
    const paused = structuredClone(scaffoldProjectState);
    paused.workSessions = [projectStateFixtures.pausedSession];
    const completed = structuredClone(scaffoldProjectState);
    completed.checkpoints = [projectStateFixtures.completedMilestone];
    completed.milestone.status = "complete";
    const missing = structuredClone(scaffoldProjectState);
    missing.packages = projectStateFixtures.missingEvidence;
    const missingValidation = structuredClone(scaffoldProjectState);
    missingValidation.validations = [
      projectStateFixtures.missingValidationEvidence,
    ];
    const contradictory = structuredClone(scaffoldProjectState);
    contradictory.contradictions = projectStateFixtures.contradictoryEvidence;

    for (const state of [
      pushed,
      paused,
      completed,
      missing,
      missingValidation,
      contradictory,
    ]) {
      expect(projectStateContractSchema.parse(state)).toEqual(state);
    }
    expect(contradictory.contradictions.map(({ kind }) => kind)).toEqual(
      expect.arrayContaining([
        "branch-mismatch",
        "completion-without-pushed-commit",
        "policy-scope-conflict",
      ]),
    );
  });

  it("fails closed for versions, unknown fields, identities, approvals, and completion", () => {
    expect(() =>
      projectStateContractSchema.parse({
        ...projectStateFixtures.activeMilestone,
        schemaVersion: 2,
      }),
    ).toThrow();
    expect(() =>
      projectStateContractSchema.parse({
        ...projectStateFixtures.activeMilestone,
        project: {
          ...projectStateFixtures.activeMilestone.project,
          id: "invalid id",
        },
      }),
    ).toThrow();
    expect(() =>
      projectStateContractSchema.parse({
        ...projectStateFixtures.activeMilestone,
        unexpected: true,
      }),
    ).toThrow();
    expect(() =>
      projectStateContractSchema.parse({
        ...projectStateFixtures.activeMilestone,
        milestone: {
          ...projectStateFixtures.activeMilestone.milestone,
          ownerApproval: {
            ...projectStateFixtures.activeMilestone.milestone.ownerApproval,
            decision: "approved",
            authority: null,
          },
        },
      }),
    ).toThrow();
    expect(() =>
      checkpointStateSchema.parse({
        ...projectStateFixtures.activeMilestone.checkpoints[0],
        status: "finished",
        completionClaimed: true,
      }),
    ).toThrow();
    const invalidTrust = structuredClone(projectStateFixtures.activeMilestone);
    invalidTrust.provenance.trust = "unsupported" as "verified";
    expect(() => projectStateContractSchema.parse(invalidTrust)).toThrow();
  });
});
