import { useEffect, useState } from "react";

import type { GitDiffSnapshot, GitWorkspaceSnapshot } from "../lib/git";
import type { ReviewPaneData } from "./types";

export default function DiffPane({
  projectId,
  loadGitStatus,
  loadGitDiff,
}: ReviewPaneData) {
  const [state, setState] = useState<"loading" | "ready" | "failure">(
    "loading",
  );
  const [diff, setDiff] = useState<GitDiffSnapshot | null>(null);
  useEffect(() => {
    if (!projectId) return;
    let active = true;
    void loadGitStatus(projectId)
      .then((status: GitWorkspaceSnapshot) => {
        const change = status.changes.find(
          (item) => item.staged || item.worktree,
        );
        if (!change) {
          if (active) setState("ready");
          return;
        }
        return loadGitDiff({
          projectId,
          path: change.path,
          area: change.staged ? "staged" : "worktree",
        }).then((value) => {
          if (active) {
            setDiff(value);
            setState("ready");
          }
        });
      })
      .catch(() => {
        if (active) setState("failure");
      });
    return () => {
      active = false;
    };
  }, [loadGitDiff, loadGitStatus, projectId]);
  if (!projectId)
    return <p role="status">No approved changed-file evidence is available.</p>;
  if (state === "loading")
    return <p role="status">Loading one bounded diff…</p>;
  if (state === "failure" || diff?.state === "unavailable")
    return <p role="status">Diff evidence is unavailable.</p>;
  if (!diff)
    return <p role="status">No approved changed-file evidence is available.</p>;
  return (
    <article className="review-pane-card">
      <h3>{diff.path}</h3>
      {diff.truncated && <p role="status">Diff is truncated.</p>}
      <pre>
        <code>{diff.lines.map((line) => line.text).join("\n")}</code>
      </pre>
    </article>
  );
}
