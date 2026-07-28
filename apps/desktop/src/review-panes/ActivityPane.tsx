import type { ReviewPaneData } from "./types";

export default function ActivityPane({ conversationEvents }: ReviewPaneData) {
  const activity = conversationEvents.filter(
    (event) => event.type === "activity",
  );
  if (!activity.length)
    return <p role="status">No bounded task activity is available.</p>;
  return (
    <ol className="review-pane-list">
      {activity.map((event) => (
        <li key={event.sequence}>
          <strong>{event.title}</strong> · {event.status}
          {event.detail ? ` — ${event.detail}` : ""}
        </li>
      ))}
    </ol>
  );
}
