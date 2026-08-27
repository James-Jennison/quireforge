import { useEffect, useId, useRef, useState } from "react";

import {
  activateObjectiveAuthority,
  createObjectiveAuthority,
  loadObjectiveAuthority,
  revokeObjectiveAuthority,
} from "./lib/bridge";
import {
  objectiveAuthorityLaneSchema,
  type ObjectiveAuthoritySnapshot,
} from "./lib/objectiveAuthority";

const lanes = objectiveAuthorityLaneSchema.options;

export function ObjectiveAuthorityWorkbench({
  projectId,
  onClose,
}: {
  projectId: string | null;
  onClose: () => void;
}) {
  const titleId = useId();
  const closeRef = useRef<HTMLButtonElement>(null);
  const [snapshot, setSnapshot] = useState<ObjectiveAuthoritySnapshot | null>(
    null,
  );
  const [title, setTitle] = useState("");
  const [objective, setObjective] = useState("");
  const [lanesSelected, setLanesSelected] = useState<string[]>([
    "work-with-code",
  ]);
  const [confirmationLanes, setConfirmationLanes] = useState<string[]>([]);
  const [minutes, setMinutes] = useState(60);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState("Loading objective authority…");

  useEffect(() => {
    closeRef.current?.focus();
    if (!projectId) return;
    void Promise.resolve()
      .then(() => loadObjectiveAuthority({ projectId }))
      .then((next) => {
        setSnapshot(next);
        setNotice(
          "Objectives are proposals until you explicitly activate them. No capability has started.",
        );
      })
      .catch(() =>
        setNotice(
          "Objective authority is unavailable; no capability has started.",
        ),
      );
  }, [projectId]);
  const apply = async (action: () => Promise<ObjectiveAuthoritySnapshot>) => {
    setBusy(true);
    try {
      const next = await action();
      setSnapshot(next);
      setNotice(
        next.diagnosticCode ??
          "Objective authority updated. No capability has started.",
      );
    } catch {
      setNotice(
        "Objective authority is unavailable; no capability has started.",
      );
    } finally {
      setBusy(false);
    }
  };
  const toggleLane = (lane: string) => {
    const wasSelected = lanesSelected.includes(lane);
    setLanesSelected((current) =>
      wasSelected
        ? current.filter((value) => value !== lane)
        : [...current, lane],
    );
    if (wasSelected) {
      setConfirmationLanes((current) =>
        current.filter((value) => value !== lane),
      );
    }
  };
  const toggleConfirmation = (lane: string) =>
    setConfirmationLanes((current) =>
      current.includes(lane)
        ? current.filter((value) => value !== lane)
        : [...current, lane],
    );
  return (
    <section
      className="mock-inference-workbench"
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
    >
      <header className="mock-inference-workbench__header">
        <div>
          <p className="eyebrow">Objective-scoped authority</p>
          <h2 id={titleId}>Authority objectives</h2>
        </div>
        <button ref={closeRef} type="button" onClick={onClose}>
          Close
        </button>
      </header>
      <p role="status">{notice}</p>
      <p>
        Objectives are project-bound, expire automatically, and can be revoked.
        They do not open a browser, run an agent, or perform an external action.
      </p>
      <label>
        Title
        <input
          value={title}
          onChange={(event) => setTitle(event.target.value)}
          maxLength={240}
        />
      </label>
      <label>
        Objective
        <textarea
          value={objective}
          onChange={(event) => setObjective(event.target.value)}
          maxLength={8192}
          rows={3}
        />
      </label>
      <fieldset>
        <legend>Allowed future lanes</legend>
        {lanes.map((lane) => (
          <label key={lane}>
            <input
              type="checkbox"
              checked={lanesSelected.includes(lane)}
              onChange={() => toggleLane(lane)}
            />{" "}
            {lane}
            <input
              type="checkbox"
              checked={confirmationLanes.includes(lane)}
              disabled={!lanesSelected.includes(lane)}
              onChange={() => toggleConfirmation(lane)}
            />{" "}
            require confirmation
          </label>
        ))}
      </fieldset>
      <label>
        Expiry in minutes
        <input
          type="number"
          min={1}
          max={10080}
          value={minutes}
          onChange={(event) => setMinutes(Number(event.target.value))}
        />
      </label>
      <button
        type="button"
        disabled={
          !projectId ||
          busy ||
          !title.trim() ||
          !objective.trim() ||
          !lanesSelected.length
        }
        onClick={() =>
          void apply(() =>
            createObjectiveAuthority({
              projectId,
              title,
              objective,
              allowedLanes: lanesSelected,
              confirmationRequiredLanes: confirmationLanes,
              expiresInMinutes: minutes,
            }),
          )
        }
      >
        Create draft objective
      </button>
      <div className="action-card">
        {snapshot?.objectives.map((item) => (
          <article key={item.id}>
            <h3>{item.title}</h3>
            <p>{item.objective}</p>
            <p>
              {item.state} · expires{" "}
              {new Date(item.expiresAtMs).toLocaleString()}
            </p>
            <p>{item.allowedLanes.join(", ")}</p>
            <button
              type="button"
              disabled={busy || item.state !== "draft"}
              onClick={() =>
                void apply(() =>
                  activateObjectiveAuthority({ objectiveId: item.id }),
                )
              }
            >
              Activate
            </button>
            <button
              type="button"
              disabled={busy || !["draft", "active"].includes(item.state)}
              onClick={() =>
                void apply(() =>
                  revokeObjectiveAuthority({ objectiveId: item.id }),
                )
              }
            >
              Revoke
            </button>
          </article>
        ))}
      </div>
    </section>
  );
}
