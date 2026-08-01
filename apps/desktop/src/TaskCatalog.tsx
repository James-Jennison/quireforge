import {
  useEffect,
  useRef,
  useState,
  type KeyboardEvent,
  type RefObject,
  type ReactNode,
} from "react";

import type { TaskCatalogSnapshot } from "./lib/taskRecords";

type SnapshotAction = () => Promise<TaskCatalogSnapshot>;
type TaskStatus = "active" | "paused" | "completed";

const diagnosticMessages: Record<
  NonNullable<TaskCatalogSnapshot["diagnosticCode"]>,
  string
> = {
  "metadata-unavailable": "Task records are unavailable.",
  "invalid-request": "The task request was invalid.",
  "capacity-reached":
    "Task storage is at capacity. Delete eligible completed or archived tasks to continue.",
  "task-not-found": "That task no longer exists.",
  "task-archived": "Restore the archived task before editing it.",
  "plan-not-found": "That plan no longer exists.",
  "invalid-status-transition": "That task status change is not allowed.",
  "duplicate-id": "A unique local task identifier could not be created.",
  "invalid-stored-value":
    "One or more invalid local task records were omitted.",
};

function ConfirmationDialog({
  title,
  children,
  confirmLabel,
  onConfirm,
  onCancel,
}: {
  title: string;
  children: ReactNode;
  confirmLabel: string;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  const dialogRef = useRef<HTMLDivElement>(null);
  const cancelRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    cancelRef.current?.focus();
  }, []);

  function handleKeyDown(event: KeyboardEvent<HTMLDialogElement>) {
    if (event.key === "Escape") {
      event.preventDefault();
      onCancel();
      return;
    }
    if (event.key !== "Tab") return;
    const controls =
      dialogRef.current?.querySelectorAll<HTMLElement>(
        'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
      ) ?? [];
    if (controls.length === 0) return;
    const first = controls[0];
    const last = controls[controls.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last?.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first?.focus();
    }
  }

  return (
    <dialog
      open
      className="task-catalog-dialog"
      aria-modal="true"
      aria-labelledby="task-catalog-dialog-title"
      onKeyDown={handleKeyDown}
    >
      <button
        className="task-catalog-dialog__backdrop"
        type="button"
        aria-label="Cancel confirmation"
        onClick={onCancel}
      />
      <div ref={dialogRef} className="task-catalog-dialog__panel">
        <h3 id="task-catalog-dialog-title">{title}</h3>
        {children}
        <div className="task-catalog__actions">
          <button ref={cancelRef} type="button" onClick={onCancel}>
            Cancel
          </button>
          <button type="button" className="danger-button" onClick={onConfirm}>
            {confirmLabel}
          </button>
        </div>
      </div>
    </dialog>
  );
}

function TaskTitleEditor({
  taskId,
  title,
  disabled,
  inputRef,
  onSave,
}: {
  taskId: string;
  title: string;
  disabled: boolean;
  inputRef: React.RefObject<HTMLInputElement | null>;
  onSave: (taskId: string, title: string) => SnapshotAction;
}) {
  const [draft, setDraft] = useState(title);
  return (
    <form
      onSubmit={(event) => {
        event.preventDefault();
        void onSave(taskId, draft)();
      }}
    >
      <label>
        Task title
        <input
          ref={inputRef}
          value={draft}
          maxLength={120}
          disabled={disabled}
          onChange={(event) => setDraft(event.target.value)}
        />
      </label>
      <button type="submit" disabled={disabled || draft === title}>
        Save title
      </button>
    </form>
  );
}

function PlanEditor({
  taskId,
  plan,
  disabled,
  onSave,
}: {
  taskId: string;
  plan: TaskCatalogSnapshot["plans"][number];
  disabled: boolean;
  onSave: (
    taskId: string,
    planId: string,
    label: string,
    body: string,
  ) => SnapshotAction;
}) {
  const [label, setLabel] = useState(plan.label);
  const [body, setBody] = useState(plan.body);
  return (
    <form
      className="task-catalog__plan-editor"
      onSubmit={(event) => {
        event.preventDefault();
        void onSave(taskId, plan.id, label, body)();
      }}
    >
      <label>
        Plan label
        <input
          value={label}
          maxLength={80}
          readOnly={disabled}
          onChange={(event) => setLabel(event.target.value)}
        />
      </label>
      <label>
        Plan text
        <textarea
          value={body}
          maxLength={8_192}
          rows={5}
          readOnly={disabled}
          onChange={(event) => setBody(event.target.value)}
        />
      </label>
      {!disabled && (
        <button
          type="submit"
          disabled={label === plan.label && body === plan.body}
        >
          Save plan
        </button>
      )}
    </form>
  );
}

export function TaskCatalog({
  snapshot,
  busy,
  onLoad,
  onCreate,
  onRename,
  onStatus,
  onArchive,
  onRestore,
  onDelete,
  onPlanCreate,
  onPlanSelect,
  onPlanEdit,
  onPlanDelete,
  onOpenTemplates,
  onOpenMockInference,
  mockInferenceTriggerRef,
}: {
  snapshot: TaskCatalogSnapshot;
  busy: boolean;
  onLoad: (request: {
    query: string | null;
    includeArchived: boolean;
    selectedTaskId: string | null;
  }) => Promise<void>;
  onCreate: SnapshotAction;
  onRename: (taskId: string, title: string) => SnapshotAction;
  onStatus: (taskId: string, status: TaskStatus) => SnapshotAction;
  onArchive: (taskId: string) => SnapshotAction;
  onRestore: (taskId: string) => SnapshotAction;
  onDelete: (taskId: string) => SnapshotAction;
  onPlanCreate: (taskId: string, copyPrimaryBody: boolean) => SnapshotAction;
  onPlanSelect: (taskId: string, planId: string) => SnapshotAction;
  onPlanEdit: (
    taskId: string,
    planId: string,
    label: string,
    body: string,
  ) => SnapshotAction;
  onPlanDelete: (taskId: string, planId: string) => SnapshotAction;
  onOpenTemplates?: () => void;
  onOpenMockInference?: () => void;
  mockInferenceTriggerRef?: RefObject<HTMLButtonElement | null>;
}) {
  const [query, setQuery] = useState("");
  const [includeArchived, setIncludeArchived] = useState(false);
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);
  const [notice, setNotice] = useState("");
  const [confirmation, setConfirmation] = useState<
    | { kind: "task"; taskId: string; title: string; planCount: number }
    | { kind: "plan"; taskId: string; planId: string; label: string }
    | null
  >(null);
  const titleRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLElement>(null);
  const confirmationTrigger = useRef<HTMLElement | null>(null);

  const report = (result: TaskCatalogSnapshot, success: string) => {
    setNotice(
      result.diagnosticCode
        ? diagnosticMessages[result.diagnosticCode]
        : success,
    );
    return result;
  };
  const apply = async (action: SnapshotAction, success: string) => {
    try {
      return report(await action(), success);
    } catch {
      setNotice(diagnosticMessages["metadata-unavailable"]);
      return null;
    }
  };
  const load = async (selectedId: string | null = selectedTaskId) => {
    try {
      await onLoad({
        query: query || null,
        includeArchived,
        selectedTaskId: selectedId,
      });
    } catch {
      setNotice(diagnosticMessages["metadata-unavailable"]);
    }
  };
  const restoreConfirmationFocus = () => {
    const trigger = confirmationTrigger.current;
    setConfirmation(null);
    window.requestAnimationFrame(() => {
      if (trigger?.isConnected) trigger.focus();
      else listRef.current?.querySelector<HTMLElement>("button")?.focus();
    });
  };
  const selected = snapshot.selectedTask;
  const selectedPlan = snapshot.plans.find(
    (plan) => plan.id === selected?.selectedPlanId,
  );
  const switchPlan = (
    taskId: string,
    planId: string,
    restoreFocus?: HTMLButtonElement,
  ) => {
    void apply(
      onPlanSelect(taskId, planId),
      "Plan selected. Transient task-plan state was cleared.",
    ).then(() => {
      if (restoreFocus) {
        window.requestAnimationFrame(() => restoreFocus.focus());
      }
    });
  };
  const payloadMiB = (snapshot.payloadBytes / (1024 * 1024)).toFixed(2);

  return (
    <section className="task-catalog" aria-labelledby="task-catalog-title">
      <div className="context-section__heading">
        <div>
          <span>Local organization</span>
          <h2 id="task-catalog-title">Tasks</h2>
        </div>
        <div className="task-catalog__actions">
          <button
            type="button"
            disabled={
              busy ||
              snapshot.state === "unavailable" ||
              snapshot.taskCount >= 200 ||
              snapshot.payloadBytes >= 8 * 1024 * 1024
            }
            onClick={() => {
              void apply(onCreate, "Task created.").then((result) => {
                if (result?.selectedTask) {
                  setSelectedTaskId(result.selectedTask.id);
                  window.requestAnimationFrame(() => titleRef.current?.focus());
                }
              });
            }}
          >
            New task
          </button>
          <button type="button" onClick={onOpenTemplates}>
            Task Templates
          </button>
          <button
            ref={mockInferenceTriggerRef}
            type="button"
            onClick={onOpenMockInference}
          >
            Fictional mock inference
          </button>
        </div>
      </div>
      <p className="context-note">
        Titles and plans stay local. They do not contain or control
        conversations, files, approvals, or execution.
      </p>
      {snapshot.state === "unavailable" ? (
        <div className="task-catalog__unavailable" role="status">
          <p>
            Task records are unavailable. Existing project and conversation
            state is unchanged.
          </p>
          <button type="button" disabled={busy} onClick={() => void load(null)}>
            Retry
          </button>
        </div>
      ) : (
        <>
          <label className="task-catalog__search">
            Search tasks
            <input
              type="search"
              value={query}
              maxLength={120}
              onChange={(event) => {
                const value = event.target.value;
                setQuery(value);
                void onLoad({
                  query: value || null,
                  includeArchived,
                  selectedTaskId,
                }).catch(() =>
                  setNotice(diagnosticMessages["metadata-unavailable"]),
                );
              }}
            />
          </label>
          <label className="task-catalog__archived">
            <input
              type="checkbox"
              checked={includeArchived}
              onChange={(event) => {
                const value = event.target.checked;
                setIncludeArchived(value);
                void onLoad({
                  query: query || null,
                  includeArchived: value,
                  selectedTaskId,
                }).catch(() =>
                  setNotice(diagnosticMessages["metadata-unavailable"]),
                );
              }}
            />
            Include archived tasks
          </label>
          <p
            className={
              snapshot.warning
                ? "task-catalog__capacity-warning"
                : "context-note"
            }
            role={snapshot.warning ? "status" : undefined}
          >
            {snapshot.taskCount} of 200 tasks · {payloadMiB} of 8.00 MiB
            {snapshot.warning
              ? " · Near capacity; cleanup remains explicit."
              : ""}
          </p>
          <nav ref={listRef} aria-label="Task list">
            <ul className="task-catalog__list">
              {snapshot.tasks.map((task) => (
                <li key={task.id}>
                  <button
                    type="button"
                    aria-current={
                      snapshot.selectedTask?.id === task.id ? "page" : undefined
                    }
                    aria-label={`${task.title}, ${task.status}, ${
                      task.archived ? "archived" : "not archived"
                    }, ${task.planCount} ${
                      task.planCount === 1 ? "plan" : "plans"
                    }${task.cleanupEligible ? ", eligible for cleanup" : ""}`}
                    onClick={() => {
                      setSelectedTaskId(task.id);
                      void load(task.id);
                    }}
                  >
                    <strong>{task.title}</strong>
                    <span>
                      {task.status}
                      {task.archived ? " · archived" : ""}
                      {task.cleanupEligible ? " · cleanup eligible" : ""}
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          </nav>
          {snapshot.state === "empty" && (
            <p className="context-empty">
              {snapshot.taskCount === 0
                ? "Create a local task to keep a title and up to four plans."
                : "No tasks match this bounded title and plan-label search."}
            </p>
          )}
          {selected && (
            <div className="task-catalog__detail">
              <TaskTitleEditor
                key={selected.id}
                taskId={selected.id}
                title={selected.title}
                disabled={busy || selected.archived}
                inputRef={titleRef}
                onSave={(taskId, title) => () =>
                  apply(onRename(taskId, title), "Task title saved.").then(
                    (result) => result ?? snapshot,
                  )
                }
              />
              <label>
                Status
                <select
                  value={selected.status}
                  disabled={busy || selected.archived}
                  onChange={(event) => {
                    const status = event.target.value as TaskStatus;
                    if (status !== selected.status) {
                      void apply(
                        onStatus(selected.id, status),
                        "Task status saved.",
                      );
                    }
                  }}
                >
                  <option value="active">Active</option>
                  {selected.status !== "completed" && (
                    <option value="paused">Paused</option>
                  )}
                  <option value="completed">Completed</option>
                </select>
              </label>
              <div className="task-catalog__actions">
                {selected.archived ? (
                  <button
                    type="button"
                    disabled={busy}
                    onClick={() =>
                      void apply(
                        onRestore(selected.id),
                        "Task restored. Select it to continue.",
                      )
                    }
                  >
                    Restore task
                  </button>
                ) : (
                  <button
                    type="button"
                    disabled={busy}
                    onClick={() =>
                      void apply(onArchive(selected.id), "Task archived.").then(
                        () =>
                          window.requestAnimationFrame(() =>
                            listRef.current
                              ?.querySelector<HTMLElement>("button")
                              ?.focus(),
                          ),
                      )
                    }
                  >
                    Archive task
                  </button>
                )}
                <button
                  type="button"
                  disabled={busy}
                  onClick={(event) => {
                    confirmationTrigger.current = event.currentTarget;
                    setConfirmation({
                      kind: "task",
                      taskId: selected.id,
                      title: selected.title,
                      planCount: selected.planCount,
                    });
                  }}
                >
                  Delete task
                </button>
              </div>
              <div className="task-catalog__plans">
                <div className="task-catalog__actions">
                  <strong>Plans ({selected.planCount} of 4)</strong>
                  {!selected.archived && (
                    <>
                      <button
                        type="button"
                        disabled={busy || selected.planCount >= 4}
                        onClick={() =>
                          void apply(
                            onPlanCreate(selected.id, false),
                            "Alternate plan created.",
                          )
                        }
                      >
                        Add empty plan
                      </button>
                      <button
                        type="button"
                        disabled={busy || selected.planCount >= 4}
                        onClick={() =>
                          void apply(
                            onPlanCreate(selected.id, true),
                            "Primary plan text copied.",
                          )
                        }
                      >
                        Copy primary text
                      </button>
                    </>
                  )}
                </div>
                <div role="tablist" aria-label="Task plans">
                  {snapshot.plans.map((plan) => (
                    <button
                      key={plan.id}
                      type="button"
                      role="tab"
                      data-plan-id={plan.id}
                      disabled={busy}
                      tabIndex={plan.id === selected.selectedPlanId ? 0 : -1}
                      aria-selected={plan.id === selected.selectedPlanId}
                      aria-controls={`task-plan-${plan.id}`}
                      onKeyDown={(event) => {
                        if (
                          !["ArrowLeft", "ArrowRight", "Home", "End"].includes(
                            event.key,
                          )
                        ) {
                          return;
                        }
                        event.preventDefault();
                        const tabs = [
                          ...event.currentTarget.parentElement!.querySelectorAll<HTMLButtonElement>(
                            '[role="tab"]:not([disabled])',
                          ),
                        ];
                        const current = tabs.indexOf(event.currentTarget);
                        const next =
                          event.key === "Home"
                            ? 0
                            : event.key === "End"
                              ? tabs.length - 1
                              : (current +
                                  (event.key === "ArrowRight" ? 1 : -1) +
                                  tabs.length) %
                                tabs.length;
                        const target = tabs[next];
                        if (target && target !== event.currentTarget) {
                          target.focus();
                          switchPlan(
                            selected.id,
                            target.dataset.planId!,
                            target,
                          );
                        }
                      }}
                      onClick={() => {
                        if (plan.id !== selected.selectedPlanId) {
                          switchPlan(selected.id, plan.id);
                        }
                      }}
                    >
                      {plan.label}
                    </button>
                  ))}
                </div>
                {selectedPlan && (
                  <div
                    id={`task-plan-${selectedPlan.id}`}
                    role="tabpanel"
                    aria-label={`${selectedPlan.label} plan`}
                  >
                    <PlanEditor
                      key={selectedPlan.id}
                      taskId={selected.id}
                      plan={selectedPlan}
                      disabled={busy || selected.archived}
                      onSave={(taskId, planId, label, body) => () =>
                        apply(
                          onPlanEdit(taskId, planId, label, body),
                          "Plan saved.",
                        ).then((result) => result ?? snapshot)
                      }
                    />
                    {!selected.archived && selected.planCount > 1 && (
                      <button
                        type="button"
                        disabled={busy}
                        onClick={(event) => {
                          confirmationTrigger.current = event.currentTarget;
                          setConfirmation({
                            kind: "plan",
                            taskId: selected.id,
                            planId: selectedPlan.id,
                            label: selectedPlan.label,
                          });
                        }}
                      >
                        Delete selected plan
                      </button>
                    )}
                  </div>
                )}
              </div>
            </div>
          )}
        </>
      )}
      <p className="task-catalog__status" role="status" aria-live="polite">
        {snapshot.diagnosticCode
          ? diagnosticMessages[snapshot.diagnosticCode]
          : notice}
      </p>
      {confirmation?.kind === "task" && (
        <ConfirmationDialog
          title={`Delete “${confirmation.title}”?`}
          confirmLabel="Delete task permanently"
          onCancel={restoreConfirmationFocus}
          onConfirm={() => {
            const target = confirmation;
            void apply(onDelete(target.taskId), "Task deleted.").then(() => {
              setSelectedTaskId(null);
              restoreConfirmationFocus();
            });
          }}
        >
          <p>
            This immediately removes the task and all {confirmation.planCount}{" "}
            local {confirmation.planCount === 1 ? "plan" : "plans"}. External
            project files, worktrees, Git history, package evidence, repository
            source, and user-saved artifacts will not change.
          </p>
          <p>
            QuireForge has no application trash for task records. Filesystem
            journals and ordinary backups may still retain recoverable copies.
          </p>
        </ConfirmationDialog>
      )}
      {confirmation?.kind === "plan" && (
        <ConfirmationDialog
          title={`Delete “${confirmation.label}”?`}
          confirmLabel="Delete selected plan"
          onCancel={restoreConfirmationFocus}
          onConfirm={() => {
            const target = confirmation;
            void apply(
              onPlanDelete(target.taskId, target.planId),
              "Plan deleted. The lowest remaining plan is selected when needed.",
            ).then(restoreConfirmationFocus);
          }}
        >
          <p>
            This immediately removes the selected local plan text. It does not
            delete or change conversations, external files, approvals,
            execution, terminals, Git state, attachments, or artifacts.
          </p>
        </ConfirmationDialog>
      )}
    </section>
  );
}
