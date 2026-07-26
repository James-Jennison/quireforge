import { describe, expect, it } from "vitest";

import fixture from "../../fixtures/advisor-foundation.json";
import {
  advisorDispatchProposalSchema,
  advisorFoundationSnapshotSchema,
} from "./advisor";

const advisorFoundationFixture = advisorFoundationSnapshotSchema.parse(fixture);

describe("reference-only Advisor foundation contract", () => {
  it("accepts the shared Rust/TypeScript fixture without prompt or transcript retention", () => {
    expect(advisorFoundationFixture.schemaVersion).toBe(1);
    expect(JSON.stringify(advisorFoundationFixture)).not.toContain('"prompt":');
    expect(JSON.stringify(advisorFoundationFixture)).not.toContain(
      '"transcript":',
    );
  });

  it("rejects unknown fields and unsafe context references", () => {
    expect(() =>
      advisorFoundationSnapshotSchema.parse({ ...fixture, unexpected: true }),
    ).toThrow();
    const unsafe = structuredClone(fixture);
    const [contextReference] = unsafe.contextReferences;
    if (!contextReference)
      throw new Error("fixture context reference is required");
    contextReference.sourceRef = "../../etc/passwd";
    expect(() => advisorFoundationSnapshotSchema.parse(unsafe)).toThrow();
  });

  it("requires explicit approval and fixed digests for a future dispatch", () => {
    const proposal = structuredClone(fixture.dispatchProposals[0]);
    expect(() =>
      advisorDispatchProposalSchema.parse({
        ...proposal,
        requiresExplicitApproval: false,
      }),
    ).toThrow();
    expect(() =>
      advisorDispatchProposalSchema.parse({
        ...proposal,
        promptSha256: "not-a-digest",
      }),
    ).toThrow();
    expect(() =>
      advisorDispatchProposalSchema.parse({
        ...proposal,
        promptSha256: "A".repeat(64),
      }),
    ).toThrow();
  });

  it("rejects context or dispatch records that are not owned by an Advisor conversation", () => {
    const orphanedContext = structuredClone(fixture);
    const [contextReference] = orphanedContext.contextReferences;
    if (!contextReference)
      throw new Error("fixture context reference is required");
    contextReference.advisorConversationId =
      "019d4e3c-3b19-7e50-9f35-3f27d4f7aa15";
    expect(() =>
      advisorFoundationSnapshotSchema.parse(orphanedContext),
    ).toThrow();

    const orphanedProposal = structuredClone(fixture);
    const [proposal] = orphanedProposal.dispatchProposals;
    if (!proposal) throw new Error("fixture dispatch proposal is required");
    proposal.advisorConversationId = "019d4e3c-3b20-7e50-9f35-3f27d4f7aa16";
    expect(() =>
      advisorFoundationSnapshotSchema.parse(orphanedProposal),
    ).toThrow();
  });
});
