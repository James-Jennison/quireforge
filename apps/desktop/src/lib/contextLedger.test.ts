import { describe, expect, it } from "vitest";
import { contextLedgerSnapshotSchema } from "./contextLedger";

const snapshot = {
  schemaVersion: 1,
  projectId: "019fbee6-476f-71b0-853c-f067657aa69c",
  diagnostic: null,
  entries: [
    {
      recordKind: "connector-operation",
      recordId: "019fbee6-476f-71b0-853c-f067657aa69b",
      projectId: "019fbee6-476f-71b0-853c-f067657aa69c",
      taskId: null,
      state: "prepared",
      bundleDigest:
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      itemCount: 0,
      expiresAtMs: 2,
      createdAtMs: 1,
      completedAtMs: null,
      auditOutcome: "prepared",
    },
  ],
};

describe("contextLedgerSnapshotSchema", () => {
  it("accepts only the bounded metadata projection", () => {
    expect(contextLedgerSnapshotSchema.parse(snapshot)).toEqual(snapshot);
  });

  it("rejects content-bearing fields", () => {
    expect(() =>
      contextLedgerSnapshotSchema.parse({
        ...snapshot,
        entries: [
          { ...snapshot.entries[0], content: "must not cross the bridge" },
        ],
      }),
    ).toThrow();
  });
});
