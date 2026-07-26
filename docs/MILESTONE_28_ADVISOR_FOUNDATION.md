# Milestone 28 — Reference-Only Advisor Foundation

Status: active implementation checkpoint.

## Objective

Establish the smallest safe local foundation for a future Advisor workspace and
Approval/Dispatch controller without adding an Advisor UI, model turn,
repository reader, project attachment, prompt dispatch, or autonomous action.

## Implemented checkpoint

The foundation introduces a strict version-1 Rust/TypeScript contract for:

- opaque Advisor conversation references owned by the supported Codex
  app-server;
- user-selected, closed context-reference kinds only: project-state, roadmap,
  current-state, and execution-report;
- trust, provenance, and freshness kept as separate values;
- reference-only dispatch proposals with prompt and context-manifest digests;
- explicit-approval-required drafts, approvals, and rejections.

The accompanying SQLite migration creates only bounded metadata tables. They
contain opaque references, closed source labels, commits, timestamps, and
SHA-256 digests. They deliberately contain no prompt body, response,
transcript, credential, token, arbitrary project path, or browser/session data.
The foundation exposes no Tauri command and executes no model, Git, terminal,
filesystem, or network operation.

## Boundaries

- Codex remains authoritative for account state, browser authentication,
  transcripts, and threads. QuireForge does not collect or retain credentials,
  cookies, API keys, or external tokens.
- A future UI may hold editable prompt text transiently, but this foundation
  persists only its SHA-256 digest. A decision to retain verbatim local Advisor
  text requires separately approved privacy, storage, and migration work.
- Context is represented as an explicit user selection. This checkpoint does
  not read project files, screenshot images, or attached-project paths, and it
  never transfers Codex attachment or execution authority into Advisor.
- A dispatch proposal is not a dispatch. No text is parsed for `Approve`,
  `Proceed`, or `Confirmed`; a later controller must require an explicit user
  action and separately confirm any model or reasoning change.
- Milestone 24D operational handoff/consistency behavior remains deferred.

## Verification

The shared `apps/desktop/fixtures/advisor-foundation.json` fixture is parsed by
strict Rust/Serde and TypeScript/Zod contracts. Tests reject unknown fields,
unsafe path-like references, non-digest proposal records, and a proposal that
does not require explicit approval. SQLite migration tests assert that the
application-owned schema excludes credential, session, prompt, transcript, and
content columns.

## Deferred work

The remaining Milestone 28 work is a separately reviewed, deterministic
foundation completion audit and its normal desktop validation/package decision.
No Advisor workspace, project-context reader, screenshot staging, managed
Advisor model call, prompt editor, approval UI, dispatch bridge, Python
sidecar, watcher, automatic handoff, contradiction resolution, or repository
write capability has been added.
