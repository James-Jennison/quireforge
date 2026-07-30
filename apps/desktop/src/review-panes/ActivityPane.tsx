import type { ReviewPaneData } from "./types";
import { useLocalReviewActivity } from "./localReviewSession";

export default function ActivityPane({ conversationEvents }: ReviewPaneData) {
  const localReviewEvents = useLocalReviewActivity();
  const activity = conversationEvents.filter(
    (event) => event.type === "activity",
  );
  if (!activity.length && !localReviewEvents.length)
    return <p role="status">No bounded task activity is available.</p>;
  return (
    <ol className="review-pane-list">
      {localReviewEvents.map((event) => (
        <li key={event.id}>
          <strong>Local review · {event.kind}</strong> · {event.status} ·{" "}
          {event.label} · {event.timestamp}
          {event.digest ? (
            <code aria-label={`SHA-256 ${event.digest}`}>
              {event.digest.slice(0, 12)}
            </code>
          ) : null}
          {event.reason ? ` — ${event.reason}` : ""}
        </li>
      ))}
      {activity.map((event) => (
        <li key={event.sequence}>
          <strong>{event.title}</strong> · {event.status}
          {event.detail ? ` — ${event.detail}` : ""}
        </li>
      ))}
    </ol>
  );
}
