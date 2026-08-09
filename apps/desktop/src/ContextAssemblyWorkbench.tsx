import { useEffect, useId, useRef, useState } from "react";
import {
  cancelContextAssembly,
  acknowledgeContextAssemblyReview,
  runContextAssemblyLocalRuntime,
  reviewContextAssembly,
  prepareContextAssembly,
  revokeContextAssembly,
  loadDurableSources,
  loadTaskCatalog,
  loadLocalReview,
} from "./lib/bridge";
import type {
  ContextAssemblySnapshot,
  LocalRuntimeSnapshot,
} from "./lib/contextAssembly";
import type { DurableSourceSummary } from "./lib/durableSources";
import type { TaskCatalogSnapshot } from "./lib/taskRecords";
import type { LocalReviewSnapshot } from "./lib/localReview";
export function ContextAssemblyWorkbench({
  projectId,
  projectLabel,
  onClose,
}: {
  projectId: string | null;
  projectLabel?: string | null;
  onClose: () => void;
}) {
  return (
    <ContextAssemblyWorkbenchScope
      key={projectId ?? "no-project"}
      projectId={projectId}
      projectLabel={projectLabel}
      onClose={onClose}
    />
  );
}

function ContextAssemblyWorkbenchScope({
  projectId,
  projectLabel,
  onClose,
}: {
  projectId: string | null;
  projectLabel?: string | null;
  onClose: () => void;
}) {
  const title = useId(),
    close = useRef<HTMLButtonElement>(null),
    [text, setText] = useState(""),
    [sources, setSources] = useState<DurableSourceSummary[]>([]),
    [tasks, setTasks] = useState<TaskCatalogSnapshot | null>(null),
    [taskId, setTaskId] = useState<string | null>(null),
    [includePlan, setIncludePlan] = useState(false),
    [review, setReview] = useState<LocalReviewSnapshot | null>(null),
    [reviewCollectionId, setReviewCollectionId] = useState<string | null>(null),
    [reviewEvidenceIds, setReviewEvidenceIds] = useState<string[]>([]),
    [includeScopeMetadata, setIncludeScopeMetadata] = useState(false),
    [selectedSources, setSelectedSources] = useState<string[]>([]),
    [snapshot, setSnapshot] = useState<ContextAssemblySnapshot | null>(null),
    [runtime, setRuntime] = useState<LocalRuntimeSnapshot | null>(null),
    [runtimeRunning, setRuntimeRunning] = useState(false),
    [busy, setBusy] = useState(false),
    [notice, setNotice] = useState(
      "Local-only reviewed context. Nothing is selected by default.",
    );
  const mounted = useRef(true);
  const projectScope = useRef(projectId);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  useEffect(() => {
    let current = true;
    projectScope.current = projectId;
    if (!projectId)
      return () => {
        current = false;
      };
    void loadDurableSources({ projectId })
      .then((value) => {
        if (current) setSources(value.sources);
      })
      .catch(() => {
        if (current) setSources([]);
      });
    void loadTaskCatalog({
      projectId,
      query: null,
      includeArchived: false,
      selectedTaskId: null,
    })
      .then((value) => {
        if (current) setTasks(value);
      })
      .catch(() => {
        if (current) setTasks(null);
      });
    return () => {
      current = false;
    };
  }, [projectId]);
  useEffect(() => {
    let current = true;
    if (!taskId) {
      return () => {
        current = false;
      };
    }
    void loadLocalReview({ selectedCollectionId: null })
      .then((value) => {
        if (current) setReview(value);
      })
      .catch(() => {
        if (current) setReview(null);
      });
    return () => {
      current = false;
    };
  }, [taskId]);
  useEffect(() => {
    let current = true;
    if (!reviewCollectionId) {
      return () => {
        current = false;
      };
    }
    void loadLocalReview({ selectedCollectionId: reviewCollectionId })
      .then((value) => {
        if (current) setReview(value);
      })
      .catch(() => {
        if (current) setReview(null);
      });
    return () => {
      current = false;
    };
  }, [reviewCollectionId]);
  const run = async (action: () => Promise<ContextAssemblySnapshot>) => {
    const actionProjectId = projectId;
    setBusy(true);
    try {
      const next = await action();
      if (mounted.current && projectScope.current === actionProjectId) {
        setSnapshot(next);
        setNotice(next.diagnostic ?? next.auditState);
      }
    } catch {
      if (mounted.current && projectScope.current === actionProjectId) {
        setNotice("Context assembly is unavailable; no dispatch occurred.");
      }
    } finally {
      if (mounted.current && projectScope.current === actionProjectId) {
        setBusy(false);
      }
    }
  };
  const canConfirm =
    snapshot?.state === "awaiting_confirmation" &&
    snapshot.bundleId &&
    snapshot.authorizationId &&
    snapshot.bundleDigest;
  const invalidatePreparedBundle = () => {
    setSnapshot(null);
    setRuntime(null);
    setRuntimeRunning(false);
    setNotice("Selection changed. Prepare a new local-only review.");
  };
  return (
    <section
      className="mock-inference-workbench"
      role="dialog"
      aria-modal="true"
      aria-labelledby={title}
    >
      <header className="task-template-workbench__header">
        <div>
          <p className="eyebrow">Credential-free local runtime</p>
          <h2 id={title}>Governed context review</h2>
        </div>
        <button ref={close} type="button" onClick={onClose}>
          Close
        </button>
      </header>
      <p role="status">{notice}</p>
      <p>
        Project scope: {projectLabel ?? "Current attached project"}. This is not
        provider transmission, credential, connector, browser, MCP, automation,
        or mutation authority. A confirmed review can run once in this open
        local view only.
      </p>
      <label>
        Explicit user instruction
        <textarea
          value={text}
          maxLength={8192}
          rows={5}
          disabled={busy}
          onChange={(e) => {
            setText(e.target.value);
            invalidatePreparedBundle();
          }}
        />
      </label>
      <p className="context-note">
        Source content is resolved only by native app-private storage. No source
        bytes can be supplied by this browser UI, and no source is selected by
        default.
      </p>
      <label>
        <input
          type="checkbox"
          checked={includeScopeMetadata}
          disabled={busy}
          onChange={(event) => {
            setIncludeScopeMetadata(event.target.checked);
            invalidatePreparedBundle();
          }}
        />
        Include bounded project/task metadata as untrusted evidence
      </label>
      <label>
        Optional task scope
        <select
          value={taskId ?? ""}
          disabled={busy}
          onChange={(event) => {
            setTaskId(event.target.value || null);
            setIncludePlan(false);
            setReview(null);
            setReviewCollectionId(null);
            setReviewEvidenceIds([]);
            invalidatePreparedBundle();
          }}
        >
          <option value="">Project only</option>
          {tasks?.tasks.map((task) => (
            <option key={task.id} value={task.id}>
              {task.title}
            </option>
          ))}
        </select>
      </label>
      {taskId && (
        <label>
          <input
            type="checkbox"
            checked={includePlan}
            disabled={busy}
            onChange={(event) => {
              setIncludePlan(event.target.checked);
              invalidatePreparedBundle();
            }}
          />
          Include the selected task plan as untrusted plan evidence
        </label>
      )}
      {taskId && (
        <fieldset disabled={busy}>
          <legend>Explicit local-review evidence selection</legend>
          <select
            value={reviewCollectionId ?? ""}
            onChange={(event) => {
              setReviewCollectionId(event.target.value || null);
              setReviewEvidenceIds([]);
              invalidatePreparedBundle();
            }}
          >
            <option value="">No review evidence</option>
            {review?.collections
              .filter((collection) => collection.taskId === taskId)
              .map((collection) => (
                <option
                  key={collection.collectionId}
                  value={collection.collectionId}
                >
                  {collection.title}
                </option>
              ))}
          </select>
          {review?.selectedCollection?.taskId === taskId &&
            review.items
              .filter((item) => item.class === "evidence")
              .map((item) => (
                <label key={item.itemId}>
                  <input
                    type="checkbox"
                    checked={reviewEvidenceIds.includes(item.itemId)}
                    onChange={() => {
                      setReviewEvidenceIds((current) =>
                        current.includes(item.itemId)
                          ? current.filter((id) => id !== item.itemId)
                          : [...current, item.itemId],
                      );
                      invalidatePreparedBundle();
                    }}
                  />
                  {item.title}
                </label>
              ))}
        </fieldset>
      )}
      <fieldset disabled={busy}>
        <legend>Explicit durable-source selection</legend>
        {sources.length ? (
          sources.map((source) => (
            <label key={source.sourceId}>
              <input
                type="checkbox"
                checked={selectedSources.includes(source.sourceId)}
                onChange={() => {
                  setSelectedSources((current) =>
                    current.includes(source.sourceId)
                      ? current.filter((value) => value !== source.sourceId)
                      : [...current, source.sourceId],
                  );
                  invalidatePreparedBundle();
                }}
              />
              {source.title} · {source.sourceClass} · {source.byteSize} bytes
            </label>
          ))
        ) : (
          <p>No active M55 sources are available; none will be included.</p>
        )}
      </fieldset>
      <div className="mock-inference-workbench__actions">
        <button
          disabled={snapshot?.state !== "prepared" || busy}
          onClick={() =>
            void run(() =>
              reviewContextAssembly({ bundleId: snapshot!.bundleId }),
            )
          }
        >
          Review prepared bundle
        </button>
        <button
          disabled={snapshot?.state !== "awaiting_review" || busy}
          onClick={() =>
            void run(() =>
              acknowledgeContextAssemblyReview({
                bundleId: snapshot!.bundleId,
              }),
            )
          }
        >
          Acknowledge exact review
        </button>
        <button
          disabled={
            !projectId ||
            (!text &&
              selectedSources.length === 0 &&
              reviewEvidenceIds.length === 0 &&
              !includeScopeMetadata &&
              !includePlan) ||
            busy
          }
          onClick={() =>
            void run(() =>
              prepareContextAssembly({
                projectId,
                taskId,
                userInstruction: text,
                durableSourceIds: selectedSources,
                selectedPlanId:
                  includePlan && taskId
                    ? (tasks?.tasks.find((task) => task.id === taskId)
                        ?.selectedPlanId ?? null)
                    : null,
                reviewEvidenceIds,
                includeScopeMetadata,
              }),
            )
          }
        >
          Prepare review
        </button>
        <button
          disabled={!canConfirm || busy}
          onClick={() => {
            const actionProjectId = projectId;
            setRuntime(null);
            setRuntimeRunning(true);
            setNotice(
              "One local-only CPU attempt is running. It has a fixed deadline and no automatic retry.",
            );
            setBusy(true);
            void runContextAssemblyLocalRuntime({
              bundleId: snapshot!.bundleId,
              authorizationId: snapshot!.authorizationId,
              bundleDigest: snapshot!.bundleDigest,
            })
              .then((next) => {
                if (
                  mounted.current &&
                  projectScope.current === actionProjectId
                ) {
                  setRuntime(next);
                  setNotice(
                    next.diagnostic ??
                      "Local-only runtime completed. Output stays in this open view.",
                  );
                }
              })
              .catch(() => {
                if (
                  mounted.current &&
                  projectScope.current === actionProjectId
                ) {
                  setRuntimeRunning(false);
                  setNotice("Local runtime is unavailable; no retry occurred.");
                }
              })
              .finally(() => {
                if (
                  mounted.current &&
                  projectScope.current === actionProjectId
                ) {
                  setRuntimeRunning(false);
                  setBusy(false);
                }
              });
          }}
        >
          Run once with local-only model
        </button>
        <button
          disabled={!snapshot?.bundleId || busy}
          onClick={() =>
            void run(() =>
              cancelContextAssembly({ bundleId: snapshot!.bundleId }),
            )
          }
        >
          Cancel
        </button>
        <button
          disabled={!snapshot?.bundleId || busy}
          onClick={() =>
            void run(() =>
              revokeContextAssembly({ bundleId: snapshot!.bundleId }),
            )
          }
        >
          Revoke
        </button>
      </div>
      {snapshot && (
        <section aria-label="Prepared context summary">
          <p>
            Items: {snapshot.items.length}; total: {snapshot.totalBytes} bytes;
            estimated tokens: {snapshot.estimatedTokens}.
          </p>
          <ul>
            {snapshot.items.map((item, index) => (
              <li key={`${item.digest}-${index}`}>
                {item.ordinal + 1}. {item.sourceClass} · {item.provenance} ·{" "}
                {item.byteSize} bytes
                {item.redactionCount
                  ? ` · ${item.redactionCount} redactions`
                  : ""}
                {item.truncated ? " · truncated" : ""}
              </li>
            ))}
          </ul>
        </section>
      )}
      {(runtimeRunning || runtime) && (
        <section aria-label="Local runtime result">
          <p>
            Local-only attempt: {runtimeRunning ? "running" : runtime!.state}.{" "}
            CPU-only; one attempt; maximum {runtime?.inputTokenLimit ?? 4096}{" "}
            input tokens, {runtime?.outputTokenLimit ?? 512} output tokens, and{" "}
            {runtime?.deadlineSeconds ?? 60} seconds.
          </p>
          {runtimeRunning && (
            <p className="context-note">
              This view is waiting for the one bounded result. Closing it does
              not authorize another attempt or retain a result.
            </p>
          )}
          {runtime?.output && <pre>{runtime.output}</pre>}
        </section>
      )}
    </section>
  );
}
