import { describe, expect, it, vi } from "vitest";
import {
  LOCAL_REVIEW_IMAGE_PICK_COMMAND,
  LOCAL_REVIEW_IMAGE_PREVIEW_COMMAND,
  pickLocalReviewImage,
  previewLocalReviewImage,
  previewLocalReviewText,
  LOCAL_REVIEW_TEXT_PREVIEW_COMMAND,
  createLocalReviewManualEvidence,
  createLocalReviewM48ArtifactCopy,
  createLocalReviewM48GeneratedArtifactMetadataEvidence,
  LOCAL_REVIEW_M48_ARTIFACT_COPY_COMMAND,
  LOCAL_REVIEW_M48_GENERATED_ARTIFACT_METADATA_EVIDENCE_CREATE_COMMAND,
  previewLocalReviewManualEvidence,
  previewLocalReviewM48GeneratedArtifactMetadataEvidence,
  LOCAL_REVIEW_MANUAL_EVIDENCE_CREATE_COMMAND,
  LOCAL_REVIEW_MANUAL_EVIDENCE_PREVIEW_COMMAND,
  LOCAL_REVIEW_M48_GENERATED_ARTIFACT_METADATA_EVIDENCE_PREVIEW_COMMAND,
  LOCAL_REVIEW_ANNOTATION_CREATE_COMMAND,
  LOCAL_REVIEW_ANNOTATION_EDIT_COMMAND,
  LOCAL_REVIEW_ANNOTATION_RESOLVE_COMMAND,
  LOCAL_REVIEW_ANNOTATION_REOPEN_COMMAND,
  LOCAL_REVIEW_ANNOTATION_DELETE_COMMAND,
  createLocalReviewAnnotation,
  editLocalReviewAnnotation,
  resolveLocalReviewAnnotation,
  reopenLocalReviewAnnotation,
  deleteLocalReviewAnnotation,
  LOCAL_REVIEW_COMPARISON_CREATE_COMMAND,
  LOCAL_REVIEW_COMPARISON_READ_COMMAND,
  LOCAL_REVIEW_COMPARISON_DISCARD_COMMAND,
  createLocalReviewComparison,
  readLocalReviewComparison,
  discardLocalReviewComparison,
  LOCAL_REVIEW_PROMOTION_PREPARE_COMMAND,
  LOCAL_REVIEW_PROMOTION_CONFIRM_COMMAND,
  LOCAL_REVIEW_PROMOTION_CANCEL_COMMAND,
  prepareLocalReviewPromotion,
  confirmLocalReviewPromotion,
  cancelLocalReviewPromotion,
  createLocalReviewPackageManifestSummaryEvidence,
  LOCAL_REVIEW_PACKAGE_MANIFEST_SUMMARY_EVIDENCE_CREATE_COMMAND,
} from "./bridge";

const id = "018f0000-0000-7000-8000-000000000001";
const sha = "a".repeat(64);
const annotationId = "018f0000-0000-7000-8000-000000000002";
const snapshot = {
  schemaVersion: 1,
  collections: [],
  selectedCollection: null,
  items: [],
  comparisons: [],
  collectionCount: 0,
  payloadBytes: 0,
  warning: false,
  packageManifestSummaryAvailable: false,
  gitStatusDiffSummaryAvailable: false,
  diagnosticCode: null,
};
const annotationSnapshot = {
  schemaVersion: 1,
  collections: [
    {
      collectionId: id,
      taskId: id,
      planId: null,
      title: "Collection",
      state: "active",
      itemCount: 1,
      payloadBytes: 1,
      updatedAtMs: 1,
      warning: false,
      annotationCountWarning: false,
      annotationByteWarning: false,
      comparisonCountWarning: false,
    },
  ],
  selectedCollection: {
    collectionId: id,
    taskId: id,
    planId: null,
    title: "Collection",
    state: "active",
    itemCount: 1,
    payloadBytes: 1,
    updatedAtMs: 1,
    warning: false,
    annotationCountWarning: false,
    annotationByteWarning: false,
    comparisonCountWarning: false,
  },
  items: [
    {
      itemId: id,
      class: "text",
      textFormat: "plain",
      sourceKind: "user-authored-text",
      state: "ready",
      title: "Item",
      mimeType: "text/plain; charset=utf-8",
      width: null,
      height: null,
      byteSize: 1,
      lineCount: 1,
      sha256: sha,
      createdAtMs: 0,
      annotations: [
        {
          schemaVersion: 1,
          annotationId,
          itemId: id,
          text: "note",
          state: "open",
          createdAtMs: 0,
          updatedAtMs: 0,
        },
      ],
    },
  ],
  comparisons: [],
  collectionCount: 1,
  payloadBytes: 1,
  warning: false,
  packageManifestSummaryAvailable: false,
  gitStatusDiffSummaryAvailable: false,
  diagnosticCode: null,
};

describe("local review image bridge", () => {
  it("uses the strict native-owned package summary envelope", async () => {
    const request = { collectionId: id, expectedCollectionUpdatedAtMs: 1 };
    const response = { outcome: "failed", snapshot };
    const invoke = vi.fn().mockResolvedValue(response);
    await expect(createLocalReviewPackageManifestSummaryEvidence(request, invoke)).resolves.toEqual(response);
    expect(invoke).toHaveBeenCalledWith(LOCAL_REVIEW_PACKAGE_MANIFEST_SUMMARY_EVIDENCE_CREATE_COMMAND, { request });
  });
  it("uses the fixed digest-bound text preview envelope", async () => {
    const request = { collectionId: id, itemId: annotationId, sha256: sha };
    const preview = {
      schemaVersion: 1,
      collectionId: id,
      itemId: annotationId,
      title: "Text",
      textFormat: "plain",
      byteSize: 4,
      sha256: sha,
      createdAtMs: 0,
      state: "ready",
      text: "text",
      projectedByteSize: 4,
      projectedLineCount: 1,
      projectedCodePointCount: 4,
      truncated: false,
      diagnosticCode: null,
    };
    const invoke = vi.fn().mockResolvedValue(preview);
    await expect(
      previewLocalReviewText(request, invoke),
    ).resolves.toMatchObject(preview);
    expect(invoke).toHaveBeenCalledWith(LOCAL_REVIEW_TEXT_PREVIEW_COMMAND, {
      request,
    });
    expect(JSON.stringify(invoke.mock.calls[0]?.[1])).not.toMatch(
      /path|url|content|format|source|command|shell|terminal|git|provider|connector|approval|dispatch|execution|operation/i,
    );
    await expect(
      previewLocalReviewText(
        request,
        vi.fn().mockResolvedValue({ ...preview, path: "/tmp/x" }),
      ),
    ).rejects.toThrow();
  });
  it("uses the fixed native-owned M48 artifact copy envelope", async () => {
    const request = {
      collectionId: id,
      expectedCollectionUpdatedAtMs: 1,
      artifactId: annotationId,
      manifestSha256: sha,
    };
    const invoke = vi.fn().mockResolvedValue(snapshot);
    await createLocalReviewM48ArtifactCopy(request, invoke);
    expect(invoke).toHaveBeenCalledWith(
      LOCAL_REVIEW_M48_ARTIFACT_COPY_COMMAND,
      { request },
    );
    expect(JSON.stringify(invoke.mock.calls[0]?.[1])).not.toMatch(
      /path|filename|directory|url|content|textFormat|sourceKind|taskId|provenance|command|git|shell|terminal|provider|connector|approval|dispatch|execution|operation/i,
    );
    await expect(
      createLocalReviewM48ArtifactCopy(
        request,
        vi.fn().mockResolvedValue({ ...snapshot, unknown: true }),
      ),
    ).rejects.toThrow();
  });
  it("uses fixed path-free picker and preview envelopes", async () => {
    const invoke = vi
      .fn()
      .mockResolvedValueOnce({ outcome: "canceled", snapshot })
      .mockResolvedValueOnce({
        schemaVersion: 1,
        itemId: id,
        mimeType: "image/png",
        width: 1,
        height: 1,
        byteSize: 1,
        sha256: sha,
        dataUrl: "data:image/png;base64,AA==",
      });
    await pickLocalReviewImage(
      { collectionId: id, expectedCollectionUpdatedAtMs: 1, title: "Mockup" },
      invoke,
    );
    await previewLocalReviewImage({ itemId: id, sha256: sha }, invoke);
    expect(invoke).toHaveBeenNthCalledWith(1, LOCAL_REVIEW_IMAGE_PICK_COMMAND, {
      request: {
        collectionId: id,
        expectedCollectionUpdatedAtMs: 1,
        title: "Mockup",
      },
    });
    expect(invoke).toHaveBeenNthCalledWith(
      2,
      LOCAL_REVIEW_IMAGE_PREVIEW_COMMAND,
      { request: { itemId: id, sha256: sha } },
    );
  });
  it("rejects malformed native responses", async () => {
    await expect(
      pickLocalReviewImage(
        { collectionId: id, expectedCollectionUpdatedAtMs: 1, title: "Mockup" },
        vi.fn().mockResolvedValue({ outcome: "created", path: "/tmp/x" }),
      ),
    ).rejects.toThrow();
  });
  it("uses strict manual evidence envelopes", async () => {
    const created = {
      ...snapshot,
      items: [
        {
          itemId: id,
          class: "evidence",
          textFormat: null,
          sourceKind: "typed-evidence-snapshot",
          evidenceSource: "manual-validation-summary",
          state: "ready",
          title: "Validation",
          mimeType: "application/json; profile=evidence-envelope-v1",
          width: null,
          height: null,
          byteSize: 12,
          lineCount: null,
          sha256: sha,
          createdAtMs: 0,
          annotations: [],
        },
      ],
    };
    const invoke = vi
      .fn()
      .mockResolvedValueOnce({
        outcome: "created",
        createdItemId: id,
        source: "manual-validation-summary",
        snapshot: created,
      })
      .mockResolvedValueOnce({
        schemaVersion: 1,
        itemId: id,
        source: "manual-validation-summary",
        title: "Validation",
        summary: "line\nsummary",
        byteSize: 12,
        sha256: sha,
        createdAtMs: 0,
      });
    await createLocalReviewManualEvidence(
      {
        collectionId: id,
        expectedCollectionUpdatedAtMs: 1,
        title: "Validation",
        summary: "line\r\nsummary",
      },
      invoke,
    );
    await previewLocalReviewManualEvidence({ itemId: id, sha256: sha }, invoke);
    expect(invoke).toHaveBeenNthCalledWith(
      1,
      LOCAL_REVIEW_MANUAL_EVIDENCE_CREATE_COMMAND,
      {
        request: {
          collectionId: id,
          expectedCollectionUpdatedAtMs: 1,
          title: "Validation",
          summary: "line\r\nsummary",
        },
      },
    );
    expect(invoke).toHaveBeenNthCalledWith(
      2,
      LOCAL_REVIEW_MANUAL_EVIDENCE_PREVIEW_COMMAND,
      { itemId: id, sha256: sha },
    );
  });
  it("uses fixed metadata-only M48 evidence capture and preview envelopes", async () => {
    const request = {
      collectionId: id,
      expectedCollectionUpdatedAtMs: 1,
      artifactId: annotationId,
      manifestSha256: sha,
    };
    const created = {
      ...snapshot,
      items: [
        {
          itemId: id,
          class: "evidence",
          textFormat: null,
          sourceKind: "typed-evidence-snapshot",
          evidenceSource: "m48-generated-artifact-metadata",
          state: "ready",
          title: "Generated artifact metadata: Safe",
          mimeType: "application/json; profile=evidence-envelope-v1",
          width: null,
          height: null,
          byteSize: 12,
          lineCount: null,
          sha256: sha,
          createdAtMs: 0,
          annotations: [],
        },
      ],
    };
    const preview = {
      schemaVersion: 1,
      itemId: id,
      source: "m48-generated-artifact-metadata",
      title: "Generated artifact metadata: Safe",
      summary: "Captured live generated-artifact metadata only.",
      details: {
        artifactState: "ready",
        artifactKind: "text",
        format: "plain",
        byteLength: 4,
        truncated: false,
        manifestSha256: sha,
      },
      byteSize: 12,
      sha256: sha,
      createdAtMs: 0,
    };
    const invoke = vi
      .fn()
      .mockResolvedValueOnce({
        outcome: "created",
        createdItemId: id,
        source: "m48-generated-artifact-metadata",
        snapshot: created,
      })
      .mockResolvedValueOnce(preview);
    await createLocalReviewM48GeneratedArtifactMetadataEvidence(
      request,
      invoke,
    );
    await previewLocalReviewM48GeneratedArtifactMetadataEvidence(
      { itemId: id, sha256: sha },
      invoke,
    );
    expect(invoke).toHaveBeenNthCalledWith(
      1,
      LOCAL_REVIEW_M48_GENERATED_ARTIFACT_METADATA_EVIDENCE_CREATE_COMMAND,
      { request },
    );
    expect(invoke).toHaveBeenNthCalledWith(
      2,
      LOCAL_REVIEW_M48_GENERATED_ARTIFACT_METADATA_EVIDENCE_PREVIEW_COMMAND,
      { itemId: id, sha256: sha },
    );
    expect(JSON.stringify(invoke.mock.calls[0]?.[1])).not.toMatch(
      /path|filename|url|content|format|state|title|summary|approval|dispatch|execution|git|command|operation/i,
    );
    await expect(
      createLocalReviewM48GeneratedArtifactMetadataEvidence(
        request,
        vi.fn().mockResolvedValue({ ...created, unknown: true }),
      ),
    ).rejects.toThrow();
  });
  it("rejects a created evidence result whose identity or source is not in its snapshot", async () => {
    const invoke = vi.fn().mockResolvedValue({
      outcome: "created",
      createdItemId: id,
      source: "manual-validation-summary",
      snapshot,
    });
    await expect(
      createLocalReviewManualEvidence(
        {
          collectionId: id,
          expectedCollectionUpdatedAtMs: 1,
          title: "Validation",
          summary: "ok",
        },
        invoke,
      ),
    ).rejects.toThrow();
  });
  it("uses fixed strict annotation mutation envelopes", async () => {
    const create = {
      collectionId: id,
      itemId: id,
      expectedCollectionUpdatedAtMs: 1,
      text: "line\r\ntext",
    };
    const edit = { ...create, annotationId, text: "edited\r\ntext" };
    const mutation = {
      collectionId: id,
      itemId: id,
      annotationId,
      expectedCollectionUpdatedAtMs: 1,
    };
    const invoke = vi.fn().mockResolvedValue(annotationSnapshot);
    await createLocalReviewAnnotation(create, invoke);
    await editLocalReviewAnnotation(edit, invoke);
    await resolveLocalReviewAnnotation(mutation, invoke);
    await reopenLocalReviewAnnotation(mutation, invoke);
    await deleteLocalReviewAnnotation(mutation, invoke);
    expect(invoke).toHaveBeenNthCalledWith(
      1,
      LOCAL_REVIEW_ANNOTATION_CREATE_COMMAND,
      { request: create },
    );
    expect(invoke).toHaveBeenNthCalledWith(
      2,
      LOCAL_REVIEW_ANNOTATION_EDIT_COMMAND,
      { request: edit },
    );
    expect(invoke).toHaveBeenNthCalledWith(
      3,
      LOCAL_REVIEW_ANNOTATION_RESOLVE_COMMAND,
      { request: mutation },
    );
    expect(invoke).toHaveBeenNthCalledWith(
      4,
      LOCAL_REVIEW_ANNOTATION_REOPEN_COMMAND,
      { request: mutation },
    );
    expect(invoke).toHaveBeenNthCalledWith(
      5,
      LOCAL_REVIEW_ANNOTATION_DELETE_COMMAND,
      { request: mutation },
    );
    for (const [, envelope] of invoke.mock.calls) {
      expect(JSON.stringify(envelope)).not.toMatch(
        /path|filename|directory|url|operation|shell|terminal|git|provider|connector|approval|dispatch|execution/i,
      );
    }
  });
  it("rejects malformed annotation snapshots before returning them", async () => {
    await expect(
      createLocalReviewAnnotation(
        {
          collectionId: id,
          itemId: id,
          expectedCollectionUpdatedAtMs: 1,
          text: "note",
        },
        vi.fn().mockResolvedValue({ ...snapshot, unknown: true }),
      ),
    ).rejects.toThrow();
  });
  it("uses fixed, strict comparison command envelopes", async () => {
    const comparisonId = "018f0000-0000-7000-8000-000000000003";
    const create = {
      collectionId: id,
      leftItemId: id,
      rightItemId: annotationId,
      expectedCollectionUpdatedAtMs: 1,
    };
    const read = { collectionId: id, comparisonId };
    const discard = { ...read, expectedCollectionUpdatedAtMs: 1 };
    const lines = {
      comparisonId,
      leftItemId: id,
      leftSha256: sha,
      rightItemId: annotationId,
      rightSha256: "b".repeat(64),
      textFormat: "plain",
      state: "ready",
      lines: [
        {
          kind: "unchanged",
          text: "same",
          leftLineNumber: 1,
          rightLineNumber: 1,
        },
      ],
    };
    const invoke = vi
      .fn()
      .mockResolvedValueOnce(annotationSnapshot)
      .mockResolvedValueOnce(lines)
      .mockResolvedValueOnce(annotationSnapshot);
    await createLocalReviewComparison(create, invoke);
    await readLocalReviewComparison(read, invoke);
    await discardLocalReviewComparison(discard, invoke);
    expect(invoke).toHaveBeenNthCalledWith(
      1,
      LOCAL_REVIEW_COMPARISON_CREATE_COMMAND,
      { request: create },
    );
    expect(invoke).toHaveBeenNthCalledWith(
      2,
      LOCAL_REVIEW_COMPARISON_READ_COMMAND,
      { request: read },
    );
    expect(invoke).toHaveBeenNthCalledWith(
      3,
      LOCAL_REVIEW_COMPARISON_DISCARD_COMMAND,
      { request: discard },
    );
    for (const [, envelope] of invoke.mock.calls)
      expect(JSON.stringify(envelope)).not.toMatch(
        /path|url|git|repository|command|shell|terminal|provider|connector|approval|dispatch|execution|operation/i,
      );
  });
  it("rejects malformed comparison native responses", async () => {
    const comparisonId = "018f0000-0000-7000-8000-000000000003";
    await expect(
      readLocalReviewComparison(
        { collectionId: id, comparisonId },
        vi.fn().mockResolvedValue({ comparisonId, path: "/tmp/x" }),
      ),
    ).rejects.toThrow();
    await expect(
      createLocalReviewComparison(
        {
          collectionId: id,
          leftItemId: id,
          rightItemId: annotationId,
          expectedCollectionUpdatedAtMs: 1,
        },
        vi.fn().mockResolvedValue({ ...snapshot, unknown: true }),
      ),
    ).rejects.toThrow();
  });
  it("uses fixed, strict promotion reservation envelopes", async () => {
    const reservationId = "018f0000-0000-7000-8000-000000000004";
    const prepare = {
      collectionId: id,
      itemId: annotationId,
      expectedCollectionUpdatedAtMs: 1,
    };
    const candidate = {
      reservationId,
      collectionId: id,
      itemId: annotationId,
      title: "Text",
      sha256: sha,
      textFormat: "plain",
      destinationClass: "text",
      taskId: id,
      planId: null,
      createdAtMs: 0,
      expiresAtMs: 300000,
      state: "prepared",
    };
    const manifest = {
      schemaVersion: 1,
      artifactId: "018f0000-0000-7000-8000-000000000005",
      class: "text",
      mimeType: "text/plain; charset=utf-8",
      sourceKind: "explicit-review-promotion",
      displayLabel: "Text",
      suggestedFilename: "review-promotion.txt",
      byteSize: 4,
      sha256: sha,
      createdAt: 0,
      expiresAt: 1,
      state: "ready",
      disposal: "transient-memory-one-successful-save",
    };
    const invoke = vi
      .fn()
      .mockResolvedValueOnce(candidate)
      .mockResolvedValueOnce(manifest)
      .mockResolvedValueOnce({ ...candidate, state: "expired" });
    await prepareLocalReviewPromotion(prepare, invoke);
    await confirmLocalReviewPromotion({ reservationId }, invoke);
    await cancelLocalReviewPromotion({ reservationId }, invoke);
    expect(invoke).toHaveBeenNthCalledWith(
      1,
      LOCAL_REVIEW_PROMOTION_PREPARE_COMMAND,
      { request: prepare },
    );
    expect(invoke).toHaveBeenNthCalledWith(
      2,
      LOCAL_REVIEW_PROMOTION_CONFIRM_COMMAND,
      { request: { reservationId } },
    );
    expect(invoke).toHaveBeenNthCalledWith(
      3,
      LOCAL_REVIEW_PROMOTION_CANCEL_COMMAND,
      { request: { reservationId } },
    );
    for (const [, envelope] of invoke.mock.calls)
      expect(JSON.stringify(envelope)).not.toMatch(
        /path|url|filename|directory|git|shell|terminal|provider|connector|approval|dispatch|execution|save|publish|deploy/i,
      );
  });
});
