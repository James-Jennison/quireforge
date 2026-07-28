import { useEffect, useState } from "react";

import type { GitWorkspaceSnapshot } from "../lib/git";
import type { ReviewPaneData } from "./types";

export default function GitPane({ projectId, loadGitStatus }: ReviewPaneData) {
  const [state, setState] = useState<"loading" | "ready" | "failure">(
    "loading",
  );
  const [snapshot, setSnapshot] = useState<GitWorkspaceSnapshot | null>(null);
  useEffect(() => {
    if (!projectId) return;
    let active = true;
    void loadGitStatus(projectId)
      .then((value) => {
        if (active) {
          setSnapshot(value);
          setState("ready");
        }
      })
      .catch(() => {
        if (active) setState("failure");
      });
    return () => {
      active = false;
    };
  }, [loadGitStatus, projectId]);
  if (!projectId) return <p role="status">Git evidence is unavailable.</p>;
  if (state === "loading") return <p role="status">Loading Git evidence…</p>;
  if (state === "failure")
    return <p role="status">Git evidence is unavailable.</p>;
  if (!snapshot || snapshot.state === "unavailable")
    return <p role="status">Git evidence is unavailable.</p>;
  if (snapshot.state === "clean")
    return <p role="status">The approved Git status is clean.</p>;
  return (
    <div className="review-pane-list">
      <p>
        {snapshot.branch?.head ?? "Detached HEAD"} · {snapshot.changes.length}{" "}
        changes
      </p>
      {snapshot.truncated && <p role="status">Status is truncated.</p>}
      <ul>
        {snapshot.changes.map((change) => (
          <li key={change.path}>
            {change.path} · {change.staged ?? change.worktree}
          </li>
        ))}
      </ul>
    </div>
  );
}
