import { describe, expect, it } from "vitest";

import {
  advisorTextExportCandidates,
  advisorTextExportRequestSchema,
  advisorTextAttachmentSnapshotSchema,
} from "./advisorAttachment";

describe("Advisor text attachment contracts", () => {
  it("offers only visible bounded reply text and supported fenced data outputs", () => {
    const candidates = advisorTextExportCandidates(
      "Here is a script:\n```python\nprint('safe')\n```\nAnd data:\n```csv\na,b\n1,2\n```",
    );
    expect(candidates).toHaveLength(3);
    expect(candidates.map((candidate) => candidate.contentType)).toEqual([
      "text",
      "python",
      "csv",
    ]);
    expect(
      advisorTextExportRequestSchema.parse({
        suggestedName: candidates[1]!.suggestedName,
        contentType: candidates[1]!.contentType,
        content: candidates[1]!.content,
      }),
    ).toMatchObject({
      suggestedName: "advisor-output.py",
    });
  });

  it("rejects path-shaped names and oversized UI export content", () => {
    expect(() =>
      advisorTextExportRequestSchema.parse({
        suggestedName: "../unsafe.txt",
        contentType: "text",
        content: "no",
      }),
    ).toThrow();
    expect(() =>
      advisorTextExportRequestSchema.parse({
        suggestedName: "answer.txt",
        contentType: "text",
        content: "x".repeat(512 * 1024 + 1),
      }),
    ).toThrow();
  });

  it("keeps F1 confined to the text-data registry entry", () => {
    expect(() =>
      advisorTextAttachmentSnapshotSchema.parse({
        schemaVersion: 1,
        state: "ready",
        confirmationState: "confirmation-required",
        diagnosticCode: null,
        attachment: {
          attachmentId: "018f0000-0000-7000-8000-000000000001",
          displayName: "notes.md",
          contentCategory: "image",
          contentType: "markdown",
          byteSize: 5,
          sha256: "a".repeat(64),
          projection: { kind: "normalized-utf8-text", normalizedByteSize: 5 },
          disposal: "transient-memory-one-send",
        },
      }),
    ).toThrow();
  });
});
