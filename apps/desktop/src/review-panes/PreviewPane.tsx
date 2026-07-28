import { useEffect, useState } from "react";

import type {
  GeneratedArtifactPreview,
  GeneratedArtifactSnapshot,
} from "../lib/advisorGeneratedArtifact";
import type { ReviewPaneData } from "./types";

export default function PreviewPane({
  loadArtifacts,
  previewArtifact,
}: ReviewPaneData) {
  const [artifacts, setArtifacts] = useState<GeneratedArtifactSnapshot | null>(
    null,
  );
  const [preview, setPreview] = useState<GeneratedArtifactPreview | null>(null);
  const [state, setState] = useState<"loading" | "ready" | "failure">(
    "loading",
  );
  useEffect(() => {
    let active = true;
    void loadArtifacts()
      .then((value) => {
        if (active) {
          setArtifacts(value);
          setState("ready");
        }
      })
      .catch(() => {
        if (active) setState("failure");
      });
    return () => {
      active = false;
    };
  }, [loadArtifacts]);
  if (state === "loading")
    return <p role="status">Loading safe generated-artifact evidence…</p>;
  if (state === "failure")
    return <p role="status">Preview evidence is unavailable.</p>;
  if (!artifacts?.artifacts.length)
    return (
      <p role="status">No generated artifacts are available for preview.</p>
    );
  return (
    <div className="review-pane-list">
      <ul>
        {artifacts.artifacts.map((artifact) => (
          <li key={artifact.artifactId}>
            <button
              type="button"
              onClick={() =>
                void previewArtifact({
                  artifactId: artifact.artifactId,
                  manifestSha256: artifact.sha256,
                })
                  .then(setPreview)
                  .catch(() => setPreview(null))
              }
            >
              {artifact.displayLabel}
            </button>{" "}
            · {artifact.byteSize} bytes
          </li>
        ))}
      </ul>
      {preview ? (
        <pre>
          <code>{preview.text}</code>
        </pre>
      ) : (
        <p>Select an artifact to request its bounded text preview.</p>
      )}
    </div>
  );
}
