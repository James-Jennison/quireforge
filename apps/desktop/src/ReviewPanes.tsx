/* eslint-disable jsx-a11y/no-noninteractive-tabindex -- ARIA separators are the documented keyboard resize control. */
import {
  lazy,
  Suspense,
  useEffect,
  useRef,
  type KeyboardEvent,
  type PointerEvent,
} from "react";

import {
  clampLayoutDimension,
  reviewPaneWidthMaximum,
  reviewPaneWidthMinimum,
} from "./layoutPreferences";
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
  width,
  selectedPane,
  onWidthChange,
  onSelectedPaneChange,
  ...data
}: ReviewPaneData & {
  onClose: () => void;
  width: number;
  selectedPane: ReviewPaneId;
  onWidthChange: (width: number) => void;
  onSelectedPaneChange: (pane: ReviewPaneId) => void;
}) {
  const closeRef = useRef<HTMLButtonElement>(null);
  const resizeCleanup = useRef<(() => void) | null>(null);
  const restore = useRef<HTMLElement | null>(
    document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null,
  );
  useEffect(
    () => () => {
      resizeCleanup.current?.();
      if (restore.current?.isConnected) restore.current.focus();
    },
    [],
  );
  const select = (pane: ReviewPaneId) => {
    onSelectedPaneChange(pane);
  };
  const resizeFromKeyboard = (event: KeyboardEvent<HTMLDivElement>) => {
    const delta =
      event.key === "ArrowLeft" ? 20 : event.key === "ArrowRight" ? -20 : 0;
    if (!delta) return;
    event.preventDefault();
    onWidthChange(
      clampLayoutDimension(
        width + delta,
        reviewPaneWidthMinimum,
        reviewPaneWidthMaximum,
      ),
    );
  };
  const beginResize = (event: PointerEvent<HTMLDivElement>) => {
    event.preventDefault();
    const resize = (pointerEvent: globalThis.PointerEvent) =>
      onWidthChange(
        clampLayoutDimension(
          window.innerWidth - pointerEvent.clientX,
          reviewPaneWidthMinimum,
          reviewPaneWidthMaximum,
        ),
      );
    const stop = () => {
      document.removeEventListener("pointermove", resize);
      document.removeEventListener("pointerup", stop);
      document.removeEventListener("pointercancel", stop);
      resizeCleanup.current = null;
    };
    resizeCleanup.current?.();
    resizeCleanup.current = stop;
    document.addEventListener("pointermove", resize);
    document.addEventListener("pointerup", stop, { once: true });
    document.addEventListener("pointercancel", stop, { once: true });
  };
  const Pane = panes[selectedPane];
  return (
    <aside
      className="review-panes"
      aria-labelledby="review-panes-title"
      style={
        { "--review-pane-width": `${width}px` } as import("react").CSSProperties
      }
    >
      {/* eslint-disable-next-line jsx-a11y/no-noninteractive-element-interactions -- ARIA separators are the documented keyboard resize control. */}
      <div
        className="review-panes__resize"
        role="separator"
        aria-label="Resize task evidence"
        aria-orientation="vertical"
        aria-valuemin={reviewPaneWidthMinimum}
        aria-valuemax={reviewPaneWidthMaximum}
        aria-valuenow={width}
        tabIndex={0}
        onPointerDown={beginResize}
        onKeyDown={resizeFromKeyboard}
      />
      <header>
        <div>
          <p className="eyebrow">Closed review surfaces</p>
          <h2 id="review-panes-title">Task evidence</h2>
        </div>
        <button
          ref={closeRef}
          type="button"
          aria-label="Close task evidence"
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
            aria-selected={selectedPane === id}
            aria-controls={`review-pane-${id}`}
            onClick={() => select(id)}
          >
            {labels[id]}
          </button>
        ))}
      </div>
      <section
        id={`review-pane-${selectedPane}`}
        role="tabpanel"
        aria-label={`${labels[selectedPane]} review pane`}
        tabIndex={-1}
      >
        <Suspense
          fallback={
            <p role="status">Loading {labels[selectedPane]} review pane…</p>
          }
        >
          <Pane {...data} />
        </Suspense>
      </section>
    </aside>
  );
}
