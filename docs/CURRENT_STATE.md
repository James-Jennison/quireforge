# Current State

## Identity and platform status

QuireForge is an unofficial Linux workspace for Codex: “Build boldly. Work
locally.” Tauri + React + TypeScript is the current functional prototype. ADR
0028 accepts retaining Tauri conditionally; no Qt migration has been selected.

- **Branch:** `feat/milestone-24a-project-state-contract`
- **Checkpoint:** Milestones 22 and 22B are complete; Milestone 23 records the
  UI-platform decision; Milestone 24A project-state contract is active.
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

Milestone 24A — **Project State Contract** defines a strict versioned Rust/Zod
project-state contract with provenance, approvals, checkpoints, validation and
package evidence, blockers, contradictions, next actions, and stable handoff
states. It creates no reader, UI, generated handoff, contradiction engine, or
canonical persistence location; those remain 24B–24D. See
[Milestone 24A](MILESTONE_24A_PROJECT_STATE_CONTRACT.md).

Milestone 23 — **UI Platform Feasibility Decision** is complete as a
documentation-only decision on `docs/milestone-23-ui-platform-feasibility`.
It maps the Tauri façade, React presentation, reusable Rust services, and Linux
adapters; compares retaining Tauri, a Qt 6 migration, and a deferred conditional
migration; and accepts **retain Tauri conditionally and reconsider Qt only on
defined measurable triggers**. No source, dependency, package, prototype, or
migration changed. See [Milestone 23](MILESTONE_23_UI_PLATFORM_FEASIBILITY.md)
and [ADR 0028](DECISIONS/0028-ui-platform-decision.md).

The final Milestone 22B baseline remains implementation commit
`6e880672b3faca50170f56fc67e187337d1b11d9`, with package evidence recorded by
`db1b729b8324b6f12952b3fe627e03a0457902ba`. Its complete gate includes 172
desktop unit tests, 38 desktop/mobile Playwright scenarios, Rust/Tauri
validation, and Ubuntu 22.04 package lifecycle and visible-launch evidence.

### Maintenance handoff

The focused `fix/sidebar-codex-usage-window` branch summarizes the exact
Codex-reported 10,080-minute window from the general upstream `codex` meter.
The ChatGPT browser Usage page was confirmed stale until refreshed; its
refreshed value matched that weekly Codex window. Model-specific meters remain
in the full details panel and cannot replace the general weekly summary. No
short-window fallback exists: missing, invalid, preview, loading, and failed
refresh states are nonnumeric, and reset timestamps stay paired to their
source window. No Qt work is included.

The final Milestone 22B package candidates were built from clean commit
`6e880672b3faca50170f56fc67e187337d1b11d9` through the digest-pinned Ubuntu
22.04 container on 2026-07-25. The ignored artifact manifest at
`target/ubuntu-22.04/release/packages/release-manifest.json` records a clean
`release-candidate` source state. Its companion checksum file is
`target/ubuntu-22.04/release/packages/SHA256SUMS`.

- `target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.2_amd64.deb` —
  4,475,472 bytes; SHA-256
  `2261d317d3748cf7dbe59d87e5d4b99161bed19a4f032c99f416c2d78c51caa1`.
- `target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.2-x86_64.AppImage`
  — 83,655,160 bytes; SHA-256
  `91ddc232068afa0f58f1504246451bacdb9e1ea49edb42cc8aa949f347c9d1c8`.

The pinned workflow passed manifest and SHA-256 validation, a maximum required
GLIBC version of `2.34`, Ubuntu 22.04 compatibility, canonical desktop-entry
and icon checks, the disposable Debian installation/upgrade/removal lifecycle,
visible installed-Debian and AppImage launches, and the representative
installed-app smoke test. The lifecycle validator installed the Debian package
with `dpkg --root "$root" --admindir "$admin" --instdir "$root" --install
"$package"`; a local manual installation would use `sudo apt install
./target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.2_amd64.deb`.
Launch the AppImage with `./target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.2-x86_64.AppImage`.

For a fresh thread, read in this order:

1. `AGENTS.md`
2. `docs/CURRENT_STATE.md`
3. The active roadmap entry
4. The relevant ADR
5. Only files within the approved scope

Do not perform full-repository rescans, create a duplicate master prompt,
develop Tauri and Qt features in parallel, or use Fast mode or subagents
without explicit justification.
