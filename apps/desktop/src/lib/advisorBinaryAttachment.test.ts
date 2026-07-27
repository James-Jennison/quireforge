import { describe, expect, it } from "vitest";

import {
  advisorBinaryAttachmentSnapshotSchema,
  scaffoldAdvisorBinaryAttachment,
} from "./advisorBinaryAttachment";

describe("Advisor ELF attachment contract", () => {
  it("accepts a bounded path-free static-binary manifest", () => {
    const snapshot = {
      schemaVersion: 1,
      state: "ready" as const,
      attachment: {
        attachmentId: "018f0000-0000-7000-8000-000000000101",
        displayName: "candidate",
        contentCategory: "static-binary" as const,
        mediaType: "elf" as const,
        byteSize: 4096,
        sha256: "a".repeat(64),
        projection: {
          kind: "static-binary-manifest-v1" as const,
          schemaVersion: 1,
          elfClass: "elf64" as const,
          endianness: "little" as const,
          fileType: "executable" as const,
          machine: 62,
          osAbi: 3,
          programHeaderCount: 8,
          sectionHeaderCount: 16,
          dynamicSectionPresent: true,
          dynamicEntryCount: 4,
          manifestByteSize: 256,
        },
        disposal: "transient-memory-one-send" as const,
      },
      confirmationState: "confirmation-required" as const,
      diagnosticCode: null,
    };
    expect(advisorBinaryAttachmentSnapshotSchema.parse(snapshot)).toEqual(
      snapshot,
    );
    expect(JSON.stringify(snapshot)).not.toContain("/");
  });

  it("rejects raw paths and impossible ready states", () => {
    expect(() =>
      advisorBinaryAttachmentSnapshotSchema.parse({
        ...scaffoldAdvisorBinaryAttachment,
        state: "ready",
      }),
    ).toThrow();
  });
});
