import { useEffect, useRef, useState } from "react";

import {
  cancelDurableSource,
  confirmArtifactReference,
  confirmArtifactReferenceDeletion,
  confirmDurableSource,
  confirmDurableSourceDeletion,
  loadDurableSources,
  loadArtifactReferences,
  prepareArtifactReference,
  prepareArtifactReferenceDeletion,
  prepareDurableSourceArtifact,
  prepareDurableSourceDeletion,
  prepareDurableSourceFile,
  prepareDurableSourceManual,
} from "./lib/bridge";
import type {
  ArtifactReferencePreparation,
  ArtifactReferenceSnapshot,
} from "./lib/artifactReferences";
import type {
  DurableSourcePreparation,
  DurableSourceSnapshot,
} from "./lib/durableSources";

export function DurableSourcesWorkbench({
  projectId,
  onClose,
  surface = "code",
}: {
  projectId: string | null;
  onClose: () => void;
  surface?: "code" | "studio";
}) {
  const closeRef = useRef<HTMLButtonElement>(null);
  const [snapshot, setSnapshot] = useState<DurableSourceSnapshot | null>(null);
  const [references, setReferences] =
    useState<ArtifactReferenceSnapshot | null>(null);
  const [title, setTitle] = useState("");
  const [text, setText] = useState("");
  const [artifactId, setArtifactId] = useState("");
  const [artifactSha256, setArtifactSha256] = useState("");
  const [preparation, setPreparation] =
    useState<DurableSourcePreparation | null>(null);
  const [referencePreparation, setReferencePreparation] =
    useState<ArtifactReferencePreparation | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const refresh = async () => {
    if (!projectId) return;
    setSnapshot(await loadDurableSources({ projectId }));
    setReferences(await loadArtifactReferences({ projectId }));
  };
  useEffect(() => {
    if (!projectId) return;
    void Promise.all([
      loadDurableSources({ projectId }),
      loadArtifactReferences({ projectId }),
    ]).then(([sources, artifactReferences]) => {
      setSnapshot(sources);
      setReferences(artifactReferences);
    });
  }, [projectId]);
  const prepare = async (kind: "manual" | "file" | "artifact") => {
    if (!projectId || busy) return;
    setBusy(true);
    setError(null);
    try {
      const result =
        kind === "manual"
          ? await prepareDurableSourceManual({
              projectId,
              taskId: null,
              title,
              text,
            })
          : kind === "file"
            ? await prepareDurableSourceFile({ projectId, taskId: null, title })
            : await prepareDurableSourceArtifact({
                projectId,
                taskId: null,
                title,
                artifactId,
                artifactSha256,
              });
      if (result.diagnosticCode) setError(result.diagnosticCode);
      else setPreparation(result);
    } catch {
      setError("Admission preparation is unavailable.");
    } finally {
      setBusy(false);
    }
  };
  const prepareReference = async () => {
    if (!projectId || busy) return;
    setBusy(true);
    setError(null);
    try {
      const result = await prepareArtifactReference({
        projectId,
        taskId: null,
        artifactId,
        artifactSha256,
      });
      if (result.diagnosticCode) setError(result.diagnosticCode);
      else setReferencePreparation(result);
    } catch {
      setError("Artifact reference preparation is unavailable.");
    } finally {
      setBusy(false);
    }
  };
  const confirm = async () => {
    if (!preparation || busy) return;
    setBusy(true);
    try {
      await confirmDurableSource({
        preparationId: preparation.preparationId,
        nonce: preparation.nonce,
        sha256: preparation.sha256,
      });
      setPreparation(null);
      setTitle("");
      setText("");
      await refresh();
    } catch {
      setError(
        "The admission result is ambiguous. Refresh durable sources before manually trying again.",
      );
      setPreparation(null);
    } finally {
      setBusy(false);
    }
  };
  const studio = surface === "studio";
  return (
    <section
      className="task-template-workbench"
      aria-label={studio ? "Local Work Studio" : "Durable Sources"}
    >
      <header>
        <div>
          <p className="eyebrow">
            {studio ? "Local Work Studio" : "M55 local admission"}
          </p>
          <h2>{studio ? "Sources and artifacts" : "Durable Sources"}</h2>
        </div>
        <button ref={closeRef} type="button" onClick={onClose}>
          {studio ? "Close Studio" : "Close Durable Sources"}
        </button>
      </header>
      <p>
        {studio
          ? "Organize explicitly admitted local material. Durable source admission creates a private copy; it does not send material, include it in model context, retrieve it, or grant tool authority."
          : "Admission creates a private local copy. It does not send material to a provider, include it in model context, retrieve it, or grant any tool authority."}
      </p>
      {!projectId ? (
        <p role="status">
          Select an authoritative project to manage durable sources.
        </p>
      ) : (
        <>
          <div className="task-template-workbench__form">
            <label>
              Title
              <input
                value={title}
                maxLength={240}
                onChange={(event) => setTitle(event.target.value)}
              />
            </label>
            <label>
              Manual text
              <textarea
                value={text}
                maxLength={128 * 1024}
                onChange={(event) => setText(event.target.value)}
              />
            </label>
            <label>
              Reviewed M48 artifact ID
              <input
                value={artifactId}
                onChange={(event) => setArtifactId(event.target.value)}
              />
            </label>
            <label>
              Reviewed artifact SHA-256
              <input
                value={artifactSha256}
                onChange={(event) => setArtifactSha256(event.target.value)}
              />
            </label>
            <div>
              <button
                type="button"
                disabled={busy || !title.trim() || !text}
                onClick={() => void prepare("manual")}
              >
                Review manual text admission
              </button>
              <button
                type="button"
                disabled={busy || !title.trim()}
                onClick={() => void prepare("file")}
              >
                Choose local text file to review
              </button>
              <button
                type="button"
                disabled={
                  busy || !title.trim() || !artifactId || !artifactSha256
                }
                onClick={() => void prepare("artifact")}
              >
                Review selected artifact admission
              </button>
              <button
                type="button"
                disabled={busy || !artifactId || !artifactSha256}
                onClick={() => void prepareReference()}
              >
                Review artifact reference
              </button>
            </div>
          </div>
          {error && <p role="alert">{error}</p>}
          {preparation && (
            <dialog open aria-labelledby="durable-source-review-title">
              <h3 id="durable-source-review-title">
                Review durable source admission
              </h3>
              <dl>
                <dt>Title</dt>
                <dd>{preparation.title}</dd>
                <dt>Class</dt>
                <dd>{preparation.sourceClass}</dd>
                <dt>Size</dt>
                <dd>
                  {preparation.byteSize} bytes / {preparation.lineCount} lines
                </dd>
                <dt>SHA-256</dt>
                <dd>
                  <code>{preparation.sha256}</code>
                </dd>
              </dl>
              <pre>{preparation.preview}</pre>
              <p>
                This admits a private local copy only. It does not transmit or
                attach the source to a provider.
              </p>
              <button
                type="button"
                disabled={busy}
                onClick={() => void confirm()}
              >
                Confirm admission
              </button>
              <button
                type="button"
                disabled={busy}
                onClick={() =>
                  void (async () => {
                    setBusy(true);
                    try {
                      await cancelDurableSource({
                        preparationId: preparation.preparationId,
                        nonce: preparation.nonce,
                      });
                      setPreparation(null);
                    } catch {
                      setError("Admission cancellation is unavailable.");
                      setPreparation(null);
                    } finally {
                      setBusy(false);
                    }
                  })()
                }
              >
                Cancel
              </button>
            </dialog>
          )}
          {referencePreparation && (
            <dialog open aria-labelledby="artifact-reference-review-title">
              <h3 id="artifact-reference-review-title">
                Review artifact reference
              </h3>
              <dl>
                <dt>Label</dt>
                <dd>{referencePreparation.displayLabel}</dd>
                <dt>Class</dt>
                <dd>{referencePreparation.artifactClass}</dd>
                <dt>SHA-256</dt>
                <dd>
                  <code>{referencePreparation.artifactSha256}</code>
                </dd>
              </dl>
              <p>
                This records only an opaque ID, digest, class, label, and
                project/task binding. It retains no artifact content, path,
                filename, or preview. The original can expire independently.
              </p>
              <button
                type="button"
                disabled={busy}
                onClick={() =>
                  void (async () => {
                    setBusy(true);
                    try {
                      await confirmArtifactReference({
                        preparationId: referencePreparation.preparationId,
                        nonce: referencePreparation.nonce,
                        artifactSha256: referencePreparation.artifactSha256,
                      });
                      setReferencePreparation(null);
                      await refresh();
                    } catch {
                      setError(
                        "The artifact reference could not be confirmed. Refresh before trying again.",
                      );
                      setReferencePreparation(null);
                    } finally {
                      setBusy(false);
                    }
                  })()
                }
              >
                Confirm reference
              </button>
              <button
                type="button"
                disabled={busy}
                onClick={() => setReferencePreparation(null)}
              >
                Cancel
              </button>
            </dialog>
          )}
          <h3>Artifact references</h3>
          <div className="task-catalog__list" role="list">
            {references?.references.length ? (
              references.references.map((reference) => (
                <article key={reference.referenceId} role="listitem">
                  <strong>{reference.displayLabel}</strong>
                  <p>
                    {reference.artifactClass} · original{" "}
                    {reference.availability === "live"
                      ? "available"
                      : "unavailable"}
                  </p>
                  <code title={reference.artifactSha256}>
                    {reference.artifactSha256}
                  </code>
                  <button
                    type="button"
                    disabled={busy}
                    onClick={() =>
                      void (async () => {
                        try {
                          const deletion =
                            await prepareArtifactReferenceDeletion({
                              referenceId: reference.referenceId,
                            });
                          if (
                            !deletion.diagnosticCode &&
                            window.confirm(
                              `Delete reference to ${reference.displayLabel}? The original artifact is unchanged.`,
                            )
                          ) {
                            await confirmArtifactReferenceDeletion({
                              preparationId: deletion.preparationId,
                              nonce: deletion.nonce,
                              referenceId: reference.referenceId,
                            });
                            await refresh();
                          }
                        } catch {
                          setError("Reference deletion is unavailable.");
                        }
                      })()
                    }
                  >
                    Delete reference
                  </button>
                </article>
              ))
            ) : (
              <p role="status">
                No artifact references are recorded for this project.
              </p>
            )}
          </div>
          <h3>Active sources</h3>
          <div className="task-catalog__list" role="list">
            {snapshot?.sources.length ? (
              snapshot.sources.map((source) => (
                <article key={source.sourceId} role="listitem">
                  <strong>{source.title}</strong>
                  <p>
                    {source.sourceClass} · {source.byteSize} bytes ·{" "}
                    {source.lineCount} lines
                  </p>
                  <code title={source.sha256}>{source.sha256}</code>
                  <button
                    type="button"
                    onClick={() =>
                      void (async () => {
                        const deletion = await prepareDurableSourceDeletion({
                          sourceId: source.sourceId,
                        });
                        if (
                          !deletion.diagnosticCode &&
                          window.confirm(
                            `Delete ${source.title}? This removes the copied source.`,
                          )
                        ) {
                          await confirmDurableSourceDeletion({
                            preparationId: deletion.preparationId,
                            nonce: deletion.nonce,
                            sourceId: source.sourceId,
                          });
                          await refresh();
                        }
                      })()
                    }
                  >
                    Delete source
                  </button>
                </article>
              ))
            ) : (
              <p role="status">
                No active durable sources are admitted for this project.
              </p>
            )}
          </div>
        </>
      )}
    </section>
  );
}
