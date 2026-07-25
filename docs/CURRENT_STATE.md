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

The corrected UI was packaged from `3d1280846eeacfbd226e4abeb5990844ffe27fa7`
through the existing digest-pinned Ubuntu 22.04 container workflow on
2026-07-25. `quireforge_0.1.0.beta.2_amd64.deb` (4,474,376 bytes;
`b1b96cfacc1996085db12eaf21587d92a0b703d814b1b39bd7b171cfcf208f40`) and
`QuireForge-0.1.0-beta.2-x86_64.AppImage` (83,651,064 bytes;
`91300b3a5f369187f6831faf4c9ca0ed678ec53f628dc4f2ff2bc60937b2cd26`) are
ignored local artifacts in `target/ubuntu-22.04/release/packages/`. Both passed
the official manifest/checksum, GLIBC (maximum `2.34`), disposable Debian
lifecycle, and visible X11 launch checks on the Ubuntu 22.04 baseline. The
component suite verifies the corrected unavailable shared-usage sidebar and
runtime-meter details; package launch testing verifies the built executable.

For a fresh thread, read in this order:

1. `AGENTS.md`
2. `docs/CURRENT_STATE.md`
3. The active roadmap entry
4. The relevant ADR
5. Only files within the approved scope

Do not perform full-repository rescans, create a duplicate master prompt,
develop Tauri and Qt features in parallel, or use Fast mode or subagents
without explicit justification.
