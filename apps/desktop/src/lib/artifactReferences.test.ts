import { describe, expect, it } from "vitest";

import {
  artifactReferencePrepareRequestSchema,
  artifactReferenceSnapshotSchema,
} from "./artifactReferences";

const projectId = "018f0000-0000-7000-8000-000000000001";
const artifactId = "018f0000-0000-7000-8000-000000000002";
const sha256 = "a".repeat(64);

describe("artifact reference contracts", () => {
  it("accepts only the opaque digest-bound preparation envelope", () => {
    expect(
      artifactReferencePrepareRequestSchema.parse({
        projectId,
        taskId: null,
        artifactId,
        artifactSha256: sha256,
      }),
    ).toMatchObject({ projectId, artifactId, artifactSha256: sha256 });
    expect(() =>
      artifactReferencePrepareRequestSchema.parse({
        projectId,
        artifactId,
        artifactSha256: sha256,
        path: "/private/artifact.txt",
      }),
    ).toThrow();
  });

  it("projects an independently unavailable original without content fields", () => {
    const snapshot = artifactReferenceSnapshotSchema.parse({
      schemaVersion: 1,
      references: [
        {
          referenceId: "018f0000-0000-7000-8000-000000000003",
          projectId,
          taskId: null,
          artifactId,
          artifactSha256: sha256,
          artifactClass: "markdown",
          displayLabel: "Reviewed outline",
          state: "active",
          availability: "unavailable",
          createdAtMs: 1,
        },
      ],
      diagnosticCode: null,
    });
    expect(snapshot.references[0]).not.toHaveProperty("preview");
    expect(snapshot.references[0]).not.toHaveProperty("path");
  });
});
