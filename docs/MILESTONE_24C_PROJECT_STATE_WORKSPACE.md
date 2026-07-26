# Milestone 24C — Project State Workspace

Status: implementation validation complete on
`feat/milestone-24c-project-state-workspace`; final pinned Ubuntu package
evidence remains required.

## Objective

Present the existing Milestone 24B repository-state snapshot as a clear,
read-only workspace for the currently attached project. The route makes
repository, milestone, policy, validation, package, handoff, and diagnostic
evidence visible without changing its source or interpreting reported evidence
as approval.

## Approved boundary

The workspace consumes only the existing version-1
`repository_state_read` response. It requests:

- the already selected QuireForge project ID;
- `local-only` remote behavior; and
- `metadata-only` artifact behavior.

It is demand-driven when the route opens and may be refreshed explicitly by the
user. It does not fetch, watch, scan in the background, accept a path, run
validation, inspect local package artifacts, write project metadata, modify Git,
approve policy, resolve diagnostics, generate a handoff, or repair evidence.

Milestone 24D remains responsible for any separately approved handoff
generation or operational consistency rules. Canonical policy and persistence
ownership are unchanged.

## Presentation

The persistent desktop shell adds a **Project state** destination under the
Workspace navigation group. Its content:

- identifies the active project and normalized worktree state;
- shows current branch or detached state, local HEAD, tracking counts, and
  observed trust;
- presents the active milestone and the recorded owner, merge, and release
  approval decisions without offering approval controls;
- inventories validation, package, and reported handoff evidence;
- renders reader diagnostics and their inferred next actions without resolving
  them; and
- identifies browser preview, loading, missing-project, native, and read-error
  states honestly.

The existing context inspector displays only snapshot provenance, trust, and
diagnostic count. The workspace uses established responsive route components
and shell navigation instead of introducing another router or visual system.

## Contract and security

Rust remains the owner of attached-project identity, filesystem access, Git
inspection, evidence parsing, provenance, and diagnostics. TypeScript validates
the returned closed snapshot before presentation. React receives no arbitrary
path or Git argument and exposes no remote or mutation request.

Reported Markdown remains reported. Verified Git evidence remains verified.
Freshness remains independent from trust. The UI does not reinterpret unknown,
stale, or conflicting evidence and does not expose credential, token, raw
transcript, or arbitrary-file surfaces.

## Verification plan

The implementation gate requires:

- focused React route, component, bridge-request, navigation, and state tests;
- desktop and mobile Playwright evidence;
- axe accessibility and overflow checks;
- repository formatting, ESLint, TypeScript, and complete frontend tests;
- production frontend build and unchanged bundle-budget enforcement;
- Rust formatting, warning-denying Clippy, locked workspace tests, and Tauri
  compilation because the route ships in the desktop application;
- repository/document validation; and
- fresh pinned Ubuntu 22.04 Debian and AppImage lifecycle, launch, and smoke
  evidence from the clean implementation commit.

## Implementation evidence

The implementation gate passes:

- TypeScript and ESLint;
- 183 frontend unit/component tests across 36 files;
- 40 desktop/mobile Playwright scenarios, including the focused Project state
  route, axe, keyboard, responsive drawer, visual capture, and overflow checks;
- production frontend build and distribution budgets: 189.16 KiB entry
  JavaScript, 231.58 KiB application JavaScript, 845.34 KiB total JavaScript,
  and 99.03 KiB total CSS against the unchanged 100 KiB CSS limit;
- repository and package-contract validation plus the full repository formatter;
- warning-denying workspace Clippy;
- locked Rust workspace tests: 192 passed and 3 deliberate live probes ignored;
  and
- the unbundled optimized Tauri application build.

Final completion remains package-gated because the new route changes the
installed application.

## Deferred work

This milestone adds no repository reader behavior, fetch UI, background scan,
watcher, project-state edit, automatic approval, generated handoff,
contradiction resolution, autonomous repair, persistence migration, Qt work,
merge, release, or publication.
