# Milestone 51 — Durable Task Records and Alternate-Plan Proposal

Status: decision-ready proposal. This document is the sole authoritative M51
proposal and does not implement runtime behavior, a migration, a package, or
`0.1.0-beta.46`.

## Decision

M52 should add a small, local-only task catalogue to QuireForge's existing
private SQLite metadata store. A task is an organizational record, not a
Codex thread, project, worktree, execution, approval, terminal, attachment,
artifact, browser, provider, connector, or Advisor authority. Each task has a
small user-controlled title, a closed organizational status, and one to four
inspectable user-controlled plan records. Plans are separate proposed approaches
to the same goal; they are never Git branches, worktrees, cloned sessions, or
agents.

The design deliberately retains no conversation text. Existing Codex session
references remain governed by their established metadata contract, the Advisor
viewport remains transient, and the M43 handoff remains one-use/transient. M52
creates no implied linkage from a task or plan to any of those systems.

## Closed identity and record schema

### Scope, IDs, ownership, and reset

The native `TaskRecordService`, owned by the Rust application core, creates
opaque UUIDv7 identifiers with `Uuid::now_v7()`:

- Task IDs and plan IDs are canonical lowercase hyphenated UUIDv7 strings
  (36 ASCII characters). They are generated natively, never supplied by the
  frontend, and are globally unique within the local QuireForge profile.
- The profile is the OS user and QuireForge application-data directory that
  owns `metadata.sqlite3`; IDs do not claim global, account, provider, or
  cross-device identity. There is no import/export or synchronization in M52.
- IDs encode no path, worktree, username, account, provider, terminal, Git
  reference, credential, or task content. A collision is a fatal local
  `duplicate-id` failure: retry native generation before the transaction; do
  not merge or overwrite a row.
- Tasks are created only by an explicit user action. Creation atomically creates
  the task and one selected primary plan. Resetting a task surface clears only
  transient UI and capability state; it neither creates nor deletes a durable
  task. Explicit deletion removes the record as defined below.
- Unsupported newer task schema data is unavailable read-only; it is never
  downgraded, rewritten, or guessed. There is no current-state import: existing
  transient conversation, Advisor, attachment, artifact, review, approval,
  dispatch, execution, terminal, Git, and project state stays transient and is
  not converted to a task.

### Exact tables (schema version 1)

M52 adds migration 11, `durable-task-records-v1`, to the existing migration
array. `schema_version` is a record wire-contract version, fixed at `1` in
both tables. Times are non-negative Unix milliseconds generated natively.

```sql
CREATE TABLE task_records (
  id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
  schema_version INTEGER NOT NULL CHECK(schema_version = 1),
  title TEXT NOT NULL CHECK(length(title) BETWEEN 1 AND 120),
  status TEXT NOT NULL CHECK(status IN ('active', 'paused', 'completed')),
  created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
  updated_at_ms INTEGER NOT NULL CHECK(updated_at_ms >= 0),
  archived_at_ms INTEGER CHECK(archived_at_ms >= 0),
  last_opened_at_ms INTEGER CHECK(last_opened_at_ms >= 0),
  selected_plan_id TEXT NOT NULL CHECK(length(selected_plan_id) = 36)
);

CREATE TABLE task_plans (
  id TEXT PRIMARY KEY NOT NULL CHECK(length(id) = 36),
  schema_version INTEGER NOT NULL CHECK(schema_version = 1),
  task_id TEXT NOT NULL CHECK(length(task_id) = 36)
    REFERENCES task_records(id) ON DELETE CASCADE,
  label TEXT NOT NULL CHECK(length(label) BETWEEN 1 AND 80),
  position INTEGER NOT NULL CHECK(position BETWEEN 0 AND 3),
  body TEXT NOT NULL CHECK(length(body) <= 8192),
  created_at_ms INTEGER NOT NULL CHECK(created_at_ms >= 0),
  updated_at_ms INTEGER NOT NULL CHECK(updated_at_ms >= 0),
  UNIQUE(task_id, position)
);

CREATE INDEX task_records_visible_recent
  ON task_records(archived_at_ms, updated_at_ms DESC, id);
CREATE INDEX task_records_status_recent
  ON task_records(archived_at_ms, status, updated_at_ms DESC, id);
CREATE INDEX task_plans_task_position ON task_plans(task_id, position, id);
```

The service validates that `selected_plan_id` names a plan belonging to the
task in the same immediate transaction; SQLite does not add a circular foreign
key. The first plan is `{position: 0, label: "Primary plan", body: ""}`.
`body` is optional planning text represented as the empty string, not a hidden
conversation or model context. There are no safe-summary, path, deletion,
content-index, authority, attachment, artifact, or conversation columns.

The record sizes include Unicode UTF-8 byte caps enforced before SQLite:
120 title characters/480 bytes, 80 label characters/320 bytes, 8,192 plan-body
characters/32 KiB, 48 KiB serialized maximum for one task plus all its plans,
and 8 MiB aggregate task/plan payload. Database pages, indexes, and unrelated
QuireForge metadata do not count toward that 8 MiB application limit.

## Titles, status, and search

### Titles

The default title is **Untitled task**. Only an explicit user rename may change
it. Model-generated titles are not allowed in M52; a future proposal may allow
a visible suggestion only after the user explicitly accepts it.

Titles and plan labels normalize CR/LF/TAB and Unicode whitespace to one ASCII
space, trim ends, reject C0/C1 and bidirectional-format controls, and enforce
the limits above after normalization. Empty input is rejected and leaves the
existing title intact; creation uses the default rather than an empty title.
Duplicate titles are permitted because identity is the opaque task ID. The UI
exposes the full normalized title as the accessible name, never a path-derived
fallback, and announces successful rename or validation failure through a
short status message.

### Closed status model

`active`, `paused`, and `completed` are the complete task status enumeration.
Archive is a separate visibility/lifecycle state (`archived_at_ms`), preserving
the organizational status.

| From | Explicit allowed transition | Not allowed |
| --- | --- | --- |
| active | paused, completed | implicit completion or archive-as-deletion |
| paused | active, completed | implicit resume |
| completed | active (explicit reopen) | automatic reopen |

Any status may be explicitly archived; an archived task cannot have its status
or plans edited until restored. Restore clears `archived_at_ms`, retains the
status, and does not reopen a completed task. Status changes are organizational
only: they grant, restore, clone, consume, cancel, or imply no execution,
approval, dispatch, attachment, artifact, terminal, Git, project, browser,
connector, or Advisor authority.

### Bounded local search

Search queries normalized with the title rules are 1–120 characters; an empty
query lists records. Search matches case-insensitively by Unicode simple
case-folding against task title and plan label only. It does not index plan body,
conversation, safe summaries, paths, attachment/artifact metadata, activity,
or hidden content. Results are at most 50, ordered unarchived before archived,
then `updated_at_ms DESC, id ASC`; archived records are excluded by default and
included only by an explicit archived filter. Search is synchronous on explicit
input/change, uses no embedding, cloud, connector, browser, daemon, background
index, or retrieval system. Invalid/corrupt rows are omitted, surface one
bounded unavailable warning, and never cause a broad scan or silent repair.

## Archive, retention, and deletion

Archive and restore are explicit native actions. Archive hides a task from the
default list, clears task selection to the next visible task or the empty state,
and returns focus to the task list. An open task therefore visibly closes; no
background plan, retrieval, saving, dispatch, approval, or execution starts.
Archived tasks remain searchable only with the archived filter, read-only, and
retain their small task/plan rows. Attachments and generated artifacts retain
their existing transient expiry and are neither retained nor attached to an
archived task. Existing pending approvals, dispatch, execution, terminal, and
save reservations remain their own process-lifetime systems; archive neither
changes nor preserves them. Restore is explicit, focuses the restored task only
after user selection, and revives no expired, disposed, consumed, completed
transient authority.

The exact retention policy is: at most **200 retained tasks**, **four plans per
task**, **48 KiB per task**, and **8 MiB aggregate task/plan payload**. Active
and paused tasks have no time expiry. Completed and archived tasks become
"eligible for cleanup" 180 days after their most recent update (or archive,
whichever is later), but are never automatically deleted or evicted. At 160
tasks or 6 MiB, the UI warns with a count/size indicator; at either hard limit,
create and plan-add actions refuse with a clear cleanup action. Limits never
auto-expand and oldest-task eviction is prohibited. A task record is included
in the count even when archived.

There are no cloud backups, account sync, or automatic snapshots. The ordinary
OS backup policy may capture the local database as it does other application
data; QuireForge offers no restore/import flow in M52. A malformed/corrupt
record is unavailable and consumes no replacement identity; recovery is from a
user's external system backup only, never from hidden task logs.

Deletion is immediate, explicit, and irreversible at the application layer;
there is no trash state or tombstone. The confirmation must name the normalized
task title, state that all up-to-four plan records will be removed, and state
that external project files, worktrees, Git history, package evidence,
repository source, and user-saved artifacts will not be changed. The native
service deletes `task_records` in one immediate SQLite transaction, relying on
the plan foreign-key cascade, and clears in-memory selection after commit.

Deletion does not request or delete attachment bytes/claims, generated-artifact
bytes/entries/reservations/receipts, approvals, dispatch records, execution
references, terminal sessions, Codex thread metadata, or external files; none
is owned by a task record. It creates no audit record, because an audit/tombstone
would retain task metadata and undermine deletion. SQLite/SSD journaling and
ordinary backups cannot promise cryptographic secure erasure; the UI and docs
must state that physical recovery may remain subject to filesystem, journal,
and backup behavior. A failed or interrupted transaction rolls back and leaves
the task intact; after restart, the service reloads the database and never
recreates a committed-deleted row.

Plan deletion is similarly immediate after confirmation when it would delete
the selected plan or the final remaining plan. A task must retain one plan, so
the last plan has no delete action; delete the task instead. Deleting the
selected non-final plan selects the lowest remaining position atomically.

## Alternate-plan contract and authority isolation

An alternate plan is a separate, inspectable, user-created implementation
approach, strategy, step sequence, or branch of planning thought for the same
task. A plan's durable label and body are shared organizational metadata only.
Plans share the task title, status, archive state, timestamps, retention limits,
and task-level selection; they do not share mutable authority or conversation.

Creation is explicit and atomic, assigns the next position (0–3), defaults to
`Alternate plan N` and an empty body, and selects it only after creation.
Rename/edit are explicit and subject to the title/body bounds. Reordering is
not in M52: creation order is the stable displayed order. There is no automatic
duplication. A user may explicitly use **Copy primary plan text** when creating
an alternate; it copies only the bounded visible plan body into the new draft,
not its identity, history, authority, or anything else. Model content can enter
a plan only through a future explicit, visible user save action; M52 has no
model-to-plan transport.

Plan selection updates `selected_plan_id`, `last_opened_at_ms`, and
`updated_at_ms` atomically. It does not require confirmation because it grants
no authority, but the existing Advisor/QuireForge workspace boundary
confirmation remains unchanged whenever a separate mode switch is requested.
Selection or switch failure leaves the current plan selected and announces a
bounded error. A deleted or missing selected plan is repaired only by choosing
the lowest extant plan in a transaction; if none exists the whole task is
invalid/unavailable, not silently recreated.

The following table is normative. `Clear` means remove process/UI transient
state on switch; `exclude` means it is never in either schema; `task-global`
means an independent existing subsystem may remain available in the workbench
but is not a plan claim and is never selected, copied, or re-bound by a switch.

| State | M52 rule |
| --- | --- |
| Approvals, digests, receipts | Clear visible pending plan presentation; exclude from records; existing approvals remain exact conversation/turn-scoped and cannot be decided through a plan. |
| Advisor dispatch and execution IDs/state | Clear and exclude. No plan switch dispatches or resumes anything. |
| Terminal sessions and contents | Clear any task-plan presentation; terminal registry remains task-global existing UI only, with no plan binding or cloned contents. |
| Git mutation authority | Exclude; existing project-scoped authority is unaffected and never implied by task/plan status or selection. |
| Attachments, claims, bytes, expiry | Clear current attachment tray selection on switch; exclude from records; native attachment service remains independently transient. |
| Generated artifacts, bytes, reservations, receipts | Clear task-plan presentation; exclude; M48 registry remains process-local and does not gain a task/plan key. |
| Browser, connector sessions, cookies, credentials, model tools | Exclude and never read or change them. |
| Advisor hidden context/model memory | Exclude. Advisor retains only its existing opaque reference metadata; no hidden plan context exists. |

Switching never triggers execution, retrieval, saving, dispatch, approval,
transport, an attachment/artifact operation, or a Codex/Advisor request. It
does not create a Git branch or worktree. It also does not clone an existing
Codex conversation. The selected plan body is the only durable plan-specific
content.

## Conversation, evidence, panes, and layout

M52 retains no task or plan conversation. The current visible QuireForge
conversation remains the dominant surface and existing Codex-owned history is
accessed only through its established session controls; it is not assigned to a
task plan. Advisor remains a separate no-project mode with a transient visible
viewport; returning to a plan shows the durable title, plan body, and an honest
empty/unavailable conversation state, not reconstructed messages. No task or
plan can inherit approved Project State or any Advisor context. A future
explicit handoff must have its own decision gate and cannot be inferred here.

M44–M46 attachments and M48 artifacts remain process-local/transient. Task
records retain **no historical attachment/artifact metadata**, including type,
name, hash, path, existence, expiry, receipt, or count. A saved artifact stays
an external user-selected file and is never task-owned. Review panes preserve
their M49 read-only sources: Files, Diff, Git, Preview, Activity, and Approval
selection is the M50 application-local layout preference, not task- or
plan-specific. Closing/opening a task or plan never loads a pane. Activity
remains bounded existing presentation, not a task audit log; Approval remains
non-durable exact existing authority.

M50's `quireforge-workbench-layout` stays a separate 512-byte browser-local
presentation record. It may retain only review width, terminal height, and
selected review pane. It must never receive task/plan IDs, titles, body,
content, paths, approval/execution/terminal state, or any capability data.

## Persistence, migration, privacy, and failures

The implementation extends `ProjectRepository`/`ProjectService` rather than
introducing a second engine or database. Rust owns ID generation, validation,
transactions, filtering, size accounting, and all SQLite access; fixed typed
Tauri commands expose only opaque IDs and bounded task/plan DTOs. TypeScript
uses strict Zod schemas (`.strict()`) for every response/request and never
accepts paths or capability-bearing fields. Native validation repeats all UI
rules and rejects unknown fields/enums.

Use the existing mode-0700 parent and mode-0600 `metadata.sqlite3` boundary,
`foreign_keys=ON`, `trusted_schema=OFF`, five-second busy timeout, and
`TransactionBehavior::Immediate`. Add migration 11 to the ordered migration
array and table checks to `verify_schema`; run it at ordinary startup in the
same all-or-nothing transaction as current migrations. List/search load only a
maximum 50-row projection; a selected task lazily loads its maximum four plans.
No background scanning, compaction, indexing daemon, vacuum, or cleanup runs.
SQLite provides atomic commit/rollback and crash recovery; a lock timeout,
disk-full, permission error, invalid row, schema mismatch, or transaction
failure returns a closed stable diagnostic and makes no partial change.

On a first M52 launch, an existing profile has no rows and displays the empty
state. No current active/transient state is imported. Migration failure leaves
the prior database unmodified by transaction rollback and marks task records
unavailable; existing unrelated metadata must continue only if safely opened,
otherwise the application follows its existing unavailable-metadata posture.
An application downgrade seeing migration 11 refuses task-record access as
future schema rather than modifying the database. Test fixtures cover empty
pre-M52 migration, every migration version, future schema, partial migration,
and deterministic UUID/time injection.

This is local-only organizational storage. It stores no credentials, provider
or account data, browser state, cookie, hidden telemetry, raw prompt/reply,
hidden system prompt, model context, shell/terminal history, commands, Git
credentials, project/worktree paths, full source/destination paths, approval
secret/digest/receipt, dispatch token, execution handle, unrestricted log,
attachment/artifact content, or support-bundle content. Diagnostics contain
closed codes only, never title/body text or database paths. Corrupt databases
are not auto-rebuilt, overwritten, uploaded, or exposed to support bundles.
The relevant threat model is local-database disclosure/tampering, stale UI,
resource exhaustion, accidental task deletion, migration incompatibility, and
authority confusion; permissions, minimization, exact IDs, explicit actions,
bounded sizes, transactional operations, and fail-closed rendering mitigate
those threats. This does not defend against a local attacker who can read the
user's private application-data directory or backups.

## Minimal M52 UI and accessibility

Place a compact task switcher/list in the existing QuireForge workbench context
area, with the selected task's compact plan strip beside the dominant
conversation—not a dashboard, board, or project-management workspace. It
provides create, title rename, status selection, bounded search, archived
filter, archive/restore, delete confirmation, plan create/switch/edit/delete,
capacity indication, and explicit empty/error/migration-unavailable states.

The list is semantic navigation with labelled search; task and plan controls
are keyboard-operable with visible focus. Create/rename place focus in the
corresponding field; archive/delete return focus to the list or opening control;
dialogs trap then restore focus; switch never steals focus from an unrelated
composer. Status, save, capacity, and error feedback use a concise live status
region and never color alone. Narrow layouts stack the list and plan strip
above/below the conversation with no horizontal overflow; zoom/reflow and
reduced-motion retain the existing rules. Screen-reader labels identify title,
status, archive visibility, selected plan, plan count, and destructive actions.

## Required deterministic M52 tests

M52 must add deterministic Rust and TypeScript/React tests, with injected
clock/UUID fixtures where needed, for:

1. strict schemas, unknown-field/path rejection, opaque UUIDv7 generation,
   duplicate-ID retry/refusal, title/label/body limits and normalization;
2. migration from no-task records, ordered migration integrity, future-schema
   refusal, failed/partial migration rollback, upgrade/restart, and downgrade
   no-write behavior;
3. task/plan CRUD, one primary plan, selected-plan integrity, status transition
   table, archive/restore, completed reopen, and last-opened ordering;
4. title/label-only case-insensitive search, query/result bounds, ordering,
   archived filtering, corrupt row handling, and absence of body/transcript
   indexing;
5. 200-task, four-plan, per-task, and aggregate capacity refusal with no silent
   eviction or automatic deletion; 180-day cleanup eligibility display;
6. destructive confirmation, atomic task/plan deletion, rollback on injected
   failure, interrupted deletion recovery, and proof external files, Git state,
   Codex metadata, attachments, artifacts, approvals, dispatch, execution, and
   terminals are unchanged;
7. plan creation, explicit visible-body copy only, switch/delete failure,
   plan capacity, selected-plan fallback, and no automatic model save;
8. authority isolation proving no approval/digest/receipt, dispatch/execution
   state/ID, terminal/content, Git authority, attachment claim/bytes/expiry,
   artifact entry/bytes/reservation/receipt, browser/connector/credential, or
   hidden Advisor context is cloned, inherited, or activated;
9. Advisor/QuireForge boundary confirmation is unchanged; no cloud/provider,
   retrieval, transport, model request, or hidden context inheritance occurs;
10. keyboard, screen-reader names, dialogs/focus restoration, live feedback,
    zoom/reflow, narrow layout, reduced motion, empty/error/capacity/migration
    states, and no focus theft; and
11. SQLite permissions, diagnostic redaction, no support-bundle inclusion,
    lock contention, disk-full/permission failure, stale task/plan selection,
    missing referenced data, and restart crash behavior.

## Exact M52 acceptance criteria — target `0.1.0-beta.46`

1. Implement only this document's schema-version-1 local task catalogue in the
   existing private `metadata.sqlite3` through migration 11; do not add another
   database, sync, provider, connector, browser, shell, terminal, Git, project,
   execution, approval, dispatch, or Advisor capability.
2. Native code owns UUIDv7 task/plan IDs, validation, SQLite access, immediate
   transactions, migrations, capacity accounting, and closed diagnostics;
   TypeScript/Zod mirrors only the bounded typed DTOs and rejects unknown or
   path/capability-bearing data.
3. Create exactly the `task_records` and `task_plans` tables, indexes, limits,
   default title/primary plan, status enum/transitions, search contract,
   archive/restore, deletion semantics, and retention policy specified above.
4. Retain at most 200 tasks, four plans/task, 48 KiB/task, and 8 MiB aggregate;
   warn at 160 tasks/6 MiB, refuse at capacity, never auto-delete/evict/expand,
   and display 180-day completed/archived cleanup eligibility.
5. Store only title, organizational status/timestamps, selected-plan identity,
   bounded label/order/body, and required schema/foreign-key fields. Do not
   persist a transcript, path, attachment/artifact history, authority, account,
   credential, provider/browser data, terminal/Git data, hidden Advisor state,
   prompt, log, or support-bundle task content.
6. Render the accessible compact workbench task list and plan strip, preserving
   one dominant conversation surface and all listed accessibility/focus/narrow
   layout behavior. Review-pane/layout preferences remain application-local.
7. On every plan switch, clear/exclude the normative isolation table state; do
   not clone, inherit, restore, trigger, or imply approvals, dispatch,
   execution, terminals, Git authority, attachments, artifacts, browser,
   credentials, Advisor context, model tools, saving, or transport.
8. Add and pass all deterministic tests in the preceding section, including
   migration, failure/rollback, restart, deletion non-effects, isolation, and
   accessibility tests; routine tests require no model call or authorization.
9. Update README status if user-visible task behavior is added, `CURRENT_STATE`,
   `ROADMAP`, `CHANGELOG`, `ARCHITECTURE`, `THREAT-MODEL`, relevant fixtures,
   and tests. Run `python3 scripts/validate_repository.py` and applicable
   desktop formatting/lint/test checks.
10. Only after implementation and validation, set every package source and
    package contract to `0.1.0-beta.46`, perform the repository-required
    package/evidence lifecycle, record immutable Debian/worker evidence bound
    to the clean commit, and commit/push normally. M51 itself changes none of
    those package sources or evidence.
