# Changelog

## 0.1.0-beta.101 — M76 isolated browser-research candidate

- Adds a separate native, owner-confirmed browser-research controller scoped
  exactly to `https://google.com/` at `https://google.com`.
- The ephemeral WebKitGTK adapter disables JavaScript, retains only bounded
  digest provenance in memory, and stops on redirect/origin drift or
  prompt-injection indicators. It has no ambient browser session, credentials,
  downloads, mutation, automatic context transfer, or agent access.

## 0.1.0-beta.100 — M71 Evidence Linkage candidate

- Adds a private native claim → evidence → conclusion chain beside the M70
  Knowledge Ledger. It links M48 artifact references, M52 typed local-review
  evidence, package-validation summaries, and bounded owner-trial receipts.
- Evidence captures only source identity, closed class, digest, timestamp, and
  bounded owner-trial outcome. It stores no raw evidence, and its explicit
  conclusion cannot change a knowledge record status automatically.
- The candidate passed full source validation, desktop and website E2E, and the
  clean pinned Ubuntu package/release-artifact/visible-launch gate.

## 0.1.0-beta.95 — Explicit linked-project chat bridge candidate

- Local Chat now names its actual scope as “Project context not attached” and
  offers a visible, user-initiated Continue with linked project action.
- That action opens the existing managed Codex project conversation and never
  transfers the Local Chat transcript implicitly.

## 0.1.0-beta.94 — Chat-first entry candidate

- When the persisted workspace route is Threads and the user has selected Chat
  mode, QuireForge opens the existing Local Chat workspace instead.
- Explicit Threads deep links remain Threads; Codex mode and every workspace
  route retain their prior behavior.
- Threads remains available as a workspace, including the beta.93 recovery
  labels and Start a local chat action.
- The pinned Ubuntu package/release-artifact gate, root-owned staging,
  installer-daemon installation, supervised launch, and canonical
  installed-host validation passed for the fresh candidate.

## 0.1.0-beta.93 — M69 Threads recovery candidate

- Adds a prominent Start a local chat action to Threads and routes it through
  the existing Chat mode boundary.
- Reconciliation misses now appear as unavailable threads rather than invented
  untitled conversations. The local reference remains unchanged and cannot
  open a substitute thread.
- The fresh candidate passed full validation, desktop and website E2E, the
  pinned Ubuntu package/release-artifact gate, root-owned staging,
  installer-daemon installation, supervised launch, and canonical
  installed-host validation.

## 0.1.0-beta.92 — M69C Action Card renderer candidate

- Adds an accessible Local Chat Actions menu and visible Action Card renderer
  for the closed non-executing lifecycle introduced in beta.91.
- A card can prepare, approve, or revoke only its opaque native ID. It names
  its boundary in the UI: no data is selected, and approval does not run an
  action. No receipt consumer or capability is introduced.
- The fresh candidate passed full validation, desktop and website E2E, the
  pinned Ubuntu package/release-artifact gate, root-owned staging,
  installer-daemon installation, supervised launch, and canonical
  installed-host validation. Its immutable content-free receipt reports two
  artifacts and completed validation.

## 0.1.0-beta.91 — M69C Action Card authority candidate

- Adds a native-owned, strict Action Card lifecycle for the closed
  `attach-project`, `use-source`, `draft-artifact`, and `work-with-code`
  proposal classes.
- Cards carry no user text, project/source/artifact identity, path, provider,
  credential, tool, or execution argument. Approval creates only an opaque,
  process-local content-free receipt; it has no data or execution authority.
- The fresh candidate passed `pnpm validate` (423 desktop tests, 455 Rust
  tests, and two sandbox tests), desktop/mobile E2E, the pinned package and
  release-artifact gates, root-owned staging, daemon installation, supervised
  launch, and canonical installed-host validation. The immutable receipt is
  content-free and reports two artifacts with all phases passed.
- This completes only M69C's non-executing native foundation. Its accessible
  renderer and any capability-specific receipt consumer remain future work.

## 0.1.0-beta.90 — M69B Threads-first candidate

- Makes Threads the default workspace and adds a top-level Threads, Projects,
  and Settings navigation layer while retaining every legacy Chat, Work, and
  Code hash route as a migration entry point.
- Adds a bounded, presentation-only thread tree with session-local unread
  state, a closed status enum, and fail-closed folder aggregation. No pending
  decision is fabricated because M69C has not introduced a native
  pending-decision event.
- Refines the M69A local composer: Enter sends, Shift+Enter remains multiline,
  turns render as a natural local transcript, and direct local date/time
  questions use the host clock without exposing model details.
- The pinned Ubuntu 22.04 package/lifecycle/visible-launch and release-artifact
  gates, `pnpm validate`, and desktop/mobile E2E passed. Both Debian packages
  were staged through the root-owned boundary, installed by the root-owned
  daemon, and launched by the supervised 6 GiB service.
- The attached project was non-destructively relinked to the beta.90 candidate.
  The canonical installed-host receipt then completed with `created`: both
  installed packages matched beta.90 and the immutable record reports two
  artifacts with every package-validation phase passed. The receipt is
  content-free and does not record model, source, or project content.

## 0.1.0-beta.89 — M69A local chat candidate

- Adds the first ordinary local-only Chat turn: bounded typed user text enters
  a dedicated native `LocalChatService` and renders an ephemeral inline result.
- Keeps M60 governed review separate: local chat has no project, source,
  artifact, ledger, provider, tool, filesystem, or credential capability.
- Preserves M63 CPU-only, one-attempt, 4,096-input-token, 512-output-token,
  60-second, and 6 GiB limits; loader/model details remain unavailable to the
  renderer.
- Beta.89 is a fresh candidate. Package, installed-host, and supervised local
  acceptance evidence is recorded only after those gates complete.

## 0.1.0-beta.88 — M63 service-cgroup lookup

- Resolves the fixed 6 GiB memory limit from the process's exact cgroup v2
  path, rather than the unconstrained cgroup mount root.
- Beta.87 failed closed before review acknowledgement, consumption, output, or
  retry because its cgroup admission lookup used that mount root. It remains
  immutable failed acceptance evidence.
- The fresh beta.88 package passed the pinned release-artifact gate, trusted
  root-owned staging, daemon installation, supervised launch, and the explicit
  governed local-only attempt. The acceptance record remains content-free.

## 0.1.0-beta.87 — M63 cgroup memory admission

- Replaces the unsuitable `RLIMIT_AS` address-space limit with a required
  supervisor cgroup memory limit at the fixed 6 GiB M63 ceiling. File-backed
  model mappings no longer consume the admission budget merely by being mapped.
- Beta.86 passed package, local installation, and supervised-launch gates, but
  native availability failed closed before review acknowledgement, consumption,
  output, or retry. It remains immutable failed acceptance evidence.

## 0.1.0-beta.86 — M63 precise loader classification

- Limits the bounded-memory diagnostic to explicit native allocation failures;
  routine loader status that merely mentions memory is classified as neither
  memory failure nor retained loader output.
- Beta.85 is immutable failed acceptance evidence: native admission failed
  closed before review acknowledgement, consumption, output, or retry. The
  observed category requires this fresh candidate before a cause is claimed.

## 0.1.0-beta.85 — M63 content-free loader diagnosis

- Reduces native preflight loader feedback to fixed, non-sensitive categories:
  access, format, bounded-memory, or generic load failure. It never returns,
  retains, or logs a loader message, model location, or model content.
- Restores llama.cpp's process-global redacting callback immediately after the
  preflight load, before the temporary classifier leaves scope.
- Beta.84 is immutable failed acceptance evidence: native admission failed
  closed before acknowledgement, consumption, output, or retry. This candidate
  preserves all M48, M55, and M60 contracts.

## 0.1.0-beta.84 — M63 native-load admission correction

- Makes native availability prove a CPU-only, in-process model load and
  immediate release before a reviewed bundle may be consumed. The preflight
  accepts no reviewed bytes and returns no model-derived data.
- Separates model-load, context-initialization, and sampler-initialization
  failures without exposing a model location, content, loader output, or
  filesystem observation.
- Beta.83 is immutable failed installed-host acceptance evidence: its bound
  contract reached the native loader, which failed before output and without a
  retry. This candidate preserves all M48, M55, and M60 contracts.

## 0.1.0-beta.83 — M63 availability-contract binding correction

- Binds a successfully verified, supervisor-provided local-model contract to
  the exact in-memory native reservation, preventing a second environment read
  from invalidating the reviewed action before model loading. The contract is
  never serialized, logged, or persisted.
- Reserves `model-unavailable` for a missing or invalid supervisor contract;
  a later in-process loader failure now reports the distinct bounded
  `runtime-unavailable` outcome.
- Beta.82 remains immutable failed acceptance evidence. This candidate changes
  no model, provider, network, package contents, or execution bounds.

## 0.1.0-beta.82 — M63 acknowledgement response correction

- Returns the existing opaque one-use authorization ID with a successfully
  acknowledged reviewed bundle, allowing the local-only action to remain gated
  by the exact native authorization instead of becoming permanently disabled.
- Beta.81 remains immutable failed acceptance evidence; this fresh candidate
  does not change the external model, provider, network, or execution bounds.

## 0.1.0-beta.81 — Project-only M63 acceptance candidate

- Restores project-only governed context preparation when no local-review
  evidence was selected and optional review storage is unavailable.
- Preserves fail-closed behavior for any non-empty review-evidence selection.

## 0.1.0-beta.80 — M63 installed-host acceptance candidate

- Reserves a fresh, unreleased candidate version for the documented M63
  installed-host governed-review acceptance workflow.
- Beta.79 remains immutable accessibility and package evidence; this candidate
  does not change its Work-lane authority boundaries or package provenance.
- The pinned Ubuntu 22.04 package/lifecycle/smoke/artifact gate and installed
  package identity checks passed. The governed review then failed closed before
  review: its project-only preparation returned a zero-item, zero-byte snapshot,
  so no bundle, availability recheck, model attempt, output, or retry occurred.
  Beta.80 is immutable failed acceptance evidence, not a release claim.

## 0.1.0-beta.79 — Accessibility semantics candidate

- Restored a complete H1-to-H2 hierarchy for the Ledger and Adapter Work
  routes without changing their local-only authority boundaries.
- Added desktop and mobile end-to-end coverage for bounded Ledger metadata,
  fixture-only Adapters behavior, and route accessibility.

## 0.1.0-beta.78 — Work surface clarity candidate

- Refined Ledger receipt cards to prioritize lifecycle facts over raw digests.
- Consolidated the Adapter fixture list into a compact selector and selected
  fixture detail panel.

## 0.1.0-beta.77 — M66–M68 local-governance candidate

- Added the read-only, content-free Context and Authority Ledger over existing
  governed local records.
- Added the deterministic local-fixture Work adapter catalogue with no
  connection, credential, or execution authority.
- Recorded external browser, connector, delivery, and scheduled-work decisions
  without implementing any external capability.

## 0.1.0-beta.73 — M63 native-revalidation recovery candidate

- Recovers an acknowledged governed review when native revalidation finds the
  local model unavailable after an earlier successful preflight. No attempt is
  consumed; the view requires an explicit availability recheck before a later
  manual run. The desktop fixture covers the complete recovery flow.
- The model remains external to the package and is never started by routine
  package gates. Package promotion and installed Debian desktop
  governed-review acceptance remain pending.

## 0.1.0-beta.72 — M63 availability-recovery acceptance candidate

- Shows a content-free, local-only availability confirmation after an explicit
  governed-review recheck, while preserving the separate one-time reviewed
  action requirement. The desktop fixture covers that recovery flow.
- The model remains external to the package and is never started by routine
  package gates. Package promotion and installed Debian desktop
  governed-review acceptance remain pending.

## 0.1.0-beta.71 — M63 local-runtime recovery acceptance candidate

- Lets a governed review recheck its content-free local-runtime availability
  after an unavailable result, without consuming the acknowledged bundle or
  starting a local attempt. The explicit one-time action remains disabled until
  native availability succeeds.
- The model remains external to the package and is never started by routine
  package gates. Package promotion and installed Debian desktop
  governed-review acceptance remain pending.

## 0.1.0-beta.70 — M63 availability-failure acceptance candidate

- Resolves an IPC-level local-runtime availability preflight to a bounded,
  visible unavailable state. The governed review leaves its one-time local
  action disabled and does not consume the acknowledged bundle.
- The model remains external to the package and is never started by routine
  package gates. Package promotion and installed Debian desktop
  governed-review acceptance remain pending.

## 0.1.0-beta.69 — M63 availability-preflight acceptance candidate

- Keeps the governed-review local-only action disabled until the typed,
  content-free local-runtime availability preflight completes, and shows that
  checking state in the open review. This preserves the native authoritative
  check and prevents a review from being submitted during initial availability
  resolution.
- The model remains external to the package and is never started by routine
  package gates. Installed Debian desktop governed-review acceptance remains
  pending.

## 0.1.0-beta.68 — M63 relaunch-privacy acceptance candidate

- Extends the governed-review browser fixture through an application reload,
  proving a completed local-only result is not restored after the open view
  closes and the desktop application relaunches.
- Retains the fixed CPU-only local model limits and governed review flow; the
  model remains external to the package and is never started by package gates.
- The clean-tree beta.68 Debian pair from source commit
  `5c4ca198f94553dd760f20734f765c8abb5a488e` passed the authoritative pinned
  Ubuntu 22.04 package, lifecycle, visible-launch, and release-artifact gates.
  It excludes the model and does not start the runtime; installed Debian
  desktop governed-review acceptance remains pending.

## 0.1.0-beta.67 — M63 governed-review acceptance candidate

- Reissues the credential-free M63 local candidate from the source state that
  records the completed bounded in-process adapter check.
- Retains the fixed CPU-only local model limits and governed review flow; the
  model remains external to the package and is never started by package gates.
- The authoritative package/lifecycle/visible-launch gate and the explicit
  installed Debian governed-review acceptance remain pending for beta.67.

## 0.1.0-beta.66 — M63 installed-host acceptance candidate

- Carries forward the completed M63 preparation, authorization, concurrent
  admission, and runtime-availability corrections in a uniquely versioned
  candidate. The focused host-native adapter gate has completed one bounded
  local-only attempt without retaining model-location or generated-output
  data. Package and installed Debian desktop acceptance evidence remains
  pending.

## 0.1.0-beta.65 — Local-runtime acceptance correction

- Restores M63 preparation for an explicit instruction without an optional
  durable source, keeps cancellation responsive during the CPU-bound attempt,
  and presents the governed review as a contained local workbench.
- The fresh clean-tree beta.65 Debian candidate at source commit
  `6f77d0c4d73cefe5ed335898830872ca63ad203a` passed the pinned Ubuntu 22.04
  package, lifecycle, and visible-launch gates. Installed-host local-model
  acceptance remains required before any release-ready claim.

## 0.1.0-beta.64 — In-process local runtime candidate

- Carries the M63 credential-free, CPU-only local-runtime candidate forward
  with canonical completed, failed, and cancelled lifecycle outcomes, bounded
  cancellation, and visible local-only terminal status.
- The model remains local, read-only, outside Git and packages. This candidate
  adds no credential, provider, network, tool, browser, connector, or external
  action authority; installed-host model execution acceptance remains required.

## 0.1.0-beta.63 — Governed Context Assembly

- Implements M60's explicit, bounded, native-owned context selection,
  canonical assembly, structural redaction, review, digest-bound one-use
  fictional local-only delivery, expiry/revocation/replay handling, migration,
  and content-free audit linkage.
- Adds no provider, credential, network transmission, inference, connector,
  browser, MCP, automation, external mutation, or deployment authority.

## 0.1.0-beta.62 — Controlled browser verification

- Adds the M58 fictional, deterministic, local-only, read-only controlled
  browser-verification slice: project-scoped digest-bound preparation and
  one-use confirmation, an ephemeral WebKitGTK custom-scheme fixture helper,
  bounded evidence, terminal failure handling, replay prevention, recovery,
  and immutable audit linkage.
- The fixture can neither navigate externally nor reuse a browser profile or
  credential. It adds no real browser target, connector, provider, MCP,
  automation, file chooser, download, upload, form, shell, Git, deployment, or
  external mutation authority. Beta.61 is retained as a failed reproducibility
  candidate; beta.62 normalizes worker-package timestamps, passed complete
  installed-host acceptance, and is published as a prerelease.

## M59 — Context Assembly and Transmission Contract

- Ratify the decision-only contract for explicit, deterministic, bounded,
  redacted context selection, prepared-bundle review, digest-bound one-use
  fictional transmission authorization, ambiguity handling, retention, and
  audit. It starts no runtime implementation and performs no transmission.

## 0.1.0-beta.60 — Fictional connector governance

- Implement the M57 fictional/local-only connector-governance vertical slice:
  static descriptor/capability declaration, project/task validation, separate
  local read and fictional mutation paths, digest-bound one-use confirmation,
  cancellation, replay rejection, ambiguous outcome/no-retry handling, and
  content-free durable binding/operation/audit records.
- Add no network, real connector, credential, OAuth, browser/M58, MCP dispatch,
  provider transmission, retrieval, automation, external mutation, shell, Git,
  cloud, or deployment authority. Beta.60 completed installed-host validation
  and is published as a prerelease; deployment remains unapproved.

## 0.1.0-beta.59 — Durable Source Admission fail-safe correction

- Replaces beta.58 after installed-host acceptance found that its synchronous
  local-file chooser could block the main-thread command path and a cancelled
  prepared review could retain staged private bytes until a later controller
  call or restart. File selection is now callback-based and bounded; cancelled
  reviews and normal application exit remove staged bytes without creating a
  durable-source record or final private copy.
- Beta.58 artifacts, installed receipt, evidence, and release state remain
  preserved. No retrieval, provider/context transmission, credentials,
  connectors, browser authority, MCP, or automation is added.

## 0.1.0-beta.58 — Durable Source Admission

- Adds governed project-owned local textual source admission with explicit
  review and confirmation, immutable identity, SHA-256, bounded preview, and
  private copied content.
- Keeps retrieval, provider/context transmission, credentials, connectors,
  browser authority, MCP, and automation out of scope.

## 0.1.0-beta.57 — Project-bound durable Task Catalog creation

- Replace beta.56 after installed-host acceptance proved that its Task Catalog
  created an unbound `Untitled task`. New task creation now requires an explicit
  normalized title and selected opaque project identity, which native validates
  atomically against a live attached association before persistence. The
  creation dialog discloses the existing empty primary-plan contract and blocks
  repeated submission. No conversation, provider, credential, network,
  retrieval, tool, browser, connector, or external authority is added.
- Beta.56 remains release-ineligible. Its source commits, package artifacts,
  receipt, logs, and installed-host failure evidence remain preserved.

## 0.1.0-beta.56 — Installed durable-task workflow correction

- Restore the governed Task Catalog and its explicit New task control to the
  primary New task workspace. A normal durable task can now be created for the
  selected project and selected by the local-only fictional mock workbench.
  Production-preview acceptance covers this exact path; no provider, network,
  credential, or external authority is added.

## 0.1.0-beta.55 — Provider-Neutral Mock Workflow

- Harden the local fictional mock-inference workflow with registry-backed
  Lantern/Ember destinations, bounded polling, separately observable
  cancellation confirmation, fail-closed authority fixtures, and representative
  desktop/mobile browser acceptance. This beta packages only a fictional,
  deterministic, local workflow: it adds no provider, network, credential,
  retrieval, connector, browser, tool, or external authority.
- Beta.55 passed source, package, installation, and native-receipt gates but is
  release-ineligible: its installed New task workspace did not expose the
  required Task Catalog/New task path. Its artifacts and receipt remain
  preserved as failed-candidate evidence and are not reinterpreted.

## Unreleased — Provider-Neutral AI Foundation local mock slice

- Add a user-visible, deterministic local mock-inference workbench bound to an
  existing project task. It uses fictional static destinations, bounded
  authored text, a reviewed transient manifest, inert opaque lease, one-use
  digest-bound authorization, canonical mock events, bounded usage, and
  content-free evidence. It adds no provider route, network access, credential
  material, context transmission, retrieval, native operation, persistence,
  package, or release authority.

## 0.1.0-beta.54 — Milestone 56 complete

- Close M56 at package/source commit
  `e2b084ed0bdf17fb6f4b0b47663cdf6952ec8e73`, tagged
  `v0.1.0-beta.54`. Inspectable local templates use migration 21 and bounded
  digest-only application reservations use migration 22; four static built-ins
  remain outside SQLite. The closed lifecycle, strict bridge, lazy management
  UI, digest-bound application workflow, accessibility, and focused browser
  acceptance are complete.
- G9 source acceptance, G10 canonical pinned-Ubuntu-22.04 packaging, and G11
  restricted installed-host validation passed. The canonical beta.54 pair is
  `quireforge_0.1.0.beta.54_amd64.deb` (5,864,924 bytes,
  `643e6bc3caf9068f7ed521ecd949f9f3f5d38b9c6a82bcce19384370f644d131`) and
  `quireforge-sandboxd_0.1.0.beta.54_amd64.deb` (3,233,492 bytes,
  `bd9c0682c0e9dd7761b28f03eb2e801ab7a925e7c5f5587eefc68bd7578bd21f`).
  The `James-Jennison/quireforge` draft prerelease contains those packages,
  `SHA256SUMS`, and `release-manifest.json`.
- Both beta.54 packages remain installed after clean integrity and restricted
  boundary checks; headless completion returned `created`, then `existing`, and
  no rollback was required. The prerelease remains draft: it has not been
  published or deployed. Beta.53 remains preserved as the prior released
  rollback generation; beta.52 remains a preserved failed installed-host
  candidate. Research, retrieval, providers, connectors, MCP, OAuth, browser
  authority, credentials, import/export, hidden instructions, automatic
  execution, approval, and dispatch remain excluded.

## 0.1.0-beta.53 — Milestone 54 complete

- Close M54 at package/source commit
  `c4c2752466f36f791fde47edbc5c6b02b0e21320`: the pinned Ubuntu 22.04
  Debian package pair, lifecycle, smoke, visible-launch, and restricted
  installed-host validation passed. Headless completion returned `created`,
  then `existing`; beta.53 is installed.
- Release the seven closed M54 evidence sources: manual validation, M48
  generated-artifact metadata, safe-preview metadata, Git status/diff summary,
  Activity presentation, Approval presentation, and package-manifest summary.
  Activity uses its native append-only ownership ledger; Approval uses the
  immutable Advisor-dispatch task origin.
- Preserve beta.52 as a failed installed-host candidate. It is not released;
  beta.47–beta.51 history remains unchanged and no beta.47 package/release is
  fabricated.

## Preserved beta.51 history

### Added

- Extend the fixed `--complete-installed-host-validation` executable branch
  with native migration, unprivileged-record bootstrap, installed-version
  gating, and digest-bound predecessor revalidation before the installed-host
  phase. It starts no Tauri/GUI surface and emits only a bounded status.

- Preserve a coherent canonical release set in a versioned archive before a
  newer candidate is promoted; malformed, partial, or conflicting sets fail
  closed without replacement.

- Treat canonical `SHA256SUMS` as a closed artifact filename-to-digest mapping,
  preserving valid prior checksum ordering byte-for-byte during archival.

- Centralize release checksum parsing, emission, and validation so every
  desktop-release authority uses the same order-independent closed mapping.

- Begin the approved M54 local-review implementation: native-owned, bounded
  task-scoped SQLite collections for copied text, static image mockups, and
  typed evidence; inert previews; local annotations; non-Git text comparison;
  and explicit, digest-bound one-time promotion into the existing transient
  M48 artifact registry. Review remains unable to access paths, arbitrary
  files, network, Git, terminal, browser, provider, approval, dispatch,
  execution, publishing, or deployment authority.

- Add unreleased post-beta.51 M54 package-manifest-summary capture. It accepts
  only a Local Review collection and optimistic collection timestamp, resolves
  the immutable same-project completed migration-18 record natively, and
  persists only the redacted closed summary envelope with an inert stored-bytes
  preview. No package build, installation, validation run, or host operation
  is performed.

- Add unreleased post-beta.51 M54 git-status-diff-summary capture using the
  bound project’s native Git service. It persists only closed aggregate facts
  and offers a persisted-bytes-only inert preview; it cannot alter repository
  state or expose paths, refs, remotes, patches, commands, or diagnostics.

- Add native append-only Local Review activity ownership for future Activity
  presentation evidence; frontend session events remain non-authoritative.

- Implement M52's native-owned local task catalogue with migration 11, opaque
  UUIDv7 task and plan identities, explicit status/archive/delete lifecycle,
  bounded title/label search, one to four visible plan records, closed capacity
  limits, and an accessible compact workbench surface. Plans retain no
  conversation, path, attachment/artifact history, approval, dispatch,
  execution, terminal, Git, credential, provider, browser, or Advisor state.
- Clear the current transient conversation-attachment selection before a plan
  switch and keep every other task-plan authority excluded. Switching a plan
  cannot start, resume, clone, save, dispatch, approve, execute, or transport
  anything.
- Record M51's decision-ready durable-task and alternate-plan proposal. It
  selects bounded local SQLite organizational records and strict plan authority
  isolation. M52 now implements that separately approved contract.

### Changed

- Refine M50's existing workbench with bounded, keyboard and pointer operable
  task-evidence and terminal-dock resizing, explicit collapse/restore controls,
  and a strict local presentation-only layout preference record. No execution,
  terminal, review-pane, task, path, transcript, approval, Git, or provider
  authority changes.

### Added

- Add M49's closed-by-default, independently lazy Files, Diff, Git, Preview,
  Activity, and Approval review panes. They consume only existing typed,
  bounded evidence services and request Git or generated-artifact data only
  after the corresponding pane opens.

### Added

- Implement M48 `advisor-generated-artifact-registry-v1`: a native-only,
  process-local, five-class text registry with bounded preview, discard,
  expiry, and explicit one-time no-replace native Save.

### Added

- Add Milestone 46's bounded Advisor attachment collection: up to three
  existing typed attachments, one image at most, native aggregate preflight,
  and one explicit collection confirmation. No generic upload, new file type,
  new app-server method, persistence, or authority is added.
- Record Milestone 47's decision-ready generated-artifact workflow proposal:
  a bounded transient text/data registry and explicit atomic no-replace native
  Save boundary for later approval. This documentation-only milestone changes
  no desktop package or runtime behavior.

## 0.1.0-beta.41

### Added

- Add Milestone 44's compact single-file attachment tray. One **Attach a file**
  entry selects a closed, existing text/data, PNG/JPEG, PDF, ZIP, or ELF native
  picker. It keeps one attachment at a time and does not add generic upload,
  drag-and-drop, new file types, transport, persistence, or authority.

## 0.1.0-beta.40

### Changed

- Replace the preliminary beta.39 envelope with one bounded, full M44–M58
  construction-period policy: 448 KiB application shell, 1.5 MiB total
  JavaScript, and 160 KiB CSS. The 256 KiB startup entry remains unchanged.
  All limits remain enforced, do not increase automatically, and require a
  future evidence-based permanent-budget reconciliation before product readiness.

## 0.1.0-beta.38

### Added

- Add Milestone 43's explicit, transient Shared Task Continuity envelope. A
  reviewed Advisor brief can be opened once in QuireForge, and a bounded
  QuireForge completion receipt can be reviewed once in Advisor. The envelope
  is native-memory-only and carries no attachment payload, transcript, project,
  terminal, Git, approval, dispatch, execution, path, or authority state.

### Changed

- Add the post-M41 packaging-efficiency checkpoint. The pinned Ubuntu 22.04
  release workflow now reuses only checksum-verified immutable Linux-kernel and
  Firecracker source archives; it still rebuilds guest outputs in a disposable
  directory and verifies every source before extraction. Release provenance,
  ABI, lifecycle, and visible-launch gates remain mandatory.

## 0.1.0-beta.36

### Added

- Add Milestone 41's bounded Advisor conversation usability refinement. The
  transcript is the independent scrolling region, readers can pause
  follow-latest and jump back to the newest reply, the anchored composer no
  longer obscures the final response, and the optional details drawer is
  independently scrollable with keyboard focus restoration. No Advisor
  authority, transport, attachment, retention, or execution boundary changes.

- Add Milestone 40's QuireForge Task Workbench Shell. The task conversation
  remains primary while an opt-in workbench context drawer, compact safe-actions
  palette, and collapsed existing managed-terminal dock improve task navigation
  without adding shell, PTY, execution, dispatch, project-write, provider, or
  transport authority.

- Add a post-M39 corrective checkpoint for the Advisor/QuireForge workspace
  boundary acknowledgement. The first ordinary switch under
  `advisor-quireforge-boundary-v1` requires confirmation; the exact closed,
  non-sensitive acknowledgement suppresses later ordinary prompts under that
  unchanged policy. Every completed switch still clears transient Advisor state
  and preserves the existing capability and context-isolation boundary. No
  project, transcript, attachment, approval, dispatch, terminal, Git,
  worktree, execution, path, or credential data is persisted with the
  acknowledgement.

- Add Milestone 39's separately installed, root-owned Linux x86_64 KVM worker
  for one confirmed static ELF64 x86_64 sample. The worker uses pinned
  Firecracker 1.15.1 and matching jailer with a zero-network, no-host-mount
  disposable guest and returns only bounded `dynamic-analysis-result-v1`
  metadata. Distribution is Debian-only. Dynamically linked
  samples, generic uploads, Advisor execution, terminal output, guest-file
  export, and automatic execution remain unavailable.

- Complete Milestone 38 as a decision-only dynamic-sandbox discovery gate. It
  records a no-go for dynamic or malware analysis in the present desktop
  architecture; no product behavior, sandbox, execution authority, dependency,
  package version, release, or deployment was added.

- Complete Milestone 37 as an evidence-only human-in-the-loop acceptance gate.
  Existing digest-bound approval, one-time dispatch, bounded completion,
  mode-reset, and recovery contracts passed alongside user-authorized managed
  Codex Advisor and read-only execution-profile checks. This adds no product
  behavior, authority, persistence, package version, release, or deployment.

- Add Milestone 36's closed ELF-only Advisor static-binary entry. One confirmed
  transient ELF32/ELF64 relocatable, executable, or shared-object source is
  reduced to a bounded `static-binary-manifest-v1` text projection. QuireForge
  never loads, executes, debugs, emulates, transports, or retains binary bytes,
  paths, symbol names, section names, interpreter/RPATH/DT_NEEDED values, notes,
  debug data, raw headers, or addresses.

- Harden Linux packaging so host development bundles are isolated and
  non-distributable, while only the pinned Ubuntu 22.04 workflow can generate
  authoritative release artifacts with fail-closed provenance and shipped-ELF
  GLIBC evidence. This post-M35 corrective checkpoint does not change M35's
  ZIP archive capability scope; beta.30 remains its immutable release set.

## 0.1.0-beta.30

### Added

- Add Milestone 35's closed ZIP-only Advisor archive entry. QuireForge accepts
  one explicitly confirmed transient ZIP source per send, rejects unsafe entry
  metadata, and sends only a bounded `archive-manifest-v1` text projection;
  archive contents are never extracted, decompressed, rendered, or transported.

- Add Milestone 34's compact Advisor/QuireForge workspace selector with the
  existing explicit capability-boundary confirmation and state clearing. Remove
  the redundant page-level conversation-mode selector and use QuireForge in
  user-facing workspace wording without changing managed Codex integration
  terminology.
- Adopt enforced temporary 1,280 KiB JavaScript and 134 KiB CSS desktop bundle
  ceilings for the active UI-construction period, with recorded actual sizes and
  a mandatory permanent-budget reconciliation before product readiness.

- Added one transient, bounded PDF projection for Advisor. Selected PDF bytes
  and source paths stay native-only; Advisor receives only a confirmed,
  path-free plain-text projection for one send.
- Harden PDF active-content inspection across bounded nested and indirect
  objects, and make projection page accounting accurate when truncation occurs.
- Supersede the unintegrated beta.21 and beta.22 candidates with final beta.23
  PDF-boundary coverage and release evidence.
- Define the PDF parser boundary: pinned lopdf accepts low-level PDF syntax;
  QuireForge owns the source boundary, bounded post-parse policy, projection,
  lifecycle, and path-free diagnostics. Beta.24 supersedes beta.23 locally.
- Add direct QuireForge-owned PDF source-boundary and path-free diagnostic
  coverage. Beta.25 supersedes beta.24 locally.
- Add direct reset and bounded Advisor transport assertions; beta.26 supersedes
  beta.25 locally.
- Refine Advisor into a transient chat-first workspace with a compact header,
  centered scrollable conversation, sticky composer, and optional closed details
  drawer. This changes no Advisor authority, transcript retention, or execution
  boundary; beta.28 supersedes the pre-commit beta.27 candidate locally.

- Add Milestone 32’s bounded, transient Advisor conversation viewport and
  confirmed Advisor/Codex mode picker. Mode changes clear temporary Advisor
  context, attachments, approvals, dispatch state, completion reports, and
  visible transcript data; no transcript storage or execution authority is
  added.

- Begin Milestone 31’s closed Advisor image registry entry: one transient,
  explicitly confirmed PNG or JPEG with native type/decode/dimension checks,
  a path-free manifest and preview, and a documented memory-backed `localImage`
  handoff. It adds no project, execution, credential, or generic-upload path.
- Formalize the post-M30 roadmap sequence: bounded image analysis, history
  viewport and Advisor/Codex mode picker, document, archive, and static-binary
  analysis, an end-to-end acceptance gate, and an isolated dynamic-analysis
  discovery gate. No future capability is implemented by this documentation.
- Add Milestone 30’s first closed Advisor Content Ingestion Registry entry: one bounded,
  user-confirmed normalized UTF-8 text/data file held only in native memory for
  one send. Image, document, archive, and static-binary registry categories are
  reserved without handlers, parsers, transport, or UI.
- Add explicit user-selected, previewed single-file export of visible Advisor
  text, Python, CSV, JSON, or Markdown output through the native Save dialog.
  Exports do not overwrite existing files or grant Advisor project authority.
- Raise only the measured desktop total-JavaScript regression ceiling from 884
  KiB to 890 KiB for the bounded Advisor text/data entry; entry,
  application-shell, and CSS ceilings are
  unchanged.

- Fix managed Advisor startup to send the reviewed QuireForge `clientInfo`
  initialization identity required by the Codex app-server. Advisor remains
  no-project, read-only, no-tools, no-network, and non-executable.
- Start Advisor with the minimal read-only app-server profile and retry once
  with an explicit empty-capability compatibility profile only after a rejected
  thread start. The UI reports that closed failure category without retaining
  or exposing raw server diagnostics.
- Render consecutive transient Advisor message fragments as one readable
  message while keeping reasoning and error entries separate.
- Accumulate same-conversation Advisor poll fragments only in transient UI
  memory, preserving complete replies across polls without local transcript
  storage.
- Add bounded recovery copy for managed-Codex protocol and runtime failures;
  raw app-server diagnostics remain undisclosed.
- Raise only the measured desktop total-JavaScript regression ceiling from 880
  KiB to 884 KiB for the integrated Approval/Dispatch B2/B3 flow and Advisor
  initialization compatibility fix; the entry, application-shell, and CSS
  ceilings remain unchanged.

## 0.1.0-beta.13

- Pin the transitive Node `brace-expansion` resolution to `5.0.8`, removing
  the high-severity Dependabot advisory from the linting toolchain.
- Retain ESLint 9 and its React/Astro accessibility coverage while adding a
  time-bounded, fail-closed review record for its remaining development-only
  `brace-expansion` audit path. Raw Node audit output remains available.
- Raise only the measured desktop total-JavaScript regression ceiling from 875
  KiB to 880 KiB for the shipped digest-bound Advisor Phase A controller; the
  entry, application-shell, and CSS ceilings remain unchanged.
- Add the unnumbered Advisor Approval/Dispatch Phase A controller: transient
  editable drafts and expiring digest-only explicit approval/rejection records.
  It cannot dispatch, start Codex, run commands, or change a project.
- Separate the Advisor capability notice from its action row so the disabled
  send control remains readable at desktop and narrow widths. The visible
  disabled reason now explains whether sign-in, an Advisor message, or a
  pending native action prevents sending; Project State remains optional and
  still requires explicit selection and per-send confirmation.
- Close the clean `0.1.0-beta.7` pinned Ubuntu 22.04 Debian/AppImage gate with
  manifest/checksum agreement, disposable lifecycle validation, visible
  container launches, and approved installed-host visual confirmation.

## 0.1.0-beta.6

- Complete Milestone 29’s managed Advisor conversation foundation. Its fixed
  Codex app-server profile requires managed ChatGPT browser authentication and
  rejects project, tool, approval, terminal, Git, worktree, network, and
  API-key authority. QuireForge retains only opaque Advisor thread metadata;
  Advisor prompt/response text stays transient. A temporary safe Project State
  summary requires a second, per-send confirmation before inclusion.
- Close the clean `0.1.0-beta.6` pinned Ubuntu 22.04 Debian/AppImage package
  gate with manifest/checksum agreement, desktop/icon and GLIBC checks,
  disposable lifecycle validation, visible launches, and installed-host smoke.

## Unreleased — 0.1.0-beta.5

- Begin Milestone 28’s reference-only Advisor foundation with strict
  Rust/TypeScript contracts and bounded SQLite metadata for opaque references,
  provenance/freshness, selected context, and non-dispatching proposal digests.
  The fixed `#advisor` route receives and renders only a strict safe-summary
  projection; it stores no prompt, response, transcript, credential, session,
  or arbitrary project-path data and introduces no model call or execution
  capability.
- Add an explicit confirmation gate for one temporary Advisor Project State
  source. Its native command is fixed to the existing attached-project-only
  `local-only`/`metadata-only` reader and returns only a strict safe projection;
  it never receives a path, Git argument, remote mode, artifact mode, document,
  image, project identity, or raw repository content.
- Close Milestone 28 with the clean `0.1.0-beta.5` pinned Ubuntu 22.04
  Debian/AppImage package gate: manifest/checksum agreement, desktop/icon and
  GLIBC checks, disposable lifecycle validation, visible launches, and
  installed-host smoke all passed. No release was published.
- Complete Milestone 27’s managed-ChatGPT-only conversation engine: closed
  Chat/Codex capability contracts, bounded no-project Chat metadata, and a
  fixed native Chat bridge that rejects project, tool, approval, and API-key
  authority.
- Persist only the explicitly confirmed Chat/Codex workspace preference, with
  a safe Codex fallback for missing or invalid local values and no transfer of
  project context or conversation content.
- Close the clean `0.1.0-beta.4` pinned Ubuntu 22.04 Debian/AppImage package
  gate with manifest/checksum agreement, desktop/icon checks, disposable
  lifecycle validation, visible launches, installed-host smoke, and no public
  release publication.

All notable project changes will be documented here. The project has not
released a usable application.

## Unreleased

### Fixed

- Documented same-version Debian candidate replacement with `apt install
--reinstall` so a locally rebuilt candidate cannot leave an older installed
  frontend bundle in place.
- Kept multiline native conversation tasks in one guarded start action and one
  app-server text input. Re-entrant submissions are ignored while the first
  start is pending, and request, native-command, and native-response failures
  now produce distinct bounded recovery guidance without exposing raw errors.
- Corrected the compact desktop usage summary so unscoped Codex runtime meters
  do not replace the general Codex weekly meter. The sidebar shows only the
  exact reported `codex` seven-day window and its paired reset time; it remains
  nonnumeric when that window is unavailable.
- Prepared immutable `0.1.0-beta.2` artifacts after GitHub Releases normalized
  the tilde in the beta 1 Debian asset name. The downloadable file is now
  `quireforge_0.1.0.beta.2_amd64.deb`, while its Debian control version remains
  `0.1.0~beta.2` so the eventual stable package sorts after the prerelease.
- Added a release-contract helper and Python/website assertions that keep the
  GitHub-safe outer Debian filename distinct from its internal package version.

### Added

- Milestone 27 foundation: managed-ChatGPT-account feasibility contract,
  explicit Chat/Codex capability separation, and accessible Settings
  destinations. Chat rejects API-key readiness and has no project, terminal,
  Git, worktree, integration, or native-action authority.
- Milestone 26 Appearance Themes: eight closed, accessible local palettes;
  live settings preview; keyboard selection; local restoration; and direct
  desktop/mobile visual regression coverage without a native, backend,
  repository-state, or automation change.
- Milestone 25 desktop visual polish: denser branded sidebar and top bar,
  clearer dark-theme hierarchy, centered task entry, rounded conversation
  composer, and desktop/mobile visual-accessibility coverage without new
  native behavior, external branded assets, or automation.
- Milestone 24C Project state workspace: a demand-driven, read-only route over
  the existing normalized repository-state snapshot, with explicit
  local-only/metadata-only access and no fetch, mutation, approval, handoff
  generation, contradiction resolution, or background scanning. Its complete
  frontend/native gate and fresh pinned Ubuntu 22.04 Debian/AppImage lifecycle
  and launch evidence are recorded without publishing a release.
- Milestone 24B repository-state reader checkpoint: attached-project-only,
  typed local/tracking/explicit-fetch reads with contract diagnostics and no UI
  or repository mutation except the separately authorized fetch mode.
- Milestone 24B final reader and Ubuntu 22.04 package evidence: strict
  producer-compatible package/validation/handoff evidence, deterministic
  fixture safety coverage, and clean Debian/AppImage lifecycle and launch smoke
  validation without a project-state UI or automation.
- Milestone 24A project-state contract: strict Rust/Zod serialization,
  provenance/trust, approvals, checkpoints, validation/package evidence,
  blockers, contradictions, and handoff state with no reader, UI, or automation;
  its final implementation commit has fresh pinned Ubuntu 22.04 package evidence.
- Milestone 23 UI-platform feasibility evidence and ADR 0028 decision: retain
  Tauri conditionally, preserve the Rust-domain/Tauri-adapter boundary, and
  reconsider Qt 6 only on documented measurable triggers. This documentation
  change adds no Qt prototype, migration, dependency, package, or capability.
- Milestone 22B presentation refinements: Home now surfaces the current
  workspace using existing project data, while shared desktop header rules keep
  New task, Threads, Projects, Changes, Worktrees, and Terminal aligned without
  changing route or native-bridge behavior.
- Milestone 22B route-surface refinement: Scheduled, Integrations, Files, and
  Settings now share calmer surface elevation, heading rhythm, responsive
  status wrapping, and mobile-drawer Playwright evidence without changing
  native actions or route ownership.
- Completed Milestone 22B's presentation, accessibility, responsive, browser,
  native, formatting, and pinned Ubuntu 22.04 package-validation gates without
  adding product capabilities or changing approved ownership boundaries.
- Milestone 22 routed desktop workspace: ten typed, deep-linkable primary
  destinations; preserved mounted tool state; route-aware toolbar and status
  bar; independent pane scrolling; contextual inspectors; compact navigation;
  and a responsive off-canvas drawer without a second routing dependency.
- A dedicated QuireForge Settings workspace. The interactive account row opens
  Accounts & connections with only supported normalized Codex refresh, usage,
  and two-step logout controls; Appearance and About remain explicitly local.
- Desktop/mobile routing, focus, accessibility, state-restoration, no-page-
  scroll, and opt-in visual-evidence coverage for the routed shell.
- A dedicated full-history public-disclosure audit covering every reachable
  branch, tag, pull-request head, Git object, collaboration record, Actions
  run/log, and exact release artifact; no credential, token, private key, or
  private server address was found.
- ADR 0027 and enforced workflow validation for the public-source boundary:
  persistent organization runners remain selected for trusted QuireForge
  branches, while fork-origin pull requests and `pull_request_target` are
  prohibited from reaching self-hosted jobs.
- Public source, issue, pull-request, and contribution links in the static
  website content model without changing the owner-hosted deployment.
- Milestone 21B local beta-readiness preflight: a fresh clean Ubuntu 22.04
  package/lifecycle pass, repeated byte-identical normalization, current-host
  AppImage and extracted-Debian launches, signed-out package pixel review, and
  reviewed initial installation, limitation, distribution, and rollback copy.
- Dormant website release validation for exactly one version-coherent AppImage
  and Debian package with positive sizes, lowercase SHA-256 values, UTC
  publication time, and credential-free same-origin HTTPS package, checksum,
  and manifest URLs. Public download state remains unavailable.
- Milestone 21A product readiness: an authenticated startup gate, original
  responsive QuireForge home and three-region workspace hierarchy, internal
  milestone-label removal, account summary, recent threads, project and quick
  actions, and compact/full remaining-usage presentation.
- A fixed read-only `account/rateLimits/read` native service and strict
  TypeScript contract that expose only bounded usage percentages, reset times,
  window durations, labels, and reached state. Plan, balance, account,
  reset-credit, and raw protocol metadata are discarded; unavailable and
  unmetered states are never estimated.
- Pre-authentication loader gating and deterministic desktop/mobile coverage
  proving that workspace, project, conversation, session, terminal,
  integration, and usage reads do not start before normalized Codex access is
  granted.
- Milestone 20 local Linux packaging for `0.1.0-beta.1`: normalized x86_64
  AppImage and Debian candidates, canonical desktop/AppStream metadata,
  checksums, a strict release manifest, and an Ubuntu 22.04 baseline container
  with digest-pinned Node, Rust, and operating-system inputs.
- Checksum-pinned Tauri Linux tool acquisition, deterministic package
  normalization, GLIBC-baseline and metadata inspection, disposable
  install/upgrade/uninstall preservation tests, isolated visible X11 launch
  probes, and inactive typed website download data.
- A manual-only, immutable-revision GitHub release workflow with verify-only
  artifact review and separately gated tag, confirmation, protected
  environment, clean-source, attestation, and prerelease publication controls.
- Dedicated repository and an explicit unofficial-project disclaimer.
- Milestone 0 Codex integration, compatibility, feature-parity, architecture,
  threat-model, GitHub Pages, roadmap, and architecture-decision documentation.
- Permanent QuireForge identity contract covering application, package,
  repository, GitHub Pages, integration-client, and XDG storage identifiers.
- Original path-based QuireForge mark, wordmark, light/dark lockups, favicon,
  application-icon source, social card, palette, and brand usage guidance.
- Cloudflare Pages capability findings and a deployment plan.
- Webuzo static-hosting architecture, isolated deployment/rollback planning,
  and an Apache-compatible artifact policy that supersede the unimplemented
  Cloudflare Pages deployment plan.
- Origin-only Webuzo staging for the private-safe static artifact, including an
  isolated provider-managed destination, route/header/redirect validation,
  trusted origin TLS, and a verified restoration rehearsal. No public DNS or
  Cloudflare setting changed during staging.
- Approved production activation at
  `https://quireforge.jamesjennison.net` using one proxied Cloudflare record,
  Full (Strict) origin validation, domain-scoped HSTS, immutable hashed-asset
  caching, verified backup/restore, and public desktop/mobile quality checks.
  No `www`, mail, wildcard, analytics, public source, or unrelated DNS record
  was added.
- Enrolled only the canonical QuireForge hostname in Webuzo-managed automatic
  origin TLS. Trusted certificate coverage, provider-managed renewal state, and
  pre/post recovery checks passed without retaining operational identifiers.
- Sanitized owner-mediated Cloudflare account audit covering the Free plan,
  Workers & Pages availability, DNS, managed TLS, and security settings.
- ADR 0006 selecting Cloudflare Pages as the production website host and
  authoritative DNS.
- Apache License 2.0 and repository-wide contributor guidance.
- Security, contribution, conduct, support, issue, and pull-request policies.
- GitHub Actions dependency updates and a minimum-permission repository-checks
  workflow pinned to a reviewed checkout revision.
- A dependency-free repository validator for required files, local links,
  QuireForge identity contracts, SVG XML, text encoding, and high-confidence
  secret patterns.
- A pinned pnpm monorepo and Astro 7 static website under `apps/website` with
  15 public pages, a custom 404, sitemap, robots policy, manifest, and canonical
  and social metadata.
- Reusable QuireForge website components, centralized light/dark design tokens,
  responsive navigation, visible keyboard focus, reduced-motion support, and
  generated favicon, application-touch, and social-card assets derived from the
  approved vector sources.
- Deterministic website type, lint, format, unit, artifact, route, responsive,
  theme, and axe-core accessibility checks in local scripts and minimum-
  permission GitHub Actions.
- Strict Apache/Cloudflare security headers with a static-site content policy
  and domain-scoped HSTS enabled only after live origin and edge HTTPS passed.
- A private-safe public content model that removes source, issues, releases,
  contribution workflows, detailed milestones, and development activity from
  the generated website while preserving all established routes.
- Website build and testing documentation with an explicit no-deployment
  boundary.
- A pinned Tauri 2, React 19, TypeScript, Vite, and Rust desktop package under
  `apps/desktop`, with the accepted Linux application identity and original
  QuireForge icon exports.
- Generalized local build-performance and milestone-forecast histories for
  system-calibrated planning.
- A cumulative real-world milestone time ledger that reconstructs every prior
  milestone with explicit evidence/confidence and prospectively separates
  active, automated-wait, user-blocked, counted-project, and elapsed time.
- A responsive, accessible light/dark desktop shell and one versioned
  `desktop_bootstrap` command validated against a shared Rust/TypeScript fixture.
- Desktop type, lint, format, unit, native contract, Clippy, build, responsive,
  theme, overflow, and axe-core accessibility gates in local scripts and CI.
- A versioned `CodexBackend` boundary, fixed-command CLI detection, supervised
  JSONL app-server lifecycle, normalized capability/model contracts, and
  explicit unavailable/degraded diagnostics.
- Deterministic Codex mocks, bounded protocol failure tests, and a reproducible
  generator that commits only the reviewed initialize and `model/list` schema
  subset for Codex CLI 0.144.6.
- A narrow `codex_runtime_probe` Tauri command and strict TypeScript runtime
  schema that prevent raw app-server, account, installation, path, or user-agent
  fields from reaching React.
- A Codex-owned authentication service with normalized account status,
  allowlisted browser/device handoffs, exact login correlation, cancellation,
  explicit two-step logout, stable redacted diagnostics, and deterministic
  lifecycle/failure tests.
- An accessible Milestone 5 onboarding panel that never displays or persists
  email, account IDs, tokens, raw errors, or completed sign-in URLs.
- A native Milestone 6 project core with app-owned migrated SQLite metadata,
  UUIDv7 identities, selected/resolved directory evidence, Git/worktree and
  project-instruction detection, explicit attach/relink/detach/archive
  lifecycle operations, and fail-closed cwd preflight.
- Deterministic project security tests for symlink and configuration changes,
  duplicate roots, linked worktrees, invalid Git pointers, read-only and
  missing directories, malformed IDs, schema ownership, metadata permissions,
  and the no-source-deletion boundary.
- A strict normalized project-workspace contract, fixed-command TypeScript
  bridge, and accessible responsive project UI for native selection,
  confirmation, missing/read-only states, preflight, relink, and two-step
  detach/archive actions.
- A native Milestone 11A managed-worktree service with normalized bounded
  inventory, app-generated private destinations, native-picker attachment,
  source-HEAD and identity revalidation, disabled hooks/configured checkout
  filters, and five-minute one-use confirmations.
- Schema migration 4 linking each managed or attached worktree to an ordinary
  QuireForge project, plus a responsive accessible inventory/create/attach UI
  that deliberately exposes no remove, prune, or cleanup action.
- A bounded Milestone 11B native conversation registry for up to four distinct
  project tasks, with per-task process locks, project reservations, exact
  polling/approval/interruption routing, refresh recovery, and deterministic
  multi-process child-reaping and capacity tests.
- A strict normalized active-task contract and responsive parallel worktree
  monitor with independent live activity selection and read-only changed-file
  and conflict counts. It exposes no Codex IDs, cwd, raw protocol/process
  fields, conflict resolution, or cleanup action.
- A Milestone 11C native recovery and cleanup boundary that issues opaque
  recovery IDs for retained private app-managed checkouts and removes only
  clean, unlocked, inactive `managed` worktrees after repository-group
  reservation and confirmation-time relation, identity, branch, `HEAD`, and
  status revalidation.
- Strict recovery/remove IPC, an accessible destructive-review interface, and
  adversarial fixtures for dirty-after-preview, attached/external ownership,
  symlink replacement, active work, configured hooks/filters, branch retention,
  metadata-only recovery, and post-Git metadata failure. Force removal, generic
  prune, branch deletion, direct directory deletion, and conflict resolution
  remain unavailable.
- A native Milestone 12 PTY service with up to eight app-owned tabs, freshly
  revalidated project cwd, controlled noncredential environment inheritance,
  bounded byte-safe input/output and resize, Linux session-member cleanup, and
  metadata-only restart recovery.
- A strict terminal IPC/Zod contract and responsive accessible xterm interface
  with independent tabs, honest browser preview, explicit foreground/background
  process-ending confirmation, truncation/recovery notices, and a clear
  Linux-account privilege boundary separate from Codex approvals. Shell input,
  output, history, paths, environment, TTY, and process identity are neither
  persisted nor exposed through the webview contract.
- A serialized native Milestone 7A conversation service that revalidates the
  attached cwd, starts a supported Codex thread and turn with explicit model,
  reasoning, sandbox, and approval controls, normalizes bounded stream events,
  and interrupts the exact owned turn.
- Reference-only conversation persistence, active-project execution
  reservation, reviewed Codex 0.144.6 thread/turn schemas, strict Rust and
  TypeScript contracts, and deterministic lifecycle, cancellation, policy,
  mismatch, path-boundary, and redaction tests.
- A responsive Milestone 7B conversation workspace with a bounded task
  composer, runtime-derived model/reasoning choices, explicit filesystem and
  approval controls, normalized progress and response rendering, stable
  diagnostics, and exact app-owned stop behavior.
- Deterministic conversation UI tests for prerequisite gating, unsafe-policy
  rejection, start/poll/terminal transitions, event deduplication and bounds,
  browser-preview honesty, responsive layout, and accessibility.
- A native Milestone 8A lifecycle boundary for app-reference-only session list,
  read, resume, fork, archive, and restore operations against revalidated
  attached directories and supervised Codex app-server processes.
- SQLite schema version 3 with bounded parent-app lineage and archive timestamps,
  plus startup reconciliation that marks stale active work interrupted and
  clears obsolete active-turn ownership without deleting Codex or project data.
- Strict Rust/TypeScript lifecycle fixtures, fixed Tauri commands, reviewed
  Codex 0.144.6 lifecycle schemas, bounded exact-cwd reconciliation, and
  deterministic mismatch, recovery, fork-lineage, archive/restore, child-
  cleanup, and raw-identity/path rejection tests.
- A Milestone 8B session-history workspace with bounded Codex-authoritative
  title search, project and fork grouping, keyboard-operable tabs, transient
  titles, and accessible exact-reference resume, fork, archive, and restore
  controls.
- Deterministic native, component, shell-integration, responsive, overflow, and
  axe-core coverage proving that title filtering does not corrupt complete
  reconciliation or expose paths, Codex IDs, previews, transcripts, or raw
  protocol records.
- A Milestone 9A native approval boundary for exact command, file, and
  permission requests, with app-owned UUIDv7 correlation, one-turn
  approve/decline/cancel decisions, turn-scoped permission grants, safe pending
  cancellation, stale/unavailable-decision refusal, and conservative crash
  recovery without approval persistence.
- Detailed schema-v2 activity events with stable app IDs, sanitized command and
  tool/file presentation, bounded line-buffered command output, MCP progress,
  exit status, approval lifecycle events, terminal-control stripping,
  credential redaction, and project-relative/outside-project path reduction.
- Reviewed Codex 0.144.6 approval request/response, command-output, MCP-progress,
  and request-resolution schemas plus deterministic tests for ID collisions,
  malformed/unsupported requests, raw argument/diff discard, split-secret
  redaction, exact decision responses, and cancel-before-interrupt ordering.
- A Milestone 9B selectable activity presentation that aggregates normalized
  lifecycle and output events by stable app-owned ID, expands in place with
  bounded live detail, preserves expansion through polling updates, and remains
  keyboard accessible on desktop and mobile layouts.
- A prominent approval interface that displays only normalized reason/detail
  fields, exposes only native-advertised approve/decline/cancel choices,
  prevents duplicate submissions, pauses polling during a decision, and sends
  only the exact app conversation ID, app approval ID, and closed decision enum.
- A Milestone 10A native read-only Git service with fixed shell-free status and
  diff commands, attachment revalidation, read-only repository support, bounded
  process/output handling, strict path containment, stable diagnostics, and no
  index, worktree, reference, configuration, or object mutation.
- Strict shared Git status/diff fixtures and a responsive source-review
  interface with branch divergence, changed files, staged/working selections,
  normalized line numbers, binary/truncation states, refresh, and explicit
  revalidated default-editor handoff.
- Deterministic native, TypeScript, component, bridge, desktop/mobile overflow,
  and axe-core coverage for Git parsing, raw-field rejection, deceptive paths,
  read-only/non-repository states, fixed-command temporary-repository review,
  and honest browser-preview degradation.
- A Milestone 10B native Git mutation coordinator with operation-specific
  preview, expiring native-held UUIDv7 confirmation, project ownership, exact
  evidence/postconditions, narrow rollback, and stable failure diagnostics.
- Fixed file-level stage/unstage, bounded regular-file revert with 30-minute
  single-use process-local recovery, and attachment-scoped commit plumbing that
  disables hooks, signing, editors, prompts, and inherited/global/system Git
  configuration.
- Commit refusal for outside-attachment staged paths, conflicts, submodules,
  repository operations in progress, missing repository-local identity,
  oversized/unscannable staged blobs, sensitive filenames, and high-confidence
  secrets in staged content or the commit message.
- Strict shared mutation fixtures and an accessible responsive confirmation UI
  with per-file controls, commit preview, destructive revert labeling, focus
  containment, exact-token confirmation, recovery, browser-preview honesty,
  desktop/mobile overflow checks, and axe-core coverage.
- A reviewed Codex CLI 0.145.0 schema subset covering connector/app,
  plugin/marketplace, skill, MCP, configuration-requirement,
  permission-profile, invalidation, and client-owned dynamic-tool contracts
  while retaining the 0.144.6 compatibility fixtures.
- The Milestone 13A `codex-integration-v1` Rust/TypeScript contract,
  deterministic category-preserving mock catalog, strict partial-failure,
  confirmation, and raw-field tests, and ADR 0018. Every capability remains
  explicitly contract-only; live discovery, installation, authorization, and
  UI are not claimed.
- The Milestone 13B serialized native integration catalog service and narrow
  `integration_catalog_read` IPC command. It discovers connectors, skills, MCP
  state, and policy through reviewed app-server methods; uses bounded stable
  CLI JSON for plugins and marketplaces; refreshes only from normalized
  invalidation reasons; and fails closed on unsupported CLI minors or malformed
  upstream data without exposing raw paths, URLs, configuration, credentials,
  account identity, or tool arguments.
- The Milestone 14A native plugin and marketplace lifecycle: fixed stable CLI
  add/remove/upgrade commands, bounded source review, one-use UUIDv7
  preview/confirm tokens, fresh policy/catalog/source revalidation, exact JSON
  result checks, postcondition refresh, closed Rust/TypeScript IPC, and an
  isolated real-CLI test lifecycle under temporary `CODEX_HOME` and `HOME`.
  Personal Codex state, generic command execution, connector authorization, and
  the user-facing Integration Center are not part of this checkpoint.
- The Milestone 14B Integration Center: a responsive category-preserving
  catalog with bounded search, category and health filters, normalized source,
  scope, installation, enablement, authentication, policy, publisher, version,
  permission, requirement, and health details. Fixed 14A plugin and marketplace
  operations appear only when the corresponding capability is both available
  and implemented; pinned repository marketplace adds reuse the strict 14A
  request contract. The preview dialog discloses permissions, warnings,
  destructive status, and separate hook trust with focus entry, containment,
  restoration, and Escape handling. Deterministic component and application
  tests plus desktop/mobile Playwright, overflow, and axe-core coverage do not
  read or mutate personal integration state. Connector/MCP authorization,
  enable/disable, skill configuration, prompt mentions, package, release, and
  deployment remain outside this checkpoint.
- The Milestone 14C confirmed integration-control boundary: fixed native
  preview/confirm operations for connector authorization, MCP OAuth, and skill
  enable/disable; one-use UUIDv7 evidence; native-only validated browser
  handoffs; exact MCP completion correlation; skill postconditions; explicit
  catalog/health refresh; and native-constructed connector mentions for new
  turns. Strict Rust/TypeScript fixtures and deterministic process/UI tests keep
  authorization URLs, app paths, skill paths, MCP names, credentials, and raw
  protocol/configuration outside IPC and personal state outside routine tests.
  Generic connector installation/configuration, plugin enablement, MCP
  management, arbitrary repair, package, release, and deployment remain
  unavailable.
- The Milestone 15A safe project-file preview foundation: one native-picker
  command with opaque project identity, fresh attachment and opened-file
  revalidation, strict shared Rust/TypeScript snapshots, bounded normalized
  UTF-8 text and PNG/JPEG rendering, metadata-only PDF recognition, stable
  failure diagnostics, honest browser degradation, responsive UI, and
  deterministic temporary-file/unit/Playwright accessibility coverage. Absolute
  paths, active HTML/SVG/APNG rendering, unknown binary content, PDF bytes,
  persistence, drag/drop, conversation attachments, generic filesystem reads,
  packages, releases, and deployments remain excluded. UTF-8 markup can appear
  only as inert normalized text.
- The Milestone 15B bounded conversation-image attachment flow: explicit native
  picker, bounded browser-byte drop, and native-only Linux file-manager drop
  sources; strict PNG/JPEG byte, dimension, name, count, and total limits;
  private mode-`0700`/`0600` staging; opaque one-use UUIDv7 draft IDs;
  cancellation, expiry, startup, failure, and terminal-turn cleanup; and
  documented `localImage` inputs on start, resume, and fork. Tauri's
  path-bearing events remain disabled; a short-lived GTK URI capture fixes
  WebKitGTK's empty HTML `FileList` without sending source/staging paths through
  IPC or persisting them in QuireForge metadata. Generic files, browser path
  events, arbitrary filesystem reads, live model calls in routine tests,
  packages, releases, and deployments remain excluded.
- The Milestone 15C reviewed desktop-integration checkpoint: five-minute,
  one-use UUIDv7 preview handoffs; confirmation that names the relative file and
  fixed system-default-application destination; fresh attachment, containment,
  symlink, descriptor, and device/inode revalidation; plus native-focused,
  deduplicated approval/completion/block/failure notifications using fixed copy
  without project names, prompts, paths, output, or raw diagnostics. React can
  provide neither a path nor an application/command, the webview receives no
  direct opener/notification plugin permission, and full display-session
  acceptance remains separately evidenced. The configured production artifact
  now has verified native Wayland project/file/image picker, bounded-preview,
  real Nautilus-drop, and notification evidence plus complete XWayland and
  true-X11 picker, preview, confirmation, default-application, attachment, and
  consumed-action paths. A
  disabled-by-default native-only probe verifies real desktop notification
  delivery with fixed production copy and no webview command or arbitrary
  content; the normal artifact is rebuilt without that feature after the probe.
- The Milestone 17A read-only scheduled task catalog: a schema-v2 shared
  Rust/TypeScript contract, installed-and-enabled plugin lookup through stable
  `plugin/read`, native-only marketplace roots, bounded and sanitized inert
  prompt previews, strict typed schedules, independent degraded diagnostics,
  and a responsive accessible Scheduled workspace with no create, edit, enable,
  run, pause, or delete controls. Deterministic native/component/browser tests
  use no personal plugin state and perform no task mutation or execution.
- Milestone 18 app-owned next-turn model selection: a migrated metadata policy,
  closed dynamic-tool registration and correlated response lifecycle, strict
  Manual/Recommend/Automatic modes, user lock, allowlists and reasoning
  ceilings, completion-time staging, restart-safe bounded provenance, fresh
  resume revalidation, typed IPC, and responsive effective/pending controls.
  Registration rejection degrades visibly without private endpoints, website
  automation, Codex configuration edits, credentials, or billable test calls.
- Milestone 19 pre-packaging hardening: warning-denying Node/Rust dependency
  audits, Cargo Dependabot, exact reviewed RustSec exceptions, immutable-action
  and frontend active-content validation, explicit Tauri asset/global-API
  disablement, command pruning, narrow CSP/response headers, production asset
  budgets, raw-error-free render recovery, keyboard skip targets, reduced
  motion, forced colors, and terminal confirmation focus ownership.

### Changed

- Split the desktop startup entry, application shell, and stable xterm terminal
  renderer into separate production chunks. The 193,549-byte entry is about
  76% smaller than the former 805,736-byte monolith, while the 459,684-byte
  pre-terminal path remains about 43% smaller and all three asset classes have
  enforced ceilings.
- Kept a bounded native startup overlay visible through the first committed
  application paints so cold WebKit compilation never presents an unexplained
  black window.
- Replaced invalid terminal tablist/close-button nesting with an accessible
  selector list, and replaced desktop navigation buttons with semantic
  workspace anchors.
- Validated the documented app-owned dynamic-tool lifecycle through
  `thread/start` registration and correlated `item/tool/call` requests, giving
  Milestone 18 a supported next-turn control dependency without web automation,
  private endpoints, or mid-turn model replacement.
- Completed the previously reserved Milestone 18 scope after integration
  discovery and the intervening product milestones, without modifying Codex
  configuration or credentials.
- Selected GLib's local filesystem backend at Linux process startup when the
  caller has not provided an override, preventing harmless GVFS activation
  warnings when the optional user service is masked.
- Kept the exact Git index lock file handle open for the full lock lifetime so
  a replacement lock cannot be removed if an ephemeral filesystem immediately
  reuses the original inode number.
- Adopted **QuireForge** as the permanent product name, with the tagline
  “Build boldly. Work locally.” The former “Codex Linux Workbench” name was a
  temporary discovery-stage label.
- Updated the planned repository-project website to
  `https://codeframe78.github.io/quireforge/` with base path `/quireforge/`.
- Replaced the initially proposed `quireforge.desktop` filename with the
  freedesktop-aligned `io.github.codeframe78.QuireForge.desktop`; the executable
  and Debian package remain `quireforge`.
- Defined app-server initialization as `clientInfo.name = "quireforge"`,
  `clientInfo.title = "QuireForge"`, and the real application version.
- Selected `https://quireforge.jamesjennison.net` as the production website.
- Recorded the former Cloudflare Pages hosting choice without creating a
  provider project, DNS record, or deployment; ADR 0024 later supersedes that
  choice with a Webuzo static origin and private-source boundary.
- Recorded the owner's separately completed move of authoritative DNS to
  Cloudflare and the temporary absence of the QuireForge hostname in the new
  zone; no DNS record was created by Codex.
- Recorded owner confirmation that Cloudflare two-factor authentication is now
  enabled without retaining factor or recovery details.
- Removed obsolete provider-specific hosting audits and deployment plans from
  the current project tree.
- Completed Milestone 0 feasibility documentation locally; no hosting project,
  DNS record, deployment, push, or release was created by Codex.
- Refreshed account-scoped Codex discovery without publishing catalog entries
  or integration identifiers.
- Reconciled the completed QuireForge path/repository migration, classified all
  remaining former-name references as intentional history, confirmed that no
  pre-release application data requires migration, and completed Milestone 1
  locally without pushing or changing repository settings.
- Completed the Milestone 2 brand and static website foundation locally without
  creating a Cloudflare project, changing DNS, deploying, pushing, or merging.
- Completed the Milestone 3 desktop scaffold locally, including an unbundled
  Wayland launch and runtime application-identity check, without implementing
  Codex workflows, packaging, pushing, or merging.
- Completed the Milestone 4 Codex process adapter locally, including a
  non-billable live app-server probe, bounded failure recovery, exact process
  cleanup, and normalized desktop status without login, conversation turns,
  configuration writes, packaging, pushing, or merging.
- Completed Milestone 5 authentication and onboarding locally. A non-mutating
  live `account/read` probe verified normalized state and exact child cleanup;
  no login, browser authorization, logout, model turn, push, package, or
  deployment was performed by Codex.
- Completed Milestone 6 project attachment locally, including its native
  storage/identity core, strict frontend boundary, accessible workspace, browser
  verification, unbundled release build, and isolated native launch. No source
  directory, Codex state, package, deployment, or release was changed.
- Completed the Milestone 7A native conversation-runtime checkpoint locally.
  No live model turn, approval decision, Codex-owned session mutation, package,
  deployment, or release was performed.
- Completed the Milestone 7B conversation UI and native-shell integration
  locally. No live model call, approval decision, deployment, package, or
  release was performed.
- Completed the Milestone 8A native session-lifecycle and crash-recovery
  checkpoint locally. No live model call, approval decision, thread deletion,
  project-file mutation, deployment, package, or release was performed; the
  history/search/tabs interface remains Milestone 8B.
- Completed the Milestone 8B history/search/tabs interface locally. Titles
  remain transient, lifecycle actions use app-owned IDs, and no live model
  call, approval decision, deletion, deployment, package, or release was
  performed.
- Completed the Milestone 9A native approval and detailed-activity contract
  locally. Routine verification used deterministic fixtures only; no live model
  call, real approval response, persistent policy grant, deployment, package,
  or release was performed. The polished selectable/expanded interface remains
  Milestone 9B.
- Completed the Milestone 9B selectable activity and approval interface
  locally with deterministic fixtures. No live model call, real command
  approval, persistent policy grant, deployment, package, or release was
  performed.
- Completed the Milestone 10A read-only Git-review checkpoint locally. Routine
  verification did not stage, revert, commit, mutate a user repository, make a
  model call, deploy, package, or release; Git mutations were deferred to the
  separately gated Milestone 10B.
- Completed Milestone 10B locally with deterministic temporary repositories.
  Routine verification mutated only those disposable fixtures; it did not
  alter a user repository, run a live model call, deploy, package, publish, or
  release.
- Completed Milestone 11A locally with deterministic disposable repositories
  and app-data roots. Verification created/attached only fixture worktrees,
  forced and preserved a recoverable post-create metadata failure, and did not
  alter a user repository, run concurrent Codex tasks, clean a worktree, make a
  live model call, deploy, package, publish, or release.
- Completed Milestone 11B locally with deterministic mock app-server processes
  and disposable repositories. Verification ran four independently owned
  fixture tasks, exercised exact interruption and capacity refusal, and did not
  start a live model turn, resolve conflicts, clean a worktree, alter a user
  repository, deploy, package, publish, or release.

### Fixed

- Fixed Linux release packaging depending on appimagetool's external homepage
  reachability check. AppImage repacking now skips that duplicate network
  probe while the artifact gate still validates both packaged AppStream
  records explicitly with `appstreamcli validate --no-net`.
- Fixed the native production window rendering only its black background. The
  Tauri `freezePrototype` option made the current production bundle fail before
  mounting with a read-only `Object.prototype.toString` error. QuireForge now
  uses Tauri's documented default (`false`), retains its strict CSP and narrow
  capability/IPC boundaries, validates the compatible setting in the repository
  gate, and smoke-tests the rendered unbundled executable on Linux.

### Migration note

- The existing Git history and discovery work were migrated in place; no
  replacement project or rewritten history was involved.
- The GitHub repository was renamed in place to `codeframe78/quireforge`, and
  the intact working copy moved to `/mnt/faststorage/quireforge`, through
  separate approval-gated operations.
- The repository was later transferred to the
  `James-Jennison/quireforge` organization location without changing the
  installed application identifier, and was deliberately made public only
  after a dedicated full-history disclosure audit.
- No released or development application data was detected under the temporary
  identity, so there is currently no user configuration to move. Future
  releases must preserve old data and never modify Codex-owned authentication
  or sessions.

### Known limitations

- The desktop adapter, Codex-owned authentication workflow, project attachment,
  conversation MVP, native session lifecycle/history, complete Milestone 9
  approval and detailed-activity interface, complete Milestone 10 Git review
  and controlled mutations, Milestone 11A worktree inventory/create/attach,
  Milestone 11B bounded parallel execution and monitor, and website are locally
  verified, but durable task recovery, automatic conflict resolution, worktree
  cleanup/recovery, advanced remote workflows, installable packages, website
  deployment, and a release workflow do not exist.
- Integration compatibility is based on Codex CLI 0.144.6 and must be probed at
  runtime.
