# Current State

## Identity and platform status

QuireForge is an unofficial Linux workspace for Codex: “Build boldly. Work
locally.” Tauri + React + TypeScript is the current functional prototype. ADR
0028 accepts retaining Tauri conditionally; no Qt migration has been selected.

- **Branch:** `main`
- **Checkpoint:** Milestones 22, 22B, 23, 24A, 24B, 24C, 25, 26, 27, and 28
  are complete; Milestone 29 is in progress.
  Milestone 27 is complete at implementation commit
  `cc4d0cea7d28d275e5ad1c8aa9d7a2a4f0627d6c`; its final package evidence is
  recorded by this documentation checkpoint. The clean `0.1.0-beta.4` Ubuntu
  22.04 Debian/AppImage set passed the pinned container lifecycle, desktop/icon,
  visible launch, and smoke gates, plus installed-host Debian and AppImage
  smoke. M27 adds only the managed-ChatGPT native conversation boundary,
  confirmed local Chat/Codex preference, and bounded settings foundation; no
  consumer ChatGPT API, credential handling, project-context transfer,
  automatic handoff, watcher, or autonomous behavior was added.

  Milestone 26 is complete and its cumulative branch was integrated into
  `main` at `81be6882b76aa17df454497b58fe5a69ae84f56d`. Its palette-only
  implementation is
  `0ae0de7995f10128728116b148d49f2cb5b2cf79`. Its selector-hotfix candidate is
  `8f7b505f24a489d468f02e82d0e6197606a83abe`; it records the required
  same-version Debian `--reinstall` path after verifying that the installed
  host binary was stale. The fresh pinned Ubuntu 22.04 candidate passed its
  container gate and the user confirmed Settings → Appearance after the
  documented interactive sudo reinstall. It adds eight closed, locally
  persisted built-in appearances and settings preview behavior without changing
  the
  native bridge, repository-state contract, layout, typography, or automation
  boundaries.

  Milestone 25's presentation-only implementation is
  `9ae07167448d81c18c8e6fb293ffe52a146b346b`; its fresh pinned Ubuntu 22.04
  package evidence is recorded by the following documentation checkpoint.
  The milestone refines only the existing branded shell, Home entry surface,
  and conversation composer; it introduces no backend, repository-state,
  watcher, automation, voice, or Qt behavior.

  Milestone 24C's final combined implementation is
  `6d8f302297fe01f2afb0dad855a4e81f1a8782b2`; its fresh Ubuntu 22.04 package
  evidence is recorded by this documentation checkpoint. Milestone 24B
  implementation is `ecc556f9a7025e9e5da3ab63dc34eb1c9f6c3d47`; its final
  Ubuntu 22.04 package evidence remains documented in its milestone report. No
  24D handoff/consistency behavior has begun.

  The post-24B native-conversation hotfix keeps a multiline task as one
  unchanged `turn/start` text item, blocks re-entrant starts before React busy
  state propagates, and distinguishes invalid requests, failed native IPC, and
  invalid native responses with bounded actionable UI diagnostics. It adds no
  project-state workspace or later-milestone behavior.

  The completed 24B fixture suite proves local-only and existing-tracking
  reads preserve an inspected repository, while the explicitly authorized
  fetch mode can advance only its `origin` tracking ref with no `FETCH_HEAD`,
  HEAD, index, worktree, or configuration mutation. Package, validation, and
  handoff evidence readers are bounded, diagnostic-only 24B evidence sources.

  The reader now returns bounded package, validation, and reported-handoff
  evidence with commit-based freshness and diagnostics; it does not execute
  validation, rebuild packages, or generate a handoff.

  Package evidence retains distinct manifest, checksum-file, and optional
  local-verification observations; conflicting accepted records remain visible.
  The release-manifest producer's version-1 `appimage`/`x86_64` wire values are
  normalized by the strict Rust/TypeScript reader contract.

  Milestone 28 is complete as a reference-only Advisor foundation. It adds
  strict Rust/Zod contracts and SQLite schema support for
  opaque Advisor conversation references, explicit selected-context references,
  provenance/freshness, and non-dispatching proposal digests. Its fixed
  `#advisor` route receives only a strict safe-summary projection and renders
  it or an empty state. A user may explicitly confirm one temporary
  local-only/metadata-only normalized Project State source; its safe projection
  excludes identity, paths, Git refs, source content, artifacts, diagnostics,
  and images, and is never retained in SQLite. It has no model call,
  prompt/transcript retention, approval action, dispatch, Python sidecar,
  watcher, handoff generation, contradiction resolution, or repository-write
  capability. See
  [Milestone 28](MILESTONE_28_ADVISOR_FOUNDATION.md). The clean
  `0.1.0-beta.5` Ubuntu 22.04 Debian/AppImage set passed the pinned container
  lifecycle, desktop/icon, visible-launch, and installed-host smoke gates.

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

Milestone 27 — **Unified Conversation Engine** is complete. It adds strict
Rust/Zod Chat/Codex capability profiles, a Settings navigation foundation,
bounded mode-aware metadata, a fixed no-project native Chat bridge, and an
explicitly confirmed Chat/Codex local workspace preference. Chat requires only
the documented Codex-managed browser ChatGPT sign-in; it remains unavailable
for API-key, external-token, browser-session, or consumer-ChatGPT paths. Its
fixed native profile has no project root, dynamic tools, integrations,
approvals, terminal, Git, or worktree authority. The selection restores only
`chat` or `codex` and safely defaults to Codex for absent or invalid saved
values; it never transfers project context, attachments, approvals,
integrations, transcripts, or credentials. The clean, incremented
`0.1.0-beta.4` package set passed the pinned Ubuntu 22.04 and installed-host
gates. See [Milestone 27](MILESTONE_27_UNIFIED_CONVERSATION_ENGINE.md).

Milestone 29 — **Managed Advisor Conversation Foundation** is in progress. It
adds a distinct, fixed no-project Advisor turn through the existing
Codex-managed browser ChatGPT sign-in boundary. Advisor prompts and replies are
transient in QuireForge; only opaque thread metadata is retained. Including
the temporary Project State safe projection requires a second per-send
confirmation. It adds no execution, dispatch, tool, terminal, Git, project
write, API-key, or transcript-retention capability. See
[Milestone 29](MILESTONE_29_MANAGED_ADVISOR_CONVERSATION.md).

Milestone 28 — **Reference-Only Advisor Foundation** is complete. It includes a
read-only Advisor metadata route and one explicitly confirmed, temporary safe
projection of the existing local Project State reader; arbitrary context
reading, model calls, prompt text retention, approval/dispatch flow, and
automation remain separately gated. The next action requires a separate
Advisor integration and Approval/Dispatch proposal gate.

Milestone 26 — **Appearance Themes** is complete. It retains Forge as the
default and adds seven closed local palettes through Settings → Appearance with
local restoration, direct accessibility/visual coverage, fresh Ubuntu 22.04
Debian/AppImage evidence, and installed-host selector confirmation. See
[Milestone 26](MILESTONE_26_APPEARANCE_THEMES.md).

Milestone 25 — **Desktop visual polish** is complete at implementation commit
`9ae07167448d81c18c8e6fb293ffe52a146b346b`. Its final Ubuntu 22.04 package
evidence is recorded below. The presentation-only pass retains QuireForge
branding and existing native behavior while strengthening sidebar density,
dark-surface hierarchy, Home composition, and the responsive conversation
composer. See [Milestone 25](MILESTONE_25_DESKTOP_VISUAL_POLISH.md).

Milestone 24D — **Handoff and Consistency Engine** remains only a proposed next
milestone. It requires a separate scope proposal and James's approval before
any operational consistency, handoff generation, contradiction detection, or
state-changing behavior begins.

Milestone 24C — **Project State Workspace** is complete at final combined
implementation commit `6d8f302297fe01f2afb0dad855a4e81f1a8782b2`. Its one
demand-driven route
requests the existing reader in `local-only`, `metadata-only` mode and presents
normalized evidence without mutation, approval, resolution, automation, or
persistence changes. Frontend, responsive/accessibility, repository, Rust,
bundle, Tauri, and fresh pinned Ubuntu package gates pass. See
[Milestone 24C](MILESTONE_24C_PROJECT_STATE_WORKSPACE.md).

Milestone 24B — **Repository State Reader** remains complete. It provides
explicit, attached-project-only local/tracking/fetch-authorized read modes and
a validated contract plus diagnostics, with no watcher, document rewrite, or
contradiction resolution. See
[Milestone 24B](MILESTONE_24B_REPOSITORY_STATE_READER.md).

Milestone 24A — **Project State Contract** is complete at implementation commit
`f62ba5c68fe0002d3d3f6b5faa0bd2d522d81f0d`. Its versioned Rust/Zod contract
has provenance, approvals, checkpoints, validation/package evidence, blockers,
contradictions, next actions, and stable handoff states. It creates no reader,
UI, generated handoff, contradiction engine, or canonical persistence location;
those remain 24B–24D pending explicit approval. See
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
"$package"`; a local manual installation would use `sudo apt install --reinstall
./target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.2_amd64.deb`.
Launch the AppImage with `./target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.2-x86_64.AppImage`.

Milestone 24A refreshed those ignored candidates from clean implementation
commit `f62ba5c68fe0002d3d3f6b5faa0bd2d522d81f0d`. The same manifest and checksum
paths now record 4,475,272-byte Debian SHA-256
`ce9a854e34964b57f125bdb723266023ef80e625b6d9e2fb56ab87f73dd02fc5` and
83,655,160-byte AppImage SHA-256
`933c09194720f12cc8ed991d4261f6acb0254be7f4241792d23548fc0564416e`.
The packaged executable requires maximum GLIBC `2.34`, within the Ubuntu 22.04
`2.35` policy baseline; the pinned workflow passed desktop/icon, Debian
lifecycle, installed launch, AppImage launch, and smoke checks.

Milestone 24C refreshed the ignored candidates from clean final combined
implementation commit `6d8f302297fe01f2afb0dad855a4e81f1a8782b2` on 2026-07-26:

- `target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.2_amd64.deb`
  — 4,635,656 bytes; SHA-256
  `d04a114f6c1b1eba822da6a5133f684b27532a8aa81874246347ae8797ff09c7`.
- `target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.2-x86_64.AppImage`
  — 83,855,864 bytes; SHA-256
  `f7c4a1c6dd651438fa1b1403269aa62331096bd9137c105cb99a8ab5c59de36a`.

The pinned Ubuntu 22.04 workflow passed manifest/checksum, maximum GLIBC `2.34`,
desktop entry, icon, Debian install/upgrade/remove, visible Debian launch,
visible AppImage launch, and representative smoke validation. Exact commands
and reader-boundary evidence are in
[Milestone 24C](MILESTONE_24C_PROJECT_STATE_WORKSPACE.md).

Milestone 25 refreshed the ignored candidates from clean presentation-only
implementation commit `9ae07167448d81c18c8e6fb293ffe52a146b346b` on
2026-07-26:

- `target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.2_amd64.deb`
  — 4,636,464 bytes; SHA-256
  `0592f36f6a00ec1e4e835fc52821cab205625dd3a80598f4dd8a12a5b4792b93`.
- `target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.2-x86_64.AppImage`
  — 83,855,864 bytes; SHA-256
  `240c4927aad116e6ab4fbb7d0d7ea86bd69dee3f84327bd6b149e7f3b48b487e`.

The digest-pinned Ubuntu 22.04 container includes `/usr/bin/xvfb-run` and
passed manifest/checksum validation, a maximum required GLIBC `2.34` against
the Ubuntu 22.04 `2.35` baseline, desktop-entry and PNG icon validation, the
disposable Debian install/upgrade/remove lifecycle, visible Debian and AppImage
launches, and representative smoke validation. The ignored release evidence is
the version-1 manifest and checksum pair under
`target/ubuntu-22.04/release/packages/`; local installation uses `sudo apt
install --reinstall ./target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.2_amd64.deb`
and AppImage launch uses
`./target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.2-x86_64.AppImage`.

Milestone 27 refreshed the ignored candidates from clean implementation commit
`cc4d0cea7d28d275e5ad1c8aa9d7a2a4f0627d6c` on 2026-07-26, incrementing the
candidate version to `0.1.0-beta.4`:

- `target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.4_amd64.deb`
  — 4,674,456 bytes; SHA-256
  `3cdb4eda670a9b771efbb53b8ac84c70ea92189330a334a06788f262268cc9f7`.
- `target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.4-x86_64.AppImage`
  — 83,905,016 bytes; SHA-256
  `b527286c55565690b9b26f52fe18a8d7d4904f4466bec5ba07d909d580815ba9`.

The release-manifest source is clean and identifies that implementation commit.
The digest-pinned Ubuntu 22.04 container includes `/usr/bin/xvfb-run` and
passed manifest/checksum, maximum GLIBC `2.34` against the `2.35` baseline,
desktop-entry and icon validation, disposable Debian install/upgrade/remove,
visible Debian/AppImage launches, and smoke validation. The installed host was
upgraded to Debian version `0.1.0~beta.4` with `sudo apt install --reinstall
-y ./target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.4_amd64.deb`;
visible smoke then passed for `/usr/bin/quireforge` and the AppImage. Exact
commands, implementation validation, and deferred scope are in
[Milestone 27](MILESTONE_27_UNIFIED_CONVERSATION_ENGINE.md).

For a fresh thread, read in this order:

1. `AGENTS.md`
2. `docs/CURRENT_STATE.md`
3. The active roadmap entry
4. The relevant ADR
5. Only files within the approved scope

Do not perform full-repository rescans, create a duplicate master prompt,
develop Tauri and Qt features in parallel, or use Fast mode or subagents
without explicit justification.
