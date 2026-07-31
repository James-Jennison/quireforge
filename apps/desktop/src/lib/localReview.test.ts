import { describe, expect, it } from "vitest";
import {
  localReviewImagePickOutcomeSchema,
  localReviewImagePickRequestSchema,
  localReviewImagePreviewSchema,
  localReviewTextPreviewRequestSchema,
  localReviewTextPreviewSchema,
  localReviewItemSchema,
  localReviewManualEvidenceCreateRequestSchema,
  localReviewManualEvidenceCreateResultSchema,
  localReviewEvidenceEnvelopeSchema,
  localReviewEvidenceSourceSchema,
  localReviewManualEvidencePreviewSchema,
  localReviewM48GeneratedArtifactMetadataEvidenceCreateRequestSchema,
  localReviewM48GeneratedArtifactMetadataEvidenceCreateResultSchema,
  localReviewM48GeneratedArtifactMetadataEvidencePreviewSchema,
  localReviewM48ArtifactCopyRequestSchema,
  localReviewAnnotationSchema,
  localReviewAnnotationCreateRequestSchema,
  localReviewAnnotationEditRequestSchema,
  localReviewAnnotationMutationRequestSchema,
  localReviewCollectionSchema,
  localReviewComparisonCreateRequestSchema,
  localReviewComparisonDiscardRequestSchema,
  localReviewComparisonReadRequestSchema,
  localReviewComparisonSchema,
  localReviewLineComparisonSchema,
  localReviewPromotionPrepareRequestSchema,
  localReviewPromotionCandidateSchema,
  localReviewPromotionReservationRequestSchema,
  localReviewPackageManifestSummaryEvidenceCreateRequestSchema,
  localReviewPackageManifestSummaryEvidencePreviewSchema,
} from "./localReview";

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
  activityPresentationAvailable: false,
  diagnosticCode: null,
};

describe("local review image contracts", () => {
  it("accepts only the native-owned package summary capture request", () => {
    const request = { collectionId: id, expectedCollectionUpdatedAtMs: 1 };
    expect(
      localReviewPackageManifestSummaryEvidenceCreateRequestSchema.parse(
        request,
      ),
    ).toEqual(request);
    expect(() =>
      localReviewPackageManifestSummaryEvidenceCreateRequestSchema.parse({
        ...request,
        projectId: id,
        applicationVersion: "0.1.0-beta.51",
      }),
    ).toThrow();
    expect(() =>
      localReviewPackageManifestSummaryEvidencePreviewSchema.parse({
        schemaVersion: 1,
        itemId: id,
        source: "package-manifest-summary",
        title: "Package validation summary",
        summary: "Captured completed package-validation summary.",
        details: {
          applicationVersion: "0.1.0-beta.51",
          debianVersion: "0.1.0~beta.51",
          manifestState: "passed",
          checksumState: "passed",
          abiState: "passed",
          provenanceState: "passed",
          visibleLaunchState: "passed",
          installedHostState: "passed",
          artifactCount: 2,
          validationComplete: true,
          path: "/unsafe",
        },
        byteSize: 1,
        sha256: sha,
        createdAtMs: 1,
      }),
    ).toThrow();
  });
  it("accepts only the fixed path-free picker request", () => {
    expect(
      localReviewImagePickRequestSchema.parse({
        collectionId: id,
        expectedCollectionUpdatedAtMs: 1,
        title: "Mockup",
      }),
    ).toBeTruthy();
    expect(() =>
      localReviewImagePickRequestSchema.parse({
        collectionId: id,
        expectedCollectionUpdatedAtMs: 1,
        title: "Mockup",
        path: "/tmp/a.png",
      }),
    ).toThrow();
  });
  it("distinguishes closed picker outcomes", () => {
    expect(
      localReviewImagePickOutcomeSchema.parse({
        outcome: "canceled",
        snapshot,
      }),
    ).toBeTruthy();
    expect(() =>
      localReviewImagePickOutcomeSchema.parse({ outcome: "unknown", snapshot }),
    ).toThrow();
  });
  it("accepts only bounded PNG or JPEG data previews", () => {
    expect(
      localReviewImagePreviewSchema.parse({
        schemaVersion: 1,
        itemId: id,
        mimeType: "image/png",
        width: 1,
        height: 1,
        byteSize: 1,
        sha256: sha,
        dataUrl: "data:image/png;base64,AA==",
      }),
    ).toBeTruthy();
    expect(() =>
      localReviewImagePreviewSchema.parse({
        schemaVersion: 1,
        itemId: id,
        mimeType: "image/png",
        width: 1,
        height: 1,
        byteSize: 1,
        sha256: sha,
        dataUrl: "https://example.test/a.png",
      }),
    ).toThrow();
  });
  it("rejects prohibited picker fields and fabricated cancellation", () => {
    for (const key of [
      "path",
      "filePath",
      "filename",
      "directory",
      "url",
      "urls",
      "files",
      "multiple",
      "operation",
    ]) {
      expect(() =>
        localReviewImagePickRequestSchema.parse({
          collectionId: id,
          expectedCollectionUpdatedAtMs: 1,
          title: "Mockup",
          [key]: "x",
        }),
      ).toThrow();
    }
    expect(() =>
      localReviewImagePickOutcomeSchema.parse({ outcome: "created" }),
    ).toThrow();
    expect(() =>
      localReviewImagePickOutcomeSchema.parse({
        outcome: "canceled",
        snapshot,
        itemId: id,
      }),
    ).toThrow();
  });
  it("closes image item MIME, dimensions, source, and unknown fields", () => {
    const image = {
      itemId: id,
      class: "image-mockup",
      textFormat: null,
      sourceKind: "native-image-input",
      state: "ready",
      title: "Mockup",
      mimeType: "image/png",
      width: 1,
      height: 1,
      byteSize: 1,
      lineCount: null,
      sha256: sha,
      createdAtMs: 0,
      annotations: [],
    };
    expect(localReviewItemSchema.parse(image)).toMatchObject(image);
    for (const patch of [
      { mimeType: "image/svg+xml" },
      { width: 0 },
      { height: 4097 },
      { byteSize: -1 },
      { sha256: "bad" },
      { sourceKind: "user-authored-text" },
      { path: "/tmp/x" },
      { url: "https://x" },
    ]) {
      expect(() =>
        localReviewItemSchema.parse({ ...image, ...patch }),
      ).toThrow();
    }
  });
  it("rejects mismatched and non-data preview URLs", () => {
    const base = {
      schemaVersion: 1,
      itemId: id,
      mimeType: "image/png",
      width: 1,
      height: 1,
      byteSize: 1,
      sha256: sha,
    };
    for (const dataUrl of [
      "data:image/jpeg;base64,AA==",
      "https://example.test/a",
      "file:///tmp/a",
      "AA==",
    ]) {
      expect(() =>
        localReviewImagePreviewSchema.parse({ ...base, dataUrl }),
      ).toThrow();
    }
  });
  it("strictly parses manual validation evidence envelopes", () => {
    expect(
      localReviewManualEvidenceCreateRequestSchema.parse({
        collectionId: id,
        expectedCollectionUpdatedAtMs: 1,
        title: "Validation",
        summary: "line\r\nsummary",
      }),
    ).toBeTruthy();
    expect(() =>
      localReviewManualEvidenceCreateRequestSchema.parse({
        collectionId: id,
        expectedCollectionUpdatedAtMs: 1,
        title: "Validation",
        summary: "ok",
        path: "/tmp/x",
      }),
    ).toThrow();
    expect(
      localReviewManualEvidencePreviewSchema.parse({
        schemaVersion: 1,
        itemId: id,
        source: "manual-validation-summary",
        title: "Validation",
        summary: "ok",
        byteSize: 1,
        sha256: sha,
        createdAtMs: 0,
      }),
    ).toBeTruthy();
  });
  it("closes every ratified evidence source and its strict envelope", () => {
    const sources = [
      "manual-validation-summary",
      "m48-generated-artifact-metadata",
      "safe-preview-metadata",
      "git-status-diff-summary",
      "activity-presentation",
      "approval-presentation",
      "package-manifest-summary",
    ];
    expect(
      sources.map((source) => localReviewEvidenceSourceSchema.parse(source)),
    ).toEqual(sources);
    expect(() =>
      localReviewEvidenceSourceSchema.parse("unknown-evidence"),
    ).toThrow();
    const envelope = {
      schemaVersion: 1,
      source: "manual-validation-summary",
      sourceSchemaVersion: 1,
      title: "Validation",
      summary: "ok",
      details: { validationState: "passed" },
    };
    expect(localReviewEvidenceEnvelopeSchema.parse(envelope)).toMatchObject(
      envelope,
    );
    for (const sourceEnvelope of [
      {
        ...envelope,
        source: "m48-generated-artifact-metadata",
        details: {
          artifactState: "ready",
          artifactKind: "text",
          format: "plain",
          byteLength: 1,
          truncated: false,
          manifestSha256: sha,
        },
      },
      {
        ...envelope,
        source: "safe-preview-metadata",
        details: {
          previewState: "ready",
          kind: "text",
          rendering: "normalized-text",
          mediaType: "text/plain; charset=utf-8",
          byteLength: 1,
          truncated: false,
          widthPx: null,
          heightPx: null,
        },
      },
      {
        ...envelope,
        source: "git-status-diff-summary",
        details: {
          workspaceState: "ready",
          dirty: true,
          stagedCount: 0,
          modifiedCount: 1,
          addedCount: 0,
          deletedCount: 0,
          renamedCount: 0,
          untrackedCount: 0,
          conflictedCount: 0,
          changedFileCount: 1,
          additions: 1,
          deletions: 0,
          diffAvailable: true,
          diffTruncated: false,
        },
      },
      {
        ...envelope,
        source: "activity-presentation",
        details: {
          scope: "current-session",
          eventCount: 1,
          itemAddedCount: 1,
          itemDiscardedCount: 0,
          annotationChangedCount: 0,
          comparisonChangedCount: 0,
          promotionPreparedCount: 0,
          promotionCompletedCount: 0,
          collectionChangedCount: 0,
          truncated: false,
        },
      },
      {
        ...envelope,
        source: "approval-presentation",
        details: {
          approvalState: "none",
          requestPresent: false,
          decisionPresent: false,
          dispatchPresent: false,
          executionPresent: false,
        },
      },
      {
        ...envelope,
        source: "package-manifest-summary",
        details: {
          applicationVersion: "0.1.0-beta.46",
          debianVersion: "0.1.0~beta.46",
          manifestState: "passed",
          checksumState: "passed",
          abiState: "passed",
          provenanceState: "passed",
          visibleLaunchState: "passed",
          installedHostState: "skipped",
          artifactCount: 2,
          validationComplete: true,
        },
      },
    ])
      expect(
        localReviewEvidenceEnvelopeSchema.parse(sourceEnvelope),
      ).toMatchObject(sourceEnvelope);
    for (const patch of [
      { path: "/tmp/x" },
      { url: "https://x" },
      { filename: "x" },
      { details: { validationState: "passed", approvalBody: "x" } },
      { source: "unknown-evidence" },
    ]) {
      expect(() =>
        localReviewEvidenceEnvelopeSchema.parse({ ...envelope, ...patch }),
      ).toThrow();
    }
  });
  it("requires an authoritative created evidence identity before reporting success", () => {
    const item = {
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
      byteSize: 1,
      lineCount: null,
      sha256: sha,
      createdAtMs: 0,
      annotations: [],
    };
    const result = {
      outcome: "created",
      createdItemId: id,
      source: "manual-validation-summary",
      snapshot: {
        schemaVersion: 1,
        collections: [],
        selectedCollection: null,
        items: [item],
        comparisons: [],
        collectionCount: 0,
        payloadBytes: 0,
        warning: false,
        packageManifestSummaryAvailable: false,
        gitStatusDiffSummaryAvailable: false,
        activityPresentationAvailable: false,
        diagnosticCode: null,
      },
    };
    expect(
      localReviewManualEvidenceCreateResultSchema.parse(result),
    ).toMatchObject(result);
    expect(() =>
      localReviewManualEvidenceCreateResultSchema.parse({
        ...result,
        createdItemId: annotationId,
      }),
    ).toThrow();
  });
  it("strictly parses M48 metadata-only evidence capture and stored preview", () => {
    const request = {
      collectionId: id,
      expectedCollectionUpdatedAtMs: 1,
      artifactId: annotationId,
      manifestSha256: sha,
    };
    expect(
      localReviewM48GeneratedArtifactMetadataEvidenceCreateRequestSchema.parse(
        request,
      ),
    ).toMatchObject(request);
    expect(() =>
      localReviewM48GeneratedArtifactMetadataEvidenceCreateRequestSchema.parse({
        ...request,
        content: "no",
      }),
    ).toThrow();
    const item = {
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
      byteSize: 1,
      lineCount: null,
      sha256: sha,
      createdAtMs: 0,
      annotations: [],
    };
    const result = {
      outcome: "created",
      createdItemId: id,
      source: "m48-generated-artifact-metadata",
      snapshot: { ...snapshot, items: [item] },
    };
    expect(
      localReviewM48GeneratedArtifactMetadataEvidenceCreateResultSchema.parse(
        result,
      ),
    ).toMatchObject(result);
    const preview = {
      schemaVersion: 1,
      itemId: id,
      source: "m48-generated-artifact-metadata",
      title: item.title,
      summary: "Captured live generated-artifact metadata only.",
      details: {
        artifactState: "ready",
        artifactKind: "markdown",
        format: "markdown",
        byteLength: 12,
        truncated: false,
        manifestSha256: sha,
      },
      byteSize: 256,
      sha256: sha,
      createdAtMs: 0,
    };
    expect(
      localReviewM48GeneratedArtifactMetadataEvidencePreviewSchema.parse(
        preview,
      ),
    ).toMatchObject(preview);
    expect(() =>
      localReviewM48GeneratedArtifactMetadataEvidencePreviewSchema.parse({
        ...preview,
        filename: "x.md",
      }),
    ).toThrow();
  });
  it("strictly parses item-level annotations and closed requests", () => {
    const annotation = {
      schemaVersion: 1,
      annotationId,
      itemId: id,
      text: "line\ntext",
      state: "open",
      createdAtMs: 1,
      updatedAtMs: 1,
    };
    expect(localReviewAnnotationSchema.parse(annotation)).toMatchObject(
      annotation,
    );
    expect(
      localReviewAnnotationSchema.parse({
        ...annotation,
        state: "resolved",
        updatedAtMs: 2,
      }),
    ).toBeTruthy();
    for (const patch of [
      { state: "pending" },
      { annotationId: "not-an-id" },
      { text: "x".repeat(1025) },
      { path: "/tmp/x" },
      { url: "https://x" },
      { author: "a" },
      { range: 1 },
      { coordinate: 1 },
      { mention: "a" },
      { provider: "x" },
      { approval: true },
      { dispatch: true },
      { execution: true },
    ]) {
      expect(() =>
        localReviewAnnotationSchema.parse({ ...annotation, ...patch }),
      ).toThrow();
    }
    const create = {
      collectionId: id,
      itemId: id,
      expectedCollectionUpdatedAtMs: 1,
      text: "line\r\ntext",
    };
    const edit = { ...create, annotationId };
    const mutation = {
      collectionId: id,
      itemId: id,
      annotationId,
      expectedCollectionUpdatedAtMs: 1,
    };
    expect(
      localReviewAnnotationCreateRequestSchema.parse(create),
    ).toMatchObject(create);
    expect(localReviewAnnotationEditRequestSchema.parse(edit)).toMatchObject(
      edit,
    );
    expect(
      localReviewAnnotationMutationRequestSchema.parse(mutation),
    ).toMatchObject(mutation);
    for (const request of [create, edit, mutation]) {
      expect(() =>
        "text" in request
          ? localReviewAnnotationEditRequestSchema.parse({
              ...request,
              operation: "x",
            })
          : localReviewAnnotationMutationRequestSchema.parse({
              ...request,
              operation: "x",
            }),
      ).toThrow();
    }
    expect(() =>
      localReviewAnnotationCreateRequestSchema.parse({
        collectionId: id,
        itemId: id,
        expectedCollectionUpdatedAtMs: 1,
      }),
    ).toThrow();
  });
  it("parses closed annotation warning projections", () => {
    const base = {
      collectionId: id,
      taskId: id,
      planId: null,
      title: "Collection",
      state: "active",
      itemCount: 0,
      payloadBytes: 0,
      updatedAtMs: 1,
      warning: false,
    };
    expect(
      localReviewCollectionSchema.parse({
        ...base,
        annotationCountWarning: false,
        annotationByteWarning: false,
        comparisonCountWarning: false,
      }),
    ).toBeTruthy();
    expect(
      localReviewCollectionSchema.parse({
        ...base,
        warning: true,
        annotationCountWarning: true,
        annotationByteWarning: false,
        comparisonCountWarning: false,
      }),
    ).toBeTruthy();
    expect(
      localReviewCollectionSchema.parse({
        ...base,
        warning: true,
        annotationCountWarning: false,
        annotationByteWarning: true,
        comparisonCountWarning: false,
      }),
    ).toBeTruthy();
    expect(
      localReviewCollectionSchema.parse({
        ...base,
        warning: true,
        annotationCountWarning: true,
        annotationByteWarning: true,
        comparisonCountWarning: false,
      }),
    ).toBeTruthy();
  });
  it("strictly parses digest-bound comparison bindings, lines, requests, and warnings", () => {
    const comparisonId = "018f0000-0000-7000-8000-000000000003";
    const binding = {
      schemaVersion: 1,
      comparisonId,
      collectionId: id,
      leftItemId: id,
      rightItemId: annotationId,
      leftSha256: sha,
      rightSha256: "b".repeat(64),
      textFormat: "plain",
      state: "ready",
      createdAtMs: 1,
    };
    expect(localReviewComparisonSchema.parse(binding)).toMatchObject(binding);
    expect(
      localReviewComparisonSchema.parse({ ...binding, state: "stale" }),
    ).toBeTruthy();
    expect(
      localReviewComparisonSchema.parse({ ...binding, state: "unavailable" }),
    ).toBeTruthy();
    for (const patch of [
      { state: "pending" },
      { comparisonId: "bad" },
      { leftSha256: "bad" },
      { path: "/tmp/x" },
      { url: "https://x" },
      { command: "diff" },
    ])
      expect(() =>
        localReviewComparisonSchema.parse({ ...binding, ...patch }),
      ).toThrow();
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
        {
          kind: "added",
          text: "right",
          leftLineNumber: null,
          rightLineNumber: 2,
        },
        {
          kind: "removed",
          text: "left",
          leftLineNumber: 2,
          rightLineNumber: null,
        },
      ],
    };
    expect(localReviewLineComparisonSchema.parse(lines)).toMatchObject(lines);
    expect(() =>
      localReviewLineComparisonSchema.parse({
        ...lines,
        lines: [
          {
            kind: "changed",
            text: "x",
            leftLineNumber: null,
            rightLineNumber: null,
          },
        ],
      }),
    ).toThrow();
    const create = {
      collectionId: id,
      leftItemId: id,
      rightItemId: annotationId,
      expectedCollectionUpdatedAtMs: 1,
    };
    const read = { collectionId: id, comparisonId };
    const discard = { ...read, expectedCollectionUpdatedAtMs: 1 };
    expect(
      localReviewComparisonCreateRequestSchema.parse(create),
    ).toMatchObject(create);
    expect(localReviewComparisonReadRequestSchema.parse(read)).toMatchObject(
      read,
    );
    expect(
      localReviewComparisonDiscardRequestSchema.parse(discard),
    ).toMatchObject(discard);
    for (const schema of [
      localReviewComparisonCreateRequestSchema,
      localReviewComparisonReadRequestSchema,
      localReviewComparisonDiscardRequestSchema,
    ])
      for (const prohibited of [
        "path",
        "url",
        "git",
        "repository",
        "command",
        "shell",
        "approval",
        "dispatch",
        "execution",
        "operation",
      ])
        expect(() =>
          schema.parse({
            ...(schema === localReviewComparisonCreateRequestSchema
              ? create
              : schema === localReviewComparisonReadRequestSchema
                ? read
                : discard),
            [prohibited]: "x",
          }),
        ).toThrow();
    const base = {
      collectionId: id,
      taskId: id,
      planId: null,
      title: "Collection",
      state: "active",
      itemCount: 0,
      payloadBytes: 0,
      updatedAtMs: 1,
      annotationCountWarning: false,
      annotationByteWarning: false,
    };
    expect(
      localReviewCollectionSchema.parse({
        ...base,
        warning: false,
        comparisonCountWarning: false,
      }),
    ).toBeTruthy();
    expect(
      localReviewCollectionSchema.parse({
        ...base,
        warning: true,
        comparisonCountWarning: true,
      }),
    ).toBeTruthy();
  });
  it("strictly parses path-free promotion reservations", () => {
    const reservationId = "018f0000-0000-7000-8000-000000000004";
    const request = {
      collectionId: id,
      itemId: annotationId,
      expectedCollectionUpdatedAtMs: 1,
    };
    expect(
      localReviewPromotionPrepareRequestSchema.parse(request),
    ).toMatchObject(request);
    expect(
      localReviewPromotionReservationRequestSchema.parse({ reservationId }),
    ).toBeTruthy();
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
    expect(localReviewPromotionCandidateSchema.parse(candidate)).toMatchObject(
      candidate,
    );
    for (const key of [
      "path",
      "url",
      "filename",
      "directory",
      "git",
      "shell",
      "approval",
      "dispatch",
      "execution",
      "saveDestination",
      "publish",
      "deploy",
    ])
      expect(() =>
        localReviewPromotionPrepareRequestSchema.parse({
          ...request,
          [key]: "x",
        }),
      ).toThrow();
    expect(() =>
      localReviewPromotionCandidateSchema.parse({
        ...candidate,
        state: "saving",
      }),
    ).toThrow();
  });
});

describe("local review M48 artifact copy contracts", () => {
  it("accepts only the fixed digest-bound artifact claim", () => {
    const request = {
      collectionId: id,
      expectedCollectionUpdatedAtMs: 1,
      artifactId: annotationId,
      manifestSha256: sha,
    };
    expect(
      localReviewM48ArtifactCopyRequestSchema.parse(request),
    ).toMatchObject(request);
    for (const key of [
      "path",
      "filePath",
      "filename",
      "directory",
      "url",
      "content",
      "textFormat",
      "sourceKind",
      "taskId",
      "provenance",
      "operation",
      "command",
      "approval",
      "dispatch",
      "execution",
    ]) {
      expect(() =>
        localReviewM48ArtifactCopyRequestSchema.parse({
          ...request,
          [key]: "x",
        }),
      ).toThrow();
    }
  });
  it("rejects malformed copy identities and concurrency claims", () => {
    const request = {
      collectionId: id,
      expectedCollectionUpdatedAtMs: 1,
      artifactId: annotationId,
      manifestSha256: sha,
    };
    expect(() =>
      localReviewM48ArtifactCopyRequestSchema.parse({
        ...request,
        artifactId: "artifact",
      }),
    ).toThrow();
    expect(() =>
      localReviewM48ArtifactCopyRequestSchema.parse({
        ...request,
        manifestSha256: "bad",
      }),
    ).toThrow();
    expect(() =>
      localReviewM48ArtifactCopyRequestSchema.parse({
        ...request,
        expectedCollectionUpdatedAtMs: -1,
      }),
    ).toThrow();
  });
});

describe("local review text preview contracts", () => {
  it("strictly parses bounded inert text previews", () => {
    const request = { collectionId: id, itemId: annotationId, sha256: sha };
    expect(localReviewTextPreviewRequestSchema.parse(request)).toMatchObject(
      request,
    );
    for (const key of [
      "path",
      "url",
      "content",
      "textFormat",
      "sourceKind",
      "command",
      "operation",
      "approval",
      "dispatch",
      "execution",
    ])
      expect(() =>
        localReviewTextPreviewRequestSchema.parse({ ...request, [key]: "x" }),
      ).toThrow();
    for (const textFormat of [
      "plain",
      "markdown",
      "json",
      "csv",
      "python",
    ] as const) {
      const text =
        textFormat === "json"
          ? '{"value":1}'
          : textFormat === "csv"
            ? "a,b\n1,2"
            : "line\ntext";
      const preview = {
        schemaVersion: 1,
        collectionId: id,
        itemId: annotationId,
        title: "Text",
        textFormat,
        byteSize: text.length,
        sha256: sha,
        createdAtMs: 0,
        state: "ready",
        text,
        projectedByteSize: text.length,
        projectedLineCount: 2,
        projectedCodePointCount: [...text].length,
        truncated: false,
        diagnosticCode: null,
      };
      expect(localReviewTextPreviewSchema.parse(preview)).toMatchObject(
        preview,
      );
    }
  });
  it("withholds unsafe text previews and rejects malformed projections", () => {
    const unavailable = {
      schemaVersion: 1,
      collectionId: id,
      itemId: annotationId,
      title: null,
      textFormat: null,
      byteSize: null,
      sha256: null,
      createdAtMs: null,
      state: "unavailable",
      text: null,
      projectedByteSize: 0,
      projectedLineCount: 0,
      projectedCodePointCount: 0,
      truncated: false,
      diagnosticCode: "integrity-failed",
    };
    expect(localReviewTextPreviewSchema.parse(unavailable)).toMatchObject(
      unavailable,
    );
    for (const patch of [
      { state: "ready" },
      { text: "unsafe" },
      { path: "/tmp/x" },
      { url: "https://x" },
      { projectedByteSize: 128 * 1024 + 1 },
    ])
      expect(() =>
        localReviewTextPreviewSchema.parse({ ...unavailable, ...patch }),
      ).toThrow();
  });
});
