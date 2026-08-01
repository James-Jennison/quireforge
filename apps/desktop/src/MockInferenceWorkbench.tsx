import { useEffect, useId, useRef, useState } from "react";

import {
  authorizeMockInference,
  cancelMockInference,
  loadMockInferenceCatalog,
  loadTaskCatalog,
  pollMockInference,
  prepareMockInference,
  submitMockInference,
} from "./lib/bridge";
import type {
  MockInferenceCatalog,
  MockInferenceSnapshot,
} from "./lib/mockInference";
import type { TaskCatalogSnapshot } from "./lib/taskRecords";

type Operations = {
  catalog: () => Promise<MockInferenceCatalog>;
  tasks: () => Promise<TaskCatalogSnapshot>;
  prepare: (request: {
    taskId: string;
    profileId: string;
    input: string;
  }) => Promise<MockInferenceSnapshot>;
  authorize: (request: {
    taskId: string;
    attemptId: string;
    authorizationId: string;
  }) => Promise<MockInferenceSnapshot>;
  submit: (request: {
    taskId: string;
    attemptId: string;
    authorizationId: string;
  }) => Promise<MockInferenceSnapshot>;
  cancel: (request: {
    taskId: string;
    attemptId: string;
  }) => Promise<MockInferenceSnapshot>;
  poll: (request: {
    taskId: string;
    attemptId: string;
  }) => Promise<MockInferenceSnapshot>;
};

const nativeOperations: Operations = {
  catalog: loadMockInferenceCatalog,
  tasks: () =>
    loadTaskCatalog({
      query: null,
      includeArchived: false,
      selectedTaskId: null,
    }),
  prepare: prepareMockInference,
  authorize: authorizeMockInference,
  submit: submitMockInference,
  cancel: cancelMockInference,
  poll: pollMockInference,
};

function diagnostic(snapshot: MockInferenceSnapshot | null) {
  if (!snapshot?.diagnostic) return null;
  return `Mock inference did not proceed: ${snapshot.diagnostic.replaceAll("-", " ")}. Your authored input remains local.`;
}

export function MockInferenceWorkbench({
  onClose,
  operations = nativeOperations,
}: {
  onClose: () => void;
  operations?: Operations;
}) {
  const titleId = useId();
  const closeRef = useRef<HTMLButtonElement>(null);
  const [catalog, setCatalog] = useState<MockInferenceCatalog | null>(null);
  const [tasks, setTasks] = useState<TaskCatalogSnapshot | null>(null);
  const [taskId, setTaskId] = useState("");
  const [profileId, setProfileId] = useState("");
  const [input, setInput] = useState("");
  const [snapshot, setSnapshot] = useState<MockInferenceSnapshot | null>(null);
  const [priorAttempt, setPriorAttempt] =
    useState<MockInferenceSnapshot | null>(null);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState(
    "Loading fictional local mock fixtures…",
  );

  useEffect(() => {
    closeRef.current?.focus();
  }, []);

  useEffect(() => {
    let active = true;
    void Promise.all([operations.catalog(), operations.tasks()])
      .then(([nextCatalog, nextTasks]) => {
        if (!active) return;
        setCatalog(nextCatalog);
        setTasks(nextTasks);
        setTaskId(nextTasks.selectedTask?.id ?? nextTasks.tasks[0]?.id ?? "");
        setProfileId(nextCatalog.profiles[0]?.id ?? "");
        setNotice(
          "Fictional local mock inference is ready for an explicit review.",
        );
      })
      .catch(
        () => active && setNotice("Mock inference fixtures are unavailable."),
      );
    return () => {
      active = false;
    };
  }, [operations]);

  const apply = async (action: () => Promise<MockInferenceSnapshot>) => {
    setBusy(true);
    try {
      const next = await action();
      setSnapshot(next);
      setNotice(
        diagnostic(next) ??
          `Mock attempt is ${next.state.replaceAll("-", " ")}.`,
      );
    } catch {
      setNotice("Mock inference is unavailable; no provider action occurred.");
    } finally {
      setBusy(false);
    }
  };
  const attemptId = snapshot?.attemptId;
  const authorizationId = snapshot?.authorization?.id;
  const canSubmit =
    snapshot?.state === "authorized" && attemptId && authorizationId;
  const canAuthorize =
    snapshot?.state === "ready" && attemptId && authorizationId;
  const canCancel =
    snapshot?.state === "submitted" || snapshot?.state === "streaming";
  const canPoll =
    snapshot?.state === "submitted" ||
    snapshot?.state === "streaming" ||
    snapshot?.state === "cancelling";
  const activeAttempt =
    snapshot?.state === "submitted" ||
    snapshot?.state === "streaming" ||
    snapshot?.state === "cancelling";
  const invalidateVisibleReview = () => {
    if (snapshot)
      setNotice(
        "The reviewed binding changed. Prepare a fresh local mock review.",
      );
    setSnapshot(null);
  };

  return (
    <section className="mock-inference-workbench" aria-labelledby={titleId}>
      <header className="task-template-workbench__header">
        <div>
          <p className="eyebrow">Local fixture</p>
          <h2 id={titleId}>Fictional mock inference</h2>
          <p>
            Private, deterministic mock behavior only. No provider, account,
            network, or credential is used.
          </p>
        </div>
        <button ref={closeRef} type="button" onClick={onClose}>
          Close
        </button>
      </header>
      <p role="status" className="context-note">
        {notice}
      </p>
      <form
        className="mock-inference-workbench__form"
        onSubmit={(event) => {
          event.preventDefault();
          if (!taskId || !profileId) return;
          void apply(() => operations.prepare({ taskId, profileId, input }));
        }}
      >
        <label>
          Durable task
          <select
            value={taskId}
            onChange={(event) => {
              setTaskId(event.target.value);
              invalidateVisibleReview();
            }}
            disabled={busy || activeAttempt || !tasks?.tasks.length}
          >
            {tasks?.tasks.map((task) => (
              <option key={task.id} value={task.id}>
                {task.title}
              </option>
            ))}
          </select>
        </label>
        <label>
          Fictional destination
          <select
            value={profileId}
            onChange={(event) => {
              setProfileId(event.target.value);
              invalidateVisibleReview();
            }}
            disabled={busy || activeAttempt || !catalog}
          >
            {catalog?.profiles.map((profile) => (
              <option key={profile.id} value={profile.id}>
                {profile.providerLabel} · {profile.modelLabel} ·{" "}
                {profile.scenario}
              </option>
            ))}
          </select>
        </label>
        <label>
          Bounded authored input
          <textarea
            value={input}
            onChange={(event) => {
              setInput(event.target.value);
              invalidateVisibleReview();
            }}
            maxLength={2000}
            rows={5}
            disabled={busy || activeAttempt}
            aria-describedby="mock-inference-input-note"
          />
        </label>
        <p id="mock-inference-input-note" className="context-note">
          Only this visible text is selected. Files, repositories, sessions,
          credentials, retrieval, and browser state are excluded.
        </p>
        <button
          type="submit"
          disabled={busy || !taskId || !profileId || !input.trim()}
        >
          Prepare local mock review
        </button>
      </form>
      {snapshot?.destination && (
        <section
          className="mock-inference-workbench__details"
          aria-label="Mock inference review"
        >
          <h3>Exact local review</h3>
          <dl>
            <dt>Attempt</dt>
            <dd>{snapshot.attemptId}</dd>
            <dt>Destination digest</dt>
            <dd>{snapshot.destination.descriptorSha256}</dd>
            <dt>Context manifest</dt>
            <dd>{snapshot.manifest?.sha256}</dd>
            <dt>Input digest</dt>
            <dd>{snapshot.manifest?.inputSha256}</dd>
            <dt>Lease</dt>
            <dd>{snapshot.lease?.state} (fictional opaque reference)</dd>
            <dt>Authorization</dt>
            <dd>{snapshot.authorization?.bindingSha256}</dd>
          </dl>
          <p className="context-note">
            Exclusions: {snapshot.manifest?.exclusions.join(", ")}
          </p>
          <div className="mock-inference-workbench__actions">
            <button
              type="button"
              disabled={busy || !canAuthorize}
              onClick={() =>
                attemptId &&
                authorizationId &&
                void apply(() =>
                  operations.authorize({ taskId, attemptId, authorizationId }),
                )
              }
            >
              Authorize one local mock submission
            </button>
            {canPoll && (
              <button
                type="button"
                disabled={busy || !attemptId}
                onClick={() =>
                  attemptId &&
                  void apply(() => operations.poll({ taskId, attemptId }))
                }
              >
                Continue bounded local fixture stream
              </button>
            )}
            <button
              type="button"
              disabled={busy || !canSubmit}
              onClick={() =>
                attemptId &&
                authorizationId &&
                void apply(() =>
                  operations.submit({ taskId, attemptId, authorizationId }),
                )
              }
            >
              Submit deterministic mock
            </button>
            {canCancel && (
              <button
                type="button"
                disabled={busy || !attemptId}
                onClick={() =>
                  attemptId &&
                  void apply(() => operations.cancel({ taskId, attemptId }))
                }
              >
                Cancel
              </button>
            )}
            <button
              type="button"
              disabled={busy || !attemptId}
              onClick={() => {
                setPriorAttempt(snapshot);
                setSnapshot(null);
                setNotice(
                  "Retry requires a newly authored input and a fresh mock review.",
                );
              }}
            >
              Prepare fresh retry or regeneration
            </button>
          </div>
        </section>
      )}
      {snapshot && (
        <section
          className="mock-inference-workbench__details"
          aria-label="Mock interaction evidence"
        >
          <h3>Canonical mock interaction events</h3>
          <ol>
            {snapshot.events.map((event) => (
              <li key={event.id}>
                <strong>
                  {event.sequence}. {event.kind}
                </strong>
                {event.text ? ` — ${event.text}` : ""}
              </li>
            ))}
          </ol>
          {snapshot.usage && (
            <p>
              Fictional reported usage:{" "}
              {snapshot.usage.units
                .map((unit) => `${unit.quantity} ${unit.unit}`)
                .join(", ")}
              .
            </p>
          )}
          <p className="context-note">
            Evidence is content-free, local mock metadata. Provider session
            references are subordinate to this durable task.
          </p>
        </section>
      )}
      {priorAttempt && !snapshot && (
        <section
          className="mock-inference-workbench__details"
          aria-label="Prior mock attempt evidence"
        >
          <h3>Prior local attempt</h3>
          <p>
            {priorAttempt.attemptId} ended as {priorAttempt.state}. A fresh
            review is required; no lease, authorization, event sequence, or
            result is reused.
          </p>
        </section>
      )}
    </section>
  );
}
