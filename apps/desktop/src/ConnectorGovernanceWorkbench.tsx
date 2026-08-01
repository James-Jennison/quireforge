import { useEffect, useId, useMemo, useRef, useState } from "react";

import {
  cancelConnectorGovernance,
  confirmConnectorGovernance,
  loadConnectorGovernance,
  loadTaskCatalog,
  prepareConnectorGovernance,
  revokeConnectorGovernance,
} from "./lib/bridge";
import type { ConnectorSnapshot } from "./lib/connectorGovernance";
import type { TaskCatalogSnapshot } from "./lib/taskRecords";

type Operations = {
  catalog: () => Promise<ConnectorSnapshot>;
  tasks: () => Promise<TaskCatalogSnapshot>;
  prepare: (request: {
    taskId: string;
    operation: "read" | "mutation";
    target: string;
  }) => Promise<ConnectorSnapshot>;
  confirm: (request: {
    taskId: string;
    operationId: string;
    authorizationId: string;
  }) => Promise<ConnectorSnapshot>;
  cancel: (request: {
    taskId: string;
    authorizationId: string;
  }) => Promise<ConnectorSnapshot>;
  revoke: (request: {
    taskId: string;
    operationId: string;
  }) => Promise<ConnectorSnapshot>;
};

export function ConnectorGovernanceWorkbench({
  projectId,
  onClose,
  operations,
}: {
  projectId: string | null;
  onClose: () => void;
  operations?: Operations;
}) {
  const titleId = useId();
  const closeRef = useRef<HTMLButtonElement>(null);
  const [tasks, setTasks] = useState<TaskCatalogSnapshot | null>(null);
  const [taskId, setTaskId] = useState("");
  const [snapshot, setSnapshot] = useState<ConnectorSnapshot | null>(null);
  const [notice, setNotice] = useState(
    "Loading fictional local connector governance…",
  );
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    closeRef.current?.focus();
  }, []);
  const nativeOperations = useMemo<Operations | null>(
    () =>
      projectId
        ? {
            catalog: loadConnectorGovernance,
            tasks: () =>
              loadTaskCatalog({
                projectId,
                query: null,
                includeArchived: false,
                selectedTaskId: null,
              }),
            prepare: prepareConnectorGovernance,
            confirm: confirmConnectorGovernance,
            cancel: cancelConnectorGovernance,
            revoke: revokeConnectorGovernance,
          }
        : null,
    [projectId],
  );
  const activeOperations = operations ?? nativeOperations;
  useEffect(() => {
    if (!activeOperations) return;
    void Promise.all([activeOperations.catalog(), activeOperations.tasks()])
      .then(([next, taskCatalog]) => {
        setSnapshot(next);
        setTasks(taskCatalog);
        setTaskId(
          taskCatalog.selectedTask?.id ?? taskCatalog.tasks[0]?.id ?? "",
        );
        setNotice(
          "Fictional local-only connector fixture ready. No network, credentials, browser, MCP, automation, or external mutation is involved.",
        );
      })
      .catch(() =>
        setNotice(
          "Connector governance fixture is unavailable; no operation occurred.",
        ),
      );
  }, [activeOperations]);
  const apply = async (action: () => Promise<ConnectorSnapshot>) => {
    setBusy(true);
    try {
      const next = await action();
      setSnapshot(next);
      setNotice(
        next.diagnostic
          ? `Connector operation rejected: ${next.diagnostic.replaceAll("-", " ")}.`
          : next.auditState,
      );
    } catch {
      setNotice(
        "Connector governance fixture is unavailable; no operation occurred.",
      );
    } finally {
      setBusy(false);
    }
  };
  const canConfirm =
    snapshot?.state === "prepared" &&
    snapshot.operationId &&
    snapshot.authorizationId &&
    taskId;
  return (
    <section
      className="mock-inference-workbench"
      role="dialog"
      aria-modal="true"
      aria-labelledby={titleId}
    >
      <header className="mock-inference-workbench__header">
        <div>
          <p className="eyebrow">Fictional local-only connector governance</p>
          <h2 id={titleId}>Connector review</h2>
        </div>
        <button ref={closeRef} type="button" onClick={onClose}>
          Close
        </button>
      </header>
      <p role="status">{notice}</p>
      <p>
        This is a deterministic local fixture. Declared capabilities do not
        grant authority, and a successful fictional read never authorizes
        mutation.
      </p>
      <label>
        Task{" "}
        <select
          value={taskId}
          onChange={(event) => setTaskId(event.target.value)}
          disabled={busy}
        >
          {tasks?.tasks.map((task) => (
            <option key={task.id} value={task.id}>
              {task.title}
            </option>
          ))}
        </select>
      </label>
      <p>
        Declared: read and fictional mutation. Granted:{" "}
        {snapshot?.grantedAuthority.join(", ") || "none"}.
      </p>
      <div className="mock-inference-workbench__actions">
        <button
          type="button"
          disabled={!taskId || busy}
          onClick={() =>
            void apply(() =>
              activeOperations!.prepare({
                taskId,
                operation: "read",
                target: "mock-object-read",
              }),
            )
          }
        >
          Run fictional read
        </button>
        <button
          type="button"
          disabled={!taskId || busy}
          onClick={() =>
            void apply(() =>
              activeOperations!.prepare({
                taskId,
                operation: "mutation",
                target: "mock-object-alpha",
              }),
            )
          }
        >
          Prepare fictional mutation
        </button>
        <button
          type="button"
          disabled={!taskId || busy}
          onClick={() =>
            void apply(() =>
              activeOperations!.prepare({
                taskId,
                operation: "mutation",
                target: "mock-object-ambiguous",
              }),
            )
          }
        >
          Prepare ambiguous fixture
        </button>
        <button
          type="button"
          disabled={!canConfirm || busy}
          onClick={() =>
            void apply(() =>
              activeOperations!.confirm({
                taskId,
                operationId: snapshot!.operationId!,
                authorizationId: snapshot!.authorizationId!,
              }),
            )
          }
        >
          Confirm once
        </button>
        <button
          type="button"
          disabled={!snapshot?.authorizationId || !taskId || busy}
          onClick={() =>
            void apply(() =>
              activeOperations!.cancel({
                taskId,
                authorizationId: snapshot!.authorizationId!,
              }),
            )
          }
        >
          Cancel review
        </button>
        <button
          type="button"
          disabled={!snapshot?.operationId || !taskId || busy}
          onClick={() =>
            void apply(() =>
              activeOperations!.revoke({
                taskId,
                operationId: snapshot!.operationId!,
              }),
            )
          }
        >
          Revoke fixture
        </button>
      </div>
      {snapshot?.state === "outcome-unknown" && (
        <p role="alert">
          Outcome is ambiguous. Automatic retry is prohibited; prepare a fresh
          review if a future approved route permits it.
        </p>
      )}
    </section>
  );
}
