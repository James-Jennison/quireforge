import type { ReviewPaneData } from "./types";

export default function ApprovalPane({ conversation }: ReviewPaneData) {
  const approval = conversation.pendingApproval;
  if (!approval)
    return <p role="status">No current approval proposal is available.</p>;
  return (
    <article className="review-pane-card">
      <h3>{approval.title}</h3>
      <p>{approval.kind} · proposal pending</p>
      {approval.reason && <p>{approval.reason}</p>}
      <dl>
        {approval.details.map((detail) => (
          <div key={detail.label}>
            <dt>{detail.label}</dt>
            <dd>{detail.value}</dd>
          </div>
        ))}
      </dl>
      <p role="status">
        This review pane cannot decide or dispatch the proposal.
      </p>
    </article>
  );
}
