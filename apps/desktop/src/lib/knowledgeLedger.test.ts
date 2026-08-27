import { describe, expect, it } from "vitest";
import {
  knowledgeLedgerCreateRequestSchema,
  knowledgeEvidenceLinkCreateRequestSchema,
  knowledgeLedgerSnapshotSchema,
  knowledgeLedgerTransitionRequestSchema,
} from "./knowledgeLedger";

const projectId = "019fbee6-476f-71b0-853c-f067657aa69c";

describe("knowledge ledger contracts", () => {
  it("accepts a bounded owner decision snapshot", () => {
    expect(
      knowledgeLedgerSnapshotSchema.parse({
        schemaVersion: 2,
        records: [
          {
            id: "019fbee6-476f-71b0-853c-f067657aa69b",
            projectId,
            taskId: null,
            kind: "owner-decision",
            provenance: "owner",
            status: "active",
            title: "Linux only",
            body: "The product is Linux only.",
            supersedesId: null,
            createdAtMs: 1,
            updatedAtMs: 2,
          },
        ],
        evidenceLinks: [],
        evidenceConclusions: [],
        diagnosticCode: null,
      }).records[0]!.status,
    ).toBe("active");
  });

  it("rejects unknown fields and unbounded bodies", () => {
    expect(() =>
      knowledgeLedgerCreateRequestSchema.parse({
        projectId,
        kind: "assumption",
        title: "Bounded",
        body: "x",
        execution: "forbidden",
      }),
    ).toThrow();
    expect(() =>
      knowledgeLedgerCreateRequestSchema.parse({
        projectId,
        kind: "assumption",
        title: "Bounded",
        body: "x".repeat(8193),
      }),
    ).toThrow();
  });

  it("accepts only a strict lifecycle transition request", () => {
    expect(
      knowledgeLedgerTransitionRequestSchema.parse({
        recordId: "019fbee6-476f-71b0-853c-f067657aa69b",
        status: "pending-owner-binding",
      }).status,
    ).toBe("pending-owner-binding");
    expect(() =>
      knowledgeLedgerTransitionRequestSchema.parse({
        recordId: "019fbee6-476f-71b0-853c-f067657aa69b",
        status: "active",
        bypass: true,
      }),
    ).toThrow();
  });

  it("permits only bounded owner-trial evidence without a source id", () => {
    expect(
      knowledgeEvidenceLinkCreateRequestSchema.parse({
        recordId: "019fbee6-476f-71b0-853c-f067657aa69b",
        kind: "owner-trial",
        ownerTrialKind: "device",
        ownerTrialResult: "passed",
      }).kind,
    ).toBe("owner-trial");
    expect(() =>
      knowledgeEvidenceLinkCreateRequestSchema.parse({
        recordId: "019fbee6-476f-71b0-853c-f067657aa69b",
        kind: "owner-trial",
        sourceId: "019fbee6-476f-71b0-853c-f067657aa69c",
        ownerTrialKind: "device",
        ownerTrialResult: "passed",
      }),
    ).toThrow();
  });
});
