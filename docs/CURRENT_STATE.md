# Current State

## Identity and platform status

QuireForge is an unofficial Linux workspace for Codex: “Build boldly. Work
locally.” Tauri + React + TypeScript is the current functional prototype. The
long-term UI-platform decision is pending; no Qt migration has been selected.

- **Branch:** `feat/milestone-22b-visual-workspace-refinement`
- **Checkpoint:** Milestone 22B is complete locally; Milestone 22 remains complete.
- **Milestone 22B:** the routed desktop architecture was refined without
  changing route, native-bridge, authentication, or account-data ownership.
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

Milestone 22B — **Visual Workspace Refinement** is complete on
`feat/milestone-22b-visual-workspace-refinement`. The completed checkpoints
consolidate responsive shell CSS, preserve reduced-motion behavior, refine
shell hierarchy, add Home's current-workspace surface, and establish shared
header rules for New task, Threads, Projects, Changes, Worktrees, and Terminal.
Scheduled, Integrations, Files, and Settings share aligned surface and
responsive conventions while retaining their distinct route structures. The
full repository format gate now passes after the approved mechanical cleanup of
the unrelated App and usage files. The final gate covers 172 desktop unit tests,
38 desktop/mobile Playwright scenarios, full Rust/Tauri validation, and fresh
Ubuntu 22.04 package lifecycle and visible-launch evidence, with 99.03 KiB
total CSS. The next product work is approval-gated Milestone 23 feasibility
research; do not begin it implicitly.

### Maintenance handoff

The focused `fix/sidebar-codex-usage-window` branch summarizes the exact
Codex-reported 10,080-minute window from the general upstream `codex` meter.
The ChatGPT browser Usage page was confirmed stale until refreshed; its
refreshed value matched that weekly Codex window. Model-specific meters remain
in the full details panel and cannot replace the general weekly summary. No
short-window fallback exists: missing, invalid, preview, loading, and failed
refresh states are nonnumeric, and reset timestamps stay paired to their
source window. No Qt work is included.

The restored weekly-summary UI was packaged from
`ba5b984d9d4de449b0c2ad3ed328d133b3412e55` through the digest-pinned Ubuntu
22.04 container on 2026-07-25. The ignored local artifacts in
`target/ubuntu-22.04/release/packages/` are
`quireforge_0.1.0.beta.2_amd64.deb` (4,475,040 bytes;
`c3bac356fb3ff8f99603165532917110a339e6c00025e652592c0ddb31e31e3b`) and
`QuireForge-0.1.0-beta.2-x86_64.AppImage` (83,655,160 bytes;
`5ab4a41503f48897f78ec64d9be3ce0b473d49d51254501635b11b0cda7d7768`). Both
passed manifest/checksum, GLIBC (maximum `2.34`), disposable Debian lifecycle,
and visible X11 launch checks on the Ubuntu 22.04 baseline. The component suite
verifies the general weekly selection, nonnumeric unavailable states, and
runtime details.

For a fresh thread, read in this order:

1. `AGENTS.md`
2. `docs/CURRENT_STATE.md`
3. The active roadmap entry
4. The relevant ADR
5. Only files within the approved scope

Do not perform full-repository rescans, create a duplicate master prompt,
develop Tauri and Qt features in parallel, or use Fast mode or subagents
without explicit justification.
