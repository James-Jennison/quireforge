import type { ReviewPaneData } from "./types";

export default function FilesPane({ filePreview }: ReviewPaneData) {
  if (filePreview.state === "unavailable") {
    return <p role="status">Files are unavailable for this task.</p>;
  }
  if (filePreview.state === "empty") {
    return <p role="status">No bounded file evidence is open.</p>;
  }
  return (
    <article className="review-pane-card">
      <h3>{filePreview.displayPath}</h3>
      <p>
        {filePreview.kind} · {filePreview.byteSize} bytes ·{" "}
        {filePreview.rendering}
      </p>
      {filePreview.truncated && (
        <p role="status">The file evidence is truncated.</p>
      )}
    </article>
  );
}
