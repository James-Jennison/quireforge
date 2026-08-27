import { describe, expect, it } from "vitest";

import {
  actionCardDecisionRequestSchema,
  actionCardPrepareRequestSchema,
  actionCardSnapshotSchema,
} from "./actionCard";

describe("M69C Action Card contract", () => {
  it("permits only a closed non-executing action class", () => {
    expect(
      actionCardPrepareRequestSchema.parse({ action: "attach-project" }),
    ).toEqual({ action: "attach-project" });
    expect(() =>
      actionCardPrepareRequestSchema.parse({
        action: "attach-project",
        path: "/not-allowed",
      }),
    ).toThrow();
  });

  it("rejects decision payloads with text or capability fields", () => {
    expect(() =>
      actionCardDecisionRequestSchema.parse({
        cardId: "018f0000-0000-7000-8000-000000000000",
        prompt: "do work",
      }),
    ).toThrow();
  });

  it("requires snapshots to state that no data or execution is authorized", () => {
    expect(
      actionCardSnapshotSchema.parse({
        schemaVersion: 1,
        cardId: "018f0000-0000-7000-8000-000000000000",
        action: "work-with-code",
        state: "approved",
        dataScope: "none",
        execution: "not-authorized",
        receiptId: "018f0000-0000-7000-8000-000000000001",
        expiresAtMs: 1,
      }),
    ).toMatchObject({ dataScope: "none", execution: "not-authorized" });
    expect(() =>
      actionCardSnapshotSchema.parse({
        schemaVersion: 1,
        cardId: "018f0000-0000-7000-8000-000000000000",
        action: "work-with-code",
        state: "approved",
        dataScope: "project",
        execution: "not-authorized",
        receiptId: null,
        expiresAtMs: 1,
      }),
    ).toThrow();
  });
});
