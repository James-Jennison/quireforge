import type { ReviewPaneData } from "./types";
import { useLocalReviewPromotionPresentation } from "./localReviewSession";

export default function ApprovalPane({ conversation }: ReviewPaneData) {
  const approval = conversation.pendingApproval;
  const promotion = useLocalReviewPromotionPresentation();
  return (
    <article className="review-pane-card">
      {approval ? (
        <>
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
        </>
      ) : (
        <p role="status">No current approval proposal is available.</p>
      )}
      <section aria-label="Local review promotion status">
        <h3>Local review promotion</h3>
        <p>{promotion.state}</p>
        {promotion.label ? <p>{promotion.label}</p> : null}
        {promotion.destinationClass ? (
          <p>Destination: {promotion.destinationClass}</p>
        ) : null}
        {promotion.sha256 ? (
          <code aria-label={`SHA-256 ${promotion.sha256}`}>
            {promotion.sha256.slice(0, 12)}
          </code>
        ) : null}
        {promotion.expiresAtMs ? <p>Expiry: {promotion.expiresAtMs}</p> : null}
        <p>Review and promotion do not approve or dispatch work.</p>
      </section>
      <p role="status">
        This review pane cannot decide or dispatch the proposal.
      </p>
    </article>
  );
}
