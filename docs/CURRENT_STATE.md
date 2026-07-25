# Current State

## Identity and platform status

QuireForge is an unofficial Linux workspace for Codex: “Build boldly. Work
locally.” Tauri + React + TypeScript is the current functional prototype. The
long-term UI-platform decision is pending; no Qt migration has been selected.

- **Branch:** `feat/milestone-22-routed-desktop-workspace`
- **Checkpoint:** `a1c4249974429571ff390904d81864af99b29cf7`
- **Milestone 22:** complete, committed, documented, and validated.
- **Host readiness:** Qt 6.10.2/QML tooling is installed on this host only.

No Qt frontend, Qt migration, CXX-Qt/Rust bridge, or native Windows/macOS
portability work has started.

## Reuse and boundary

The Rust Codex, project/SQLite, Git, worktree, terminal, preview, attachment,
settings, and integration services are candidates for reuse. The current Tauri
boundary is the command façade, app/plugin wiring, native dialogs/openers/
notifications, and one native drop-capture path. React/TypeScript/Vite remains
the current presentation layer and calls that façade through its bridge.

## Next action

The next action is the read-only **Milestone 23 — UI Platform Feasibility
Decision**. It must produce evidence for ADR 0028 before any platform decision
or implementation work.

### Maintenance handoff

The focused `fix/sidebar-codex-usage-window` branch now keeps Codex runtime
meters distinct from shared account usage. A redacted live app-server capture
on Codex CLI 0.145.0 showed `rateLimits` and `rateLimitsByLimitId` meter fields,
but no explicit shared-account ownership field. Consequently the compact
sidebar displays shared usage as unavailable, while the full panel preserves
every upstream-reported runtime meter with `Scope not verified`. It never
selects a meter by duration, name, ID, kind, or order, and refresh failures
clear current values instead of showing stale data. No Qt work is included.

The same branch has verified local beta-2 package candidates through the
existing digest-pinned Ubuntu 22.04 container workflow. Both the Debian and
AppImage candidates passed manifest/checksum, GLIBC, disposable lifecycle, and
visible X11 launch checks; their source commit and hashes remain in the ignored
`target/ubuntu-22.04/release/packages/` manifest rather than source control.

For a fresh thread, read in this order:

1. `AGENTS.md`
2. `docs/CURRENT_STATE.md`
3. The active roadmap entry
4. The relevant ADR
5. Only files within the approved scope

Do not perform full-repository rescans, create a duplicate master prompt,
develop Tauri and Qt features in parallel, or use Fast mode or subagents
without explicit justification.
