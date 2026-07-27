import { describe, expect, it } from "vitest";

import { advisorImageAttachmentSnapshotSchema } from "./advisorImageAttachment";

const ready = {
  schemaVersion: 1,
  state: "ready",
  attachment: {
    attachmentId: "018f0000-0000-7000-8000-000000000099",
    displayName: "diagram.png",
    contentCategory: "image",
    mediaType: "png",
    byteSize: 42,
    width: 1,
    height: 1,
    sha256: "a".repeat(64),
    projection: { kind: "local-image", width: 1, height: 1 },
    disposal: "transient-memory-one-send",
  },
  previewDataUrl: "data:image/png;base64,AAAA",
  confirmationState: "confirmation-required",
  diagnosticCode: null,
};

describe("Advisor image attachment contract", () => {
  it("accepts only a consistent path-free bounded image manifest", () => {
    expect(advisorImageAttachmentSnapshotSchema.parse(ready)).toEqual(ready);
  });
  it("rejects unknown fields and mismatched preview dimensions", () => {
    expect(() =>
      advisorImageAttachmentSnapshotSchema.parse({
        ...ready,
        sourcePath: "/tmp/a",
      }),
    ).toThrow();
    expect(() =>
      advisorImageAttachmentSnapshotSchema.parse({
        ...ready,
        attachment: {
          ...ready.attachment,
          projection: { kind: "local-image", width: 2, height: 1 },
        },
      }),
    ).toThrow();
  });
});
