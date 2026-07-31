import { useEffect, useRef, useState, type KeyboardEvent } from "react";

import {
  createLocalReviewCollection,
  createLocalReviewTextItem,
  createLocalReviewM48ArtifactCopy,
  createLocalReviewM48GeneratedArtifactMetadataEvidence,
  loadLocalReview,
  pickLocalReviewImage,
  previewLocalReviewImage,
  previewLocalReviewText,
  createLocalReviewManualEvidence,
  previewLocalReviewManualEvidence,
  previewLocalReviewM48GeneratedArtifactMetadataEvidence,
  claimLocalReviewSafePreviewMetadata,
  createLocalReviewSafePreviewMetadataEvidence,
  previewLocalReviewSafePreviewMetadataEvidence,
  createLocalReviewPackageManifestSummaryEvidence,
  previewLocalReviewPackageManifestSummaryEvidence,
  createLocalReviewGitStatusDiffSummaryEvidence,
  previewLocalReviewGitStatusDiffSummaryEvidence,
  createLocalReviewActivityPresentationEvidence,
  previewLocalReviewActivityPresentationEvidence,
  createLocalReviewApprovalPresentationEvidence,
  discardLocalReviewItem,
  createLocalReviewAnnotation,
  editLocalReviewAnnotation,
  resolveLocalReviewAnnotation,
  reopenLocalReviewAnnotation,
  deleteLocalReviewAnnotation,
  createLocalReviewComparison,
  readLocalReviewComparison,
  discardLocalReviewComparison,
  prepareLocalReviewPromotion,
  confirmLocalReviewPromotion,
  cancelLocalReviewPromotion,
} from "../lib/bridge";
import type { LocalReviewSnapshot } from "../lib/localReview";
import type { ReviewPaneData } from "./types";
import {
  recordLocalReviewActivity,
  setLocalReviewPromotionPresentation,
} from "./localReviewSession";

const unavailable: LocalReviewSnapshot = {
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
  approvalPresentationAvailable: false,
  diagnosticCode: "metadata-unavailable",
};

export default function LocalReviewPane({
  taskCatalog,
  loadArtifacts,
}: ReviewPaneData) {
  const [snapshot, setSnapshot] = useState<LocalReviewSnapshot | null>(null);
  const [title, setTitle] = useState("");
  const [evidenceTitle, setEvidenceTitle] = useState("");
  const [text, setText] = useState("");
  const [busy, setBusy] = useState(false);
  const [selectedItemId, setSelectedItemId] = useState<string | null>(null);
  const [imagePreview, setImagePreview] = useState<Awaited<
    ReturnType<typeof previewLocalReviewImage>
  > | null>(null);
  const [textPreview, setTextPreview] = useState<Awaited<
    ReturnType<typeof previewLocalReviewText>
  > | null>(null);
  const [textPreviewFailedKey, setTextPreviewFailedKey] = useState<
    string | null
  >(null);
  const [error, setError] = useState<string | null>(null);
  const [evidenceSummary, setEvidenceSummary] = useState("");
  const [evidencePreview, setEvidencePreview] = useState<Awaited<
    ReturnType<typeof previewLocalReviewManualEvidence>
  > | null>(null);
  const [annotationText, setAnnotationText] = useState("");
  const [editingAnnotationId, setEditingAnnotationId] = useState<string | null>(
    null,
  );
  const [editingAnnotationText, setEditingAnnotationText] = useState("");
  const [deletingAnnotationId, setDeletingAnnotationId] = useState<
    string | null
  >(null);
  const [comparisonChooserOpen, setComparisonChooserOpen] = useState(false);
  const [comparisonRightItemId, setComparisonRightItemId] = useState("");
  const [selectedComparisonId, setSelectedComparisonId] = useState<
    string | null
  >(null);
  const [comparisonResult, setComparisonResult] = useState<Awaited<
    ReturnType<typeof readLocalReviewComparison>
  > | null>(null);
  const [deletingComparisonId, setDeletingComparisonId] = useState<
    string | null
  >(null);
  const [promotionCandidate, setPromotionCandidate] = useState<Awaited<
    ReturnType<typeof prepareLocalReviewPromotion>
  > | null>(null);
  const [promotionResult, setPromotionResult] = useState<Awaited<
    ReturnType<typeof confirmLocalReviewPromotion>
  > | null>(null);
  const [artifactCopyChooserOpen, setArtifactCopyChooserOpen] = useState(false);
  const [artifactMetadataChooserOpen, setArtifactMetadataChooserOpen] =
    useState(false);
  const [artifactMetadataPreview, setArtifactMetadataPreview] =
    useState<Awaited<
      ReturnType<typeof previewLocalReviewM48GeneratedArtifactMetadataEvidence>
    > | null>(null);
  const [safePreviewClaim, setSafePreviewClaim] = useState<Awaited<
    ReturnType<typeof claimLocalReviewSafePreviewMetadata>
  > | null>(null);
  const [safePreviewEvidencePreview, setSafePreviewEvidencePreview] =
    useState<Awaited<
      ReturnType<typeof previewLocalReviewSafePreviewMetadataEvidence>
    > | null>(null);
  const [packageManifestEvidencePreview, setPackageManifestEvidencePreview] =
    useState<Awaited<ReturnType<typeof previewLocalReviewPackageManifestSummaryEvidence>> | null>(null);
  const [gitEvidencePreview, setGitEvidencePreview] = useState<Awaited<ReturnType<typeof previewLocalReviewGitStatusDiffSummaryEvidence>> | null>(null);
  const [activityEvidencePreview, setActivityEvidencePreview] = useState<Awaited<ReturnType<typeof previewLocalReviewActivityPresentationEvidence>> | null>(null);
  const [artifactCandidates, setArtifactCandidates] = useState<
    Awaited<ReturnType<ReviewPaneData["loadArtifacts"]>>["artifacts"]
  >([]);
  const [selectedArtifactId, setSelectedArtifactId] = useState("");
  const [compact, setCompact] = useState(
    () =>
      window.matchMedia?.("(max-width: 760px), (max-height: 520px)").matches ??
      false,
  );
  const [compactLevel, setCompactLevel] = useState<
    "collections" | "items" | "detail"
  >("collections");
  const evidenceHeading = useRef<HTMLHeadingElement>(null);
  const annotationsHeading = useRef<HTMLHeadingElement>(null);
  const focusAnnotationsRequested = useRef(false);
  const errorSummary = useRef<HTMLParagraphElement>(null);
  const imageTrigger = useRef<HTMLButtonElement>(null);
  const previewHeading = useRef<HTMLHeadingElement>(null);
  const compareTrigger = useRef<HTMLButtonElement>(null);
  const comparisonHeading = useRef<HTMLHeadingElement>(null);
  const comparisonsHeading = useRef<HTMLHeadingElement>(null);
  const promotionTrigger = useRef<HTMLButtonElement>(null);
  const promotionDialogHeading = useRef<HTMLHeadingElement>(null);
  const promotionCancel = useRef<HTMLButtonElement>(null);
  const promotionResultHeading = useRef<HTMLHeadingElement>(null);
  const artifactCopyTrigger = useRef<HTMLButtonElement>(null);
  const artifactMetadataTrigger = useRef<HTMLButtonElement>(null);
  const safePreviewMetadataTrigger = useRef<HTMLButtonElement>(null);
  const textPreviewRequest = useRef(0);
  const previewCollectionId =
    snapshot?.selectedCollection?.collectionId ?? null;
  const previewItem = selectedItemId
    ? snapshot?.items.find((candidate) => candidate.itemId === selectedItemId)
    : undefined;
  const previewKey = previewItem
    ? `${previewItem.itemId}:${previewItem.sha256}`
    : null;

  useEffect(() => {
    if (!window.matchMedia) return;
    const query = window.matchMedia("(max-width: 760px), (max-height: 520px)");
    const sync = () => setCompact(query.matches);
    sync();
    query.addEventListener("change", sync);
    return () => query.removeEventListener("change", sync);
  }, []);

  const load = (selectedCollectionId: string | null) => {
    setBusy(true);
    void loadLocalReview({ selectedCollectionId })
      .then(setSnapshot)
      .catch(() => setSnapshot(unavailable))
      .finally(() => setBusy(false));
  };
  useEffect(() => {
    void loadLocalReview({ selectedCollectionId: null })
      .then(setSnapshot)
      .catch(() => setSnapshot(unavailable));
  }, []);
  useEffect(() => {
    if (error) errorSummary.current?.focus();
  }, [error]);
  useEffect(() => {
    if (
      evidencePreview ||
      artifactMetadataPreview ||
      safePreviewEvidencePreview
      || packageManifestEvidencePreview
    )
      evidenceHeading.current?.focus();
  }, [artifactMetadataPreview, evidencePreview, safePreviewEvidencePreview, packageManifestEvidencePreview]);
  useEffect(() => {
    if (focusAnnotationsRequested.current) {
      annotationsHeading.current?.focus();
      focusAnnotationsRequested.current = false;
    }
  }, [snapshot]);
  useEffect(() => {
    if (comparisonResult) comparisonHeading.current?.focus();
  }, [comparisonResult]);
  useEffect(() => {
    if (promotionCandidate) {
      promotionCancel.current?.focus();
    }
  }, [promotionCandidate]);
  useEffect(() => {
    if (promotionResult) promotionResultHeading.current?.focus();
  }, [promotionResult]);
  useEffect(() => {
    const requestId = ++textPreviewRequest.current;
    if (!previewCollectionId || !previewItem || previewItem.class !== "text")
      return;
    void previewLocalReviewText({
      collectionId: previewCollectionId,
      itemId: previewItem.itemId,
      sha256: previewItem.sha256,
    })
      .then((preview) => {
        if (
          textPreviewRequest.current === requestId &&
          selectedItemId === previewItem.itemId
        ) {
          setTextPreviewFailedKey(null);
          setTextPreview(preview);
        }
      })
      .catch(() => {
        if (
          textPreviewRequest.current === requestId &&
          selectedItemId === previewItem.itemId
        ) {
          setTextPreview(null);
          setTextPreviewFailedKey(
            `${previewItem.itemId}:${previewItem.sha256}`,
          );
        }
      });
  }, [previewCollectionId, previewItem, selectedItemId]);
  const selectedTask = taskCatalog.selectedTask;
  const selectCollection = (collectionId: string) => {
    load(collectionId);
    if (compact) setCompactLevel("items");
  };
  const selectItem = (itemId: string) => {
    textPreviewRequest.current += 1;
    setTextPreview(null);
    setTextPreviewFailedKey(null);
    setImagePreview(null);
    setEvidencePreview(null);
    setArtifactMetadataPreview(null);
    setPackageManifestEvidencePreview(null);
    setSelectedItemId(itemId);
    if (compact) setCompactLevel("detail");
  };
  const moveItem = (
    event: KeyboardEvent<HTMLButtonElement>,
    itemId: string,
  ) => {
    const items = snapshot?.items ?? [];
    const index = items.findIndex((item) => item.itemId === itemId);
    const target =
      event.key === "Home"
        ? 0
        : event.key === "End"
          ? items.length - 1
          : event.key === "ArrowDown"
            ? index + 1
            : event.key === "ArrowUp"
              ? index - 1
              : index;
    if (target === index || target < 0 || target >= items.length) return;
    const targetItem = items[target];
    if (!targetItem) return;
    event.preventDefault();
    selectItem(targetItem.itemId);
    queueMicrotask(() =>
      document.getElementById(`review-item-${targetItem.itemId}`)?.focus(),
    );
  };
  const createCollection = () => {
    if (!selectedTask || !title.trim()) return;
    setBusy(true);
    void createLocalReviewCollection({
      taskId: selectedTask.id,
      planId: selectedTask.selectedPlanId,
      title,
    })
      .then((next) => {
        setSnapshot(next);
        setTitle("");
        recordLocalReviewActivity({
          kind: "collection-created",
          label: next.selectedCollection?.title ?? title,
          status: "success",
        });
      })
      .catch(() => setSnapshot(unavailable))
      .finally(() => setBusy(false));
  };
  const createText = () => {
    const collection = snapshot?.selectedCollection;
    if (!collection || !title.trim() || !text) return;
    setBusy(true);
    void createLocalReviewTextItem({
      collectionId: collection.collectionId,
      expectedCollectionUpdatedAtMs: collection.updatedAtMs,
      title,
      textFormat: "plain",
      content: text,
    })
      .then((next) => {
        setSnapshot(next);
        setTitle("");
        setText("");
        recordLocalReviewActivity({
          kind: "item-added",
          label: title,
          status: "success",
        });
      })
      .catch(() => setSnapshot(unavailable))
      .finally(() => setBusy(false));
  };
  const openArtifactCopyChooser = () => {
    const collection = snapshot?.selectedCollection;
    if (!collection || collection.state !== "active") return;
    setBusy(true);
    setError(null);
    void loadArtifacts()
      .then((artifacts) => {
        const eligible = artifacts.artifacts.filter(
          (artifact) =>
            artifact.state === "ready" &&
            ["text", "markdown", "json", "csv", "python"].includes(
              artifact.class,
            ),
        );
        if (!eligible.length) {
          setError("No live generated text artifact is available to copy.");
          return;
        }
        setArtifactCandidates(eligible);
        setSelectedArtifactId(eligible[0]?.artifactId ?? "");
        setArtifactCopyChooserOpen(true);
      })
      .catch(() => setError("Live generated artifacts are unavailable."))
      .finally(() => setBusy(false));
  };
  const cancelArtifactCopy = () => {
    setArtifactCopyChooserOpen(false);
    setArtifactCandidates([]);
    setSelectedArtifactId("");
    setTimeout(() => artifactCopyTrigger.current?.focus(), 0);
  };
  const copyArtifact = () => {
    const collection = snapshot?.selectedCollection;
    const artifact = artifactCandidates.find(
      (candidate) => candidate.artifactId === selectedArtifactId,
    );
    if (!collection || !artifact) return;
    setBusy(true);
    setError(null);
    void createLocalReviewM48ArtifactCopy({
      collectionId: collection.collectionId,
      expectedCollectionUpdatedAtMs: collection.updatedAtMs,
      artifactId: artifact.artifactId,
      manifestSha256: artifact.sha256,
    })
      .then((next) => {
        setSnapshot(next);
        setArtifactCopyChooserOpen(false);
        setArtifactCandidates([]);
        setSelectedArtifactId("");
        const copied = next.items.find(
          (item) =>
            item.sourceKind === "m48-artifact-copy" &&
            item.sha256 === artifact.sha256 &&
            item.title === artifact.displayLabel,
        );
        if (copied) {
          setSelectedItemId(copied.itemId);
          setTimeout(
            () =>
              document.getElementById(`review-item-${copied.itemId}`)?.focus(),
            0,
          );
          recordLocalReviewActivity({
            kind: "item-added",
            label: copied.title,
            status: "success",
            digest: copied.sha256,
          });
        }
      })
      .catch(() =>
        setError(
          "The generated artifact could not be copied into local review.",
        ),
      )
      .finally(() => setBusy(false));
  };
  const openArtifactMetadataChooser = () => {
    const collection = snapshot?.selectedCollection;
    if (!collection || collection.state !== "active") return;
    setBusy(true);
    setError(null);
    void loadArtifacts()
      .then((artifacts) => {
        const eligible = artifacts.artifacts.filter(
          (artifact) =>
            artifact.state === "ready" &&
            ["text", "markdown", "json", "csv", "python"].includes(
              artifact.class,
            ),
        );
        if (!eligible.length) {
          setError(
            "No live generated artifact metadata is available to capture.",
          );
          return;
        }
        setArtifactCandidates(eligible);
        setSelectedArtifactId(eligible[0]?.artifactId ?? "");
        setArtifactMetadataChooserOpen(true);
      })
      .catch(() => setError("Live generated artifacts are unavailable."))
      .finally(() => setBusy(false));
  };
  const cancelArtifactMetadata = () => {
    setArtifactMetadataChooserOpen(false);
    setArtifactCandidates([]);
    setSelectedArtifactId("");
    setTimeout(() => artifactMetadataTrigger.current?.focus(), 0);
  };
  const captureArtifactMetadata = () => {
    const collection = snapshot?.selectedCollection;
    const artifact = artifactCandidates.find(
      (candidate) => candidate.artifactId === selectedArtifactId,
    );
    if (!collection || !artifact) return;
    setBusy(true);
    setError(null);
    void createLocalReviewM48GeneratedArtifactMetadataEvidence({
      collectionId: collection.collectionId,
      expectedCollectionUpdatedAtMs: collection.updatedAtMs,
      artifactId: artifact.artifactId,
      manifestSha256: artifact.sha256,
    })
      .then((result) => {
        setSnapshot(result.snapshot);
        if (result.outcome !== "created") {
          setError("Generated-artifact metadata could not be captured.");
          return;
        }
        const item = result.snapshot.items.find(
          (candidate) =>
            candidate.itemId === result.createdItemId &&
            candidate.class === "evidence" &&
            candidate.evidenceSource === result.source,
        );
        if (!item) {
          setError("Generated-artifact metadata could not be captured.");
          return;
        }
        setArtifactMetadataChooserOpen(false);
        setArtifactCandidates([]);
        setSelectedArtifactId("");
        setSelectedItemId(item.itemId);
        recordLocalReviewActivity({
          kind: "item-added",
          label: item.title,
          status: "success",
          digest: item.sha256,
        });
        return previewLocalReviewM48GeneratedArtifactMetadataEvidence({
          itemId: item.itemId,
          sha256: item.sha256,
        }).then((preview) => {
          setArtifactMetadataPreview(preview);
        });
      })
      .catch(() =>
        setError("Generated-artifact metadata could not be captured."),
      )
      .finally(() => setBusy(false));
  };
  const prepareSafePreviewMetadata = () => {
    if (snapshot?.selectedCollection?.state !== "active") return;
    setBusy(true);
    setError(null);
    void claimLocalReviewSafePreviewMetadata()
      .then(setSafePreviewClaim)
      .catch(() => setError("No current safe-preview metadata is available."))
      .finally(() => setBusy(false));
  };
  const cancelSafePreviewMetadata = () => {
    setSafePreviewClaim(null);
    setTimeout(() => safePreviewMetadataTrigger.current?.focus(), 0);
  };
  const captureSafePreviewMetadata = () => {
    const collection = snapshot?.selectedCollection;
    if (!collection || !safePreviewClaim) return;
    setBusy(true);
    setError(null);
    void createLocalReviewSafePreviewMetadataEvidence({
      collectionId: collection.collectionId,
      expectedCollectionUpdatedAtMs: collection.updatedAtMs,
      previewClaimId: safePreviewClaim.claimId,
      claimSha256: safePreviewClaim.claimSha256,
    })
      .then((result) => {
        setSnapshot(result.snapshot);
        if (result.outcome !== "created") {
          setError("Safe-preview metadata could not be captured.");
          return;
        }
        const item = result.snapshot.items.find(
          (candidate) =>
            candidate.itemId === result.createdItemId &&
            candidate.class === "evidence" &&
            candidate.evidenceSource === result.source,
        );
        if (!item) {
          setError("Safe-preview metadata could not be captured.");
          return;
        }
        setSafePreviewClaim(null);
        setSelectedItemId(item.itemId);
        recordLocalReviewActivity({
          kind: "item-added",
          label: item.title,
          status: "success",
          digest: item.sha256,
        });
        return previewLocalReviewSafePreviewMetadataEvidence({
          itemId: item.itemId,
          sha256: item.sha256,
        }).then(setSafePreviewEvidencePreview);
      })
      .catch(() => setError("Safe-preview metadata could not be captured."))
      .finally(() => setBusy(false));
  };
  const chooseImage = () => {
    const collection = snapshot?.selectedCollection;
    if (!collection || !title.trim()) return;
    setBusy(true);
    setError(null);
    void pickLocalReviewImage({
      collectionId: collection.collectionId,
      expectedCollectionUpdatedAtMs: collection.updatedAtMs,
      title,
    })
      .then((result) => {
        if (result.outcome === "canceled") {
          setSnapshot(result.snapshot);
          imageTrigger.current?.focus();
          return;
        }
        setSnapshot(result.snapshot);
        setTitle("");
        const created = result.snapshot.items.find(
          (item) => item.class === "image-mockup",
        );
        if (!created) return;
        recordLocalReviewActivity({
          kind: "item-added",
          label: created.title,
          status: "success",
          digest: created.sha256,
        });
        setSelectedItemId(created.itemId);
        return previewLocalReviewImage({
          itemId: created.itemId,
          sha256: created.sha256,
        }).then((preview) => {
          setImagePreview(preview);
          queueMicrotask(() => previewHeading.current?.focus());
        });
      })
      .catch(() => setError("Image mockup could not be added."))
      .finally(() => setBusy(false));
  };
  const createEvidence = () => {
    const collection = snapshot?.selectedCollection;
    if (!collection || !evidenceTitle.trim() || !evidenceSummary) return;
    setBusy(true);
    setError(null);
    void createLocalReviewManualEvidence({
      collectionId: collection.collectionId,
      expectedCollectionUpdatedAtMs: collection.updatedAtMs,
      title: evidenceTitle,
      summary: evidenceSummary,
    })
      .then((result) => {
        setSnapshot(result.snapshot);
        if (result.outcome !== "created") {
          setError("Evidence snapshot could not be added.");
          return;
        }
        const item = result.snapshot.items.find(
          (candidate) =>
            candidate.itemId === result.createdItemId &&
            candidate.class === "evidence" &&
            candidate.evidenceSource === result.source,
        );
        if (!item) {
          setError("Evidence snapshot could not be added.");
          return;
        }
        setEvidenceTitle("");
        setEvidenceSummary("");
        recordLocalReviewActivity({
          kind: "item-added",
          label: item.title,
          status: "success",
          digest: item.sha256,
        });
        setSelectedItemId(item.itemId);
        return previewLocalReviewManualEvidence({
          itemId: item.itemId,
          sha256: item.sha256,
        }).then(setEvidencePreview);
      })
      .catch(() => {
        setError("Evidence snapshot could not be added.");
      })
      .finally(() => setBusy(false));
  };
  const capturePackageManifestSummary = () => {
    const collection = snapshot?.selectedCollection;
    if (!collection || busy || !snapshot?.packageManifestSummaryAvailable) return;
    setBusy(true);
    setError(null);
    void createLocalReviewPackageManifestSummaryEvidence({
      collectionId: collection.collectionId,
      expectedCollectionUpdatedAtMs: collection.updatedAtMs,
    }).then((result) => {
      setSnapshot(result.snapshot);
      if (result.outcome !== "created") { setError("Package validation summary could not be captured."); return; }
      const item = result.snapshot.items.find((candidate) => candidate.itemId === result.createdItemId && candidate.class === "evidence" && candidate.evidenceSource === result.source);
      if (!item) { setError("Package validation summary could not be captured."); return; }
      setSelectedItemId(item.itemId);
      recordLocalReviewActivity({ kind: "item-added", label: item.title, status: "success", digest: item.sha256 });
      return previewLocalReviewPackageManifestSummaryEvidence({ itemId: item.itemId, sha256: item.sha256 }).then(setPackageManifestEvidencePreview);
    }).catch(() => setError("Package validation summary could not be captured.")).finally(() => setBusy(false));
  };
  const captureGitStatusDiffSummary = () => {
    const collection = snapshot?.selectedCollection;
    if (!collection || busy || !snapshot?.gitStatusDiffSummaryAvailable) return;
    setBusy(true); setError(null);
    void createLocalReviewGitStatusDiffSummaryEvidence({ collectionId: collection.collectionId, expectedCollectionUpdatedAtMs: collection.updatedAtMs }).then((result) => {
      setSnapshot(result.snapshot);
      if (result.outcome !== "created") { setError("Git status and diff summary could not be captured."); return; }
      const item = result.snapshot.items.find((candidate) => candidate.itemId === result.createdItemId && candidate.class === "evidence" && candidate.evidenceSource === result.source);
      if (!item) { setError("Git status and diff summary could not be captured."); return; }
      setSelectedItemId(item.itemId);
      recordLocalReviewActivity({ kind: "item-added", label: item.title, status: "success", digest: item.sha256 });
      return previewLocalReviewGitStatusDiffSummaryEvidence({ itemId: item.itemId, sha256: item.sha256 }).then(setGitEvidencePreview);
    }).catch(() => setError("Git status and diff summary could not be captured.")).finally(() => setBusy(false));
  };
  const captureActivityPresentation = () => {
    const collection = snapshot?.selectedCollection;
    if (!collection || busy || !snapshot?.activityPresentationAvailable) return;
    setBusy(true); setError(null);
    void createLocalReviewActivityPresentationEvidence({ collectionId: collection.collectionId, expectedCollectionUpdatedAtMs: collection.updatedAtMs }).then((result) => {
      setSnapshot(result.snapshot); if (result.outcome !== "created") { setError("Activity presentation could not be captured."); return; }
      const item = result.snapshot.items.find((candidate) => candidate.itemId === result.createdItemId && candidate.class === "evidence" && candidate.evidenceSource === result.source);
      if (!item) { setError("Activity presentation could not be captured."); return; }
      setSelectedItemId(item.itemId); recordLocalReviewActivity({ kind: "item-added", label: item.title, status: "success", digest: item.sha256 });
      return previewLocalReviewActivityPresentationEvidence({ itemId: item.itemId, sha256: item.sha256 }).then(setActivityEvidencePreview);
    }).catch(() => setError("Activity presentation could not be captured.")).finally(() => setBusy(false));
  };
  const captureApprovalPresentation = () => {
    const collection = snapshot?.selectedCollection;
    if (!collection || busy || !snapshot?.approvalPresentationAvailable) return;
    setBusy(true); setError(null);
    void createLocalReviewApprovalPresentationEvidence({ collectionId: collection.collectionId, expectedCollectionUpdatedAtMs: collection.updatedAtMs }).then((result) => {
      setSnapshot(result.snapshot); if (result.outcome !== "created") { setError("Approval presentation could not be captured."); return; }
      const item = result.snapshot.items.find((candidate) => candidate.itemId === result.createdItemId && candidate.class === "evidence" && candidate.evidenceSource === result.source);
      if (!item) { setError("Approval presentation could not be captured."); return; }
      setSelectedItemId(item.itemId); recordLocalReviewActivity({ kind: "item-added", label: item.title, status: "success", digest: item.sha256 });
    }).catch(() => setError("Approval presentation could not be captured.")).finally(() => setBusy(false));
  };
  const discardSelectedItem = () => {
    const collection = snapshot?.selectedCollection;
    if (
      !collection ||
      !selectedItemId ||
      !window.confirm("Discard this local review item?")
    )
      return;
    const discardedItem = snapshot.items.find(
      (item) => item.itemId === selectedItemId,
    );
    setBusy(true);
    setError(null);
    void discardLocalReviewItem({
      collectionId: collection.collectionId,
      itemId: selectedItemId,
      expectedCollectionUpdatedAtMs: collection.updatedAtMs,
    })
      .then((next) => {
        setSnapshot(next);
        setSelectedItemId(next.items[0]?.itemId ?? null);
        setEvidencePreview(null);
        setImagePreview(null);
        recordLocalReviewActivity({
          kind: "item-discarded",
          label: discardedItem?.title ?? "Local review item",
          status: "success",
        });
      })
      .catch(() => {
        setError("Local review item could not be discarded.");
      })
      .finally(() => setBusy(false));
  };
  const createAnnotation = () => {
    const collection = snapshot?.selectedCollection;
    if (!collection || !selectedItemId || !annotationText) return;
    setBusy(true);
    setError(null);
    void createLocalReviewAnnotation({
      collectionId: collection.collectionId,
      itemId: selectedItemId,
      expectedCollectionUpdatedAtMs: collection.updatedAtMs,
      text: annotationText,
    })
      .then((next) => {
        setSnapshot(next);
        setAnnotationText("");
        focusAnnotationsRequested.current = true;
        recordLocalReviewActivity({
          kind: "annotation-added",
          label:
            snapshot.items.find((item) => item.itemId === selectedItemId)
              ?.title ?? "Local review item",
          status: "success",
        });
      })
      .catch(() => setError("Annotation could not be added."))
      .finally(() => setBusy(false));
  };
  const mutateAnnotation = (
    annotationId: string,
    operation: "edit" | "resolve" | "reopen",
  ) => {
    const collection = snapshot?.selectedCollection;
    if (!collection || !selectedItemId) return;
    setBusy(true);
    setError(null);
    const request = {
      collectionId: collection.collectionId,
      itemId: selectedItemId,
      annotationId,
      expectedCollectionUpdatedAtMs: collection.updatedAtMs,
    };
    const mutation =
      operation === "edit"
        ? editLocalReviewAnnotation({ ...request, text: editingAnnotationText })
        : operation === "resolve"
          ? resolveLocalReviewAnnotation(request)
          : reopenLocalReviewAnnotation(request);
    void mutation
      .then((next) => {
        setSnapshot(next);
        setTimeout(
          () => document.getElementById(`annotation-${annotationId}`)?.focus(),
          0,
        );
        if (operation === "edit") {
          setEditingAnnotationId(null);
          setEditingAnnotationText("");
        }
        recordLocalReviewActivity({
          kind:
            operation === "edit"
              ? "annotation-edited"
              : operation === "resolve"
                ? "annotation-resolved"
                : "annotation-reopened",
          label:
            snapshot.items.find((item) => item.itemId === selectedItemId)
              ?.title ?? "Local review item",
          status: "success",
        });
      })
      .catch(() =>
        setError(
          `Annotation could not be ${operation === "edit" ? "updated" : operation === "resolve" ? "resolved" : "reopened"}.`,
        ),
      )
      .finally(() => setBusy(false));
  };
  const cancelAnnotationEdit = (annotationId: string) => {
    setEditingAnnotationId(null);
    setEditingAnnotationText("");
    setTimeout(
      () => document.getElementById(`annotation-edit-${annotationId}`)?.focus(),
      0,
    );
  };
  const cancelAnnotationDelete = (annotationId: string) => {
    setDeletingAnnotationId(null);
    setTimeout(
      () =>
        document.getElementById(`annotation-delete-${annotationId}`)?.focus(),
      0,
    );
  };
  const deleteAnnotation = (annotationId: string) => {
    const collection = snapshot?.selectedCollection;
    if (!collection || !selectedItemId) return;
    const selectedItem = snapshot.items.find(
      (item) => item.itemId === selectedItemId,
    );
    const previous = [...(selectedItem?.annotations ?? [])].sort(
      (left, right) => {
        const state =
          (left.state === "open" ? 0 : 1) - (right.state === "open" ? 0 : 1);
        return (
          state ||
          left.createdAtMs - right.createdAtMs ||
          left.annotationId.localeCompare(right.annotationId)
        );
      },
    );
    const deletedIndex = previous.findIndex(
      (annotation) => annotation.annotationId === annotationId,
    );
    setBusy(true);
    setError(null);
    void deleteLocalReviewAnnotation({
      collectionId: collection.collectionId,
      itemId: selectedItemId,
      annotationId,
      expectedCollectionUpdatedAtMs: collection.updatedAtMs,
    })
      .then((next) => {
        setSnapshot(next);
        setDeletingAnnotationId(null);
        const nextItem = next.items.find(
          (item) => item.itemId === selectedItemId,
        );
        const remaining = [...(nextItem?.annotations ?? [])].sort(
          (left, right) => {
            const state =
              (left.state === "open" ? 0 : 1) -
              (right.state === "open" ? 0 : 1);
            return (
              state ||
              left.createdAtMs - right.createdAtMs ||
              left.annotationId.localeCompare(right.annotationId)
            );
          },
        );
        const nextId = previous[deletedIndex + 1]?.annotationId;
        const previousId =
          deletedIndex > 0
            ? previous[deletedIndex - 1]?.annotationId
            : undefined;
        const targetId = remaining.some(
          (annotation) => annotation.annotationId === nextId,
        )
          ? nextId
          : remaining.some(
                (annotation) => annotation.annotationId === previousId,
              )
            ? previousId
            : undefined;
        setTimeout(
          () =>
            (targetId
              ? document.getElementById(`annotation-${targetId}`)
              : document.getElementById("add-annotation")
            )?.focus(),
          0,
        );
        recordLocalReviewActivity({
          kind: "annotation-deleted",
          label: selectedItem?.title ?? "Local review item",
          status: "success",
        });
      })
      .catch(() => {
        setError("Annotation could not be deleted.");
        setTimeout(() => errorSummary.current?.focus(), 0);
      })
      .finally(() => setBusy(false));
  };
  const readComparison = (comparisonId: string) => {
    const collection = snapshot?.selectedCollection;
    const comparison = snapshot?.comparisons.find(
      (candidate) => candidate.comparisonId === comparisonId,
    );
    if (!collection || !comparison) return;
    setSelectedComparisonId(comparisonId);
    setComparisonResult(null);
    if (comparison.state !== "ready") return;
    setBusy(true);
    setError(null);
    void readLocalReviewComparison({
      collectionId: collection.collectionId,
      comparisonId,
    })
      .then(setComparisonResult)
      .catch(() => setError("Comparison content is unavailable."))
      .finally(() => setBusy(false));
  };
  const createComparison = () => {
    const collection = snapshot?.selectedCollection;
    if (!collection || !selectedItemId || !comparisonRightItemId) return;
    setBusy(true);
    setError(null);
    void createLocalReviewComparison({
      collectionId: collection.collectionId,
      leftItemId: selectedItemId,
      rightItemId: comparisonRightItemId,
      expectedCollectionUpdatedAtMs: collection.updatedAtMs,
    })
      .then((next) => {
        setSnapshot(next);
        setComparisonChooserOpen(false);
        const created = [...next.comparisons]
          .reverse()
          .find(
            (comparison) =>
              comparison.leftItemId === selectedItemId &&
              comparison.rightItemId === comparisonRightItemId,
          );
        if (created) {
          recordLocalReviewActivity({
            kind: "comparison-created",
            label: `${snapshot.items.find((item) => item.itemId === selectedItemId)?.title ?? "Text"} comparison`,
            status: "success",
          });
          readComparisonAfterCreate(next, created.comparisonId);
        }
      })
      .catch(() => setError("Comparison could not be created."))
      .finally(() => setBusy(false));
  };
  const readComparisonAfterCreate = (
    next: LocalReviewSnapshot,
    comparisonId: string,
  ) => {
    const comparison = next.comparisons.find(
      (candidate) => candidate.comparisonId === comparisonId,
    );
    if (!comparison || comparison.state !== "ready") {
      setSelectedComparisonId(comparisonId);
      return;
    }
    setSelectedComparisonId(comparisonId);
    void readLocalReviewComparison({
      collectionId: comparison.collectionId,
      comparisonId,
    })
      .then(setComparisonResult)
      .catch(() => setError("Comparison content is unavailable."));
  };
  const cancelComparisonChooser = () => {
    setComparisonChooserOpen(false);
    setComparisonRightItemId("");
    setTimeout(() => compareTrigger.current?.focus(), 0);
  };
  const discardComparison = (comparisonId: string) => {
    const collection = snapshot?.selectedCollection;
    if (!collection) return;
    const ordered = [...snapshot.comparisons].sort(
      (left, right) =>
        left.createdAtMs - right.createdAtMs ||
        left.comparisonId.localeCompare(right.comparisonId),
    );
    const index = ordered.findIndex(
      (comparison) => comparison.comparisonId === comparisonId,
    );
    setBusy(true);
    setError(null);
    void discardLocalReviewComparison({
      collectionId: collection.collectionId,
      comparisonId,
      expectedCollectionUpdatedAtMs: collection.updatedAtMs,
    })
      .then((next) => {
        setSnapshot(next);
        setDeletingComparisonId(null);
        setComparisonResult(null);
        const nextId = ordered[index + 1]?.comparisonId;
        const previousId =
          index > 0 ? ordered[index - 1]?.comparisonId : undefined;
        const targetId = next.comparisons.some(
          (comparison) => comparison.comparisonId === nextId,
        )
          ? nextId
          : next.comparisons.some(
                (comparison) => comparison.comparisonId === previousId,
              )
            ? previousId
            : undefined;
        setSelectedComparisonId(targetId ?? null);
        setTimeout(
          () =>
            (targetId
              ? document.getElementById(`comparison-${targetId}`)
              : (compareTrigger.current ?? comparisonsHeading.current)
            )?.focus(),
          0,
        );
        recordLocalReviewActivity({
          kind: "comparison-discarded",
          label: "Text comparison",
          status: "success",
        });
      })
      .catch(() => setError("Comparison could not be discarded."))
      .finally(() => setBusy(false));
  };
  const cancelComparisonDiscard = (comparisonId: string) => {
    setDeletingComparisonId(null);
    setTimeout(
      () =>
        document.getElementById(`comparison-discard-${comparisonId}`)?.focus(),
      0,
    );
  };
  const preparePromotion = () => {
    const collection = snapshot?.selectedCollection;
    if (!collection || !selectedItemId) return;
    setBusy(true);
    setError(null);
    void prepareLocalReviewPromotion({
      collectionId: collection.collectionId,
      itemId: selectedItemId,
      expectedCollectionUpdatedAtMs: collection.updatedAtMs,
    })
      .then((candidate) => {
        setPromotionCandidate(candidate);
        setLocalReviewPromotionPresentation({
          state: "prepared",
          label: candidate.title,
          destinationClass: candidate.destinationClass,
          sha256: candidate.sha256,
          expiresAtMs: candidate.expiresAtMs,
        });
        recordLocalReviewActivity({
          kind: "promotion-prepared",
          label: candidate.title,
          status: "success",
          digest: candidate.sha256,
        });
      })
      .catch(() => {
        setError(
          "A transient generated artifact could not be prepared. Refresh eligibility and try again.",
        );
        setLocalReviewPromotionPresentation({ state: "unavailable" });
        recordLocalReviewActivity({
          kind: "promotion-failed",
          label: "Transient generated artifact",
          status: "failed",
          reason: "Promotion preparation was unavailable.",
        });
      })
      .finally(() => setBusy(false));
  };
  const cancelPromotion = () => {
    if (!promotionCandidate) return;
    setBusy(true);
    setError(null);
    void cancelLocalReviewPromotion({
      reservationId: promotionCandidate.reservationId,
    })
      .then(() => {
        recordLocalReviewActivity({
          kind: "promotion-canceled",
          label: promotionCandidate.title,
          status: "info",
        });
        setLocalReviewPromotionPresentation({ state: "expired" });
        setPromotionCandidate(null);
        setTimeout(() => promotionTrigger.current?.focus(), 0);
      })
      .catch(() => {
        setError("The transient artifact reservation could not be canceled.");
        recordLocalReviewActivity({
          kind: "promotion-failed",
          label: promotionCandidate.title,
          status: "failed",
          reason: "Reservation cancellation was unavailable.",
        });
      })
      .finally(() => setBusy(false));
  };
  const confirmPromotion = () => {
    if (!promotionCandidate) return;
    setBusy(true);
    setError(null);
    void confirmLocalReviewPromotion({
      reservationId: promotionCandidate.reservationId,
    })
      .then((result) => {
        setPromotionResult(result);
        setPromotionCandidate(null);
        setLocalReviewPromotionPresentation({
          state: "succeeded",
          label: result.displayLabel,
          destinationClass: result.class,
          sha256: result.sha256,
          expiresAtMs: result.expiresAt,
        });
        recordLocalReviewActivity({
          kind: "promotion-succeeded",
          label: result.displayLabel,
          status: "success",
          digest: result.sha256,
        });
      })
      .catch(() => {
        setError(
          "The transient generated artifact could not be created. Prepare a new reservation if eligibility changed.",
        );
        setLocalReviewPromotionPresentation({ state: "unavailable" });
        recordLocalReviewActivity({
          kind: "promotion-failed",
          label: promotionCandidate.title,
          status: "failed",
          reason: "Promotion confirmation was unavailable.",
        });
      })
      .finally(() => setBusy(false));
  };

  if (!snapshot) return <p role="status">Loading local review…</p>;
  if (snapshot.diagnosticCode) {
    return <p role="alert">Local review is unavailable.</p>;
  }
  return (
    <div
      className="review-pane-list local-review-pane"
      data-compact={compact ? "true" : "false"}
    >
      <p>
        Copied local review is not a file, approval, dispatch, save, or
        execution surface.
      </p>
      {snapshot.warning ? (
        <p role="status">Near local review capacity.</p>
      ) : null}
      {error ? (
        <p ref={errorSummary} role="alert" tabIndex={-1}>
          {error}
        </p>
      ) : null}
      {(!compact || compactLevel === "collections") && (
        <>
          <h3>Collections</h3>
          {snapshot.collections.length ? (
            <ul aria-label="Review collections">
              {snapshot.collections.map((collection) => (
                <li key={collection.collectionId}>
                  <button
                    type="button"
                    aria-pressed={
                      snapshot.selectedCollection?.collectionId ===
                      collection.collectionId
                    }
                    id={`review-collection-${collection.collectionId}`}
                    onClick={() => selectCollection(collection.collectionId)}
                  >
                    {collection.title} — {collection.state}
                  </button>
                </li>
              ))}
            </ul>
          ) : (
            <p>No review collections/items.</p>
          )}
        </>
      )}
      {!snapshot.selectedCollection ? (
        <section aria-label="Create local review collection">
          <label>
            Collection title
            <input
              value={title}
              maxLength={120}
              onChange={(event) => setTitle(event.target.value)}
            />
          </label>
          <button
            type="button"
            disabled={busy || !selectedTask || !title.trim()}
            onClick={createCollection}
          >
            New collection
          </button>
          {!selectedTask ? (
            <p>Select a task in Task Catalog before creating a collection.</p>
          ) : null}
        </section>
      ) : !compact || compactLevel !== "collections" ? (
        <section aria-label="Selected local review collection">
          {compact ? (
            <button
              type="button"
              onClick={() => {
                setCompactLevel("collections");
                queueMicrotask(() =>
                  document
                    .getElementById(
                      `review-collection-${snapshot.selectedCollection?.collectionId}`,
                    )
                    ?.focus(),
                );
              }}
            >
              Back to collections
            </button>
          ) : null}
          <h3>{snapshot.selectedCollection.title}</h3>
          <p>
            {snapshot.selectedCollection.state} ·{" "}
            {snapshot.selectedCollection.payloadBytes} bytes
          </p>
          {(!compact || compactLevel === "items") && (
            <ul aria-label="Review items">
              {snapshot.items.map((item) => (
                <li key={item.itemId}>
                  <button
                    id={`review-item-${item.itemId}`}
                    type="button"
                    aria-pressed={selectedItemId === item.itemId}
                    onKeyDown={(event) => moveItem(event, item.itemId)}
                    onClick={() => selectItem(item.itemId)}
                  >
                    <strong>{item.title}</strong> — {item.class} · {item.state}{" "}
                    · {item.sha256.slice(0, 12)}
                  </button>
                </li>
              ))}
            </ul>
          )}
          {compact && selectedItemId && compactLevel === "detail" ? (
            <button
              type="button"
              onClick={() => {
                setCompactLevel("items");
                queueMicrotask(() =>
                  document
                    .getElementById(`review-item-${selectedItemId}`)
                    ?.focus(),
                );
              }}
            >
              Back to items
            </button>
          ) : null}
          {(() => {
            const left = selectedItemId
              ? snapshot.items.find((item) => item.itemId === selectedItemId)
              : undefined;
            const mutable = snapshot.selectedCollection.state === "active";
            const leftReason = !left
              ? "Select a text item to compare."
              : left.class !== "text"
                ? "This item is not comparable."
                : left.state !== "ready"
                  ? "This text item is not ready for comparison."
                  : !mutable
                    ? "Comparisons are unavailable because this collection is not active."
                    : left.byteSize > 128 * 1024
                      ? "This text item is more than 128 KiB."
                      : (left.lineCount ?? 0) > 2_000
                        ? "This text item has more than 2,000 lines."
                        : null;
            const candidates =
              left && !leftReason
                ? snapshot.items.filter(
                    (item) =>
                      item.itemId !== left.itemId &&
                      item.class === "text" &&
                      item.state === "ready" &&
                      item.textFormat === left.textFormat &&
                      item.byteSize <= 128 * 1024 &&
                      (item.lineCount ?? 0) <= 2_000,
                  )
                : [];
            const compareReason =
              leftReason ??
              (candidates.length
                ? null
                : "No other ready text item with the same format is eligible.");
            const comparisons = [...snapshot.comparisons].sort(
              (a, b) =>
                a.createdAtMs - b.createdAtMs ||
                a.comparisonId.localeCompare(b.comparisonId),
            );
            const label = (itemId: string) =>
              snapshot.items.find((item) => item.itemId === itemId)?.title ??
              "Unavailable item";
            const selectedComparison = selectedComparisonId
              ? comparisons.find(
                  (comparison) =>
                    comparison.comparisonId === selectedComparisonId,
                )
              : undefined;
            const nonText =
              left?.class === "image-mockup" || left?.class === "evidence";
            return (
              <section aria-label="Comparisons">
                <h4 ref={comparisonsHeading} tabIndex={-1}>
                  Comparisons
                </h4>
                <p>{comparisons.length} of 8 comparisons</p>
                {snapshot.selectedCollection.comparisonCountWarning ? (
                  <p role="status">
                    Comparison warning: six or more bindings are stored; the
                    limit is eight.
                  </p>
                ) : null}
                {comparisons.length >= 8 ? (
                  <p role="status">
                    Comparison quota reached: discard a comparison before
                    creating another.
                  </p>
                ) : null}
                {nonText ? (
                  <p>Not comparable</p>
                ) : (
                  <>
                    <button
                      ref={compareTrigger}
                      type="button"
                      disabled={busy || !!compareReason}
                      aria-describedby={
                        compareReason ? "compare-reason" : undefined
                      }
                      onClick={() => {
                        setComparisonChooserOpen(true);
                        setComparisonRightItemId(candidates[0]?.itemId ?? "");
                      }}
                    >
                      Compare
                    </button>
                    {compareReason ? (
                      <p id="compare-reason">{compareReason}</p>
                    ) : null}
                  </>
                )}
                {comparisonChooserOpen && left ? (
                  <dialog
                    open
                    aria-label="Create comparison"
                    onKeyDown={(event) => {
                      if (event.key === "Escape") {
                        event.preventDefault();
                        cancelComparisonChooser();
                      }
                    }}
                  >
                    <h5>Compare {left.title}</h5>
                    <p>
                      Left side: {left.title} · {left.textFormat} ·{" "}
                      {left.byteSize} bytes · {left.sha256.slice(0, 12)}
                    </p>
                    <label>
                      Right side
                      <select
                        value={comparisonRightItemId}
                        onChange={(event) =>
                          setComparisonRightItemId(event.target.value)
                        }
                      >
                        {candidates.map((item) => (
                          <option key={item.itemId} value={item.itemId}>
                            {item.title} · {item.textFormat} · {item.byteSize}{" "}
                            bytes · {item.sha256.slice(0, 12)}
                          </option>
                        ))}
                      </select>
                    </label>
                    <button
                      type="button"
                      disabled={busy || !comparisonRightItemId}
                      onClick={createComparison}
                    >
                      Create comparison
                    </button>
                    <button
                      type="button"
                      disabled={busy}
                      onClick={cancelComparisonChooser}
                    >
                      Cancel
                    </button>
                  </dialog>
                ) : null}
                {comparisons.length ? (
                  <ul aria-label="Comparison list">
                    {comparisons.map((comparison) => (
                      <li
                        key={comparison.comparisonId}
                        id={`comparison-${comparison.comparisonId}`}
                        tabIndex={-1}
                      >
                        <button
                          type="button"
                          aria-pressed={
                            selectedComparisonId === comparison.comparisonId
                          }
                          aria-label={`${label(comparison.leftItemId)} and ${label(comparison.rightItemId)} comparison, ${comparison.state}`}
                          onClick={() =>
                            readComparison(comparison.comparisonId)
                          }
                        >
                          {label(comparison.leftItemId)} ↔{" "}
                          {label(comparison.rightItemId)} ·{" "}
                          {comparison.textFormat} · {comparison.state} ·{" "}
                          {comparison.leftSha256.slice(0, 12)} /{" "}
                          {comparison.rightSha256.slice(0, 12)} · created{" "}
                          {comparison.createdAtMs}
                        </button>
                        <button
                          id={`comparison-discard-${comparison.comparisonId}`}
                          type="button"
                          disabled={busy || !mutable}
                          onClick={() =>
                            setDeletingComparisonId(comparison.comparisonId)
                          }
                        >
                          Discard comparison…
                        </button>
                        {deletingComparisonId === comparison.comparisonId ? (
                          <section
                            role="dialog"
                            aria-label="Discard comparison confirmation"
                          >
                            <p>
                              Discarding removes only this stored comparison
                              binding. It does not alter either review item,
                              item text or digest, annotations, task or plan,
                              files, Git, approval, dispatch, or execution.
                            </p>
                            <button
                              type="button"
                              disabled={busy}
                              onClick={() =>
                                discardComparison(comparison.comparisonId)
                              }
                            >
                              Discard comparison
                            </button>
                            <button
                              type="button"
                              disabled={busy}
                              onClick={() =>
                                cancelComparisonDiscard(comparison.comparisonId)
                              }
                            >
                              Cancel
                            </button>
                          </section>
                        ) : null}
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p>No comparisons.</p>
                )}
                {selectedComparison ? (
                  selectedComparison.state !== "ready" ? (
                    <section aria-label="Comparison result">
                      <h5 ref={comparisonHeading} tabIndex={-1}>
                        Comparison unavailable
                      </h5>
                      <p>
                        {selectedComparison.state === "stale"
                          ? "This comparison is stale because one of its text items changed."
                          : "This comparison is unavailable because a bound text item is unavailable or invalid."}
                      </p>
                    </section>
                  ) : comparisonResult?.comparisonId ===
                    selectedComparison.comparisonId ? (
                    <section aria-label="Comparison result">
                      <h5 ref={comparisonHeading} tabIndex={-1}>
                        Comparison result
                      </h5>
                      <p>
                        {label(comparisonResult.leftItemId)} ·{" "}
                        <code
                          aria-label={`SHA-256 ${comparisonResult.leftSha256}`}
                        >
                          {comparisonResult.leftSha256.slice(0, 12)}
                        </code>
                      </p>
                      <p>
                        {label(comparisonResult.rightItemId)} ·{" "}
                        <code
                          aria-label={`SHA-256 ${comparisonResult.rightSha256}`}
                        >
                          {comparisonResult.rightSha256.slice(0, 12)}
                        </code>
                      </p>
                      <p>{comparisonResult.textFormat}</p>
                      <ol>
                        {comparisonResult.lines.map((line, index) => (
                          <li key={`${line.kind}-${index}`}>
                            <strong>
                              {line.kind === "unchanged"
                                ? "Unchanged"
                                : line.kind === "added"
                                  ? "Added"
                                  : "Removed"}
                            </strong>{" "}
                            · left {line.leftLineNumber ?? "—"}, right{" "}
                            {line.rightLineNumber ?? "—"}:{" "}
                            <span>{line.text}</span>
                          </li>
                        ))}
                      </ol>
                    </section>
                  ) : null
                ) : null}
              </section>
            );
          })()}
          {selectedItemId
            ? (() => {
                const selectedItem = snapshot.items.find(
                  (item) => item.itemId === selectedItemId,
                );
                if (!selectedItem) return null;
                const annotations = [...(selectedItem.annotations ?? [])].sort(
                  (left, right) => {
                    const state =
                      (left.state === "open" ? 0 : 1) -
                      (right.state === "open" ? 0 : 1);
                    return (
                      state ||
                      left.createdAtMs - right.createdAtMs ||
                      left.annotationId.localeCompare(right.annotationId)
                    );
                  },
                );
                const mutable =
                  snapshot.selectedCollection?.state === "active" &&
                  selectedItem.state === "ready";
                return (
                  <section aria-label="Annotations">
                    <h4 ref={annotationsHeading} tabIndex={-1}>
                      Annotations
                    </h4>
                    {!mutable ? (
                      <p role="status">
                        Annotations are read-only because this item or
                        collection is not active and ready.
                      </p>
                    ) : null}
                    {annotations.length ? (
                      <ul>
                        {annotations.map((annotation) => (
                          <li
                            key={annotation.annotationId}
                            id={`annotation-${annotation.annotationId}`}
                            tabIndex={-1}
                          >
                            <p>{annotation.text}</p>
                            <p>
                              {annotation.state} · created{" "}
                              {annotation.createdAtMs} · updated{" "}
                              {annotation.updatedAtMs}
                            </p>
                            {editingAnnotationId === annotation.annotationId ? (
                              <>
                                <label>
                                  Edit annotation
                                  <textarea
                                    value={editingAnnotationText}
                                    maxLength={1024}
                                    onChange={(event) =>
                                      setEditingAnnotationText(
                                        event.target.value,
                                      )
                                    }
                                  />
                                </label>
                                <p>
                                  {editingAnnotationText.length} / 1024
                                  characters
                                </p>
                                <button
                                  type="button"
                                  disabled={busy || !editingAnnotationText}
                                  onClick={() =>
                                    mutateAnnotation(
                                      annotation.annotationId,
                                      "edit",
                                    )
                                  }
                                >
                                  Save annotation
                                </button>
                                <button
                                  type="button"
                                  disabled={busy}
                                  onClick={() =>
                                    cancelAnnotationEdit(
                                      annotation.annotationId,
                                    )
                                  }
                                >
                                  Cancel
                                </button>
                              </>
                            ) : (
                              <>
                                <button
                                  id={`annotation-edit-${annotation.annotationId}`}
                                  type="button"
                                  disabled={busy || !mutable}
                                  onClick={() => {
                                    setEditingAnnotationId(
                                      annotation.annotationId,
                                    );
                                    setEditingAnnotationText(annotation.text);
                                  }}
                                >
                                  Edit
                                </button>
                                {annotation.state === "open" ? (
                                  <button
                                    type="button"
                                    disabled={busy || !mutable}
                                    onClick={() =>
                                      mutateAnnotation(
                                        annotation.annotationId,
                                        "resolve",
                                      )
                                    }
                                  >
                                    Resolve
                                  </button>
                                ) : (
                                  <button
                                    type="button"
                                    disabled={busy || !mutable}
                                    onClick={() =>
                                      mutateAnnotation(
                                        annotation.annotationId,
                                        "reopen",
                                      )
                                    }
                                  >
                                    Reopen
                                  </button>
                                )}
                                <button
                                  id={`annotation-delete-${annotation.annotationId}`}
                                  type="button"
                                  disabled={busy || !mutable}
                                  onClick={() =>
                                    setDeletingAnnotationId(
                                      annotation.annotationId,
                                    )
                                  }
                                >
                                  Delete annotation…
                                </button>
                              </>
                            )}
                            {deletingAnnotationId ===
                            annotation.annotationId ? (
                              <section
                                role="dialog"
                                aria-label="Delete annotation confirmation"
                              >
                                <p>
                                  Delete this local annotation record only. This
                                  does not alter the review item, item content
                                  or digest, task, optional plan, evidence,
                                  generated artifacts, project files, Git,
                                  approval, dispatch, or execution state.
                                </p>
                                <button
                                  type="button"
                                  disabled={busy}
                                  onClick={() =>
                                    deleteAnnotation(annotation.annotationId)
                                  }
                                >
                                  Delete annotation
                                </button>
                                <button
                                  type="button"
                                  disabled={busy}
                                  onClick={() =>
                                    cancelAnnotationDelete(
                                      annotation.annotationId,
                                    )
                                  }
                                >
                                  Cancel
                                </button>
                              </section>
                            ) : null}
                          </li>
                        ))}
                      </ul>
                    ) : (
                      <p>No annotations.</p>
                    )}
                    <label>
                      Annotation text
                      <textarea
                        value={annotationText}
                        maxLength={1024}
                        onChange={(event) =>
                          setAnnotationText(event.target.value)
                        }
                      />
                    </label>
                    <p>{annotationText.length} / 1024 characters</p>
                    <button
                      id="add-annotation"
                      type="button"
                      disabled={busy || !mutable || !annotationText}
                      onClick={createAnnotation}
                    >
                      Add annotation
                    </button>
                  </section>
                );
              })()
            : null}
          {selectedItemId
            ? (() => {
                const item = snapshot.items.find(
                  (candidate) => candidate.itemId === selectedItemId,
                );
                if (!item || item.class !== "text") return null;
                const activePreview =
                  textPreview?.collectionId ===
                    snapshot.selectedCollection.collectionId &&
                  textPreview.itemId === item.itemId &&
                  textPreview.sha256 === item.sha256
                    ? textPreview
                    : null;
                const previewFailed =
                  previewKey === `${item.itemId}:${item.sha256}` &&
                  textPreviewFailedKey === previewKey;
                const formatLabel =
                  item.textFormat === "plain"
                    ? "Plain text"
                    : item.textFormat === "markdown"
                      ? "Markdown source"
                      : item.textFormat === "json"
                        ? "JSON source"
                        : item.textFormat === "csv"
                          ? "CSV source"
                          : "Python source";
                return (
                  <section aria-label="Text preview">
                    <h4 ref={previewHeading} tabIndex={-1}>
                      Text preview
                    </h4>
                    <p>
                      {item.title} · Text · {formatLabel}
                    </p>
                    <p>
                      {item.byteSize} bytes · created {item.createdAtMs} ·{" "}
                      {item.state}
                    </p>
                    <code aria-label={`SHA-256 ${item.sha256}`}>
                      {item.sha256.slice(0, 12)}
                    </code>
                    {activePreview === null ? (
                      previewFailed ? (
                        <>
                          <p>Safe preview unavailable.</p>
                          <p>Content withheld for safety.</p>
                        </>
                      ) : (
                        <p role="status">Loading safe text preview…</p>
                      )
                    ) : activePreview.state !== "ready" ||
                      activePreview.text === null ? (
                      <>
                        <p>Safe preview unavailable.</p>
                        <p>Content withheld for safety.</p>
                      </>
                    ) : (
                      <>
                        <p>
                          {activePreview.truncated
                            ? `Preview truncated deterministically at ${activePreview.projectedByteSize} bytes and ${activePreview.projectedLineCount} lines.`
                            : `Preview contains ${activePreview.projectedByteSize} bytes and ${activePreview.projectedLineCount} lines.`}
                        </p>
                        <pre className="local-review-text-preview">
                          <code>{activePreview.text}</code>
                        </pre>
                      </>
                    )}
                  </section>
                );
              })()
            : null}
          {selectedItemId
            ? (() => {
                const item = snapshot.items.find(
                  (candidate) => candidate.itemId === selectedItemId,
                );
                if (!item) return null;
                const reason =
                  item.class !== "text"
                    ? "Not promotion eligible"
                    : item.state !== "ready"
                      ? "This text item is not ready for promotion."
                      : snapshot.selectedCollection.state !== "active"
                        ? "Promotion requires an active review collection."
                        : item.byteSize > 512 * 1024
                          ? "This text item exceeds the 512 KiB generated-artifact limit."
                          : null;
                return (
                  <section aria-label="Transient generated artifact">
                    <h4>Transient generated artifact</h4>
                    <p>Promotion eligibility is not execution approval.</p>
                    <p>
                      Creating a generated artifact does not approve or dispatch
                      work.
                    </p>
                    {item.class === "image-mockup" ||
                    item.class === "evidence" ? (
                      <p>Not promotion eligible</p>
                    ) : (
                      <>
                        <button
                          ref={promotionTrigger}
                          type="button"
                          disabled={busy || !!reason}
                          aria-describedby={
                            reason ? "promotion-reason" : undefined
                          }
                          onClick={preparePromotion}
                        >
                          Create transient generated artifact…
                        </button>
                        {reason ? <p id="promotion-reason">{reason}</p> : null}
                      </>
                    )}
                    {busy && !promotionCandidate ? (
                      <p role="status">
                        Preparing transient generated artifact…
                      </p>
                    ) : null}
                    {promotionCandidate ? (
                      <dialog
                        open
                        aria-label="Create transient generated artifact confirmation"
                        onKeyDown={(event) => {
                          if (event.key === "Escape") {
                            event.preventDefault();
                            cancelPromotion();
                          }
                        }}
                      >
                        <h5 ref={promotionDialogHeading} tabIndex={-1}>
                          Create transient generated artifact
                        </h5>
                        <p>{promotionCandidate.title}</p>
                        <p>
                          {promotionCandidate.textFormat} →{" "}
                          {promotionCandidate.destinationClass}
                        </p>
                        <code
                          aria-label={`SHA-256 ${promotionCandidate.sha256}`}
                        >
                          {promotionCandidate.sha256.slice(0, 12)}
                        </code>
                        <p>Task context: {promotionCandidate.taskId}</p>
                        {promotionCandidate.planId ? (
                          <p>Plan context: {promotionCandidate.planId}</p>
                        ) : null}
                        <p>
                          Reservation state: {promotionCandidate.state}. This
                          reservation expires in five minutes at{" "}
                          {promotionCandidate.expiresAtMs}.
                        </p>
                        <p>
                          This creates only a transient QuireForge generated
                          artifact. It does not save a file, approve or dispatch
                          work, run code, change Git, publish, or deploy.
                        </p>
                        {busy ? (
                          <p role="status">
                            Creating transient generated artifact…
                          </p>
                        ) : null}
                        <button
                          type="button"
                          disabled={busy}
                          onClick={confirmPromotion}
                        >
                          Create transient generated artifact
                        </button>
                        <button
                          ref={promotionCancel}
                          type="button"
                          disabled={busy}
                          onClick={cancelPromotion}
                        >
                          Cancel
                        </button>
                      </dialog>
                    ) : null}
                    {promotionResult ? (
                      <section aria-label="Transient generated artifact result">
                        <h5 ref={promotionResultHeading} tabIndex={-1}>
                          Transient generated artifact created
                        </h5>
                        <p>
                          {promotionResult.displayLabel} ·{" "}
                          {promotionResult.class} · {promotionResult.state}
                        </p>
                        <p>Provenance: {promotionResult.sourceKind}</p>
                        <code aria-label={`SHA-256 ${promotionResult.sha256}`}>
                          {promotionResult.sha256.slice(0, 12)}
                        </code>
                        <p>Transient until {promotionResult.expiresAt}</p>
                        <p>
                          View the resulting artifact in Generated Artifacts.
                        </p>
                      </section>
                    ) : null}
                  </section>
                );
              })()
            : null}
          <label>
            Text item title
            <input
              value={title}
              maxLength={120}
              onChange={(event) => setTitle(event.target.value)}
            />
          </label>
          <label>
            Plain text
            <textarea
              value={text}
              maxLength={256 * 1024}
              onChange={(event) => setText(event.target.value)}
            />
          </label>
          <button
            type="button"
            disabled={busy || !title.trim() || !text}
            onClick={createText}
          >
            Add text item
          </button>
          <section aria-label="Copy live generated artifact">
            <h4>Copy live generated artifact</h4>
            <p>
              Copy one currently available generated text artifact into this
              local review collection. This does not save, approve, dispatch, or
              execute work.
            </p>
            <button
              ref={artifactCopyTrigger}
              type="button"
              disabled={busy || snapshot.selectedCollection.state !== "active"}
              onClick={openArtifactCopyChooser}
            >
              Copy generated artifact…
            </button>
            {snapshot.selectedCollection.state !== "active" ? (
              <p>
                Generated artifacts can be copied only into an active review
                collection.
              </p>
            ) : null}
            {artifactCopyChooserOpen ? (
              <dialog
                open
                aria-label="Copy live generated artifact"
                onKeyDown={(event) => {
                  if (event.key === "Escape") {
                    event.preventDefault();
                    cancelArtifactCopy();
                  }
                }}
              >
                <h5>Copy live generated artifact</h5>
                <p>
                  Only currently available generated text artifacts are listed.
                  Their bytes are resolved and verified locally when copied.
                </p>
                <label>
                  Generated artifact
                  <select
                    value={selectedArtifactId}
                    onChange={(event) =>
                      setSelectedArtifactId(event.target.value)
                    }
                  >
                    {artifactCandidates.map((artifact) => (
                      <option
                        key={artifact.artifactId}
                        value={artifact.artifactId}
                      >
                        {artifact.displayLabel} · {artifact.class} ·{" "}
                        {artifact.byteSize} bytes ·{" "}
                        {artifact.sha256.slice(0, 12)}
                      </option>
                    ))}
                  </select>
                </label>
                <button
                  type="button"
                  disabled={busy || !selectedArtifactId}
                  onClick={copyArtifact}
                >
                  Copy into local review
                </button>
                <button
                  type="button"
                  disabled={busy}
                  onClick={cancelArtifactCopy}
                >
                  Cancel
                </button>
              </dialog>
            ) : null}
          </section>
          <section aria-label="Capture generated-artifact metadata">
            <h4>Capture generated-artifact metadata</h4>
            <p>
              Capture only a bounded, copied metadata record for one live
              generated artifact. This does not copy artifact content, save a
              file, approve, dispatch, or execute work.
            </p>
            <button
              ref={artifactMetadataTrigger}
              type="button"
              disabled={busy || snapshot.selectedCollection.state !== "active"}
              onClick={openArtifactMetadataChooser}
            >
              Capture generated-artifact metadata…
            </button>
            {snapshot.selectedCollection.state !== "active" ? (
              <p>
                Generated-artifact metadata can be captured only into an active
                review collection.
              </p>
            ) : null}
            {artifactMetadataChooserOpen ? (
              <dialog
                open
                aria-label="Capture generated-artifact metadata"
                onKeyDown={(event) => {
                  if (event.key === "Escape") {
                    event.preventDefault();
                    cancelArtifactMetadata();
                  }
                }}
              >
                <h5>Capture generated-artifact metadata</h5>
                <p>
                  Only current artifact state, kind, format, size, truncation
                  state, and manifest digest are copied. Artifact content is not
                  copied.
                </p>
                <label>
                  Generated artifact
                  <select
                    value={selectedArtifactId}
                    onChange={(event) =>
                      setSelectedArtifactId(event.target.value)
                    }
                  >
                    {artifactCandidates.map((artifact) => (
                      <option
                        key={artifact.artifactId}
                        value={artifact.artifactId}
                      >
                        {artifact.displayLabel} · {artifact.class} ·{" "}
                        {artifact.byteSize} bytes ·{" "}
                        {artifact.sha256.slice(0, 12)}
                      </option>
                    ))}
                  </select>
                </label>
                <button
                  type="button"
                  disabled={busy || !selectedArtifactId}
                  onClick={captureArtifactMetadata}
                >
                  Capture metadata
                </button>
                <button
                  type="button"
                  disabled={busy}
                  onClick={cancelArtifactMetadata}
                >
                  Cancel
                </button>
              </dialog>
            ) : null}
          </section>
          <section aria-label="Capture safe-preview metadata">
            <h4>Capture safe-preview metadata</h4>
            <p>
              Capture only bounded safe-preview metadata. File content, paths,
              URLs, and open actions are not captured.
            </p>
            <button
              ref={safePreviewMetadataTrigger}
              type="button"
              disabled={busy || snapshot.selectedCollection.state !== "active"}
              onClick={prepareSafePreviewMetadata}
            >
              Capture safe-preview metadata…
            </button>
            {safePreviewClaim ? (
              <dialog
                open
                aria-label="Capture safe-preview metadata"
                onKeyDown={(event) => {
                  if (event.key === "Escape") {
                    event.preventDefault();
                    cancelSafePreviewMetadata();
                  }
                }}
              >
                <h5>Capture safe-preview metadata</h5>
                <p>
                  {safePreviewClaim.kind} · {safePreviewClaim.rendering} ·{" "}
                  {safePreviewClaim.mediaType} · {safePreviewClaim.byteLength}{" "}
                  bytes
                </p>
                <p>
                  Only metadata is captured; no file content or open handoff is
                  retained.
                </p>
                <button
                  type="button"
                  disabled={busy}
                  onClick={captureSafePreviewMetadata}
                >
                  Capture metadata
                </button>
                <button
                  type="button"
                  disabled={busy}
                  onClick={cancelSafePreviewMetadata}
                >
                  Cancel
                </button>
              </dialog>
            ) : null}
          </section>
          <section aria-label="Add image mockup">
            <h4>Add image mockup</h4>
            <p>
              Choose one static PNG or JPEG. QuireForge validates and copies it
              locally without retaining or displaying its original path.
            </p>
            <button
              ref={imageTrigger}
              type="button"
              disabled={busy || !title.trim()}
              onClick={chooseImage}
            >
              Choose PNG or JPEG…
            </button>
          </section>
          {selectedItemId && imagePreview ? (
            <section aria-label="Image mockup preview">
              <h4 ref={previewHeading} tabIndex={-1}>
                Image mockup preview
              </h4>
              <img
                src={imagePreview.dataUrl}
                alt={`${snapshot.items.find((item) => item.itemId === selectedItemId)?.title ?? "Image mockup"}, ${imagePreview.width} by ${imagePreview.height} pixels`}
              />
              <p>
                {imagePreview.mimeType} · {imagePreview.width} ×{" "}
                {imagePreview.height} · {imagePreview.byteSize} bytes
              </p>
              <code aria-label={`SHA-256 ${imagePreview.sha256}`}>
                {imagePreview.sha256.slice(0, 12)}
              </code>
              <p>Not comparable</p>
              <p>Not promotion eligible</p>
            </section>
          ) : null}
          <section aria-label="Add evidence snapshot">
            <h4>Add evidence snapshot</h4>
            <p>
              Manual validation summary copies bounded local text into this
              review collection. It retains no path, URL, command output,
              approval, or external connection.
            </p>
            <p>Manual validation summary</p>
            <label>
              Evidence label
              <input
                value={evidenceTitle}
                maxLength={120}
                onChange={(event) => setEvidenceTitle(event.target.value)}
              />
            </label>
            <label>
              Manual validation summary
              <textarea
                value={evidenceSummary}
                maxLength={16 * 1024}
                onChange={(event) => setEvidenceSummary(event.target.value)}
              />
            </label>
            <p>
              {evidenceSummary.length} / {16 * 1024} characters
            </p>
            {snapshot.selectedCollection.warning ? (
              <p role="status">
                Evidence warning: review collection capacity is near its limit.
              </p>
            ) : null}
            <button
              type="button"
              disabled={busy || !evidenceTitle.trim() || !evidenceSummary}
              onClick={createEvidence}
            >
              Add evidence snapshot
            </button>
            <p>Package validation summary is available only for this collection’s eligible native task and completed validation record.</p>
            <button type="button" disabled={busy || !snapshot.packageManifestSummaryAvailable} onClick={capturePackageManifestSummary}>
              Capture package validation summary…
            </button>
            <button type="button" disabled={busy || !snapshot.gitStatusDiffSummaryAvailable} onClick={captureGitStatusDiffSummary}>
              Capture Git status and diff summary…
            </button>
            <button type="button" disabled={busy || !snapshot.activityPresentationAvailable} onClick={captureActivityPresentation}>Capture activity presentation…</button>
            <button type="button" disabled={busy || !snapshot.approvalPresentationAvailable} onClick={captureApprovalPresentation}>Capture approval presentation…</button>
          </section>
          {evidencePreview ? (
            <section aria-label="Evidence preview">
              <h4 ref={evidenceHeading} tabIndex={-1}>
                Evidence preview
              </h4>
              <p>{evidencePreview.title}</p>
              <p>Evidence · Manual validation summary</p>
              <pre>{evidencePreview.summary}</pre>
              <p>
                {evidencePreview.byteSize} bytes · created{" "}
                {evidencePreview.createdAtMs}
              </p>
              <p>ready</p>
              <code aria-label={`SHA-256 ${evidencePreview.sha256}`}>
                {evidencePreview.sha256.slice(0, 12)}
              </code>
              <p>Not comparable</p>
              <p>Not promotion eligible</p>
            </section>
          ) : null}
          {artifactMetadataPreview ? (
            <section aria-label="Generated-artifact metadata evidence preview">
              <h4 ref={evidenceHeading} tabIndex={-1}>
                Generated-artifact metadata evidence
              </h4>
              <p>{artifactMetadataPreview.title}</p>
              <p>Evidence · Generated-artifact metadata</p>
              <pre>{artifactMetadataPreview.summary}</pre>
              <p>
                {artifactMetadataPreview.details.artifactState} ·{" "}
                {artifactMetadataPreview.details.artifactKind} ·{" "}
                {artifactMetadataPreview.details.format} ·{" "}
                {artifactMetadataPreview.details.byteLength} bytes
              </p>
              <code
                aria-label={`SHA-256 ${artifactMetadataPreview.details.manifestSha256}`}
              >
                {artifactMetadataPreview.details.manifestSha256.slice(0, 12)}
              </code>
              <p>
                {artifactMetadataPreview.byteSize} evidence bytes · created{" "}
                {artifactMetadataPreview.createdAtMs}
              </p>
              <p>Not comparable</p>
              <p>Not promotion eligible</p>
            </section>
          ) : null}
          {safePreviewEvidencePreview ? (
            <section aria-label="Safe-preview metadata evidence preview">
              <h4 ref={evidenceHeading} tabIndex={-1}>
                Safe-preview metadata evidence
              </h4>
              <p>{safePreviewEvidencePreview.title}</p>
              <p>Evidence · Safe-preview metadata</p>
              <pre>{safePreviewEvidencePreview.summary}</pre>
              <p>
                {safePreviewEvidencePreview.details.kind} ·{" "}
                {safePreviewEvidencePreview.details.rendering} ·{" "}
                {safePreviewEvidencePreview.details.mediaType} ·{" "}
                {safePreviewEvidencePreview.details.byteLength} bytes
              </p>
              <code aria-label={`SHA-256 ${safePreviewEvidencePreview.sha256}`}>
                {safePreviewEvidencePreview.sha256.slice(0, 12)}
              </code>
              <p>Not comparable</p>
              <p>Not promotion eligible</p>
            </section>
          ) : null}
          {packageManifestEvidencePreview ? (
            <section aria-label="Package validation summary evidence preview">
              <h4 ref={evidenceHeading} tabIndex={-1}>Package validation summary evidence</h4>
              <p>{packageManifestEvidencePreview.title}</p>
              <p>Evidence · Package validation summary</p>
              <pre>{packageManifestEvidencePreview.summary}</pre>
              <p>{packageManifestEvidencePreview.details.applicationVersion} · {packageManifestEvidencePreview.details.debianVersion} · 2 artifacts · complete</p>
              <p>All validation checks passed.</p>
              <p>Not comparable</p><p>Not promotion eligible</p>
            </section>
          ) : null}
          {gitEvidencePreview ? (
            <section aria-label="Git status and diff summary evidence preview">
              <h4 ref={evidenceHeading} tabIndex={-1}>Git status and diff summary evidence</h4>
              <p>{gitEvidencePreview.title}</p><p>Evidence · Git status and diff summary</p>
              <pre>{gitEvidencePreview.summary}</pre>
              <p>{gitEvidencePreview.details.workspaceState} · {gitEvidencePreview.details.changedFileCount} changed files · {gitEvidencePreview.details.stagedCount} staged</p>
              <p>Not comparable</p><p>Not promotion eligible</p>
            </section>
          ) : null}
          {activityEvidencePreview ? (<section aria-label="Activity presentation evidence preview"><h4 ref={evidenceHeading} tabIndex={-1}>Activity presentation evidence</h4><p>{activityEvidencePreview.title}</p><p>Evidence · Activity presentation</p><pre>{activityEvidencePreview.summary}</pre><p>{activityEvidencePreview.details.eventCount} native current-session events</p></section>) : null}
          {selectedItemId ? (
            <button type="button" disabled={busy} onClick={discardSelectedItem}>
              Discard selected item
            </button>
          ) : null}
        </section>
      ) : null}
    </div>
  );
}
