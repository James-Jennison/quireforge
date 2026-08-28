import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import LocalReviewPane from "./LocalReviewPane";
import type { ReviewPaneData } from "./types";
import { resetLocalReviewSessionForTest } from "./localReviewSession";
import * as localReviewSession from "./localReviewSession";

const id = "018f0000-0000-7000-8000-000000000001";
const artifactId = "018f0000-0000-7000-8000-000000000002";
const sha = "a".repeat(64);
const {
  load,
  pick,
  preview,
  previewText,
  createEvidence,
  previewEvidence,
  copyArtifact,
  captureArtifactMetadata,
  previewArtifactMetadata,
  claimSafePreview,
  captureSafePreview,
  previewSafePreview,
  discard,
  createAnnotation,
  editAnnotation,
  resolveAnnotation,
  reopenAnnotation,
  deleteAnnotation,
  createComparison,
  readComparison,
  discardComparison,
  preparePromotion,
  confirmPromotion,
  cancelPromotion,
} = vi.hoisted(() => ({
  load: vi.fn(),
  pick: vi.fn(),
  preview: vi.fn(),
  previewText: vi.fn(),
  createEvidence: vi.fn(),
  previewEvidence: vi.fn(),
  copyArtifact: vi.fn(),
  captureArtifactMetadata: vi.fn(),
  previewArtifactMetadata: vi.fn(),
  claimSafePreview: vi.fn(),
  captureSafePreview: vi.fn(),
  previewSafePreview: vi.fn(),
  discard: vi.fn(),
  createAnnotation: vi.fn(),
  editAnnotation: vi.fn(),
  resolveAnnotation: vi.fn(),
  reopenAnnotation: vi.fn(),
  deleteAnnotation: vi.fn(),
  createComparison: vi.fn(),
  readComparison: vi.fn(),
  discardComparison: vi.fn(),
  preparePromotion: vi.fn(),
  confirmPromotion: vi.fn(),
  cancelPromotion: vi.fn(),
}));

vi.mock("../lib/bridge", () => ({
  loadLocalReview: load,
  createLocalReviewCollection: vi.fn(),
  createLocalReviewTextItem: vi.fn(),
  createLocalReviewM48ArtifactCopy: copyArtifact,
  createLocalReviewM48GeneratedArtifactMetadataEvidence:
    captureArtifactMetadata,
  pickLocalReviewImage: pick,
  previewLocalReviewImage: preview,
  previewLocalReviewText: previewText,
  createLocalReviewManualEvidence: createEvidence,
  previewLocalReviewManualEvidence: previewEvidence,
  previewLocalReviewM48GeneratedArtifactMetadataEvidence:
    previewArtifactMetadata,
  claimLocalReviewSafePreviewMetadata: claimSafePreview,
  createLocalReviewSafePreviewMetadataEvidence: captureSafePreview,
  previewLocalReviewSafePreviewMetadataEvidence: previewSafePreview,
  discardLocalReviewItem: discard,
  createLocalReviewAnnotation: createAnnotation,
  editLocalReviewAnnotation: editAnnotation,
  resolveLocalReviewAnnotation: resolveAnnotation,
  reopenLocalReviewAnnotation: reopenAnnotation,
  deleteLocalReviewAnnotation: deleteAnnotation,
  createLocalReviewComparison: createComparison,
  readLocalReviewComparison: readComparison,
  discardLocalReviewComparison: discardComparison,
  prepareLocalReviewPromotion: preparePromotion,
  confirmLocalReviewPromotion: confirmPromotion,
  cancelLocalReviewPromotion: cancelPromotion,
}));

afterEach(() => {
  resetLocalReviewSessionForTest();
  vi.clearAllMocks();
  vi.restoreAllMocks();
});

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
  sha256: sha,
  createdAtMs: 0,
} as const;
const snapshot = {
  schemaVersion: 1 as const,
  collections: [
    {
      collectionId: id,
      taskId: id,
      planId: null,
      title: "Collection",
      state: "active",
      itemCount: 0,
      payloadBytes: 0,
      updatedAtMs: 1,
      warning: false,
    },
  ],
  selectedCollection: {
    collectionId: id,
    taskId: id,
    planId: null,
    title: "Collection",
    state: "active",
    itemCount: 0,
    payloadBytes: 0,
    updatedAtMs: 1,
    warning: false,
  },
  items: [],
  comparisons: [],
  collectionCount: 1,
  payloadBytes: 0,
  warning: false,
  diagnosticCode: null,
};

beforeEach(() =>
  previewText.mockResolvedValue({
    schemaVersion: 1,
    collectionId: id,
    itemId: id,
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
    diagnosticCode: "item-not-found",
  }),
);

function renderPane() {
  load.mockResolvedValue(snapshot);
  const props = {
    taskCatalog: { selectedTask: null },
    loadArtifacts: vi.fn(),
  } as unknown as ReviewPaneData;
  return render(<LocalReviewPane {...props} />);
}

describe("local review image pane", () => {
  it.each([
    ["plain", "Plain text", "plain source"],
    ["markdown", "Markdown source", "[not a link](https://example.invalid)"],
    ["json", "JSON source", '{"value":1}'],
    ["csv", "CSV source", "left,right\n1,2"],
    ["python", "Python source", "print('inert')"],
  ] as const)(
    "renders %s text only as inert %s",
    async (textFormat, formatLabel, text) => {
      const item = {
        itemId: id,
        class: "text",
        textFormat,
        sourceKind: "user-authored-text",
        state: "ready",
        title: "Text",
        mimeType:
          textFormat === "plain" ? "text/plain; charset=utf-8" : "text/plain",
        width: null,
        height: null,
        byteSize: text.length,
        lineCount: 1,
        sha256: sha,
        createdAtMs: 0,
        annotations: [],
      } as const;
      load.mockResolvedValue({
        ...snapshot,
        items: [item],
        selectedCollection: { ...snapshot.selectedCollection, itemCount: 1 },
      });
      previewText.mockResolvedValueOnce({
        schemaVersion: 1,
        collectionId: id,
        itemId: id,
        title: "Text",
        textFormat,
        byteSize: text.length,
        sha256: sha,
        createdAtMs: 0,
        state: "ready",
        text,
        projectedByteSize: text.length,
        projectedLineCount: text.endsWith("\n") ? 1 : text.split("\n").length,
        projectedCodePointCount: [...text].length,
        truncated: false,
        diagnosticCode: null,
      });
      render(
        <LocalReviewPane
          {...({
            taskCatalog: { selectedTask: null },
            loadArtifacts: vi.fn(),
          } as unknown as ReviewPaneData)}
        />,
      );
      fireEvent.click(
        await screen.findByRole("button", { name: /Text — text/i }),
      );
      const code = await screen.findByText(
        (_, element) =>
          element?.tagName === "CODE" && element.textContent === text,
      );
      expect(
        screen.getByText(
          (_, element) =>
            element?.tagName === "P" &&
            element.textContent?.includes(formatLabel) === true,
        ),
      ).toBeVisible();
      expect(code.closest("code")).not.toBeNull();
      expect(screen.queryByRole("link")).toBeNull();
    },
  );
  it("renders native text previews inertly and withholds a late prior selection", async () => {
    const first = {
      itemId: id,
      class: "text",
      textFormat: "markdown",
      sourceKind: "user-authored-text",
      state: "ready",
      title: "First",
      mimeType: "text/markdown; charset=utf-8",
      width: null,
      height: null,
      byteSize: 20,
      lineCount: 1,
      sha256: sha,
      createdAtMs: 0,
      annotations: [],
    } as const;
    const second = {
      ...first,
      itemId: artifactId,
      title: "Second",
      textFormat: "python",
      sha256: "b".repeat(64),
    } as const;
    const initial = {
      ...snapshot,
      items: [first, second],
      selectedCollection: { ...snapshot.selectedCollection, itemCount: 2 },
    };
    let resolveFirst: (value: unknown) => void = () => undefined;
    let resolveSecond: (value: unknown) => void = () => undefined;
    previewText
      .mockImplementationOnce(
        () =>
          new Promise<unknown>((resolve) => {
            resolveFirst = resolve;
          }),
      )
      .mockImplementationOnce(
        () =>
          new Promise<unknown>((resolve) => {
            resolveSecond = resolve;
          }),
      );
    load.mockResolvedValue(initial);
    render(
      <LocalReviewPane
        {...({
          taskCatalog: { selectedTask: null },
          loadArtifacts: vi.fn(),
        } as unknown as ReviewPaneData)}
      />,
    );
    fireEvent.click(
      await screen.findByRole("button", { name: /First — text/i }),
    );
    expect(screen.getByRole("status")).toHaveTextContent(
      "Loading safe text preview…",
    );
    fireEvent.click(screen.getByRole("button", { name: /Second — text/i }));
    resolveFirst({
      schemaVersion: 1,
      collectionId: id,
      itemId: id,
      title: "First",
      textFormat: "markdown",
      byteSize: 20,
      sha256: sha,
      createdAtMs: 0,
      state: "ready",
      text: "<img src=x>",
      projectedByteSize: 11,
      projectedLineCount: 1,
      projectedCodePointCount: 11,
      truncated: false,
      diagnosticCode: null,
    });
    await waitFor(() => expect(screen.queryByText("<img src=x>")).toBeNull());
    resolveSecond({
      schemaVersion: 1,
      collectionId: id,
      itemId: artifactId,
      title: "Second",
      textFormat: "python",
      byteSize: 12,
      sha256: "b".repeat(64),
      createdAtMs: 0,
      state: "ready",
      text: "print('safe')",
      projectedByteSize: 13,
      projectedLineCount: 1,
      projectedCodePointCount: 13,
      truncated: true,
      diagnosticCode: null,
    });
    await screen.findByText("print('safe')");
    expect(
      screen.getByText(/Preview truncated deterministically/),
    ).toBeVisible();
    expect(screen.queryByRole("img")).toBeNull();
    expect(document.querySelector('input[type="file"]')).toBeNull();
    expect(
      screen.queryByRole("button", {
        name: /save|run|apply|merge|patch|dispatch|publish|deploy/i,
      }),
    ).toBeNull();
  });
  it("withholds stale and unavailable text preview content", async () => {
    const item = {
      itemId: id,
      class: "text",
      textFormat: "plain",
      sourceKind: "user-authored-text",
      state: "stale",
      title: "Text",
      mimeType: "text/plain; charset=utf-8",
      width: null,
      height: null,
      byteSize: 4,
      lineCount: 1,
      sha256: sha,
      createdAtMs: 0,
      annotations: [],
    } as const;
    load.mockResolvedValue({
      ...snapshot,
      items: [item],
      selectedCollection: { ...snapshot.selectedCollection, itemCount: 1 },
    });
    previewText.mockResolvedValueOnce({
      schemaVersion: 1,
      collectionId: id,
      itemId: id,
      title: "Text",
      textFormat: "plain",
      byteSize: 4,
      sha256: sha,
      createdAtMs: 0,
      state: "stale",
      text: null,
      projectedByteSize: 0,
      projectedLineCount: 0,
      projectedCodePointCount: 0,
      truncated: false,
      diagnosticCode: null,
    });
    render(
      <LocalReviewPane
        {...({
          taskCatalog: { selectedTask: null },
          loadArtifacts: vi.fn(),
        } as unknown as ReviewPaneData)}
      />,
    );
    fireEvent.click(
      await screen.findByRole("button", { name: /Text — text/i }),
    );
    expect(
      await screen.findByText("Content withheld for safety."),
    ).toBeVisible();
    expect(screen.queryByText("safe text")).toBeNull();
  });
  it("withholds text when the native preview request fails", async () => {
    const item = {
      itemId: id,
      class: "text",
      textFormat: "plain",
      sourceKind: "user-authored-text",
      state: "ready",
      title: "Text",
      mimeType: "text/plain; charset=utf-8",
      width: null,
      height: null,
      byteSize: 4,
      lineCount: 1,
      sha256: sha,
      createdAtMs: 0,
      annotations: [],
    } as const;
    load.mockResolvedValue({
      ...snapshot,
      items: [item],
      selectedCollection: { ...snapshot.selectedCollection, itemCount: 1 },
    });
    previewText.mockRejectedValueOnce(
      new Error("storage detail /private/source.txt"),
    );
    render(
      <LocalReviewPane
        {...({
          taskCatalog: { selectedTask: null },
          loadArtifacts: vi.fn(),
        } as unknown as ReviewPaneData)}
      />,
    );
    fireEvent.click(
      await screen.findByRole("button", { name: /Text — text/i }),
    );
    expect(await screen.findByText("Safe preview unavailable.")).toBeVisible();
    expect(screen.getByText("Content withheld for safety.")).toBeVisible();
    expect(screen.queryByText(/private\/source/i)).toBeNull();
  });
  it("copies one selected live M48 text artifact only after explicit confirmation", async () => {
    const loadArtifacts = vi.fn().mockResolvedValue({
      schemaVersion: 1,
      artifacts: [
        {
          schemaVersion: 1,
          artifactId,
          class: "markdown",
          mimeType: "text/markdown; charset=utf-8",
          sourceKind: "visible-fenced-block",
          displayLabel: "Live artifact",
          suggestedFilename: "live.md",
          byteSize: 9,
          sha256: sha,
          createdAt: 0,
          expiresAt: 10,
          state: "ready",
          disposal: "transient-memory-one-successful-save",
        },
      ],
      diagnosticCode: null,
    });
    load.mockResolvedValue(snapshot);
    const props = {
      taskCatalog: { selectedTask: null },
      loadArtifacts,
    } as unknown as ReviewPaneData;
    render(<LocalReviewPane {...props} />);
    await screen.findByRole("heading", {
      name: "Copy live generated artifact",
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Copy generated artifact…" }),
    );
    await screen.findByRole("dialog", { name: "Copy live generated artifact" });
    expect(
      screen.getByText(/does not save, approve, dispatch, or execute work/i),
    ).toBeVisible();
    expect(document.querySelector('input[type="file"]')).toBeNull();
    expect(screen.queryByLabelText(/path|url|filename/i)).toBeNull();
    let resolveCopy: (value: unknown) => void = () => undefined;
    copyArtifact.mockImplementationOnce(
      () =>
        new Promise<unknown>((resolve) => {
          resolveCopy = resolve;
        }),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Copy into local review" }),
    );
    await waitFor(() =>
      expect(copyArtifact).toHaveBeenCalledWith({
        collectionId: id,
        expectedCollectionUpdatedAtMs: 1,
        artifactId,
        manifestSha256: sha,
      }),
    );
    expect(
      screen.getByRole("button", { name: "Copy into local review" }),
    ).toBeDisabled();
    const copied = {
      itemId: artifactId,
      class: "text",
      textFormat: "markdown",
      sourceKind: "m48-artifact-copy",
      state: "ready",
      title: "Live artifact",
      mimeType: "text/markdown; charset=utf-8",
      width: null,
      height: null,
      byteSize: 9,
      lineCount: 1,
      sha256: sha,
      createdAtMs: 1,
      annotations: [],
    } as const;
    resolveCopy({
      ...snapshot,
      items: [copied],
      selectedCollection: {
        ...snapshot.selectedCollection,
        itemCount: 1,
        updatedAtMs: 2,
      },
    });
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: /Live artifact — text/i }),
      ).toBeVisible(),
    );
    expect(
      screen.getByRole("button", { name: /Live artifact — text/i }),
    ).toHaveFocus();
  });
  it("cancels and reports artifact-copy failures without changing the snapshot", async () => {
    const loadArtifacts = vi.fn().mockResolvedValue({
      schemaVersion: 1,
      artifacts: [
        {
          schemaVersion: 1,
          artifactId,
          class: "text",
          mimeType: "text/plain; charset=utf-8",
          sourceKind: "visible-completed-reply",
          displayLabel: "Live artifact",
          suggestedFilename: "live.txt",
          byteSize: 9,
          sha256: sha,
          createdAt: 0,
          expiresAt: 10,
          state: "ready",
          disposal: "transient-memory-one-successful-save",
        },
      ],
      diagnosticCode: null,
    });
    load.mockResolvedValue(snapshot);
    render(
      <LocalReviewPane
        {...({
          taskCatalog: { selectedTask: null },
          loadArtifacts,
        } as unknown as ReviewPaneData)}
      />,
    );
    await screen.findByRole("button", { name: "Copy generated artifact…" });
    fireEvent.click(
      screen.getByRole("button", { name: "Copy generated artifact…" }),
    );
    await screen.findByRole("dialog", { name: "Copy live generated artifact" });
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(copyArtifact).not.toHaveBeenCalled();
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Copy generated artifact…" }),
      ).toHaveFocus(),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Copy generated artifact…" }),
    );
    await screen.findByRole("dialog", { name: "Copy live generated artifact" });
    copyArtifact.mockRejectedValueOnce(new Error("private /tmp/source"));
    fireEvent.click(
      screen.getByRole("button", { name: "Copy into local review" }),
    );
    const error = await screen.findByRole("alert");
    expect(error).toHaveTextContent(
      "The generated artifact could not be copied into local review.",
    );
    await waitFor(() => expect(error).toHaveFocus());
    expect(screen.queryByText(/tmp\/source/)).toBeNull();
    expect(
      screen.queryByRole("button", { name: /Live artifact — text/i }),
    ).toBeNull();
  });
  it("captures metadata separately from generated-artifact content", async () => {
    const loadArtifacts = vi.fn().mockResolvedValue({
      schemaVersion: 1,
      artifacts: [
        {
          schemaVersion: 1,
          artifactId,
          class: "markdown",
          mimeType: "text/markdown; charset=utf-8",
          sourceKind: "visible-fenced-block",
          displayLabel: "Live artifact",
          suggestedFilename: "live.md",
          byteSize: 9,
          sha256: sha,
          createdAt: 0,
          expiresAt: 10,
          state: "ready",
          disposal: "transient-memory-one-successful-save",
        },
      ],
      diagnosticCode: null,
    });
    const evidence = {
      itemId: artifactId,
      class: "evidence",
      textFormat: null,
      sourceKind: "typed-evidence-snapshot",
      evidenceSource: "m48-generated-artifact-metadata",
      state: "ready",
      title: "Generated artifact metadata: Live artifact",
      mimeType: "application/json; profile=evidence-envelope-v1",
      width: null,
      height: null,
      byteSize: 200,
      lineCount: null,
      sha256: sha,
      createdAtMs: 1,
      annotations: [],
    } as const;
    captureArtifactMetadata.mockResolvedValueOnce({
      outcome: "created",
      createdItemId: artifactId,
      source: "m48-generated-artifact-metadata",
      snapshot: {
        ...snapshot,
        items: [evidence],
        selectedCollection: {
          ...snapshot.selectedCollection,
          itemCount: 1,
          updatedAtMs: 2,
        },
      },
    });
    previewArtifactMetadata.mockResolvedValueOnce({
      schemaVersion: 1,
      itemId: artifactId,
      source: "m48-generated-artifact-metadata",
      title: evidence.title,
      summary: "Captured live generated-artifact metadata only.",
      details: {
        artifactState: "ready",
        artifactKind: "markdown",
        format: "markdown",
        byteLength: 9,
        truncated: false,
        manifestSha256: sha,
      },
      byteSize: 200,
      sha256: sha,
      createdAtMs: 1,
    });
    const activity = vi.spyOn(localReviewSession, "recordLocalReviewActivity");
    load.mockResolvedValue(snapshot);
    render(
      <LocalReviewPane
        {...({
          taskCatalog: { selectedTask: null },
          loadArtifacts,
        } as unknown as ReviewPaneData)}
      />,
    );
    fireEvent.click(
      await screen.findByRole("button", {
        name: "Capture generated-artifact metadata…",
      }),
    );
    await screen.findByRole("dialog", {
      name: "Capture generated-artifact metadata",
    });
    expect(screen.getByText(/artifact content is not copied/i)).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Capture metadata" }));
    await waitFor(() =>
      expect(captureArtifactMetadata).toHaveBeenCalledWith({
        collectionId: id,
        expectedCollectionUpdatedAtMs: 1,
        artifactId,
        manifestSha256: sha,
      }),
    );
    await screen.findByRole("heading", {
      name: "Generated-artifact metadata evidence",
    });
    expect(activity).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "item-added",
        label: evidence.title,
        digest: sha,
      }),
    );
    expect(screen.queryByText("live.md")).toBeNull();
    expect(document.querySelector('input[type="file"]')).toBeNull();
  });
  it("captures only a native safe-preview claim", async () => {
    const evidence = {
      itemId: artifactId,
      class: "evidence",
      textFormat: null,
      sourceKind: "typed-evidence-snapshot",
      evidenceSource: "safe-preview-metadata",
      state: "ready",
      title: "Safe preview metadata",
      mimeType: "application/json; profile=evidence-envelope-v1",
      width: null,
      height: null,
      byteSize: 200,
      lineCount: null,
      sha256: sha,
      createdAtMs: 1,
      annotations: [],
    } as const;
    claimSafePreview.mockResolvedValueOnce({
      claimId: artifactId,
      claimSha256: sha,
      previewState: "ready",
      kind: "image",
      rendering: "bounded-image",
      mediaType: "image/png",
      byteLength: 10,
      truncated: false,
      widthPx: 1,
      heightPx: 1,
    });
    captureSafePreview.mockResolvedValueOnce({
      outcome: "created",
      createdItemId: artifactId,
      source: "safe-preview-metadata",
      snapshot: {
        ...snapshot,
        items: [evidence],
        selectedCollection: {
          ...snapshot.selectedCollection,
          itemCount: 1,
          updatedAtMs: 2,
        },
      },
    });
    previewSafePreview.mockResolvedValueOnce({
      schemaVersion: 1,
      itemId: artifactId,
      source: "safe-preview-metadata",
      title: evidence.title,
      summary: "Captured current safe-preview metadata only.",
      details: {
        previewState: "ready",
        kind: "image",
        rendering: "bounded-image",
        mediaType: "image/png",
        byteLength: 10,
        truncated: false,
        widthPx: 1,
        heightPx: 1,
      },
      byteSize: 200,
      sha256: sha,
      createdAtMs: 1,
    });
    renderPane();
    fireEvent.click(
      await screen.findByRole("button", {
        name: "Capture safe-preview metadata…",
      }),
    );
    await screen.findByRole("dialog", {
      name: "Capture safe-preview metadata",
    });
    expect(screen.queryByText(/private\/|https:\/\//i)).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Capture metadata" }));
    await waitFor(() =>
      expect(captureSafePreview).toHaveBeenCalledWith({
        collectionId: id,
        expectedCollectionUpdatedAtMs: 1,
        previewClaimId: artifactId,
        claimSha256: sha,
      }),
    );
    await screen.findByRole("heading", {
      name: "Safe-preview metadata evidence",
    });
    expect(document.querySelector('input[type="file"]')).toBeNull();
  });
  it("renders only the fixed native image entry point", async () => {
    renderPane();
    await screen.findByText("Add image mockup");
    expect(
      screen.getByText(
        "Choose one static PNG or JPEG. QuireForge validates and copies it locally without retaining or displaying its original path.",
      ),
    ).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Choose PNG or JPEG…" }),
    ).toBeVisible();
    expect(document.querySelector('input[type="file"]')).toBeNull();
    expect(screen.queryByLabelText(/url|path|filename/i)).toBeNull();
  });

  it("installs created snapshot and renders inert PNG preview", async () => {
    renderPane();
    await screen.findByText("Add image mockup");
    fireEvent.change(screen.getByLabelText("Text item title"), {
      target: { value: "Mockup" },
    });
    const created = {
      ...snapshot,
      items: [image],
      selectedCollection: {
        ...snapshot.selectedCollection,
        itemCount: 1,
        updatedAtMs: 2,
      },
    };
    pick.mockResolvedValueOnce({ outcome: "created", snapshot: created });
    preview.mockResolvedValueOnce({
      schemaVersion: 1,
      itemId: id,
      mimeType: "image/png",
      width: 1,
      height: 1,
      byteSize: 1,
      sha256: sha,
      dataUrl: "data:image/png;base64,AA==",
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Choose PNG or JPEG…" }),
    );
    await waitFor(() =>
      expect(preview).toHaveBeenCalledWith({ itemId: id, sha256: sha }),
    );
    expect(
      await screen.findByRole("img", { name: "Mockup, 1 by 1 pixels" }),
    ).toHaveAttribute("src", "data:image/png;base64,AA==");
    expect(screen.getAllByText("Not comparable").length).toBeGreaterThan(0);
    expect(
      screen.getAllByText("Not promotion eligible").length,
    ).toBeGreaterThan(0);
    expect(screen.queryByRole("button", { name: "Compare" })).toBeNull();
  });
  it("creates and previews a manual validation evidence snapshot", async () => {
    renderPane();
    await screen.findByRole("heading", { name: "Add evidence snapshot" });
    expect(screen.getByLabelText("Manual validation summary")).toBeVisible();
    expect(
      screen.getByText(
        "Manual validation summary copies bounded local text into this review collection. It retains no path, URL, command output, approval, or external connection.",
      ),
    ).toBeVisible();
    expect(document.querySelector('input[type="file"]')).toBeNull();
    expect(
      screen.queryByLabelText(
        /path|url|filename|command|provider|connector|approval/i,
      ),
    ).toBeNull();
    fireEvent.change(screen.getByLabelText("Evidence label"), {
      target: { value: "Validation" },
    });
    fireEvent.change(screen.getByLabelText("Manual validation summary"), {
      target: { value: "line\r\nsummary" },
    });
    const evidence = {
      ...image,
      class: "evidence",
      sourceKind: "typed-evidence-snapshot",
      evidenceSource: "manual-validation-summary",
      mimeType: "application/json; profile=evidence-envelope-v1",
      width: null,
      height: null,
    };
    const created = {
      ...snapshot,
      items: [evidence],
      selectedCollection: {
        ...snapshot.selectedCollection,
        itemCount: 1,
        updatedAtMs: 2,
      },
    };
    createEvidence.mockResolvedValueOnce({
      outcome: "created",
      createdItemId: id,
      source: "manual-validation-summary",
      snapshot: created,
    });
    previewEvidence.mockResolvedValueOnce({
      schemaVersion: 1,
      itemId: id,
      source: "manual-validation-summary",
      title: "Validation",
      summary: "line\nsummary",
      byteSize: 12,
      sha256: sha,
      createdAtMs: 0,
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Add evidence snapshot" }),
    );
    await waitFor(() =>
      expect(createEvidence).toHaveBeenCalledWith({
        collectionId: id,
        expectedCollectionUpdatedAtMs: 1,
        title: "Validation",
        summary: "line\nsummary",
      }),
    );
    await waitFor(() =>
      expect(previewEvidence).toHaveBeenCalledWith({ itemId: id, sha256: sha }),
    );
    expect(
      screen.getByRole("heading", { name: "Evidence preview" }),
    ).toHaveFocus();
    expect(
      screen.getByText(
        (_, element) =>
          element?.tagName === "PRE" && element.textContent === "line\nsummary",
      ),
    ).toBeVisible();
    expect(screen.getByLabelText(`SHA-256 ${sha}`)).toBeVisible();
    expect(screen.getAllByText("Not comparable").length).toBeGreaterThan(0);
    expect(
      screen.getAllByText("Not promotion eligible").length,
    ).toBeGreaterThan(0);
  });

  it("preserves the snapshot and focuses a closed error when evidence creation fails", async () => {
    renderPane();
    await screen.findByRole("heading", { name: "Add evidence snapshot" });
    fireEvent.change(screen.getByLabelText("Evidence label"), {
      target: { value: "Validation" },
    });
    fireEvent.change(screen.getByLabelText("Manual validation summary"), {
      target: { value: "summary" },
    });
    createEvidence.mockRejectedValueOnce(
      new Error("internal parser path /tmp/private"),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Add evidence snapshot" }),
    );
    const error = await screen.findByRole("alert");
    expect(error).toHaveTextContent("Evidence snapshot could not be added.");
    expect(error).toHaveFocus();
    expect(
      screen.getByRole("heading", { name: "Add evidence snapshot" }),
    ).toBeVisible();
    expect(screen.queryByText(/tmp\/private/)).toBeNull();
  });

  it("does not record item-added for a failed evidence result that contains older evidence", async () => {
    const evidence = {
      ...image,
      class: "evidence",
      sourceKind: "typed-evidence-snapshot",
      evidenceSource: "manual-validation-summary",
      mimeType: "application/json; profile=evidence-envelope-v1",
      width: null,
      height: null,
    };
    const olderEvidence = {
      ...snapshot,
      items: [evidence],
      selectedCollection: { ...snapshot.selectedCollection, itemCount: 1 },
    };
    const activity = vi.spyOn(localReviewSession, "recordLocalReviewActivity");
    renderPane();
    await screen.findByRole("heading", { name: "Add evidence snapshot" });
    fireEvent.change(screen.getByLabelText("Evidence label"), {
      target: { value: "Validation" },
    });
    fireEvent.change(screen.getByLabelText("Manual validation summary"), {
      target: { value: "summary" },
    });
    createEvidence.mockResolvedValueOnce({
      outcome: "failed",
      snapshot: olderEvidence,
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Add evidence snapshot" }),
    );
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Evidence snapshot could not be added.",
    );
    expect(activity).not.toHaveBeenCalled();
    expect(previewEvidence).not.toHaveBeenCalled();
  });

  it("discards a selected evidence item using the authoritative snapshot", async () => {
    vi.spyOn(window, "confirm").mockReturnValueOnce(true);
    renderPane();
    await screen.findByRole("heading", { name: "Add evidence snapshot" });
    const evidence = {
      ...image,
      class: "evidence",
      sourceKind: "typed-evidence-snapshot",
      evidenceSource: "manual-validation-summary",
      mimeType: "application/json; profile=evidence-envelope-v1",
      width: null,
      height: null,
    };
    const withEvidence = {
      ...snapshot,
      items: [evidence],
      selectedCollection: {
        ...snapshot.selectedCollection,
        itemCount: 1,
        updatedAtMs: 2,
      },
    };
    createEvidence.mockResolvedValueOnce({
      outcome: "created",
      createdItemId: id,
      source: "manual-validation-summary",
      snapshot: withEvidence,
    });
    previewEvidence.mockResolvedValueOnce({
      schemaVersion: 1,
      itemId: id,
      source: "manual-validation-summary",
      title: "Validation",
      summary: "summary",
      byteSize: 7,
      sha256: sha,
      createdAtMs: 0,
    });
    fireEvent.change(screen.getByLabelText("Evidence label"), {
      target: { value: "Validation" },
    });
    fireEvent.change(screen.getByLabelText("Manual validation summary"), {
      target: { value: "summary" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Add evidence snapshot" }),
    );
    await screen.findByRole("heading", { name: "Evidence preview" });
    discard.mockResolvedValueOnce(snapshot);
    fireEvent.click(
      screen.getByRole("button", { name: "Discard selected item" }),
    );
    await waitFor(() =>
      expect(discard).toHaveBeenCalledWith({
        collectionId: id,
        itemId: id,
        expectedCollectionUpdatedAtMs: 2,
      }),
    );
    expect(
      screen.queryByRole("heading", { name: "Evidence preview" }),
    ).toBeNull();
  });

  it("creates annotations from an authoritative snapshot and preserves the draft on failure", async () => {
    const textItem = {
      itemId: id,
      class: "text",
      textFormat: "plain",
      sourceKind: "user-authored-text",
      state: "ready",
      title: "Item",
      mimeType: "text/plain; charset=utf-8",
      width: null,
      height: null,
      byteSize: 4,
      sha256: sha,
      createdAtMs: 0,
      annotations: [],
    } as const;
    const initial = {
      ...snapshot,
      items: [textItem],
      selectedCollection: { ...snapshot.selectedCollection, itemCount: 1 },
    };
    load.mockResolvedValueOnce(initial);
    const props = {
      taskCatalog: { selectedTask: null },
    } as unknown as ReviewPaneData;
    render(<LocalReviewPane {...props} />);
    const itemButton = await screen.findByRole("button", {
      name: /Item — text/i,
    });
    fireEvent.click(itemButton);
    expect(screen.getByRole("heading", { name: "Annotations" })).toBeVisible();
    expect(screen.getByLabelText("Annotation text")).toBeVisible();
    expect(
      screen.getByRole("button", { name: "Add annotation" }),
    ).toBeVisible();
    expect(
      screen.queryByLabelText(
        /author|mention|range|coordinate|approval|execution/i,
      ),
    ).toBeNull();
    let resolveCreate: (value: unknown) => void = () => undefined;
    createAnnotation.mockImplementationOnce(
      () =>
        new Promise<unknown>((resolve) => {
          resolveCreate = resolve;
        }),
    );
    fireEvent.change(screen.getByLabelText("Annotation text"), {
      target: { value: "line\r\nnote" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Add annotation" }));
    await waitFor(() =>
      expect(createAnnotation).toHaveBeenCalledWith({
        collectionId: id,
        itemId: id,
        expectedCollectionUpdatedAtMs: 1,
        text: "line\nnote",
      }),
    );
    expect(screen.getByText("No annotations.")).toBeVisible();
    const annotation = {
      schemaVersion: 1,
      annotationId: "018f0000-0000-7000-8000-000000000002",
      itemId: id,
      text: "line\nnote",
      state: "open",
      createdAtMs: 1,
      updatedAtMs: 1,
    } as const;
    resolveCreate({
      ...initial,
      items: [{ ...textItem, annotations: [annotation] }],
      selectedCollection: { ...initial.selectedCollection, updatedAtMs: 2 },
    });
    await waitFor(() =>
      expect(
        screen.getByText(
          (_, element) =>
            element?.tagName === "P" && element.textContent === "line\nnote",
        ),
      ).toBeVisible(),
    );
    expect(screen.getByLabelText("Annotation text")).toHaveValue("");
    await waitFor(() =>
      expect(
        screen.getByRole("heading", { name: "Annotations" }),
      ).toHaveFocus(),
    );
    fireEvent.change(screen.getByLabelText("Annotation text"), {
      target: { value: "retain this" },
    });
    createAnnotation.mockRejectedValueOnce(
      new Error("private path /tmp/source"),
    );
    fireEvent.click(screen.getByRole("button", { name: "Add annotation" }));
    const error = await screen.findByRole("alert");
    expect(error).toHaveTextContent("Annotation could not be added.");
    expect(error).toHaveFocus();
    expect(screen.getByLabelText("Annotation text")).toHaveValue("retain this");
    expect(
      screen.getByText(
        (_, element) =>
          element?.tagName === "P" && element.textContent === "line\nnote",
      ),
    ).toBeVisible();
    expect(screen.queryByText(/tmp\/source/)).toBeNull();
  });

  it("edits, resolves, and reopens annotations only through authoritative snapshots", async () => {
    const firstAnnotation = {
      schemaVersion: 1,
      annotationId: "018f0000-0000-7000-8000-000000000002",
      itemId: id,
      text: "first",
      state: "open",
      createdAtMs: 1,
      updatedAtMs: 1,
    } as const;
    const secondAnnotation = {
      schemaVersion: 1,
      annotationId: "018f0000-0000-7000-8000-000000000003",
      itemId: id,
      text: "second",
      state: "resolved",
      createdAtMs: 2,
      updatedAtMs: 2,
    } as const;
    const textItem = {
      itemId: id,
      class: "text",
      textFormat: "plain",
      sourceKind: "user-authored-text",
      state: "ready",
      title: "Item",
      mimeType: "text/plain; charset=utf-8",
      width: null,
      height: null,
      byteSize: 4,
      sha256: sha,
      createdAtMs: 0,
      annotations: [firstAnnotation, secondAnnotation],
    } as const;
    const initial = {
      ...snapshot,
      items: [textItem],
      selectedCollection: { ...snapshot.selectedCollection, itemCount: 1 },
    };
    load.mockResolvedValueOnce(initial);
    const props = {
      taskCatalog: { selectedTask: null },
    } as unknown as ReviewPaneData;
    render(<LocalReviewPane {...props} />);
    fireEvent.click(
      await screen.findByRole("button", { name: /Item — text/i }),
    );
    expect(screen.getAllByRole("button", { name: "Edit" })[0]!).toBeVisible();
    expect(screen.getByRole("button", { name: "Resolve" })).toBeVisible();
    expect(screen.getByRole("button", { name: "Reopen" })).toBeVisible();
    fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]!);
    expect(screen.getByLabelText("Edit annotation")).toHaveValue("first");
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(editAnnotation).not.toHaveBeenCalled();
    await waitFor(() =>
      expect(screen.getAllByRole("button", { name: "Edit" })[0]!).toHaveFocus(),
    );
    fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]!);
    fireEvent.change(screen.getByLabelText("Edit annotation"), {
      target: { value: "edited\r\ntext" },
    });
    const edited = {
      ...firstAnnotation,
      text: "edited\ntext",
      updatedAtMs: 2,
    } as const;
    const afterEdit = {
      ...initial,
      items: [{ ...textItem, annotations: [edited, secondAnnotation] }],
      selectedCollection: { ...initial.selectedCollection, updatedAtMs: 2 },
    };
    let resolveEdit: (value: unknown) => void = () => undefined;
    editAnnotation.mockImplementationOnce(
      () =>
        new Promise<unknown>((resolve) => {
          resolveEdit = resolve;
        }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Save annotation" }));
    await waitFor(() =>
      expect(editAnnotation).toHaveBeenCalledWith({
        collectionId: id,
        itemId: id,
        annotationId: firstAnnotation.annotationId,
        expectedCollectionUpdatedAtMs: 1,
        text: "edited\ntext",
      }),
    );
    expect(
      screen.getByText(
        (_, element) =>
          element?.tagName === "P" && element.textContent === "first",
      ),
    ).toBeVisible();
    resolveEdit(afterEdit);
    await screen.findByText(
      (_, element) =>
        element?.tagName === "P" && element.textContent === "edited\ntext",
    );
    expect(
      document.getElementById(`annotation-${firstAnnotation.annotationId}`),
    ).toHaveFocus();
    const resolved = { ...edited, state: "resolved", updatedAtMs: 3 } as const;
    const afterResolve = {
      ...afterEdit,
      items: [{ ...textItem, annotations: [secondAnnotation, resolved] }],
      selectedCollection: { ...afterEdit.selectedCollection, updatedAtMs: 3 },
    };
    resolveAnnotation.mockResolvedValueOnce(afterResolve);
    fireEvent.click(screen.getByRole("button", { name: "Resolve" }));
    await waitFor(() =>
      expect(resolveAnnotation).toHaveBeenCalledWith({
        collectionId: id,
        itemId: id,
        annotationId: firstAnnotation.annotationId,
        expectedCollectionUpdatedAtMs: 2,
      }),
    );
    expect(
      document.getElementById(`annotation-${firstAnnotation.annotationId}`),
    ).toHaveFocus();
    const reopened = { ...resolved, state: "open", updatedAtMs: 4 } as const;
    const afterReopen = {
      ...afterResolve,
      items: [{ ...textItem, annotations: [reopened, secondAnnotation] }],
      selectedCollection: {
        ...afterResolve.selectedCollection,
        updatedAtMs: 4,
      },
    };
    reopenAnnotation.mockResolvedValueOnce(afterReopen);
    fireEvent.click(screen.getAllByRole("button", { name: "Reopen" })[0]!);
    await waitFor(() =>
      expect(reopenAnnotation).toHaveBeenCalledWith({
        collectionId: id,
        itemId: id,
        annotationId: firstAnnotation.annotationId,
        expectedCollectionUpdatedAtMs: 3,
      }),
    );
    expect(
      document.getElementById(`annotation-${firstAnnotation.annotationId}`),
    ).toHaveFocus();
    fireEvent.click(screen.getAllByRole("button", { name: "Edit" })[0]!);
    fireEvent.change(screen.getByLabelText("Edit annotation"), {
      target: { value: "retain this" },
    });
    editAnnotation.mockRejectedValueOnce(
      new Error("private storage /tmp/annotation"),
    );
    fireEvent.click(screen.getByRole("button", { name: "Save annotation" }));
    const error = await screen.findByRole("alert");
    expect(error).toHaveFocus();
    expect(screen.getByLabelText("Edit annotation")).toHaveValue("retain this");
    expect(screen.queryByText(/tmp\/annotation/)).toBeNull();
    expect(
      screen.queryByRole("combobox", { name: /state|status/i }),
    ).toBeNull();
    expect(
      screen.queryByLabelText(
        /author|mention|range|coordinate|approval|execution/i,
      ),
    ).toBeNull();
  });

  it("keeps annotations visible but read-only for a non-mutable item", async () => {
    const annotation = {
      schemaVersion: 1,
      annotationId: "018f0000-0000-7000-8000-000000000002",
      itemId: id,
      text: "visible note",
      state: "open",
      createdAtMs: 1,
      updatedAtMs: 1,
    } as const;
    const staleItem = {
      itemId: id,
      class: "text",
      textFormat: "plain",
      sourceKind: "user-authored-text",
      state: "stale",
      title: "Item",
      mimeType: "text/plain; charset=utf-8",
      width: null,
      height: null,
      byteSize: 4,
      sha256: sha,
      createdAtMs: 0,
      annotations: [annotation],
    } as const;
    load.mockResolvedValueOnce({
      ...snapshot,
      items: [staleItem],
      selectedCollection: { ...snapshot.selectedCollection, itemCount: 1 },
    });
    const props = {
      taskCatalog: { selectedTask: null },
    } as unknown as ReviewPaneData;
    render(<LocalReviewPane {...props} />);
    fireEvent.click(
      await screen.findByRole("button", { name: /Item — text/i }),
    );
    expect(screen.getByText("visible note")).toBeVisible();
    expect(
      screen.getByText(
        "Annotations are read-only because this item or collection is not active and ready.",
      ),
    ).toBeVisible();
    expect(screen.getByRole("button", { name: "Edit" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Resolve" })).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Delete annotation…" }),
    ).toBeDisabled();
    expect(
      screen.queryByRole("button", { name: /delete all|bulk delete/i }),
    ).toBeNull();
  });

  it("confirms and deletes only the local annotation through an authoritative snapshot", async () => {
    const first = {
      schemaVersion: 1,
      annotationId: "018f0000-0000-7000-8000-000000000002",
      itemId: id,
      text: "first",
      state: "open",
      createdAtMs: 1,
      updatedAtMs: 1,
    } as const;
    const second = {
      schemaVersion: 1,
      annotationId: "018f0000-0000-7000-8000-000000000003",
      itemId: id,
      text: "second",
      state: "open",
      createdAtMs: 2,
      updatedAtMs: 2,
    } as const;
    const item = {
      itemId: id,
      class: "text",
      textFormat: "plain",
      sourceKind: "user-authored-text",
      state: "ready",
      title: "Item",
      mimeType: "text/plain; charset=utf-8",
      width: null,
      height: null,
      byteSize: 4,
      sha256: sha,
      createdAtMs: 0,
      annotations: [first, second],
    } as const;
    const initial = {
      ...snapshot,
      items: [item],
      selectedCollection: { ...snapshot.selectedCollection, itemCount: 1 },
    };
    load.mockResolvedValueOnce(initial);
    const props = {
      taskCatalog: { selectedTask: null },
    } as unknown as ReviewPaneData;
    render(<LocalReviewPane {...props} />);
    fireEvent.click(
      await screen.findByRole("button", { name: /Item — text/i }),
    );
    fireEvent.click(
      screen.getAllByRole("button", { name: "Delete annotation…" })[0]!,
    );
    expect(
      screen.getByRole("dialog", { name: "Delete annotation confirmation" }),
    ).toHaveTextContent(
      "does not alter the review item, item content or digest, task, optional plan, evidence, generated artifacts, project files, Git, approval, dispatch, or execution state",
    );
    expect(deleteAnnotation).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(deleteAnnotation).not.toHaveBeenCalled();
    expect(screen.getByText("first")).toBeVisible();
    await waitFor(() =>
      expect(
        screen.getAllByRole("button", { name: "Delete annotation…" })[0]!,
      ).toHaveFocus(),
    );
    fireEvent.click(
      screen.getAllByRole("button", { name: "Delete annotation…" })[0]!,
    );
    let resolveDelete: (value: unknown) => void = () => undefined;
    deleteAnnotation.mockImplementationOnce(
      () =>
        new Promise<unknown>((resolve) => {
          resolveDelete = resolve;
        }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Delete annotation" }));
    await waitFor(() =>
      expect(deleteAnnotation).toHaveBeenCalledWith({
        collectionId: id,
        itemId: id,
        annotationId: first.annotationId,
        expectedCollectionUpdatedAtMs: 1,
      }),
    );
    expect(screen.getByText("first")).toBeVisible();
    const afterDelete = {
      ...initial,
      items: [{ ...item, annotations: [second] }],
      selectedCollection: { ...initial.selectedCollection, updatedAtMs: 2 },
    };
    resolveDelete(afterDelete);
    await waitFor(() => expect(screen.queryByText("first")).toBeNull());
    expect(screen.getByText("second")).toBeVisible();
    expect(screen.getByRole("button", { name: /Item — text/i })).toBeVisible();
    await waitFor(() =>
      expect(
        document.getElementById(`annotation-${second.annotationId}`),
      ).toHaveFocus(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Delete annotation…" }));
    deleteAnnotation.mockRejectedValueOnce(new Error("sqlite /tmp/private"));
    fireEvent.click(screen.getByRole("button", { name: "Delete annotation" }));
    const error = await screen.findByRole("alert");
    expect(error).toHaveTextContent("Annotation could not be deleted.");
    expect(error).toHaveFocus();
    expect(screen.getByText("second")).toBeVisible();
    expect(screen.queryByText(/sqlite|tmp\/private/)).toBeNull();
  });

  it("creates a bounded text comparison, reads inert lines, and excludes ineligible items", async () => {
    const rightId = "018f0000-0000-7000-8000-000000000010";
    const staleId = "018f0000-0000-7000-8000-000000000011";
    const left = {
      itemId: id,
      class: "text",
      textFormat: "plain",
      sourceKind: "user-authored-text",
      state: "ready",
      title: "Left",
      mimeType: "text/plain; charset=utf-8",
      width: null,
      height: null,
      byteSize: 4,
      lineCount: 1,
      sha256: sha,
      createdAtMs: 1,
      annotations: [],
    } as const;
    const right = {
      ...left,
      itemId: rightId,
      title: "Right",
      sha256: "b".repeat(64),
    } as const;
    const markdown = {
      ...right,
      itemId: "018f0000-0000-7000-8000-000000000012",
      title: "Markdown",
      textFormat: "markdown",
    } as const;
    const stale = {
      ...right,
      itemId: staleId,
      title: "Stale",
      state: "stale",
    } as const;
    const large = {
      ...right,
      itemId: "018f0000-0000-7000-8000-000000000013",
      title: "Large",
      byteSize: 128 * 1024 + 1,
    } as const;
    const manyLines = {
      ...right,
      itemId: "018f0000-0000-7000-8000-000000000014",
      title: "Many lines",
      lineCount: 2001,
    } as const;
    const imageItem = {
      ...image,
      itemId: "018f0000-0000-7000-8000-000000000015",
      lineCount: null,
      annotations: [],
    } as const;
    const evidence = {
      ...imageItem,
      itemId: "018f0000-0000-7000-8000-000000000016",
      class: "evidence",
      title: "Evidence",
      sourceKind: "typed-evidence-snapshot",
      mimeType: "application/json",
      width: null,
      height: null,
    } as const;
    const initial = {
      ...snapshot,
      items: [
        left,
        right,
        markdown,
        stale,
        large,
        manyLines,
        imageItem,
        evidence,
      ],
      selectedCollection: { ...snapshot.selectedCollection, itemCount: 8 },
    };
    load.mockResolvedValueOnce(initial);
    render(
      <LocalReviewPane
        {...({
          taskCatalog: { selectedTask: null },
        } as unknown as ReviewPaneData)}
      />,
    );
    fireEvent.click(
      await screen.findByRole("button", { name: /Left — text/i }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Compare" }));
    expect(
      screen.getByRole("dialog", { name: "Create comparison" }),
    ).toHaveTextContent("Left side: Left");
    expect(screen.getByRole("option", { name: /Right.*plain/i })).toBeVisible();
    for (const forbidden of [
      "Left",
      "Markdown",
      "Stale",
      "Large",
      "Many lines",
      "Mockup",
      "Evidence",
    ])
      expect(
        screen.queryByRole("option", { name: new RegExp(forbidden, "i") }),
      ).toBeNull();
    const comparisonDialog = screen.getByRole("dialog", {
      name: "Create comparison",
    });
    const rightSide = screen.getByRole("combobox", { name: "Right side" });
    const cancelComparison = screen.getByRole("button", { name: "Cancel" });
    cancelComparison.focus();
    fireEvent.keyDown(cancelComparison, { key: "Tab" });
    expect(rightSide).toHaveFocus();
    fireEvent.keyDown(comparisonDialog, { key: "Escape" });
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Compare" })).toHaveFocus(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Compare" }));
    expect(
      screen.getByRole("dialog", { name: "Create comparison" }),
    ).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Compare" })).toHaveFocus(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Compare" }));
    const comparisonId = "018f0000-0000-7000-8000-000000000017";
    const binding = {
      schemaVersion: 1,
      comparisonId,
      collectionId: id,
      leftItemId: id,
      rightItemId: rightId,
      leftSha256: sha,
      rightSha256: "b".repeat(64),
      textFormat: "plain",
      state: "ready",
      createdAtMs: 2,
    } as const;
    const created = {
      ...initial,
      comparisons: [binding],
      selectedCollection: { ...initial.selectedCollection, updatedAtMs: 2 },
    };
    createComparison.mockResolvedValueOnce(created);
    readComparison.mockResolvedValueOnce({
      comparisonId,
      leftItemId: id,
      leftSha256: sha,
      rightItemId: rightId,
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
    });
    fireEvent.click(screen.getByRole("button", { name: "Create comparison" }));
    await waitFor(() =>
      expect(createComparison).toHaveBeenCalledWith({
        collectionId: id,
        leftItemId: id,
        rightItemId: rightId,
        expectedCollectionUpdatedAtMs: 1,
      }),
    );
    await waitFor(() =>
      expect(readComparison).toHaveBeenCalledWith({
        collectionId: id,
        comparisonId,
      }),
    );
    expect(
      await screen.findByRole("heading", { name: "Comparison result" }),
    ).toHaveFocus();
    expect(screen.getByText("Unchanged")).toBeVisible();
    expect(screen.getByText("Added")).toBeVisible();
    expect(screen.getByText("Removed")).toBeVisible();
    expect(screen.getByText(/left 1, right 1/)).toBeVisible();
    expect(
      screen.queryByRole("button", {
        name: /apply|accept|merge|patch|save|run|execute/i,
      }),
    ).toBeNull();
    expect(JSON.stringify(createComparison.mock.calls[0]?.[0])).not.toMatch(
      /path|url|git|shell|approval|dispatch|execution/i,
    );
  });

  it("renders closed comparison states, warnings, and authoritative discard behavior", async () => {
    readComparison.mockClear();
    const rightId = "018f0000-0000-7000-8000-000000000020";
    const left = {
      itemId: id,
      class: "text",
      textFormat: "plain",
      sourceKind: "user-authored-text",
      state: "ready",
      title: "Left",
      mimeType: "text/plain; charset=utf-8",
      width: null,
      height: null,
      byteSize: 4,
      lineCount: 1,
      sha256: sha,
      createdAtMs: 1,
      annotations: [],
    } as const;
    const right = {
      ...left,
      itemId: rightId,
      title: "Right",
      sha256: "b".repeat(64),
    } as const;
    const ready = {
      schemaVersion: 1,
      comparisonId: "018f0000-0000-7000-8000-000000000021",
      collectionId: id,
      leftItemId: id,
      rightItemId: rightId,
      leftSha256: sha,
      rightSha256: "b".repeat(64),
      textFormat: "plain",
      state: "ready",
      createdAtMs: 1,
    } as const;
    const stale = {
      ...ready,
      comparisonId: "018f0000-0000-7000-8000-000000000022",
      state: "stale",
      createdAtMs: 2,
    } as const;
    const unavailable = {
      ...ready,
      comparisonId: "018f0000-0000-7000-8000-000000000023",
      state: "unavailable",
      createdAtMs: 3,
    } as const;
    const initial = {
      ...snapshot,
      items: [left, right],
      comparisons: [unavailable, stale, ready],
      selectedCollection: {
        ...snapshot.selectedCollection,
        comparisonCountWarning: true,
        updatedAtMs: 3,
      },
    };
    load.mockResolvedValueOnce(initial);
    render(
      <LocalReviewPane
        {...({
          taskCatalog: { selectedTask: null },
        } as unknown as ReviewPaneData)}
      />,
    );
    await screen.findByText(
      "Comparison warning: six or more bindings are stored; the limit is eight.",
    );
    const staleButton = screen.getByRole("button", {
      name: /Left and Right comparison, stale/i,
    });
    fireEvent.click(staleButton);
    expect(screen.getByText(/comparison is stale/i)).toBeVisible();
    expect(readComparison).not.toHaveBeenCalled();
    fireEvent.click(
      screen.getAllByRole("button", { name: "Discard comparison…" })[0]!,
    );
    expect(
      screen.getByRole("dialog", { name: "Discard comparison confirmation" }),
    ).toHaveTextContent(
      "does not alter either review item, item text or digest, annotations, task or plan, files, Git, approval, dispatch, or execution",
    );
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await waitFor(() =>
      expect(
        screen.getAllByRole("button", { name: "Discard comparison…" })[0]!,
      ).toHaveFocus(),
    );
    fireEvent.click(
      screen.getAllByRole("button", { name: "Discard comparison…" })[0]!,
    );
    const afterDiscard = {
      ...initial,
      comparisons: [stale, unavailable],
      selectedCollection: { ...initial.selectedCollection, updatedAtMs: 4 },
    };
    discardComparison.mockResolvedValueOnce(afterDiscard);
    fireEvent.click(screen.getByRole("button", { name: "Discard comparison" }));
    await waitFor(() =>
      expect(discardComparison).toHaveBeenCalledWith({
        collectionId: id,
        comparisonId: ready.comparisonId,
        expectedCollectionUpdatedAtMs: 3,
      }),
    );
    await waitFor(() =>
      expect(
        screen.queryByRole("button", { name: /comparison, ready/i }),
      ).toBeNull(),
    );
    expect(screen.getByRole("button", { name: /Left — text/i })).toBeVisible();
    expect(screen.getByRole("button", { name: /Right — text/i })).toBeVisible();
  });

  it("prepares, cancels, and confirms only a transient generated artifact", async () => {
    preparePromotion.mockClear();
    confirmPromotion.mockClear();
    cancelPromotion.mockClear();
    const item = {
      itemId: id,
      class: "text",
      textFormat: "plain",
      sourceKind: "user-authored-text",
      state: "ready",
      title: "Promotable",
      mimeType: "text/plain; charset=utf-8",
      width: null,
      height: null,
      byteSize: 4,
      lineCount: 1,
      sha256: sha,
      createdAtMs: 0,
      annotations: [
        {
          schemaVersion: 1,
          annotationId: "018f0000-0000-7000-8000-000000000030",
          itemId: id,
          text: "note",
          state: "open",
          createdAtMs: 1,
          updatedAtMs: 1,
        },
      ],
    } as const;
    const comparison = {
      schemaVersion: 1,
      comparisonId: "018f0000-0000-7000-8000-000000000031",
      collectionId: id,
      leftItemId: id,
      rightItemId: "018f0000-0000-7000-8000-000000000032",
      leftSha256: sha,
      rightSha256: "b".repeat(64),
      textFormat: "plain",
      state: "stale",
      createdAtMs: 1,
    } as const;
    const initial = { ...snapshot, items: [item], comparisons: [comparison] };
    const reservationId = "018f0000-0000-7000-8000-000000000033";
    const candidate = {
      reservationId,
      collectionId: id,
      itemId: id,
      title: "Promotable",
      sha256: sha,
      textFormat: "plain",
      destinationClass: "text",
      taskId: id,
      planId: null,
      createdAtMs: 0,
      expiresAtMs: 300000,
      state: "prepared",
    } as const;
    const manifest = {
      schemaVersion: 1,
      artifactId: "018f0000-0000-7000-8000-000000000034",
      class: "text",
      mimeType: "text/plain; charset=utf-8",
      sourceKind: "explicit-review-promotion",
      displayLabel: "Promotable",
      suggestedFilename: "review-promotion.txt",
      byteSize: 4,
      sha256: sha,
      createdAt: 0,
      expiresAt: 900000,
      state: "ready",
      disposal: "transient-memory-one-successful-save",
    } as const;
    load.mockResolvedValueOnce(initial);
    preparePromotion
      .mockResolvedValueOnce(candidate)
      .mockResolvedValueOnce(candidate);
    cancelPromotion.mockResolvedValueOnce({ ...candidate, state: "expired" });
    confirmPromotion.mockResolvedValueOnce(manifest);
    render(
      <LocalReviewPane
        {...({
          taskCatalog: { selectedTask: null },
        } as unknown as ReviewPaneData)}
      />,
    );
    fireEvent.click(
      await screen.findByRole("button", { name: /Promotable — text/i }),
    );
    expect(
      screen.getByRole("button", {
        name: "Create transient generated artifact…",
      }),
    ).toBeVisible();
    expect(
      screen.getByText("Promotion eligibility is not execution approval."),
    ).toBeVisible();
    expect(
      screen.getByText(
        "Creating a generated artifact does not approve or dispatch work.",
      ),
    ).toBeVisible();
    fireEvent.click(
      screen.getByRole("button", {
        name: "Create transient generated artifact…",
      }),
    );
    await waitFor(() =>
      expect(preparePromotion).toHaveBeenCalledWith({
        collectionId: id,
        itemId: id,
        expectedCollectionUpdatedAtMs: 1,
      }),
    );
    expect(
      screen.getByRole("dialog", {
        name: "Create transient generated artifact confirmation",
      }),
    ).toHaveTextContent(
      "This creates only a transient QuireForge generated artifact. It does not save a file, approve or dispatch work, run code, change Git, publish, or deploy.",
    );
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Cancel" })).toHaveFocus(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await waitFor(() =>
      expect(cancelPromotion).toHaveBeenCalledWith({ reservationId }),
    );
    await waitFor(() =>
      expect(
        screen.queryByRole("dialog", {
          name: "Create transient generated artifact confirmation",
        }),
      ).toBeNull(),
    );
    expect(
      screen.getByRole("button", {
        name: "Create transient generated artifact…",
      }),
    ).toHaveFocus();
    fireEvent.click(
      screen.getByRole("button", {
        name: "Create transient generated artifact…",
      }),
    );
    await screen.findByRole("dialog", {
      name: "Create transient generated artifact confirmation",
    });
    fireEvent.click(
      screen.getByRole("button", {
        name: "Create transient generated artifact",
      }),
    );
    await waitFor(() =>
      expect(confirmPromotion).toHaveBeenCalledWith({ reservationId }),
    );
    expect(
      await screen.findByRole("heading", {
        name: "Transient generated artifact created",
      }),
    ).toHaveFocus();
    expect(
      screen.getByText("Provenance: explicit-review-promotion"),
    ).toBeVisible();
    expect(screen.getByText("note")).toBeVisible();
    expect(
      screen.getByRole("button", { name: /comparison, stale/i }),
    ).toBeVisible();
    expect(
      screen.queryByRole("button", {
        name: /save|approve|run|apply|dispatch|publish|deploy/i,
      }),
    ).toBeNull();
  });
});
