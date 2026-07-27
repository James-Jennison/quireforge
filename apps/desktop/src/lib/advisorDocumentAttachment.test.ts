import { describe, expect, it } from "vitest";

import { advisorDocumentAttachmentSnapshotSchema } from "./advisorDocumentAttachment";

describe("Advisor document attachment contract", () => {
  const ready = {
    schemaVersion: 1,
    state: "ready" as const,
    attachment: {
      attachmentId: "018f0000-0000-7000-8000-000000000099",
      displayName: "brief.pdf",
      contentCategory: "document" as const,
      mediaType: "pdf" as const,
      byteSize: 42,
      sha256: "a".repeat(64),
      projection: {
        kind: "pdf-plain-text-v1" as const,
        schemaVersion: 1 as const,
        pageCount: 1,
        processedPageCount: 1,
        includedPageCount: 1,
        omittedPageCount: 0,
        partialPageCount: 0,
        projectedByteSize: 12,
        outlineEntryCount: 0,
        truncated: false,
        warnings: [],
      },
      disposal: "transient-memory-one-send" as const,
    },
    confirmationState: "confirmation-required" as const,
    diagnosticCode: null,
  };
  it("accepts only a bounded path-free ready manifest", () => {
    expect(advisorDocumentAttachmentSnapshotSchema.parse(ready)).toEqual(ready);
    expect(() =>
      advisorDocumentAttachmentSnapshotSchema.parse({
        ...ready,
        attachment: { ...ready.attachment, displayName: "/tmp/brief.pdf" },
      }),
    ).toThrow();
  });
  it("rejects an unavailable snapshot with a retained manifest", () => {
    expect(() =>
      advisorDocumentAttachmentSnapshotSchema.parse({
        ...ready,
        state: "unavailable",
        confirmationState: null,
        diagnosticCode: "invalid-content",
      }),
    ).toThrow();
  });
  it("rejects incoherent page accounting", () => {
    expect(() =>
      advisorDocumentAttachmentSnapshotSchema.parse({
        ...ready,
        attachment: {
          ...ready.attachment,
          projection: {
            ...ready.attachment.projection,
            includedPageCount: 2,
          },
        },
      }),
    ).toThrow();
  });
  it("rejects a partial page without bounded truncation evidence", () => {
    expect(() =>
      advisorDocumentAttachmentSnapshotSchema.parse({
        ...ready,
        attachment: {
          ...ready.attachment,
          projection: {
            ...ready.attachment.projection,
            includedPageCount: 0,
            partialPageCount: 1,
          },
        },
      }),
    ).toThrow();
  });
  it("accepts a path-free partial-page truncation record", () => {
    const partial = {
      ...ready,
      attachment: {
        ...ready.attachment,
        projection: {
          ...ready.attachment.projection,
          includedPageCount: 0,
          partialPageCount: 1,
          truncated: true,
          warnings: ["projection-truncated"],
        },
      },
    };
    expect(advisorDocumentAttachmentSnapshotSchema.parse(partial)).toEqual(
      partial,
    );
  });
});
