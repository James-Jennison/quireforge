import { describe, expect, it } from "vitest";

import {
  advisorArchiveAttachmentSnapshotSchema,
  scaffoldAdvisorArchiveAttachment,
} from "./advisorArchiveAttachment";

const ready = {
  schemaVersion: 1,
  state: "ready" as const,
  attachment: {
    attachmentId: "018f0000-0000-7000-8000-000000000001",
    displayName: "source.zip",
    contentCategory: "archive" as const,
    mediaType: "zip" as const,
    byteSize: 128,
    sha256: "a".repeat(64),
    projection: {
      kind: "archive-manifest-v1" as const,
      schemaVersion: 1,
      discoveredEntryCount: 2,
      includedEntryCount: 2,
      omittedEntryCount: 0,
      declaredAggregateUncompressedBytes: 24,
      manifestByteSize: 120,
      truncated: false,
      warnings: [],
    },
    disposal: "transient-memory-one-send" as const,
  },
  entries: [
    {
      name: "docs/readme.txt",
      kind: "file" as const,
      compressedSize: 8,
      declaredUncompressedSize: 16,
      nestedArchiveLike: false,
    },
    {
      name: "notes.zip",
      kind: "file" as const,
      compressedSize: 4,
      declaredUncompressedSize: 8,
      nestedArchiveLike: true,
    },
  ],
  confirmationState: "confirmation-required" as const,
  diagnosticCode: null,
};

describe("Advisor archive attachment contract", () => {
  it("accepts a metadata-only path-free ZIP manifest", () => {
    expect(advisorArchiveAttachmentSnapshotSchema.parse(ready)).toEqual(ready);
    expect(
      advisorArchiveAttachmentSnapshotSchema.parse(
        scaffoldAdvisorArchiveAttachment,
      ),
    ).toEqual(scaffoldAdvisorArchiveAttachment);
  });

  it("rejects raw paths, entry content fields, and inconsistent accounting", () => {
    expect(() =>
      advisorArchiveAttachmentSnapshotSchema.parse({
        ...ready,
        attachment: { ...ready.attachment, displayName: "/tmp/source.zip" },
      }),
    ).toThrow();
    expect(() =>
      advisorArchiveAttachmentSnapshotSchema.parse({
        ...ready,
        entries: [{ ...ready.entries[0], content: "not permitted" }],
      }),
    ).toThrow();
    expect(() =>
      advisorArchiveAttachmentSnapshotSchema.parse({
        ...ready,
        attachment: {
          ...ready.attachment,
          projection: { ...ready.attachment.projection, includedEntryCount: 1 },
        },
      }),
    ).toThrow();
  });
});
