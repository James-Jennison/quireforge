import { describe, expect, it } from "vitest";
import {
  knowledgeLedgerCreateRequestSchema,
  knowledgeLedgerSnapshotSchema,
} from "./knowledgeLedger";

const projectId = "019fbee6-476f-71b0-853c-f067657aa69c";

describe("knowledge ledger contracts", () => {
  it("accepts a bounded owner decision snapshot", () => {
    expect(
      knowledgeLedgerSnapshotSchema.parse({
        schemaVersion: 1,
        records: [
          {
            id: "019fbee6-476f-71b0-853c-f067657aa69b",
            projectId,
            taskId: null,
            kind: "owner-decision",
            status: "active",
            title: "Linux only",
            body: "The product is Linux only.",
            supersedesId: null,
            createdAtMs: 1,
            updatedAtMs: 2,
          },
        ],
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
});
