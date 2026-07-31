# Current State

## Identity and platform status

QuireForge is an unofficial Linux workspace for Codex: “Build boldly. Work
locally.” Tauri + React + TypeScript is the current functional prototype. ADR
0028 accepts retaining Tauri conditionally; no Qt migration has been selected.

- **Branch:** `main`
- **Checkpoint:** Milestones 22, 22B, 23, 24A, 24B, 24C, 25, 26, 27, 28, and
  29 are complete.
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
  prompt/transcript retention, dispatch, Python sidecar, watcher, handoff
  generation, contradiction resolution, or repository-write capability. The
  separately approved Phase A controller adds only expiring digest-only draft
  approval/rejection; it cannot start Codex or execute a project. See
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

## Completed milestone history

Milestone 30 — **Advisor Bounded Text/Data Content Ingestion and Reviewed
Single-File Export** is complete in the integrated `0.1.0-beta.18` candidate.
It is limited to one explicitly confirmed, transient, normalized UTF-8 text/data
file and one reviewed single-file native export. Advisor remains no-project,
read-only, and non-executable.

Milestone 31 — **Advisor Bounded PNG/JPEG Image Analysis** is complete in the
integrated `0.1.0-beta.19` candidate. It is limited to one explicitly
confirmed, transient, natively validated image sent only through the documented
`localImage` path.

Milestone 32 — **Advisor Conversation History Viewport and Mode Picker** is
complete in the integrated `0.1.0-beta.20` candidate. It adds only a bounded
transient Advisor viewport and a confirmed Advisor/Codex mode choice; it does
not add transcript storage, cross-mode context transfer, provider changes, or
new execution authority.

Milestone 33 — **Advisor Bounded PDF/Office Document Analysis** is complete in
the final corrected `0.1.0-beta.26` candidate, initially limited to one explicitly
confirmed PDF. The unintegrated `0.1.0-beta.21` and local `0.1.0-beta.22`
candidates were superseded before
integration. Native code keeps the source and path transient, rejects active or
embedded content, and supplies Advisor only with a bounded, path-free text
projection whose page accounting reports included, omitted, and partial pages.
Office support, generic uploads, macros, embedded-object execution, editing,
export, and project browsing remain out of scope.

M33 pins `lopdf 0.44.0` as its delegated PDF syntax parser. A parser load
success means only that `lopdf` accepted sufficient source to construct a
document; QuireForge does not independently diagnose xref, trailer, EOF,
offset, object-stream, or other low-level syntax failures. QuireForge owns the
native source boundary, bounded post-parse active/embedded/external-content
policy, projection, transient lifecycle, and path-free diagnostics. Parser
failures are intentionally reported as malformed or unsupported documents.
In-process parser CPU and memory denial-of-service limits are deferred; no
subprocess isolation is claimed.

Milestone 34 — **Workspace Selector and QuireForge Naming** is integrated at
`0481c667df946f83d0b716bc131ada88a376499b` with its verified
`0.1.0-beta.29` package evidence. It adds the compact accessible workspace
selector and user-facing QuireForge naming without changing managed Codex
protocol terminology or authority boundaries. Milestone 35 is integrated at
`ac5beb32f08798615e2be72f6f5f55f17ec18434` with its verified immutable
`0.1.0-beta.30` package set. The post-M35 packaging-hardening corrective
checkpoint uses the fresh `0.1.0-beta.31` candidate; it changes release
provenance and ABI validation only, not M35 archive behavior.

Milestone 36 — **Advisor Static Binary/Executable Inspection** is integrated at
`3ca9f4eae7d9a03213a4bc16e1171b205633418c` with its verified immutable
`0.1.0-beta.32` package set. It accepts one signature-validated ELF32/ELF64
relocatable, executable, or shared-object file for one confirmed send and
provides only bounded `static-binary-manifest-v1` metadata through the existing
text input. It never loads, executes, debugs, emulates, detonates, or transports
the binary; raw bytes, paths, names from ELF tables, notes, debug data, headers,
and addresses remain excluded. The pinned `elf 0.8.0` parser handles low-level
ELF table parsing after QuireForge-enforced source and table limits. In-process
parser CPU and memory hard limits remain deferred.

Milestone 37 — **Advisor/Approval/Dispatch/Execution End-to-End Acceptance
Gate** is complete as an evidence-only checkpoint with no package change. Its
deterministic approval, invalidation, one-time dispatch, bounded completion,
mode-reset, and recovery contracts passed. User-authorized managed-Codex checks
completed strict no-project Advisor turns, interruption/recovery, and a
read-only/untrusted execution-profile turn in a disposable directory; no
authority request or project modification occurred. See
[Milestone 37](MILESTONE_37_END_TO_END_ACCEPTANCE.md).

Milestone 38 — **Dynamic Sandbox / Malware-Analysis Discovery Gate** is
complete as a decision-only checkpoint with no product or package change. It
finds no safe dynamic-analysis capability in the current desktop architecture:
containers, bubblewrap, and an ad-hoc QEMU process are not accepted as a
hostile-binary boundary. Its separately approved M39 successor is limited to a
root-owned Firecracker 1.15.1/jailer worker for one explicitly confirmed static
ELF64 x86_64 sample with no `PT_INTERP`, zero guest network, no host/project
mounts, immutable guest assets, and bounded metadata-only results. Dynamic
loader/library support remains deferred. See
[Milestone 38](MILESTONE_38_DYNAMIC_SANDBOX_DISCOVERY.md).

The post-M39 **Workspace Boundary Acknowledgement** corrective checkpoint is
recorded separately from completed M34 history. Its `0.1.0-beta.34` package
gate records the closed
`advisor-quireforge-boundary-v1` local preference only as
`{ schemaVersion, boundaryPolicyVersion, acknowledged }`. The first ordinary
Advisor/QuireForge switch prompts; later switches under that exact policy do
not. Missing, malformed, unknown, or stale records prompt again. Every
completed switch continues to clear transient Advisor state and retains the
existing no-context-transfer boundary. A material boundary-policy change must
increment the policy version; no project, transcript, attachment, approval,
dispatch, terminal, Git, worktree, execution, path, credential, or other
capability-bearing state is persisted.

Milestone 40 — **QuireForge Task Workbench Shell** is integrated at
`98fa8fa26d740572095c2dcd9d4c1f579156817b`. It keeps the task conversation dominant while adding an opt-in
workbench-context drawer, safe keyboard-accessible action palette, and a
collapsed re-presentation of the existing managed terminal. It does not add a
shell, PTY, command-launch, execution, project-write, upload, or transport
path. Its clean `0.1.0-beta.35` Debian-only desktop and worker release set
passed full source validation, 46 desktop and 8 website Playwright checks,
provenance/ABI validation (`GLIBC_2.34` within the Ubuntu 22.04 `GLIBC_2.35`
ceiling), container lifecycle/smoke, and installed-host visible-launch checks.

Milestone 41 — **Advisor Conversation Usability** is integrated at
`eee6a9ac7e3393fd7dcd73a2c4304894c70839d4`. It limits its changes to the
bounded transient Advisor transcript viewport, reader-controlled follow-latest
and Jump to latest, final-reply reachability above the anchored composer, and a
closed-by-default safe details drawer. Its clean `0.1.0-beta.36` Debian-only
desktop and worker release set passed full source validation, `48` desktop and
`8` website Playwright checks, provenance/ABI validation (`GLIBC_2.34` within
the Ubuntu 22.04 `GLIBC_2.35` ceiling), container lifecycle/smoke, restricted
installed-package validation, and installed-host visible-launch checks. It
does not change Advisor authority, transport, attachments, persistence,
Approval/Dispatch, or execution boundaries.

The post-M41 **Packaging-Efficiency Corrective Checkpoint** is integrated at
`502e56e46131c64e7821fc98b16152142ac50eff`. It changes only the pinned Ubuntu
22.04 release workflow: immutable Linux-kernel and Firecracker source archives
may be reused from a checksum-verified container cache, while guest outputs are
rebuilt in a disposable work directory and the existing provenance, ABI,
lifecycle, and visible-launch gates remain mandatory. Its clean
`0.1.0-beta.37` Debian-only desktop and worker release set passed full source
validation, `48` desktop and `8` website Playwright checks, provenance/ABI
validation (`GLIBC_2.34` within the Ubuntu 22.04 `GLIBC_2.35` ceiling),
container lifecycle/smoke, restricted installed-package validation, and
installed-host visible-launch checks. No application capability, dependency,
release, or deployment behavior changes.

Milestone 43 — **Shared Task Continuity** source is integrated at
`6eb526bdb0b1705414f5507081dc37872358198c`. Its clean
`0.1.0-beta.38` Debian-only desktop and worker release set passed full source
validation, provenance/ABI (`GLIBC_2.34` within the `GLIBC_2.35` ceiling),
container lifecycle/smoke, restricted installed-package validation, and visible
launch. It adds only a user-approved, one-use transient brief/receipt envelope;
it does not transfer transcripts, attachments, project, terminal, Git,
approval, dispatch, execution, path, or authority state.

The post-beta.38 **Temporary Bundle Construction-Envelope Checkpoint** is
complete locally as `0.1.0-beta.40`; a preliminary beta.39 package set was
superseded before it was recorded as authoritative package evidence after the
approved roadmap was confirmed through M58. The corrected temporary limits are
256 KiB startup entry, 448 KiB
application shell, 1.5 MiB total JavaScript, and 160 KiB CSS. The clean pinned
Ubuntu 22.04 Debian set is bound to
`0fed7983a3f32aa79ea4d1feee9947535d370a9b`, passes provenance/ABI, lifecycle,
container and installed smoke, and visible-launch gates, and adds no product
capability, dependency, authority, transport, provider, connector, release, or
deployment behavior. The existing post-workbench permanent-budget
reconciliation remains required.

Milestone 44 — **Unified Single Attachment Entry** is complete at
`891abf6d953e3b7c0dd3f0d3bd03baeb29de40fb` with its verified
`0.1.0-beta.41` Debian and worker set. It replaces the five separate
Advisor attachment entry buttons with one compact **Attach a file** tray that
selects only the existing native text/data, PNG/JPEG, PDF, ZIP, and static-ELF
handlers. It adds no generic upload, drag-and-drop, new type, collection,
transport, persistence, or authority. Full source validation, bundle ceilings,
provenance/ABI, lifecycle, container and installed smoke, and visible-launch
gates passed.

## Next action

Milestone 46 — **Bounded Multi-Attachment** is complete at
`1bc2e787ab785016041d70845c97ca9c2c4f84db` with its verified
`0.1.0-beta.42` Debian and worker set. Advisor now supports only the approved,
native-memory-only collection of at most three existing typed attachments, at
most one image, and a 40 MiB aggregate source ceiling. The existing text input
plus optional `localImage` transport is retained; generic upload, persistence,
and new authority remain out of scope. See
[Milestone 46](MILESTONE_46_BOUNDED_MULTI_ATTACHMENT.md).

Milestone 48 — **Generated Artifacts and Explicit Save** is complete at
`5d483d0c068c450bbc779ee07b048fe848c7e1f0` with verified
`0.1.0-beta.43` Debian and worker evidence. It implements only the approved
[M47](MILESTONE_47_GENERATED_ARTIFACT_WORKFLOW_PROPOSAL.md)
`advisor-generated-artifact-registry-v1` contract. See
[Milestone 48](MILESTONE_48_GENERATED_ARTIFACTS_AND_EXPLICIT_SAVE.md).

Milestone 49 — **QuireForge Review Panes** is complete at
`f1a44324859faa2ed43f24ab60db12b58e6c6836` with verified `0.1.0-beta.44`
Debian and worker evidence. Its closed, individually lazy Files, Diff, Git,
Preview, Activity, and Approval panes retain the existing typed read-only
boundaries and make no new authority available. See
[Milestone 49](MILESTONE_49_REVIEW_PANES.md).

Milestone 50 — **QuireForge Workbench Layout Refinement** is complete at
`1cc7c50ceed6d2b6c2f91274110471d71fe6292a` with verified `0.1.0-beta.45`
Debian and worker evidence. It adds only bounded local presentation preferences
and accessible existing review-shell/terminal-dock ergonomics; durable task
records and alternate plans were not part of M50.

Milestone 51 — **Durable Task Records and Alternate-Plan Proposal** is
approved. It selects a bounded,
local-only SQLite task catalogue with separate non-authoritative plan records;
it retains no conversation, path, attachment, artifact, approval, dispatch,
execution, terminal, Git, provider, or Advisor state. See
[Milestone 51](MILESTONE_51_DURABLE_TASK_RECORDS_ALTERNATE_PLAN_PROPOSAL.md).

Milestone 52 — **Durable Task Records and Alternate Plans** is complete at
`6df055999d2ad01d2385096a14bc71f8aada2a8c`. Migration 11 adds only private
local organizational task/plan metadata. The compact workbench surface
provides bounded search, explicit lifecycle actions, capacity/cleanup feedback,
and up to four user-controlled plans. Plan switching clears the current
transient attachment selection and never imports or activates authority. Its
clean `0.1.0-beta.46` desktop and worker Debian artifacts passed the pinned
Ubuntu 22.04 package lifecycle, visible-launch, provenance, ABI, sandbox-worker,
artifact, and restricted installed-host gates. See
[Milestone 52](MILESTONE_52_DURABLE_TASK_RECORDS.md).

Milestone 53 — **Local Artifact and Design Review Proposal** is complete. Its
four accepted phases select the bounded, native-owned
`local-review-collection-v1` implementation proposal: task-contextual private
SQLite review collections with path-free copied text, validated static image
mockups, typed evidence envelopes, inert preview, local item-level notes,
non-Git text comparison, and explicit digest-bound M48 promotion. Review does
not confer filesystem, network, Git, terminal, provider, approval, dispatch,
execution, publishing, or deployment authority. M54 is explicitly approved
and in progress locally, targeting `0.1.0-beta.49` / `0.1.0~beta.49`. Beta.48
is a validated but non-deployable candidate because the installed host still
lacked the migration-18 unprivileged receipt bootstrap. Beta.49 adds the fixed
native migration/unprivileged/installed-host bootstrap and preserves coherent
canonical releases before promotion. The beta.47 normalized canonical set was
lost by the former finalizer; only its recorded hashes and distinct raw Tauri
bundle remain, and no replacement beta.47 evidence was fabricated. Restricted installed-host
validation now has a fixed native beta.49 completion branch: it resolves only
the production data directory, one live attached project, and one digest-valid
migration-18 unprivileged predecessor; it starts no Tauri/GUI surface and
records only through the existing controller. The coherent beta.48 release set
is retained as the non-deployable predecessor. No M53 package,
release, publication, or deployment occurred. See
[Milestone 53](MILESTONE_53_LOCAL_ARTIFACT_DESIGN_REVIEW_PROPOSAL.md).

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

Milestone 29 — **Managed Advisor Conversation Foundation** is complete at
implementation commit `45a6d5f2219a5531cf336ed27f0cf7f389d984be`. It adds a
distinct, fixed no-project Advisor turn through the existing
Codex-managed browser ChatGPT sign-in boundary. Advisor prompts and replies are
transient in QuireForge; only opaque thread metadata is retained. Including
the temporary Project State safe projection requires a second per-send
confirmation. It adds no execution, dispatch, tool, terminal, Git, project
write, API-key, or transcript-retention capability. The clean `0.1.0-beta.6`
Ubuntu 22.04 Debian/AppImage set passed the pinned container lifecycle,
desktop/icon, visible-launch, and installed-host smoke gates. See
[Milestone 29](MILESTONE_29_MANAGED_ADVISOR_CONVERSATION.md).

The scoped Milestone 29 post-completion UI hotfix separates the Advisor
read-only capability notice from the action row and makes the disabled-send
reason explicit without changing Advisor permissions, context rules, or
execution boundaries. Its clean `0.1.0-beta.7` Ubuntu 22.04 Debian/AppImage
package set passed the pinned container lifecycle, desktop/icon,
visible-launch, and installed-host visual gate.

Milestone 28 — **Reference-Only Advisor Foundation** is complete. It includes a
read-only Advisor metadata route and one explicitly confirmed, temporary safe
projection of the existing local Project State reader; arbitrary context
reading, model calls, prompt text retention, and automation remain separately
gated. The approved unnumbered Phase A/B1 controller provides transient,
digest-bound approvals. Its approved B2 extension dispatches one immediately
revalidated request into the separate managed Codex execution workspace using
only the existing project-bound app-server boundary. B3 returns only a bounded,
correlated completion report to Advisor; it never supplies a transcript,
terminal stream, repository data, credentials, or new authority.

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
