import type { AdvisorWorkspaceSnapshot } from "./lib/advisorWorkspace";

interface AdvisorWorkspaceProps {
  availability: "checking" | "native" | "preview" | "error";
  snapshot: AdvisorWorkspaceSnapshot | null;
}

export function AdvisorWorkspace({
  availability,
  snapshot,
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
    </section>
  );
}
