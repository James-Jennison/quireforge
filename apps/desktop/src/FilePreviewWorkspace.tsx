import type { FilePreviewSnapshot } from "./lib/filePreview";
import type { ProjectWorkspaceSnapshot } from "./lib/project";

type ProjectSummary = ProjectWorkspaceSnapshot["projects"][number];

type FilePreviewAvailability = "checking" | "native" | "preview";

interface FilePreviewWorkspaceProps {
  availability: FilePreviewAvailability;
  project: ProjectSummary | undefined;
  snapshot: FilePreviewSnapshot;
  busy: boolean;
  actionError: boolean;
  onPick: (projectId: string) => Promise<void>;
  onClear: () => void;
}

const diagnostics: Record<
  NonNullable<FilePreviewSnapshot["diagnosticCode"]>,
  string
> = {
  "invalid-request": "The preview request was invalid.",
  "project-not-found": "The selected project is no longer available.",
  "directory-unavailable": "The attached project directory is unavailable.",
  "identity-changed":
    "The attached directory identity changed. Relink it before previewing files.",
  "picker-unavailable":
    "The native file picker could not return a usable selection.",
  "outside-project":
    "Only files inside the selected attached project can be previewed.",
  "unsafe-path": "The selected path is not a safe regular project file.",
  "unsupported-type":
    "This file type is not supported by the safe preview boundary.",
  "file-too-large": "The selected file exceeds the bounded preview size.",
  "read-failed": "The selected file could not be read safely.",
  "invalid-content":
    "The selected file content did not match its supported preview format.",
  "image-dimensions-too-large":
    "The image dimensions exceed the safe preview limit.",
};

function formatBytes(value: number): string {
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KiB`;
  return `${(value / (1024 * 1024)).toFixed(1)} MiB`;
}

export function FilePreviewWorkspace({
  availability,
  project,
  snapshot,
  busy,
  actionError,
  onPick,
  onClear,
}: FilePreviewWorkspaceProps) {
  const projectReady =
    project !== undefined &&
    !project.archived &&
    ["connected-accessible", "connected-read-only"].includes(
      project.directory?.state ?? "",
    );
  const visibleSnapshot =
    snapshot.projectId === null ||
    (projectReady && snapshot.projectId === project?.id)
      ? snapshot
      : null;

  return (
    <section
      className="workspace-section file-preview-workspace"
      id="files"
      aria-labelledby="file-preview-title"
    >
      <div className="section-heading">
        <div>
          <p className="eyebrow">
            <span />
            Safe local review · Milestone 15A
          </p>
          <h2 id="file-preview-title">
            Preview a file without widening trust.
          </h2>
          <p>
            The native picker chooses one project-contained regular file. Paths
            remain native-only; React receives only bounded normalized content
            and a relative display name.
          </p>
        </div>
        <div className="section-actions">
          {visibleSnapshot?.state === "ready" && (
            <button
              className="ghost-action"
              type="button"
              disabled={busy}
              onClick={onClear}
            >
              Clear preview
            </button>
          )}
          <button
            className="secondary-action"
            type="button"
            disabled={availability !== "native" || !projectReady || busy}
            onClick={() => project && void onPick(project.id)}
          >
            {busy ? "Opening picker…" : "Choose project file"}
          </button>
        </div>
      </div>

      {availability === "preview" && (
        <p className="inline-notice">
          Browser preview cannot select or read local project files.
        </p>
      )}
      {availability === "checking" && (
        <p className="inline-notice">Checking native preview support…</p>
      )}
      {availability === "native" && !projectReady && (
        <p className="inline-notice">
          Attach or relink an accessible project before selecting a file.
        </p>
      )}
      {actionError && (
        <p className="inline-error" role="alert">
          The native preview bridge failed. No file content was retained.
        </p>
      )}
      {visibleSnapshot?.state === "unavailable" &&
        visibleSnapshot.diagnosticCode && (
          <p className="inline-error" role="alert">
            {diagnostics[visibleSnapshot.diagnosticCode]}
          </p>
        )}

      {visibleSnapshot?.state === "ready" ? (
        <article
          className="file-preview-card"
          aria-label={`Preview of ${visibleSnapshot.displayPath}`}
        >
          <header>
            <div>
              <span className="detail-kicker">
                {visibleSnapshot.kind} · {visibleSnapshot.rendering}
              </span>
              <h3>{visibleSnapshot.displayPath}</h3>
            </div>
            <span>{formatBytes(visibleSnapshot.byteSize!)}</span>
          </header>
          {visibleSnapshot.kind === "text" && (
            <>
              {visibleSnapshot.truncated && (
                <p className="inline-notice">
                  Preview truncated at the native content limit.
                </p>
              )}
              <pre className="file-preview-text">
                <code>{visibleSnapshot.textContent}</code>
              </pre>
            </>
          )}
          {visibleSnapshot.kind === "image" && visibleSnapshot.imageDataUrl && (
            <figure className="file-preview-image">
              <img
                src={visibleSnapshot.imageDataUrl}
                alt={`Preview of ${visibleSnapshot.displayPath}`}
              />
              <figcaption>
                {visibleSnapshot.imageWidth} × {visibleSnapshot.imageHeight}{" "}
                pixels
              </figcaption>
            </figure>
          )}
          {visibleSnapshot.kind === "pdf" && (
            <div className="file-preview-pdf">
              <strong>
                PDF recognized; active document rendering is disabled.
              </strong>
              <p>
                Milestone 15A shows bounded metadata only. The PDF bytes are not
                embedded in the privileged webview, so scripts, links, forms,
                and remote resources cannot execute.
              </p>
            </div>
          )}
        </article>
      ) : (
        <div className="file-preview-empty">
          <strong>No file preview selected</strong>
          <p>
            Supported now: normalized UTF-8 text, bounded PNG/JPEG images, and
            metadata-only PDF recognition.
          </p>
        </div>
      )}
    </section>
  );
}
