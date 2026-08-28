import { useEffect, useId, useRef, useState } from "react";

import {
  activateObjectiveAuthority,
  createObjectiveAuthority,
  loadObjectiveAuthority,
  revokeObjectiveAuthority,
} from "./lib/bridge";
import type { ObjectiveAuthoritySnapshot } from "./lib/objectiveAuthority";

const scopeGroups = [
  {
    id: "project-work",
    label: "Work on this project",
    description: "Future code and project-work capability.",
    lanes: ["work-with-code"],
  },
  {
    id: "research-and-data",
    label: "Research and project data",
    description: "Future browser research and read-only connected data.",
    lanes: ["browser-workspace", "browser-observation", "connector-read"],
  },
  {
    id: "future-actions",
    label: "Future actions and services",
    description: "Future schedules, delivery, providers, and computer use.",
    lanes: [
      "scheduled-work",
      "connector-mutation",
      "provider-inference",
      "computer-use",
    ],
  },
] as const;

const scopeLaneLabels: Record<string, string> = {
  "work-with-code": "Work with project code",
  "browser-workspace": "Use a private browser workspace",
  "browser-observation": "Observe browser pages",
  "connector-read": "Read connected project data",
  "scheduled-work": "Schedule project work",
  "connector-mutation": "Change a connected service",
  "provider-inference": "Use an AI provider",
  "computer-use": "Use a computer",
};

function selectedGroupLanes(lanesSelected: string[], groupId: string) {
  return (
    scopeGroups
      .find((group) => group.id === groupId)
      ?.lanes.filter((lane) => lanesSelected.includes(lane)) ?? []
  );
}

function selectedScopeGroupLabels(lanes: string[]) {
  return scopeGroups
    .filter((group) => group.lanes.some((lane) => lanes.includes(lane)))
    .map((group) => group.label);
}

export function ObjectiveAuthorityWorkbench({
  projectId,
  projectName,
  onClose,
}: {
  projectId: string | null;
  projectName: string | null;
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
  const [flagForReview, setFlagForReview] = useState(false);
  const [minutes, setMinutes] = useState(60);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState("Loading authority objectives…");
  const [formNotice, setFormNotice] = useState<string | null>(null);

  useEffect(() => {
    closeRef.current?.focus();
    if (!projectId) return;
    void Promise.resolve()
      .then(() => loadObjectiveAuthority({ projectId }))
      .then((next) => {
        setSnapshot(next);
        setNotice(
          "Drafts record future scope only. No capability has started.",
        );
      })
      .catch(() =>
        setNotice(
          "Authority objectives are unavailable; no capability has started.",
        ),
      );
  }, [projectId]);

  useEffect(() => {
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [onClose]);

  const apply = async (
    action: () => Promise<ObjectiveAuthoritySnapshot>,
  ): Promise<boolean> => {
    setBusy(true);
    try {
      const next = await action();
      setSnapshot(next);
      setNotice(
        next.diagnosticCode ?? "Objective updated. No capability has started.",
      );
      return true;
    } catch {
      setNotice(
        "Authority objectives are unavailable; no capability has started.",
      );
      return false;
    } finally {
      setBusy(false);
    }
  };
  const toggleGroup = (groupId: string) => {
    const groupLanes = selectedGroupLanes(lanesSelected, groupId);
    const group = scopeGroups.find((candidate) => candidate.id === groupId);
    if (!group) return;
    setLanesSelected((current) =>
      groupLanes.length === group.lanes.length
        ? current.filter(
            (lane) => !(group.lanes as readonly string[]).includes(lane),
          )
        : [...new Set([...current, ...group.lanes])],
    );
  };

  if (!projectId)
    return (
      <section
        className="objective-authority-workbench"
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
      >
        <header className="objective-authority-workbench__header">
          <div>
            <p className="eyebrow">Project authority</p>
            <h2 id={titleId}>Choose a project first</h2>
          </div>
          <button ref={closeRef} type="button" onClick={onClose}>
            Close
          </button>
        </header>
        <p className="objective-authority__inert-notice" role="note">
          Objectives belong to one attached project. No authority form is
          available until a project is selected.
        </p>
      </section>
    );

  return (
    <section
      className="objective-authority-workbench"
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
    >
      <header className="objective-authority-workbench__header">
        <div>
          <p className="eyebrow">Project authority</p>
          <h2 id={titleId}>
            Authority objectives for {projectName ?? "this project"}
          </h2>
        </div>
        <button ref={closeRef} type="button" onClick={onClose}>
          Close
        </button>
      </header>
      <p role="status">{notice}</p>
      <p className="objective-authority__inert-notice" role="note">
        Scope choices describe future work only. They start nothing and never
        replace a future capability&apos;s own approval.
      </p>
      {snapshot?.objectives.length ? (
        <section
          className="objective-authority__lifecycle"
          aria-label="Existing authority objectives"
        >
          <h3>Current objectives</h3>
          {snapshot.objectives.map((item) => (
            <article key={item.id}>
              <div>
                <p className="eyebrow">{item.state}</p>
                <h4>{item.title}</h4>
                <p>{item.objective}</p>
                <p>Expires {new Date(item.expiresAtMs).toLocaleString()}</p>
              </div>
              <div className="objective-authority__lifecycle-actions">
                {item.state === "draft" && (
                  <button
                    type="button"
                    disabled={busy}
                    onClick={() =>
                      void apply(() =>
                        activateObjectiveAuthority({ objectiveId: item.id }),
                      )
                    }
                  >
                    Activate
                  </button>
                )}
                {["draft", "active"].includes(item.state) && (
                  <button
                    type="button"
                    disabled={busy}
                    onClick={() =>
                      void apply(() =>
                        revokeObjectiveAuthority({ objectiveId: item.id }),
                      )
                    }
                  >
                    Revoke
                  </button>
                )}
              </div>
              <details>
                <summary>Review future scope</summary>
                <p>
                  🔒 Every listed scope remains locked until its own capability
                  and approval flow exist.
                </p>
                <ul>
                  {selectedScopeGroupLabels(item.allowedLanes).map((label) => (
                    <li key={label}>{label}</li>
                  ))}
                </ul>
              </details>
            </article>
          ))}
        </section>
      ) : null}
      <form
        className="objective-authority__form"
        onSubmit={(event) => {
          event.preventDefault();
          if (
            busy ||
            !title.trim() ||
            !objective.trim() ||
            !lanesSelected.length
          )
            return;
          setFormNotice(null);
          void apply(() =>
            createObjectiveAuthority({
              projectId,
              title,
              objective,
              allowedLanes: lanesSelected,
              confirmationRequiredLanes: flagForReview ? lanesSelected : [],
              expiresInMinutes: minutes,
            }),
          ).then((created) => {
            if (!created) return;
            setTitle("");
            setObjective("");
            setFormNotice(
              "Draft created. It is listed under Current objectives above; no capability has started.",
            );
          });
        }}
      >
        <div>
          <p className="eyebrow">New objective</p>
          <h3>Describe the work you want to plan</h3>
        </div>
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
          <legend>Future scope</legend>
          <p>
            Choose broad areas to record for this project. 🔒 No capability
            executes from these choices.
          </p>
          {scopeGroups.map((group) => {
            const selected =
              selectedGroupLanes(lanesSelected, group.id).length ===
              group.lanes.length;
            return (
              <div className="objective-authority__scope-group" key={group.id}>
                <input
                  type="checkbox"
                  aria-label={group.label}
                  checked={selected}
                  onChange={() => toggleGroup(group.id)}
                />
                <span>
                  <strong>🔒 {group.label}</strong>
                  <small>{group.description}</small>
                </span>
                <details className="objective-authority__scope-details">
                  <summary>See included future scope</summary>
                  <ul>
                    {group.lanes.map((lane) => (
                      <li key={lane}>🔒 {scopeLaneLabels[lane]}</li>
                    ))}
                  </ul>
                </details>
              </div>
            );
          })}
        </fieldset>
        <div className="objective-authority__review-flag">
          <input
            type="checkbox"
            aria-label="Flag this scope for review when it becomes available"
            checked={flagForReview}
            onChange={(event) => setFlagForReview(event.target.checked)}
          />
          <span>
            <strong>
              Flag this scope for review when it becomes available
            </strong>
            <small>
              This only highlights a future Action Card; it never grants or
              lowers approval.
            </small>
          </span>
        </div>
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
          type="submit"
          disabled={
            busy || !title.trim() || !objective.trim() || !lanesSelected.length
          }
        >
          Create draft objective
        </button>
        {formNotice ? (
          <p className="objective-authority__form-notice" role="status">
            {formNotice}
          </p>
        ) : null}
      </form>
    </section>
  );
}
