import { lazy, Suspense, useEffect, useRef, useState } from "react";

import type { ReviewPaneData, ReviewPaneId } from "./review-panes/types";
import { reviewPaneIds } from "./review-panes/types";

const panes = {
  files: lazy(() => import("./review-panes/FilesPane")),
  diff: lazy(() => import("./review-panes/DiffPane")),
  git: lazy(() => import("./review-panes/GitPane")),
  preview: lazy(() => import("./review-panes/PreviewPane")),
  activity: lazy(() => import("./review-panes/ActivityPane")),
  approval: lazy(() => import("./review-panes/ApprovalPane")),
};

const labels: Record<ReviewPaneId, string> = {
  files: "Files",
  diff: "Diff",
  git: "Git",
  preview: "Preview",
  activity: "Activity",
  approval: "Approval",
};

export function ReviewPanes({
  onClose,
  ...data
}: ReviewPaneData & { onClose: () => void }) {
  const [selected, setSelected] = useState<ReviewPaneId>("files");
  const closeRef = useRef<HTMLButtonElement>(null);
  const restore = useRef<HTMLElement | null>(
    document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null,
  );
  useEffect(
    () => () => {
      if (restore.current?.isConnected) restore.current.focus();
    },
    [],
  );
  const Pane = panes[selected];
  return (
    <aside className="review-panes" aria-labelledby="review-panes-title">
      <header>
        <div>
          <p className="eyebrow">Closed review surfaces</p>
          <h2 id="review-panes-title">Task evidence</h2>
        </div>
        <button
          ref={closeRef}
          type="button"
          aria-label="Close review panes"
          onClick={onClose}
        >
          ×
        </button>
      </header>
      <div role="tablist" aria-label="Review panes">
        {reviewPaneIds.map((id) => (
          <button
            key={id}
            type="button"
            role="tab"
            aria-selected={selected === id}
            aria-controls={`review-pane-${id}`}
            onClick={() => setSelected(id)}
          >
            {labels[id]}
          </button>
        ))}
      </div>
      <section
        id={`review-pane-${selected}`}
        role="tabpanel"
        aria-label={`${labels[selected]} review pane`}
        tabIndex={-1}
      >
        <Suspense
          fallback={
            <p role="status">Loading {labels[selected]} review pane…</p>
          }
        >
          <Pane {...data} />
        </Suspense>
      </section>
    </aside>
  );
}
