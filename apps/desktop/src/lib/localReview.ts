import { z } from "zod";
import { generatedArtifactManifestSchema } from "./advisorGeneratedArtifact";

const id = z.string().uuid();
const annotationId = id.regex(
  /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/u,
);
const sha256 = z.string().regex(/^[a-f0-9]{64}$/u);
const safeLabel = z
  .string()
  .min(1)
  .max(120)
  .refine((value) => !/[\\/\p{Cc}\p{Cf}]/u.test(value));

export const localReviewTextFormatSchema = z.enum([
  "plain",
  "markdown",
  "json",
  "csv",
  "python",
]);
const collectionState = z.enum([
  "active",
  "frozen",
  "orphaned",
  "unavailable",
  "discarded",
]);
const itemClass = z.enum(["text", "image-mockup", "evidence"]);
const itemState = z.enum(["ready", "stale", "unavailable", "discarded"]);
export const localReviewEvidenceSourceSchema = z.enum([
  "manual-validation-summary",
  "m48-generated-artifact-metadata",
  "safe-preview-metadata",
  "git-status-diff-summary",
  "activity-presentation",
  "approval-presentation",
  "package-manifest-summary",
]);
export const localReviewAnnotationStateSchema = z.enum(["open", "resolved"]);
const diagnostic = z.enum([
  "metadata-unavailable",
  "invalid-request",
  "collection-capacity",
  "active-collection-capacity",
  "item-capacity",
  "image-capacity",
  "evidence-capacity",
  "payload-capacity",
  "invalid-content",
  "invalid-label",
  "invalid-reference",
  "task-unavailable",
  "task-frozen",
  "plan-unavailable",
  "plan-stale",
  "collection-not-found",
  "item-not-found",
  "integrity-failed",
  "stale-write",
]);

export const localReviewCollectionSchema = z
  .object({
    collectionId: id,
    taskId: id,
    planId: id.nullable(),
    title: safeLabel,
    state: collectionState,
    itemCount: z.number().int().min(0).max(12),
    payloadBytes: z
      .number()
      .int()
      .nonnegative()
      .max(4 * 1024 * 1024),
    updatedAtMs: z.number().int().nonnegative(),
    warning: z.boolean(),
    annotationCountWarning: z.boolean(),
    annotationByteWarning: z.boolean(),
    comparisonCountWarning: z.boolean(),
  })
  .strict();
const annotationText = z
  .string()
  .min(1)
  .max(1024)
  .refine((value) => new TextEncoder().encode(value).byteLength <= 1024)
  .refine((value) => !/\r/u.test(value));
export const localReviewAnnotationSchema = z
  .object({
    schemaVersion: z.literal(1),
    annotationId,
    itemId: id,
    text: annotationText,
    state: localReviewAnnotationStateSchema,
    createdAtMs: z.number().int().nonnegative(),
    updatedAtMs: z.number().int().nonnegative(),
  })
  .strict()
  .refine(
    (value) => value.updatedAtMs >= value.createdAtMs,
    "timestamps must be monotonic",
  );
export const localReviewItemSchema = z
  .object({
    itemId: id,
    class: itemClass,
    textFormat: localReviewTextFormatSchema.nullable(),
    sourceKind: z.enum([
      "user-authored-text",
      "m48-artifact-copy",
      "native-image-input",
      "typed-evidence-snapshot",
    ]),
    evidenceSource: localReviewEvidenceSourceSchema.nullable().default(null),
    state: itemState,
    title: safeLabel,
    mimeType: z.string().min(1).max(80),
    width: z.number().int().positive().max(4096).nullable(),
    height: z.number().int().positive().max(4096).nullable(),
    byteSize: z
      .number()
      .int()
      .positive()
      .max(1024 * 1024),
    lineCount: z.number().int().nonnegative().max(32_768).nullable(),
    sha256,
    createdAtMs: z.number().int().nonnegative(),
    annotations: z.array(localReviewAnnotationSchema).max(32),
  })
  .strict()
  .superRefine((value, context) => {
    if (value.class === "text") {
      if (
        value.textFormat === null ||
        !["user-authored-text", "m48-artifact-copy"].includes(value.sourceKind)
      )
        context.addIssue({ code: "custom", message: "text source is closed" });
      if (value.width !== null || value.height !== null)
        context.addIssue({
          code: "custom",
          message: "text dimensions are absent",
        });
    }
    if (value.class === "image-mockup") {
      if (
        !(["image/png", "image/jpeg"] as const).includes(
          value.mimeType as "image/png" | "image/jpeg",
        )
      )
        context.addIssue({ code: "custom", message: "image MIME is closed" });
      if (value.width === null || value.height === null)
        context.addIssue({
          code: "custom",
          message: "image dimensions are required",
        });
      if (value.sourceKind !== "native-image-input")
        context.addIssue({ code: "custom", message: "image source is closed" });
    }
    if (
      value.class === "evidence" &&
      (value.sourceKind !== "typed-evidence-snapshot" ||
        value.evidenceSource === null)
    )
      context.addIssue({
        code: "custom",
        message: "evidence source is closed",
      });
    if (value.class !== "evidence" && value.evidenceSource !== null)
      context.addIssue({
        code: "custom",
        message: "non-evidence source is absent",
      });
  });
export const localReviewComparisonStateSchema = z.enum([
  "ready",
  "stale",
  "unavailable",
]);
export const localReviewComparisonSchema = z
  .object({
    schemaVersion: z.literal(1),
    comparisonId: annotationId,
    collectionId: id,
    leftItemId: id,
    rightItemId: id,
    leftSha256: sha256,
    rightSha256: sha256,
    textFormat: localReviewTextFormatSchema,
    state: localReviewComparisonStateSchema,
    createdAtMs: z.number().int().nonnegative(),
  })
  .strict()
  .refine(
    (value) => value.leftItemId !== value.rightItemId,
    "comparison sides must differ",
  );
export const localReviewComparisonLineSchema = z
  .object({
    kind: z.enum(["unchanged", "added", "removed"]),
    text: z.string().max(128 * 1024),
    leftLineNumber: z.number().int().positive().nullable(),
    rightLineNumber: z.number().int().positive().nullable(),
  })
  .strict();
export const localReviewLineComparisonSchema = z
  .object({
    comparisonId: annotationId,
    leftItemId: id,
    leftSha256: sha256,
    rightItemId: id,
    rightSha256: sha256,
    textFormat: localReviewTextFormatSchema,
    state: localReviewComparisonStateSchema,
    lines: z.array(localReviewComparisonLineSchema).max(4000),
  })
  .strict();
export const localReviewSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    collections: z.array(localReviewCollectionSchema).max(24),
    selectedCollection: localReviewCollectionSchema.nullable(),
    items: z.array(localReviewItemSchema).max(12),
    comparisons: z.array(localReviewComparisonSchema).max(8),
    collectionCount: z.number().int().min(0).max(24),
    payloadBytes: z
      .number()
      .int()
      .nonnegative()
      .max(32 * 1024 * 1024),
    warning: z.boolean(),
    packageManifestSummaryAvailable: z.boolean(),
    diagnosticCode: diagnostic.nullable(),
  })
  .strict();
export const localReviewListRequestSchema = z
  .object({ selectedCollectionId: id.nullable() })
  .strict();
export const localReviewCollectionCreateRequestSchema = z
  .object({ taskId: id, planId: id.nullable(), title: safeLabel })
  .strict();
export const localReviewTextItemCreateRequestSchema = z
  .object({
    collectionId: id,
    expectedCollectionUpdatedAtMs: z.number().int().nonnegative(),
    title: safeLabel,
    textFormat: localReviewTextFormatSchema,
    content: z
      .string()
      .min(1)
      .max(256 * 1024),
  })
  .strict();
export const localReviewM48ArtifactCopyRequestSchema = z
  .object({
    collectionId: id,
    expectedCollectionUpdatedAtMs: z.number().int().nonnegative(),
    artifactId: id,
    manifestSha256: sha256,
  })
  .strict();
export const localReviewCollectionMutationRequestSchema = z
  .object({
    collectionId: id,
    expectedCollectionUpdatedAtMs: z.number().int().nonnegative(),
  })
  .strict();
export const localReviewItemDiscardRequestSchema = z
  .object({
    collectionId: id,
    itemId: id,
    expectedCollectionUpdatedAtMs: z.number().int().nonnegative(),
  })
  .strict();
export const localReviewComparisonCreateRequestSchema = z
  .object({
    collectionId: id,
    leftItemId: id,
    rightItemId: id,
    expectedCollectionUpdatedAtMs: z.number().int().nonnegative(),
  })
  .strict()
  .refine(
    (value) => value.leftItemId !== value.rightItemId,
    "comparison sides must differ",
  );
export const localReviewComparisonReadRequestSchema = z
  .object({ collectionId: id, comparisonId: annotationId })
  .strict();
export const localReviewComparisonDiscardRequestSchema = z
  .object({
    collectionId: id,
    comparisonId: annotationId,
    expectedCollectionUpdatedAtMs: z.number().int().nonnegative(),
  })
  .strict();
export const localReviewPromotionPrepareRequestSchema = z
  .object({
    collectionId: id,
    itemId: id,
    expectedCollectionUpdatedAtMs: z.number().int().nonnegative(),
  })
  .strict();
export const localReviewPromotionReservationStateSchema = z.enum([
  "prepared",
  "consumed",
  "expired",
]);
export const localReviewPromotionCandidateSchema = z
  .object({
    reservationId: annotationId,
    collectionId: id,
    itemId: id,
    title: safeLabel,
    sha256,
    textFormat: localReviewTextFormatSchema,
    destinationClass: z.enum(["text", "markdown", "json", "csv", "python"]),
    taskId: id,
    planId: id.nullable(),
    createdAtMs: z.number().int().nonnegative(),
    expiresAtMs: z.number().int().positive(),
    state: localReviewPromotionReservationStateSchema,
  })
  .strict();
export const localReviewPromotionReservationRequestSchema = z
  .object({ reservationId: annotationId })
  .strict();
export const localReviewPromotionConfirmationSchema =
  generatedArtifactManifestSchema;
export const localReviewAnnotationCreateRequestSchema = z
  .object({
    collectionId: id,
    itemId: id,
    expectedCollectionUpdatedAtMs: z.number().int().nonnegative(),
    text: z.string().min(1).max(1024),
  })
  .strict();
export const localReviewAnnotationEditRequestSchema = z
  .object({
    collectionId: id,
    itemId: id,
    annotationId,
    expectedCollectionUpdatedAtMs: z.number().int().nonnegative(),
    text: z.string().min(1).max(1024),
  })
  .strict();
export const localReviewAnnotationMutationRequestSchema = z
  .object({
    collectionId: id,
    itemId: id,
    annotationId,
    expectedCollectionUpdatedAtMs: z.number().int().nonnegative(),
  })
  .strict();
export const localReviewImagePickRequestSchema = z
  .object({
    collectionId: id,
    expectedCollectionUpdatedAtMs: z.number().int().nonnegative(),
    title: safeLabel,
  })
  .strict();
const evidenceSummary = z
  .string()
  .min(1)
  .max(16 * 1024)
  .refine((value) => new TextEncoder().encode(value).byteLength <= 16 * 1024)
  .refine((value) => !/\r/u.test(value));
const evidenceInputSummary = z
  .string()
  .min(1)
  .max(16 * 1024)
  .refine((value) => new TextEncoder().encode(value).byteLength <= 16 * 1024);
const evidenceEnvelope = <T extends z.ZodType>(source: z.ZodType, details: T) =>
  z
    .object({
      schemaVersion: z.literal(1),
      source,
      sourceSchemaVersion: z.literal(1),
      title: safeLabel,
      summary: evidenceSummary,
      details,
    })
    .strict();
const nonnegativeCount = z.number().int().nonnegative().max(1_000_000);
const sha = sha256;
export const localReviewEvidenceEnvelopeSchema = z.discriminatedUnion(
  "source",
  [
    evidenceEnvelope(
      z.literal("manual-validation-summary"),
      z
        .object({
          validationState: z.enum(["passed", "failed", "mixed", "not-run"]),
        })
        .strict(),
    ),
    evidenceEnvelope(
      z.literal("m48-generated-artifact-metadata"),
      z
        .object({
          artifactState: z.enum(["ready", "saving", "expired", "saved"]),
          artifactKind: z.enum(["text", "markdown", "json", "csv", "python"]),
          format: localReviewTextFormatSchema,
          byteLength: nonnegativeCount,
          truncated: z.boolean(),
          manifestSha256: sha,
        })
        .strict(),
    ),
    evidenceEnvelope(
      z.literal("safe-preview-metadata"),
      z
        .object({
          previewState: z.enum(["empty", "ready", "unavailable"]),
          kind: z.enum(["text", "image", "pdf"]),
          rendering: z.enum([
            "normalized-text",
            "bounded-image",
            "metadata-only",
          ]),
          mediaType: z.enum([
            "text/plain; charset=utf-8",
            "image/png",
            "image/jpeg",
            "application/pdf",
          ]),
          byteLength: nonnegativeCount,
          truncated: z.boolean(),
          widthPx: z.number().int().positive().max(4096).nullable(),
          heightPx: z.number().int().positive().max(4096).nullable(),
        })
        .strict(),
    ),
    evidenceEnvelope(
      z.literal("git-status-diff-summary"),
      z
        .object({
          workspaceState: z.enum(["clean", "ready", "unavailable"]),
          dirty: z.boolean(),
          stagedCount: nonnegativeCount,
          modifiedCount: nonnegativeCount,
          addedCount: nonnegativeCount,
          deletedCount: nonnegativeCount,
          renamedCount: nonnegativeCount,
          untrackedCount: nonnegativeCount,
          conflictedCount: nonnegativeCount,
          changedFileCount: nonnegativeCount,
          additions: nonnegativeCount,
          deletions: nonnegativeCount,
          diffAvailable: z.boolean(),
          diffTruncated: z.boolean(),
        })
        .strict(),
    ),
    evidenceEnvelope(
      z.literal("activity-presentation"),
      z
        .object({
          scope: z.literal("current-session"),
          eventCount: z.number().int().nonnegative().max(12),
          itemAddedCount: z.number().int().nonnegative().max(12),
          itemDiscardedCount: z.number().int().nonnegative().max(12),
          annotationChangedCount: z.number().int().nonnegative().max(12),
          comparisonChangedCount: z.number().int().nonnegative().max(12),
          promotionPreparedCount: z.number().int().nonnegative().max(12),
          promotionCompletedCount: z.number().int().nonnegative().max(12),
          collectionChangedCount: z.number().int().nonnegative().max(12),
          truncated: z.boolean(),
        })
        .strict(),
    ),
    evidenceEnvelope(
      z.literal("approval-presentation"),
      z
        .object({
          approvalState: z.enum([
            "none",
            "pending",
            "approved",
            "rejected",
            "expired",
            "unavailable",
          ]),
          requestPresent: z.boolean(),
          decisionPresent: z.boolean(),
          dispatchPresent: z.boolean(),
          executionPresent: z.boolean(),
        })
        .strict(),
    ),
    evidenceEnvelope(
      z.literal("package-manifest-summary"),
      z
        .object({
          applicationVersion: z.string().min(1).max(64),
          debianVersion: z.string().min(1).max(64),
          manifestState: z.enum(["passed", "failed", "skipped", "unavailable"]),
          checksumState: z.enum(["passed", "failed", "skipped", "unavailable"]),
          abiState: z.enum(["passed", "failed", "skipped", "unavailable"]),
          provenanceState: z.enum([
            "passed",
            "failed",
            "skipped",
            "unavailable",
          ]),
          visibleLaunchState: z.enum([
            "passed",
            "failed",
            "skipped",
            "unavailable",
          ]),
          installedHostState: z.enum([
            "passed",
            "failed",
            "skipped",
            "unavailable",
          ]),
          artifactCount: nonnegativeCount,
          validationComplete: z.boolean(),
        })
        .strict(),
    ),
  ],
);
export const localReviewManualEvidenceCreateRequestSchema = z
  .object({
    collectionId: id,
    expectedCollectionUpdatedAtMs: z.number().int().nonnegative(),
    title: safeLabel,
    summary: evidenceInputSummary,
  })
  .strict();
export const localReviewManualEvidenceCreateResultSchema = z.discriminatedUnion(
  "outcome",
  [
    z
      .object({
        outcome: z.literal("created"),
        createdItemId: id,
        source: z.literal("manual-validation-summary"),
        snapshot: localReviewSnapshotSchema,
      })
      .strict()
      .superRefine((value, context) => {
        const item = value.snapshot.items.find(
          (candidate) => candidate.itemId === value.createdItemId,
        );
        if (
          !item ||
          item.class !== "evidence" ||
          item.evidenceSource !== value.source
        )
          context.addIssue({
            code: "custom",
            message: "created evidence identity matches snapshot",
          });
      }),
    z
      .object({
        outcome: z.literal("failed"),
        snapshot: localReviewSnapshotSchema,
      })
      .strict(),
  ],
);
export const localReviewM48GeneratedArtifactMetadataEvidenceCreateRequestSchema =
  z
    .object({
      collectionId: id,
      expectedCollectionUpdatedAtMs: z.number().int().nonnegative(),
      artifactId: id,
      manifestSha256: sha256,
    })
    .strict();
export const localReviewSafePreviewMetadataClaimSchema = z
  .object({
    claimId: id,
    claimSha256: sha256,
    previewState: z.literal("ready"),
    kind: z.enum(["text", "image", "pdf"]),
    rendering: z.enum(["normalized-text", "bounded-image", "metadata-only"]),
    mediaType: z.enum([
      "text/plain; charset=utf-8",
      "image/png",
      "image/jpeg",
      "application/pdf",
    ]),
    byteLength: nonnegativeCount,
    truncated: z.boolean(),
    widthPx: z.number().int().positive().max(8192).nullable(),
    heightPx: z.number().int().positive().max(8192).nullable(),
  })
  .strict();
export const localReviewSafePreviewMetadataEvidenceCreateRequestSchema = z
  .object({
    collectionId: id,
    expectedCollectionUpdatedAtMs: z.number().int().nonnegative(),
    previewClaimId: id,
    claimSha256: sha256,
  })
  .strict();
export const localReviewPackageManifestSummaryEvidenceCreateRequestSchema = z
  .object({
    collectionId: id,
    expectedCollectionUpdatedAtMs: z.number().int().nonnegative(),
  })
  .strict();
export const localReviewM48GeneratedArtifactMetadataEvidenceCreateResultSchema =
  z.discriminatedUnion("outcome", [
    z
      .object({
        outcome: z.literal("created"),
        createdItemId: id,
        source: z.literal("m48-generated-artifact-metadata"),
        snapshot: localReviewSnapshotSchema,
      })
      .strict()
      .superRefine((value, context) => {
        const item = value.snapshot.items.find(
          (candidate) => candidate.itemId === value.createdItemId,
        );
        if (
          !item ||
          item.class !== "evidence" ||
          item.evidenceSource !== value.source
        )
          context.addIssue({
            code: "custom",
            message: "created evidence identity matches snapshot",
          });
      }),
    z
      .object({
        outcome: z.literal("failed"),
        snapshot: localReviewSnapshotSchema,
      })
      .strict(),
  ]);
export const localReviewSafePreviewMetadataEvidenceCreateResultSchema =
  z.discriminatedUnion("outcome", [
    z
      .object({
        outcome: z.literal("created"),
        createdItemId: id,
        source: z.literal("safe-preview-metadata"),
        snapshot: localReviewSnapshotSchema,
      })
      .strict()
      .superRefine((value, context) => {
        const item = value.snapshot.items.find(
          (candidate) => candidate.itemId === value.createdItemId,
        );
        if (
          !item ||
          item.class !== "evidence" ||
          item.evidenceSource !== value.source
        )
          context.addIssue({
            code: "custom",
            message: "created evidence identity matches snapshot",
          });
      }),
    z
      .object({
        outcome: z.literal("failed"),
        snapshot: localReviewSnapshotSchema,
      })
      .strict(),
  ]);
export const localReviewPackageManifestSummaryEvidenceCreateResultSchema =
  z.discriminatedUnion("outcome", [
    z.object({ outcome: z.literal("created"), createdItemId: id, source: z.literal("package-manifest-summary"), snapshot: localReviewSnapshotSchema }).strict(),
    z.object({ outcome: z.literal("failed"), snapshot: localReviewSnapshotSchema }).strict(),
  ]);
export const localReviewManualEvidencePreviewSchema = z
  .object({
    schemaVersion: z.literal(1),
    itemId: id,
    source: z.literal("manual-validation-summary"),
    title: safeLabel,
    summary: evidenceSummary,
    byteSize: z
      .number()
      .int()
      .positive()
      .max(16 * 1024),
    sha256,
    createdAtMs: z.number().int().nonnegative(),
  })
  .strict();
export const localReviewM48GeneratedArtifactMetadataEvidencePreviewSchema = z
  .object({
    schemaVersion: z.literal(1),
    itemId: id,
    source: z.literal("m48-generated-artifact-metadata"),
    title: safeLabel,
    summary: evidenceSummary,
    details: z
      .object({
        artifactState: z.literal("ready"),
        artifactKind: z.enum(["text", "markdown", "json", "csv", "python"]),
        format: localReviewTextFormatSchema,
        byteLength: nonnegativeCount,
        truncated: z.boolean(),
        manifestSha256: sha256,
      })
      .strict(),
    byteSize: z
      .number()
      .int()
      .positive()
      .max(16 * 1024),
    sha256,
    createdAtMs: z.number().int().nonnegative(),
  })
  .strict();
export const localReviewSafePreviewMetadataEvidencePreviewSchema = z
  .object({
    schemaVersion: z.literal(1),
    itemId: id,
    source: z.literal("safe-preview-metadata"),
    title: safeLabel,
    summary: evidenceSummary,
    details: z
      .object({
        previewState: z.literal("ready"),
        kind: z.enum(["text", "image", "pdf"]),
        rendering: z.enum([
          "normalized-text",
          "bounded-image",
          "metadata-only",
        ]),
        mediaType: z.enum([
          "text/plain; charset=utf-8",
          "image/png",
          "image/jpeg",
          "application/pdf",
        ]),
        byteLength: nonnegativeCount,
        truncated: z.boolean(),
        widthPx: z.number().int().positive().max(8192).nullable(),
        heightPx: z.number().int().positive().max(8192).nullable(),
      })
      .strict(),
    byteSize: z
      .number()
      .int()
      .positive()
      .max(16 * 1024),
    sha256,
    createdAtMs: z.number().int().nonnegative(),
  })
  .strict();
export const localReviewPackageManifestSummaryEvidencePreviewSchema = z
  .object({
    schemaVersion: z.literal(1), itemId: id, source: z.literal("package-manifest-summary"), title: safeLabel,
    summary: evidenceSummary,
    details: z.object({ applicationVersion: z.string().min(1).max(64), debianVersion: z.string().min(1).max(64), manifestState: z.literal("passed"), checksumState: z.literal("passed"), abiState: z.literal("passed"), provenanceState: z.literal("passed"), visibleLaunchState: z.literal("passed"), installedHostState: z.literal("passed"), artifactCount: z.literal(2), validationComplete: z.literal(true) }).strict(),
    byteSize: z.number().int().positive().max(16 * 1024), sha256, createdAtMs: z.number().int().nonnegative(),
  }).strict();
export const localReviewImagePreviewRequestSchema = z
  .object({ itemId: id, sha256 })
  .strict();
export const localReviewTextPreviewRequestSchema = z
  .object({ collectionId: id, itemId: id, sha256 })
  .strict();
export const localReviewTextPreviewSchema = z
  .object({
    schemaVersion: z.literal(1),
    collectionId: id,
    itemId: id,
    title: safeLabel.nullable(),
    textFormat: localReviewTextFormatSchema.nullable(),
    byteSize: z
      .number()
      .int()
      .positive()
      .max(256 * 1024)
      .nullable(),
    sha256: sha256.nullable(),
    createdAtMs: z.number().int().nonnegative().nullable(),
    state: itemState,
    text: z
      .string()
      .max(128 * 1024)
      .nullable(),
    projectedByteSize: z
      .number()
      .int()
      .nonnegative()
      .max(128 * 1024),
    projectedLineCount: z.number().int().nonnegative().max(2_000),
    projectedCodePointCount: z.number().int().nonnegative().max(32_768),
    truncated: z.boolean(),
    diagnosticCode: diagnostic.nullable(),
  })
  .strict()
  .superRefine((value, context) => {
    if (value.text !== null) {
      if (value.text.includes("\r"))
        context.addIssue({ code: "custom", message: "preview uses LF" });
      if (
        new TextEncoder().encode(value.text).byteLength !==
        value.projectedByteSize
      )
        context.addIssue({
          code: "custom",
          message: "projected bytes match text",
        });
      if ([...value.text].length !== value.projectedCodePointCount)
        context.addIssue({
          code: "custom",
          message: "projected code points match text",
        });
      const renderedLines = value.text.endsWith("\n")
        ? value.text.slice(0, -1).split(/\n/u).length
        : value.text.split(/\n/u).length;
      if (renderedLines > 2_000)
        context.addIssue({ code: "custom", message: "preview line limit" });
    }
    if (
      value.state === "ready" &&
      (value.text === null ||
        value.title === null ||
        value.textFormat === null ||
        value.byteSize === null ||
        value.sha256 === null ||
        value.createdAtMs === null ||
        value.diagnosticCode !== null)
    )
      context.addIssue({
        code: "custom",
        message: "ready preview is complete",
      });
    if (
      (value.state === "stale" ||
        value.state === "unavailable" ||
        value.state === "discarded") &&
      value.text !== null
    )
      context.addIssue({
        code: "custom",
        message: "unsafe content is withheld",
      });
  });
export const localReviewImagePreviewSchema = z
  .object({
    schemaVersion: z.literal(1),
    itemId: id,
    mimeType: z.enum(["image/png", "image/jpeg"]),
    width: z.number().int().positive().max(4096),
    height: z.number().int().positive().max(4096),
    byteSize: z
      .number()
      .int()
      .positive()
      .max(1024 * 1024),
    sha256,
    dataUrl: z
      .string()
      .regex(/^data:image\/(png|jpeg);base64,/u)
      .max(2 * 1024 * 1024),
  })
  .strict()
  .refine(
    (value) => value.dataUrl.startsWith(`data:${value.mimeType};base64,`),
    "data URL MIME must match",
  );
export const localReviewImagePickOutcomeSchema = z.discriminatedUnion(
  "outcome",
  [
    z
      .object({
        outcome: z.literal("created"),
        snapshot: localReviewSnapshotSchema,
      })
      .strict(),
    z
      .object({
        outcome: z.literal("canceled"),
        snapshot: localReviewSnapshotSchema,
      })
      .strict(),
  ],
);

export type LocalReviewSnapshot = z.infer<typeof localReviewSnapshotSchema>;
export type LocalReviewCollectionCreateRequest = z.infer<
  typeof localReviewCollectionCreateRequestSchema
>;
export type LocalReviewTextItemCreateRequest = z.infer<
  typeof localReviewTextItemCreateRequestSchema
>;
export type LocalReviewM48ArtifactCopyRequest = z.infer<
  typeof localReviewM48ArtifactCopyRequestSchema
>;
export type LocalReviewM48GeneratedArtifactMetadataEvidenceCreateRequest =
  z.infer<
    typeof localReviewM48GeneratedArtifactMetadataEvidenceCreateRequestSchema
  >;
export type LocalReviewSafePreviewMetadataEvidenceCreateRequest = z.infer<
  typeof localReviewSafePreviewMetadataEvidenceCreateRequestSchema
>;
export type LocalReviewPackageManifestSummaryEvidenceCreateRequest = z.infer<
  typeof localReviewPackageManifestSummaryEvidenceCreateRequestSchema
>;
export type LocalReviewComparisonCreateRequest = z.infer<
  typeof localReviewComparisonCreateRequestSchema
>;
export type LocalReviewPromotionPrepareRequest = z.infer<
  typeof localReviewPromotionPrepareRequestSchema
>;
