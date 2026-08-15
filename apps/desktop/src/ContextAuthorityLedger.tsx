import { useEffect, useState } from "react";
import { loadContextAuthorityLedger } from "./lib/bridge";
import type { ContextLedgerSnapshot } from "./lib/contextLedger";

export function ContextAuthorityLedger({
  projectId,
}: {
  projectId: string | null;
}) {
  const [snapshot, setSnapshot] = useState<ContextLedgerSnapshot | null>(null);
  useEffect(() => {
    let active = true;
    if (!projectId) {
      return;
    }
    void loadContextAuthorityLedger(projectId)
      .then((value) => {
        if (active) setSnapshot(value);
      })
      .catch(() => {
        if (active) setSnapshot(null);
      });
    return () => {
      active = false;
    };
  }, [projectId]);
  return (
    <section
      className="task-template-workbench"
      aria-label="Context and Authority Ledger"
    >
      <header className="task-template-workbench__header">
        <div>
          <p className="eyebrow">Read-only governance receipts</p>
          <h2>Context and Authority Ledger</h2>
        </div>
      </header>
      <p>
        Shows content-free local governance receipts only. It cannot transfer
        content, authorize a destination, or run an action.
      </p>
      {!projectId ? (
        <p role="status">Select a project to inspect its local ledger.</p>
      ) : snapshot?.diagnostic ? (
        <p role="status">Ledger is unavailable; no authority changed.</p>
      ) : !snapshot ? (
        <p role="status">Loading local ledger…</p>
      ) : snapshot.entries.length === 0 ? (
        <p role="status">No governance receipts exist for this project.</p>
      ) : (
        <ol className="context-ledger__entries">
          {snapshot.entries.map((entry) => (
            <li key={entry.recordId}>
              <strong>
                {entry.recordKind}: {entry.state}
              </strong>
              <span>
                {" "}
                · {entry.itemCount} selected item
                {entry.itemCount === 1 ? "" : "s"} · audit: {entry.auditOutcome}
              </span>
              <code>{entry.bundleDigest}</code>
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}
