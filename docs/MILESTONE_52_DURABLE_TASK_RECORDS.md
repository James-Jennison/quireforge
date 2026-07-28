# Milestone 52 — Durable Task Records and Alternate Plans

Status: complete at source commit
`6df055999d2ad01d2385096a14bc71f8aada2a8c`.

M52 implements the separately approved
[M51 proposal](MILESTONE_51_DURABLE_TASK_RECORDS_ALTERNATE_PLAN_PROPOSAL.md)
as a small local organizational catalogue. A task is not a Codex conversation,
project, worktree, execution, approval, terminal, attachment, artifact,
browser, provider, connector, credential, or Advisor authority. The package
candidate is `0.1.0-beta.46`.

## Native persistence contract

Migration 11 adds exactly `task_records`, `task_plans`, and the approved
indexes to the existing private `metadata.sqlite3`. There is no second
database. Rust owns:

- canonical opaque UUIDv7 task and plan IDs, with bounded collision retry;
- title and label Unicode-whitespace normalization, UTF-8/code-point limits,
  visible plan-body validation, and bidirectional-format-control rejection;
- one atomic default `Untitled task` plus `Primary plan` creation;
- the closed active/paused/completed transition table, archive/restore,
  selected-plan repair, plan-position compaction, and irreversible row
  deletion;
- title/plan-label-only Unicode case-insensitive search, a 50-result projection,
  archived filtering, and deterministic 180-day cleanup eligibility;
- immediate SQLite transactions, rollback, corruption omission, closed
  diagnostics, and UTF-8-byte capacity accounting.

The fixed capacity contract is 200 tasks, four plans per task, 48 KiB per task,
and 8 MiB aggregate. The UI warns at 160 tasks or 6 MiB and refuses additions
or edits at capacity without eviction, automatic cleanup, or partial mutation.

Only the approved schema fields are durable: title, organizational status and
timestamps, archive/last-opened timestamps, selected-plan identity, and
bounded plan label/order/body. Task storage has no project, conversation,
session, transcript, path, attachment/artifact, approval, dispatch, execution,
terminal, Git, browser, connector, credential, provider, account, prompt, log,
or Advisor field.

## Workbench and isolation contract

The opt-in workbench context contains a semantic task navigation list and the
selected task's compact plan strip. It supports create, explicit title/plan
save, status change, search, archived filtering, archive/restore, confirmed
task/plan deletion, capacity feedback, and honest empty/unavailable states.
Keyboard plan navigation, labelled controls, polite live feedback, destructive
dialog focus trap/restore, creation focus, read-only archived plans, narrow
stacking, reduced motion, and visible focus preserve the existing accessibility
rules. The conversation remains the dominant workspace.

Switching a plan first clears the current transient conversation-attachment
selection through the existing bounded cancellation service. If that clear
fails, selection does not proceed. Approvals, digests, receipts, Advisor
dispatch/execution state, terminal contents, Git mutation authority, generated
artifacts, browser/connector sessions, credentials, model tools, hidden Advisor
context, conversation history, and review/layout state are never represented
by task records and are neither cloned nor activated. A plan switch performs
no model request, retrieval, save, dispatch, approval, execution, transport,
Git/worktree action, or conversation restoration.

## Deterministic validation

Routine tests require no model call, billable provider request, browser login,
connector authorization, or external project mutation. The M52 suite covers:

- ordered migration, failed migration rollback, future-schema refusal,
  exact-schema/restart persistence, private database permissions, and
  corruption omission;
- UUIDv7 collision retry/refusal, strict Rust/Zod requests and responses,
  unknown/path/capability field rejection, normalization, text limits, and a
  sanitized shared fixture;
- task/plan CRUD, status lifecycle, archive/restore, selected-plan repair,
  body-copy rules, title/label-only Unicode search, archived filtering,
  cleanup timing, and no body indexing;
- 200-task, four-plan, 48-KiB task, and 8-MiB UTF-8 aggregate refusal without
  eviction or partial edits;
- cascade isolation, external-file preservation, injected deletion rollback,
  and recovery after the injected failure;
- semantic names, keyboard plan switching, live status, focus trap/restore,
  creation focus, read-only archives, empty/unavailable rendering, and
  attachment-clear-before-plan-select ordering.

The final full repository gate passed with 15 package tests, 57 desktop test
files and 298 frontend tests, seven website tests, 272 native library tests
(269 passed and three documented manual probes ignored), two sandbox-worker
tests, strict formatting/lint, TypeScript/Astro checks, production builds,
distribution budgets, Cargo check, Clippy with warnings denied, and
repository/package contract validation.

The browser gate covered 48 desktop and eight website checks. All M52 desktop
and mobile workbench, accessibility, keyboard, dialog, and overflow checks
passed. One unchanged desktop screenshot-tour check timed out at 30 seconds
while the full browser suite and repository gate competed for host resources;
the exact check passed independently without a source change in 11.1 seconds.
The combined verified browser result is therefore all 56 checks passing, with
that resource-contention rerun recorded rather than concealed.

## Package evidence

The authoritative pinned Ubuntu 22.04 package workflow ran from clean source
commit `6df055999d2ad01d2385096a14bc71f8aada2a8c` and passed build, package
contract, install/upgrade/remove/reinstall lifecycle, AppStream, desktop/icon,
visible-launch, sandbox-worker, provenance, ABI, and final artifact validation.
The builder was
`ubuntu:22.04@sha256:0e0a0fc6d18feda9db1590da249ac93e8d5abfea8f4c3c0c849ce512b5ef8982`.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `quireforge_0.1.0.beta.46_amd64.deb` | 5,537,352 | `3b32fd097c9b5a7bcc7eae62c0133bd9bdc17f834954128c0b4f73adf7db1ad6` |
| `quireforge-sandboxd_0.1.0.beta.46_amd64.deb` | 3,233,384 | `b51734bdd458aada3d8d870ba03218fd25f7a8229325ba7ae9f66ea338a90c29` |

Both packages require at most `GLIBC_2.34`, within the pinned Ubuntu 22.04
`GLIBC_2.35` ceiling. The worker retains Firecracker/jailer 1.15.1, Linux
6.1.178, disabled guest networking, no guest mounts, and the
static-ELF64-x86_64-only runtime.

The final desktop distribution contains 975,149 bytes of JavaScript and
114,186 bytes of CSS. Its 194,984-byte startup entry and 333,785-byte
application shell remain within the 262,144-byte and 458,752-byte ceilings.
The total JavaScript and CSS remain within the 1,572,864-byte and 163,840-byte
temporary construction ceilings.

The restricted installed-host validator accepted the exact desktop artifact
and upgraded the host package from beta.45 to `quireforge 0.1.0~beta.46`.
An initial wrapper call without its required package-path argument exited with
usage status 64; the corrected exact-artifact invocation passed without
weakening or bypassing the validator.
