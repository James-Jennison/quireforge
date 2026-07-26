import type { RepositoryStateReadSnapshot } from "./lib/repositoryState";

type ProjectStateAvailability =
  "idle" | "checking" | "native" | "preview" | "error";

interface ProjectStateWorkspaceProps {
  availability: ProjectStateAvailability;
  projectName: string | null;
  snapshot: RepositoryStateReadSnapshot | null;
  busy: boolean;
  onRefresh: () => void;
}

function shortCommit(commit: string | null) {
  return commit ? commit.slice(0, 12) : "Unknown";
}

function freshnessLabel(value: string) {
  return value.replace("-", " ");
}

export function ProjectStateWorkspace({
  availability,
  projectName,
  snapshot,
  busy,
  onRefresh,
}: ProjectStateWorkspaceProps) {
  const repository = snapshot?.state.repository;
  const milestone = snapshot?.state.milestone;
  const worktreeChanges = snapshot
    ? snapshot.git.stagedCount +
      snapshot.git.unstagedCount +
      snapshot.git.untrackedCount
    : 0;

  return (
    <section
      className="project-workspace project-state-workspace"
      id="project-state"
      aria-labelledby="project-state-title"
    >
      <div className="project-workspace__heading">
        <div>
          <p className="eyebrow">Verified project evidence</p>
          <h1 id="project-state-title" data-workspace-heading tabIndex={-1}>
            Project state, without automation.
          </h1>
          <p>
            Inspect the normalized repository snapshot QuireForge already knows.
            This workspace does not fetch, edit, approve, or resolve anything.
          </p>
        </div>
        <button
          className="auth-button"
          type="button"
          disabled={!projectName || busy || availability === "preview"}
          onClick={onRefresh}
        >
          Refresh local evidence
        </button>
      </div>

      <div className="project-workspace__status" aria-live="polite">
        {availability === "idle" && (
          <p>Select an attached project to inspect its normalized state.</p>
        )}
        {availability === "checking" && (
          <p role="status">Reading local repository evidence.</p>
        )}
        {availability === "preview" && (
          <p>
            Browser preview cannot read an attached native repository. No
            example state is substituted.
          </p>
        )}
        {availability === "error" && (
          <p className="project-message project-message--warning" role="alert">
            Local project state could not be read. No repository data was
            changed.
          </p>
        )}
      </div>

      {availability === "native" && snapshot && repository && milestone && (
        <div className="project-list">
          <article className="project-card">
            <div className="project-card__heading">
              <div>
                <span className="project-kicker">Current snapshot</span>
                <h2>{projectName ?? snapshot.state.project.displayName}</h2>
              </div>
              <span className="directory-state">
                {repository.worktree === "clean"
                  ? "Clean worktree"
                  : repository.worktree === "dirty"
                    ? `${worktreeChanges} local change${worktreeChanges === 1 ? "" : "s"}`
                    : "Worktree unknown"}
              </span>
            </div>
            <dl className="context-facts">
              <div>
                <dt>Branch</dt>
                <dd>
                  {repository.currentBranch ??
                    (snapshot.git.detached ? "Detached HEAD" : "Unknown")}
                </dd>
              </div>
              <div>
                <dt>Local HEAD</dt>
                <dd>
                  <code>{shortCommit(repository.localHead)}</code>
                </dd>
              </div>
              <div>
                <dt>Tracking</dt>
                <dd>
                  {repository.ahead === null || repository.behind === null
                    ? "Not established"
                    : `${repository.ahead} ahead, ${repository.behind} behind`}
                </dd>
              </div>
              <div>
                <dt>Observed trust</dt>
                <dd>{repository.provenance.trust}</dd>
              </div>
            </dl>
          </article>

          <article className="project-card">
            <div className="project-card__heading">
              <div>
                <span className="project-kicker">Active milestone</span>
                <h2>{milestone.title}</h2>
              </div>
              <span className="directory-state">{milestone.status}</span>
            </div>
            <p>{milestone.objective}</p>
            <dl className="context-facts">
              <div>
                <dt>Owner approval</dt>
                <dd>{milestone.ownerApproval.decision}</dd>
              </div>
              <div>
                <dt>Merge authorization</dt>
                <dd>{repository.mergeAuthorization.decision}</dd>
              </div>
              <div>
                <dt>Release authorization</dt>
                <dd>{repository.releaseAuthorization.decision}</dd>
              </div>
              <div>
                <dt>Policy source</dt>
                <dd>{milestone.provenance.sourceType}</dd>
              </div>
            </dl>
          </article>

          <article className="project-card">
            <div className="project-card__heading">
              <div>
                <span className="project-kicker">Evidence inventory</span>
                <h2>Validation and packages</h2>
              </div>
              <span className="directory-state">
                {snapshot.diagnostics.length} diagnostic
                {snapshot.diagnostics.length === 1 ? "" : "s"}
              </span>
            </div>
            <dl className="context-facts">
              <div>
                <dt>Validation records</dt>
                <dd>{snapshot.evidence.validations.length}</dd>
              </div>
              <div>
                <dt>Package records</dt>
                <dd>{snapshot.evidence.packages.length}</dd>
              </div>
              <div>
                <dt>Handoff report</dt>
                <dd>{snapshot.evidence.handoff?.status ?? "Not reported"}</dd>
              </div>
              <div>
                <dt>Shallow repository</dt>
                <dd>
                  {snapshot.git.shallow === null
                    ? "Unknown"
                    : snapshot.git.shallow
                      ? "Yes"
                      : "No"}
                </dd>
              </div>
            </dl>
            {snapshot.evidence.validations.length > 0 && (
              <div className="project-flags" aria-label="Validation evidence">
                {snapshot.evidence.validations.map((validation) => (
                  <span key={validation.id}>
                    {validation.id}: {validation.status},{" "}
                    {freshnessLabel(validation.freshness)}
                  </span>
                ))}
              </div>
            )}
            {snapshot.evidence.packages.length > 0 && (
              <div className="project-flags" aria-label="Package evidence">
                {snapshot.evidence.packages.map((artifact) => (
                  <span
                    key={`${artifact.kind}-${artifact.filename ?? "unknown"}`}
                  >
                    {artifact.kind}: {freshnessLabel(artifact.freshness)}
                    {artifact.localVerified ? ", locally verified" : ""}
                  </span>
                ))}
              </div>
            )}
          </article>

          {snapshot.diagnostics.length > 0 && (
            <article
              className="project-card"
              aria-labelledby="state-diagnostics"
            >
              <div className="project-card__heading">
                <div>
                  <span className="project-kicker">Read-only diagnostics</span>
                  <h2 id="state-diagnostics">Evidence needing attention</h2>
                </div>
              </div>
              <ul className="project-list">
                {snapshot.diagnostics.map((diagnostic) => (
                  <li className="project-message" key={diagnostic.id}>
                    <strong>{diagnostic.id}</strong>: {diagnostic.explanation}{" "}
                    <span>
                      Suggested next action: {diagnostic.recommendedAction}
                    </span>
                  </li>
                ))}
              </ul>
            </article>
          )}
        </div>
      )}
    </section>
  );
}
