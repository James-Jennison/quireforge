import {
  useEffect,
  useRef,
  useState,
  type KeyboardEvent,
  type ReactNode,
} from "react";

import {
  archiveTaskTemplate,
  createTaskTemplate,
  deleteTaskTemplate,
  duplicateTaskTemplate,
  editTaskTemplate,
  inspectTaskTemplate,
  loadTaskTemplateCatalog,
  restoreTaskTemplate,
} from "./lib/bridge";
import type {
  TaskTemplateApplicationOutcome,
  TaskTemplateCatalogSnapshot,
  TaskTemplateInspectionSnapshot,
} from "./lib/taskTemplates";

type Operations = {
  loadCatalog: () => Promise<TaskTemplateCatalogSnapshot>;
  inspect: (request: {
    templateId: string;
  }) => Promise<TaskTemplateInspectionSnapshot>;
  create: (request: {
    title: string;
    purpose: string;
    instructions: string;
  }) => Promise<TaskTemplateInspectionSnapshot>;
  edit: (request: {
    mutationHandle: string;
    title: string;
    purpose: string;
    instructions: string;
  }) => Promise<TaskTemplateInspectionSnapshot>;
  duplicate: (request: {
    mutationHandle: string;
  }) => Promise<TaskTemplateInspectionSnapshot>;
  archive: (request: {
    mutationHandle: string;
  }) => Promise<TaskTemplateInspectionSnapshot>;
  restore: (request: {
    mutationHandle: string;
  }) => Promise<TaskTemplateInspectionSnapshot>;
  delete: (request: {
    mutationHandle: string;
    confirmation: "confirmed";
  }) => Promise<TaskTemplateApplicationOutcome>;
};

const nativeOperations: Operations = {
  loadCatalog: loadTaskTemplateCatalog,
  inspect: inspectTaskTemplate,
  create: createTaskTemplate,
  edit: editTaskTemplate,
  duplicate: duplicateTaskTemplate,
  archive: archiveTaskTemplate,
  restore: restoreTaskTemplate,
  delete: deleteTaskTemplate,
};
const blankDraft = { title: "", purpose: "", instructions: "" };
type Draft = typeof blankDraft;

function message(code: string | null | undefined) {
  return (
    {
      "invalid-request": "Check the template text and try again.",
      "not-found": "That template is no longer available.",
      "built-in-immutable": "Built-in templates are read-only.",
      "archived-read-only": "Restore this local template before editing it.",
      "active-already": "This template is already active.",
      "archived-already": "This template is already archived.",
      stale:
        "This template changed. Your draft is preserved; review it against refreshed details.",
      "capacity-reached":
        "Native template capacity has been reached. Your draft is preserved.",
      "metadata-unavailable": "Task templates are unavailable.",
      unavailable: "Task templates are unavailable. Your draft is preserved.",
    }[code ?? ""] ?? "Task templates are unavailable. Your draft is preserved."
  );
}

function Dialog({
  title,
  children,
  onCancel,
  confirm,
  trigger,
}: {
  title: string;
  children: ReactNode;
  onCancel: () => void;
  confirm?: () => void;
  trigger: React.MutableRefObject<HTMLElement | null>;
}) {
  const panel = useRef<HTMLDivElement>(null);
  const cancel = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    cancel.current?.focus();
  }, []);
  const close = () => {
    onCancel();
    requestAnimationFrame(() => trigger.current?.focus());
  };
  const onKeyDown = (event: KeyboardEvent<HTMLDialogElement>) => {
    if (event.key === "Escape") {
      event.preventDefault();
      close();
      return;
    }
    if (event.key !== "Tab") return;
    const controls =
      panel.current?.querySelectorAll<HTMLElement>(
        "button:not([disabled]), input:not([disabled]), textarea:not([disabled])",
      ) ?? [];
    if (!controls.length) return;
    const first = controls[0];
    const last = controls[controls.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last?.focus();
    }
    if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first?.focus();
    }
  };
  return (
    <dialog
      open
      className="task-template-dialog"
      aria-modal="true"
      aria-labelledby="task-template-dialog-title"
      onKeyDown={onKeyDown}
    >
      <button
        className="task-template-dialog__backdrop"
        type="button"
        aria-label="Close dialog"
        onClick={close}
      />
      <div className="task-template-dialog__panel" ref={panel}>
        <h3 id="task-template-dialog-title">{title}</h3>
        {children}
        <div className="task-template-workbench__actions">
          <button ref={cancel} type="button" onClick={close}>
            Cancel
          </button>
          {confirm && (
            <button type="button" className="danger-button" onClick={confirm}>
              Delete template
            </button>
          )}
        </div>
      </div>
    </dialog>
  );
}

export function TaskTemplateWorkbench({
  onClose,
  operations = nativeOperations,
}: {
  onClose: () => void;
  operations?: Operations;
}) {
  const [catalog, setCatalog] = useState<TaskTemplateCatalogSnapshot | null>(
    null,
  );
  const [detail, setDetail] = useState<TaskTemplateInspectionSnapshot | null>(
    null,
  );
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [draft, setDraft] = useState<Draft>(blankDraft);
  const [form, setForm] = useState<"create" | "edit" | null>(null);
  const [deleting, setDeleting] = useState(false);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState("");
  const list = useRef<HTMLUListElement>(null);
  const createButton = useRef<HTMLButtonElement>(null);
  const trigger = useRef<HTMLElement | null>(null);

  const refresh = async (keepSelection = true) => {
    try {
      const next = await operations.loadCatalog();
      setCatalog(next);
      if (
        keepSelection &&
        selectedId &&
        next.templates.some((entry) => entry.id === selectedId)
      )
        await select(selectedId, false);
      else if (selectedId) {
        setSelectedId(null);
        setDetail(null);
      }
      return next;
    } catch {
      setCatalog(null);
      setNotice("Task templates are unavailable.");
      return null;
    }
  };
  const select = async (id: string, focus = true) => {
    setSelectedId(id);
    setDetail(null);
    try {
      const next = await operations.inspect({ templateId: id });
      setDetail(next);
      if (next.diagnosticCode) setNotice(message(next.diagnosticCode));
    } catch {
      setDetail(null);
      setNotice("Task templates are unavailable.");
    }
    if (focus)
      requestAnimationFrame(() =>
        list.current
          ?.querySelector<HTMLButtonElement>(`button[data-template-id="${id}"]`)
          ?.focus(),
      );
  };
  useEffect(() => {
    let active = true;
    void operations.loadCatalog().then(
      (next) => {
        if (active) setCatalog(next);
      },
      () => {
        if (active) {
          setCatalog(null);
          setNotice("Task templates are unavailable.");
        }
      },
    );
    return () => {
      active = false;
    };
  }, [operations]); // Native data loads only when this lazy pane mounts.

  const selected = detail?.state === "ready" ? detail.template : null;
  const handle = detail?.state === "ready" ? detail.mutationHandle : null;
  const mutate = async (
    action: () => Promise<TaskTemplateInspectionSnapshot>,
    success: string,
  ) => {
    setBusy(true);
    try {
      const result = await action();
      if (result.diagnosticCode) {
        setNotice(message(result.diagnosticCode));
        if (result.diagnosticCode === "stale") await refresh(true);
        return;
      }
      setDetail(result);
      setSelectedId(result.template?.id ?? null);
      setNotice(success);
      setForm(null);
      setDraft(blankDraft);
      await refresh(false);
      if (result.template) await select(result.template.id, false);
      requestAnimationFrame(() =>
        trigger.current?.isConnected
          ? trigger.current.focus()
          : createButton.current?.focus(),
      );
    } catch {
      setNotice("Task templates are unavailable. Your draft is preserved.");
    } finally {
      setBusy(false);
    }
  };
  const submit = () => {
    if (form === "create")
      void mutate(() => operations.create(draft), "Local template created.");
    if (form === "edit" && handle)
      void mutate(
        () => operations.edit({ mutationHandle: handle, ...draft }),
        "Local template updated.",
      );
  };
  const listKeyDown = (event: KeyboardEvent<HTMLElement>) => {
    const ids = catalog?.templates.map((entry) => entry.id) ?? [];
    const current = selectedId ? ids.indexOf(selectedId) : -1;
    let target: number | null = null;
    if (event.key === "ArrowDown")
      target = Math.min(ids.length - 1, current + 1);
    if (event.key === "ArrowUp") target = Math.max(0, current - 1);
    if (event.key === "Home") target = 0;
    if (event.key === "End") target = ids.length - 1;
    const targetId = target === null ? undefined : ids[target];
    if (targetId) {
      event.preventDefault();
      void select(targetId);
    }
  };
  return (
    <section
      className="task-template-workbench"
      aria-labelledby="task-template-title"
    >
      <div className="context-section__heading">
        <div>
          <span>Local organization</span>
          <h2 id="task-template-title">Task Templates</h2>
        </div>
        <div className="task-template-workbench__actions">
          <button
            ref={createButton}
            type="button"
            onClick={(event) => {
              trigger.current = event.currentTarget;
              setDraft(blankDraft);
              setForm("create");
            }}
            disabled={busy || catalog?.state !== "ready"}
          >
            New local template
          </button>
          <button type="button" onClick={onClose}>
            Close templates
          </button>
        </div>
      </div>
      <p className="context-note">
        Inspectable local planning templates. They do not apply work, access
        project data, or control execution.
      </p>
      <p
        className="task-template-workbench__status"
        role="status"
        aria-live="polite"
      >
        {notice}
      </p>
      {catalog === null ? (
        <p role="status">Loading task templates…</p>
      ) : catalog.state === "unavailable" ? (
        <div role="alert" className="task-template-workbench__unavailable">
          {message(catalog.diagnosticCode)}
        </div>
      ) : (
        <>
          <p className="task-template-workbench__capacity">
            {catalog.capacity?.recordCount ?? 0} of{" "}
            {catalog.capacity?.countLimit ?? 64} templates ·{" "}
            {catalog.capacity?.canonicalBytes ?? 0} of{" "}
            {catalog.capacity?.canonicalByteLimit ?? 0} canonical bytes
          </p>
          {catalog.capacity?.warning && (
            <p className="task-template-workbench__warning" role="alert">
              Native capacity warning: template storage is nearing its limit.
            </p>
          )}
          {catalog.templates.length === 0 ? (
            <p className="context-empty">No task templates are available.</p>
          ) : (
            <div
              role="listbox"
              aria-label="Task templates"
              tabIndex={0}
              onKeyDown={listKeyDown}
            >
              <ul ref={list} className="task-template-workbench__list">
                {catalog.templates.map((entry) => (
                  <li key={entry.id}>
                    <button
                      type="button"
                      data-template-id={entry.id}
                      aria-current={
                        entry.id === selectedId ? "page" : undefined
                      }
                      onClick={() => void select(entry.id)}
                    >
                      <strong>{entry.title}</strong>
                      <span>
                        {entry.origin === "built-in"
                          ? "Built-in · read-only"
                          : `Local · ${entry.state}`}
                      </span>
                    </button>
                  </li>
                ))}
              </ul>
            </div>
          )}
        </>
      )}
      {selected && (
        <article
          className="task-template-workbench__detail"
          aria-labelledby="task-template-detail-title"
        >
          <div className="context-section__heading">
            <div>
              <p className="eyebrow">
                {selected.origin === "built-in"
                  ? "Built-in template · read-only"
                  : `Local template · ${selected.state}`}
              </p>
              <h3 id="task-template-detail-title">{selected.title}</h3>
            </div>
          </div>
          <dl>
            <div>
              <dt>Purpose</dt>
              <dd>{selected.purpose}</dd>
            </div>
            <div>
              <dt>Instructions</dt>
              <dd className="task-template-workbench__instructions">
                {selected.instructions}
              </dd>
            </div>
            <div>
              <dt>Version</dt>
              <dd>{selected.version}</dd>
            </div>
            <div>
              <dt>SHA-256</dt>
              <dd>
                <code>{selected.sha256}</code>
              </dd>
            </div>
            <div>
              <dt>State</dt>
              <dd>{selected.state}</dd>
            </div>
            <div>
              <dt>Origin</dt>
              <dd>{selected.origin}</dd>
            </div>
          </dl>
          <div className="task-template-workbench__actions">
            <button
              type="button"
              onClick={() =>
                handle &&
                void mutate(
                  () => operations.duplicate({ mutationHandle: handle }),
                  "Template duplicated into a local draft.",
                )
              }
              disabled={!handle || busy}
            >
              Duplicate
            </button>
            {selected.origin === "local" && selected.state === "active" && (
              <>
                <button
                  type="button"
                  onClick={(event) => {
                    trigger.current = event.currentTarget;
                    setDraft({
                      title: selected.title,
                      purpose: selected.purpose,
                      instructions: selected.instructions,
                    });
                    setForm("edit");
                  }}
                >
                  Edit
                </button>
                <button
                  type="button"
                  onClick={() =>
                    handle &&
                    void mutate(
                      () => operations.archive({ mutationHandle: handle }),
                      "Local template archived.",
                    )
                  }
                  disabled={!handle || busy}
                >
                  Archive
                </button>
              </>
            )}
            {selected.origin === "local" && selected.state === "archived" && (
              <>
                <button
                  type="button"
                  onClick={() =>
                    handle &&
                    void mutate(
                      () => operations.restore({ mutationHandle: handle }),
                      "Local template restored.",
                    )
                  }
                  disabled={!handle || busy}
                >
                  Restore
                </button>
                <button
                  type="button"
                  className="danger-button"
                  onClick={(event) => {
                    trigger.current = event.currentTarget;
                    setDeleting(true);
                  }}
                >
                  Delete permanently
                </button>
              </>
            )}
          </div>
        </article>
      )}
      {form && (
        <Dialog
          title={
            form === "create" ? "Create local template" : "Edit local template"
          }
          onCancel={() => setForm(null)}
          trigger={trigger}
        >
          <form
            className="task-template-workbench__form"
            onSubmit={(event) => {
              event.preventDefault();
              submit();
            }}
          >
            <label>
              Title{" "}
              <input
                value={draft.title}
                maxLength={80}
                onChange={(event) =>
                  setDraft({ ...draft, title: event.target.value })
                }
                required
              />
              <small>{draft.title.length}/80 characters</small>
            </label>
            <label>
              Purpose{" "}
              <input
                value={draft.purpose}
                maxLength={240}
                onChange={(event) =>
                  setDraft({ ...draft, purpose: event.target.value })
                }
                required
              />
              <small>{draft.purpose.length}/240 characters</small>
            </label>
            <label>
              Instructions{" "}
              <textarea
                value={draft.instructions}
                maxLength={32 * 1024}
                rows={10}
                onChange={(event) =>
                  setDraft({ ...draft, instructions: event.target.value })
                }
                required
              />
              <small>{draft.instructions.length}/32768 characters</small>
            </label>
            <button type="submit" disabled={busy}>
              Save local template
            </button>
          </form>
        </Dialog>
      )}
      {deleting && (
        <Dialog
          title="Delete local template permanently?"
          onCancel={() => setDeleting(false)}
          trigger={trigger}
          confirm={() => {
            if (!handle) return;
            setDeleting(false);
            void (async () => {
              setBusy(true);
              try {
                const outcome = await operations.delete({
                  mutationHandle: handle,
                  confirmation: "confirmed",
                });
                if (outcome.diagnosticCode)
                  setNotice(message(outcome.diagnosticCode));
                else {
                  setNotice("Local template deleted.");
                  setDetail(null);
                  setSelectedId(null);
                  await refresh(false);
                  requestAnimationFrame(() => createButton.current?.focus());
                }
              } catch {
                setNotice("Task templates are unavailable.");
              } finally {
                setBusy(false);
              }
            })();
          }}
        >
          <p>
            This permanently removes the archived local template. It cannot be
            undone.
          </p>
        </Dialog>
      )}
    </section>
  );
}
