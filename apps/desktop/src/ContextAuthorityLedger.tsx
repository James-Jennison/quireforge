import { useEffect, useState } from "react";
import { loadContextAuthorityLedger } from "./lib/bridge";
import type { ContextLedgerSnapshot } from "./lib/contextLedger";

const recordLabels: Record<
  ContextLedgerSnapshot["entries"][number]["recordKind"],
  string
> = {
  "artifact-reference": "Artifact reference",
  "browser-verification": "Browser verification",
  "connector-operation": "Connector operation",
  "context-bundle": "Context bundle",
  "durable-source": "Durable source",
};

function shortDigest(digest: string) {
  return `${digest.slice(0, 12)}…${digest.slice(-8)}`;
}

function timestamp(value: number) {
  return value === 0 ? "No expiry" : new Date(value).toLocaleString();
}

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
          <h1 className="work-route-title">Context and Authority Ledger</h1>
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
            <li key={entry.recordId} className="context-ledger__entry">
              <div className="context-ledger__entry-heading">
                <strong>{recordLabels[entry.recordKind]}</strong>
                <span className="context-ledger__state">{entry.state}</span>
              </div>
              <dl className="context-ledger__facts">
                <div>
                  <dt>Outcome</dt>
                  <dd>{entry.auditOutcome}</dd>
                </div>
                <div>
                  <dt>Selected</dt>
                  <dd>
                    {entry.itemCount} item{entry.itemCount === 1 ? "" : "s"}
                  </dd>
                </div>
                <div>
                  <dt>Expiry</dt>
                  <dd>{timestamp(entry.expiresAtMs)}</dd>
                </div>
              </dl>
              <code
                title={entry.bundleDigest}
                aria-label={`Digest ${entry.bundleDigest}`}
              >
                {shortDigest(entry.bundleDigest)}
              </code>
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}
