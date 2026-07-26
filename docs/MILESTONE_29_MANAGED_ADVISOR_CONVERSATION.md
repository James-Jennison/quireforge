# Milestone 29 — Managed Advisor Conversation Foundation

Status: in progress.

## Objective

Add the smallest real Advisor conversation capability through the documented,
locally supervised Codex app-server while preserving the separate Advisor,
Approval/Dispatch, and Codex Execution boundaries.

## Implemented scope

- A dedicated Advisor conversation service owns one transient native turn at a
  time. It uses the existing managed ChatGPT browser-login state reported by
  Codex; it accepts no password, API key, browser cookie, external token, or
  consumer ChatGPT endpoint.
- The fixed profile has no cwd, environments, dynamic tools, integration,
  approval path, network access, terminal, Git, worktree, or project-write
  authority. Any unexpected tool, activity, plan, or permission request is
  blocked and reported as a bounded diagnostic.
- QuireForge persists only an opaque Codex thread reference and timestamps in
  the existing Advisor metadata table. Prompt text, response text, transcripts,
  credentials, project paths, project identity, and model/reasoning choices are
  not persisted by QuireForge.
- A selected Project State summary remains temporary. Including it in a message
  requires a second, per-send confirmation; Rust re-reads only the existing
  fixed local-only/metadata-only safe projection and sends only its closed
  trust/freshness/worktree/diagnostic-count summary.

## Explicit exclusions

This milestone does not add an Approval/Dispatch controller, editable Codex
prompt generation, a Codex dispatch bridge, stored Advisor transcripts,
model/reasoning selection, arbitrary project browsing, images, screenshots,
terminal access, Git actions, watchers, automatic retry, automatic handoff,
contradiction resolution, API-provider configuration, or repository mutation.

## Verification plan

Rust tests cover the fixed no-project app-server wire profile, managed-account
gate, bounded metadata, and safe selected-state input. TypeScript/Zod tests
reject thread identifiers and malformed requests at the bridge. Advisor UI and
shell tests cover the composer, managed-account fallback, explicit snapshot
selection, per-send context confirmation, and absence of execution controls.

The desktop bundle retains all shipped chunks in its budget calculation. The
reviewed JavaScript ceiling is 875 KiB for this milestone; the production M29
build measures 874.69 KiB. The final repository validation also covers Rust
formatting, locked checks/tests, warning-denying Clippy, TypeScript, ESLint,
Vitest, production builds, distribution checks, package-contract tests, the
unbundled Tauri build, and desktop/mobile Playwright accessibility coverage.

The final runtime gate requires the standard repository validation and a fresh
incremented pinned Ubuntu 22.04 Debian/AppImage package cycle before this
milestone can be recorded as complete.
