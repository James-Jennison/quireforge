import type {
  AdvisorSelectedProjectStateSnapshot,
  AdvisorWorkspaceSnapshot,
} from "./lib/advisorWorkspace";

interface AdvisorWorkspaceProps {
  availability: "checking" | "native" | "preview" | "error";
  snapshot: AdvisorWorkspaceSnapshot | null;
  selectedProjectState: AdvisorSelectedProjectStateSnapshot | null;
  selectionState: "idle" | "confirming" | "reading" | "error";
  canSelectProjectState: boolean;
  onRequestProjectState: () => void;
  onConfirmProjectState: () => void;
  onCancelProjectState: () => void;
  onRemoveProjectState: () => void;
}

export function AdvisorWorkspace({
  availability,
  snapshot,
  selectedProjectState,
  selectionState,
  canSelectProjectState,
  onRequestProjectState,
  onConfirmProjectState,
  onCancelProjectState,
  onRemoveProjectState,
}: AdvisorWorkspaceProps) {
  const empty =
    snapshot?.conversationCount === 0 &&
    snapshot.contextReferenceCount === 0 &&
    snapshot.proposalCount === 0;
  return (
    <section
      className="project-workspace"
      id="advisor"
      aria-labelledby="advisor-title"
    >
      <p className="eyebrow">Advisor</p>
      <h1 id="advisor-title" data-workspace-heading tabIndex={-1}>
        Reference-only planning, without execution.
      </h1>
      <p role="note">
        No project, model, approval, or dispatch capability. Prompts,
        transcripts, credentials, and paths are excluded.
      </p>
      {availability === "checking" && (
        <p role="status">Reading Advisor metadata.</p>
      )}
      {availability === "preview" && (
        <p>Browser preview cannot read Advisor metadata.</p>
      )}
      {availability === "error" && (
        <p className="project-message project-message--warning" role="alert">
          Advisor metadata could not be read; no state changed.
        </p>
      )}
      {availability === "native" && snapshot && (
        <div className="project-list">
          <dl className="context-facts">
            <div>
              <dt>Conversations</dt>
              <dd>{snapshot.conversationCount}</dd>
            </div>
            <div>
              <dt>Contexts</dt>
              <dd>{snapshot.contextReferenceCount}</dd>
            </div>
            <div>
              <dt>Proposals</dt>
              <dd>{snapshot.proposalCount}</dd>
            </div>
          </dl>
          {empty ? (
            <p className="project-message">No Advisor metadata yet.</p>
          ) : (
            <ul
              className="project-list"
              aria-label="Advisor metadata summaries"
            >
              {snapshot.contextSummaries.map(
                ({ kind, freshness, trust }, index) => (
                  <li className="project-message" key={`${kind}-${index}`}>
                    {kind}: {trust}, {freshness}
                  </li>
                ),
              )}
              {snapshot.proposalSummaries.map(({ state }, index) => (
                <li className="project-message" key={`${state}-${index}`}>
                  Proposal digest: {state}, explicit approval required.
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
      {availability === "native" && (
        <section
          className="project-card"
          aria-labelledby="advisor-source-title"
        >
          <h2 id="advisor-source-title">Selected Project State</h2>
          {selectedProjectState ? (
            <>
              <p>
                Temporary safe summary: {selectedProjectState.freshness},{" "}
                {selectedProjectState.worktree}. No project identity or source
                content is retained.
              </p>
              <button type="button" onClick={onRemoveProjectState}>
                Remove temporary snapshot
              </button>
            </>
          ) : (
            <>
              <p>
                Select one normalized local snapshot. Advisor does not browse
                repositories or retain it after restart.
              </p>
              <button
                type="button"
                disabled={
                  !canSelectProjectState || selectionState === "reading"
                }
                onClick={onRequestProjectState}
              >
                Select current Project State snapshot
              </button>
              {!canSelectProjectState && (
                <p className="project-message">
                  Select an attached project outside Advisor before choosing a
                  Project State source.
                </p>
              )}
            </>
          )}
          {selectionState === "confirming" && (
            <div
              className="project-confirmation"
              role="dialog"
              aria-modal="true"
              aria-label="Confirm Project State selection"
            >
              <p>
                Read one local Project State summary. No files, paths, images,
                remote refresh, or repository change is included.
              </p>
              <div className="project-actions">
                <button type="button" onClick={onConfirmProjectState}>
                  Confirm selection
                </button>
                <button type="button" onClick={onCancelProjectState}>
                  Cancel
                </button>
              </div>
            </div>
          )}
          {selectionState === "reading" && (
            <p role="status">Reading the selected local snapshot.</p>
          )}
          {selectionState === "error" && (
            <p
              className="project-message project-message--warning"
              role="alert"
            >
              The selected snapshot could not be read; no context was retained.
            </p>
          )}
        </section>
      )}
    </section>
  );
}
