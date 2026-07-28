import { describe, expect, it } from "vitest";
import {
  generatedArtifactClaimRequestSchema,
  generatedArtifactCreateRequestSchema,
  generatedArtifactManifestSchema,
} from "./advisorGeneratedArtifact";

describe("generated artifact bridge contract", () => {
  const hash = "a".repeat(64);
  const id = "018f8f8e-2f84-7b11-8a94-966a6e5b8a5b";
  it("accepts only closed classes and path-free creation", () => {
    for (const [artifactClass, filename] of [
      ["text", "reply.txt"],
      ["markdown", "reply.md"],
      ["json", "reply.json"],
      ["csv", "reply.csv"],
      ["python", "reply.py"],
    ] as const)
      expect(() =>
        generatedArtifactCreateRequestSchema.parse({
          class: artifactClass,
          sourceKind: "visible-fenced-block",
          displayLabel: "Visible output",
          suggestedFilename: filename,
          content: "x",
        }),
      ).not.toThrow();
    expect(() =>
      generatedArtifactCreateRequestSchema.parse({
        class: "text",
        sourceKind: "visible-completed-reply",
        displayLabel: "reply",
        suggestedFilename: "../reply.txt",
        content: "x",
      }),
    ).toThrow();
  });
  it("requires UUID/hash claims and rejects path-bearing manifests", () => {
    expect(() =>
      generatedArtifactClaimRequestSchema.parse({
        artifactId: id,
        manifestSha256: hash,
      }),
    ).not.toThrow();
    expect(() =>
      generatedArtifactClaimRequestSchema.parse({
        artifactId: "not-an-id",
        manifestSha256: hash,
      }),
    ).toThrow();
    expect(() =>
      generatedArtifactManifestSchema.parse({
        schemaVersion: 1,
        artifactId: id,
        class: "text",
        mimeType: "text/plain; charset=utf-8",
        sourceKind: "visible-completed-reply",
        displayLabel: "reply",
        suggestedFilename: "reply.txt",
        byteSize: 1,
        sha256: hash,
        createdAt: 1,
        expiresAt: 2,
        state: "ready",
        disposal: "transient-memory-one-successful-save",
        destinationPath: "/tmp/no",
      }),
    ).toThrow();
  });
});
