# Roadmap

The roadmap is gated milestone by milestone. Before each milestone, the
maintainer must inspect currently available Codex models, recommend the newest
suitable GPT model and reasoning level, provide the full milestone briefing,
and wait for manual confirmation.

**Post-M62 autonomous operating rule:** the preceding confirmation requirement
does not apply to routine, reversible, local, non-production implementation
after M62. Codex may inspect, implement, test, commit, and push the
highest-value safe QuireForge product work. It must still stop for credentials
or account/browser access, production deployment, public release, destructive
actions, third-party commitments, and genuinely irreversible product-direction
decisions.

No milestone may merge, access authenticated hosting, change DNS/SSL/provider
settings, deploy, publish a release, install an integration, or authorize a
connector without its required approval.

## Permanent identity migration

The discovery-stage name “Codex Linux Workbench” has been replaced by the
permanent product identity **QuireForge**: “Build boldly. Work locally.” The
migration preserves the repository and its history. Tracked documentation,
GitHub repository identity, original local working-copy path, and branding
assets are handled as separately verified and approval-gated migration steps.

The working copy moved through a controlled Codex-session handoff to
`/mnt/faststorage/quireforge`. The existing GitHub repository was first renamed
in place and was later transferred to the private
`James-Jennison/quireforge` organization location. None of those operations
authorized a push, public source link, website deployment, or release.

Migration status: the tracked identity contract, authoritative naming audit,
in-place GitHub repository rename, local working-copy handoff, and core vector
brand sources are complete. Milestone 1 also established the Apache-2.0 license,
repository guidance, contribution/security/conduct/support policies, issue and
pull-request templates, dependency automation, and initial repository CI. The
work through Milestone 6 is merged on `main`. Milestone 2 added the local static
website, production web exports, and automated website quality gates without
creating a hosting project or deployment. Milestone 3 added the locally verified Tauri
desktop foundation, narrow typed IPC contract, Linux app icons, and desktop
quality gates without producing an installable package. Milestone 4 added the
versioned Codex boundary, supervised app-server probe, normalized model catalog,
mock/failure tests, and selected generated schemas without starting a model
turn or modifying Codex state. Milestone 5 added normalized account status,
Codex-owned browser/device onboarding, exact cancellation/completion handling,
explicit logout, and redacted recovery without retaining secrets. Milestone 6
adds app-owned project metadata, native directory attachment, identity-aware
preflight, and an accessible project workspace without copying or deleting
source content. Milestone 7A adds the native conversation runtime, strict
normalized contracts, exact-turn interruption, and reference-only persistence;
Milestone 7B adds the responsive task composer, runtime-derived controls,
normalized progress stream, and exact stop interaction. Application packages
and external provider settings remain milestone- and approval-gated. Milestone
8A adds native resume, fork, archive/restore, Codex-authoritative reference
reconciliation, and conservative crash recovery. Milestone 8B adds the bounded
history/search/tabs presentation and accessible lifecycle actions.
Milestone 9A adds the native approval and detailed-activity contract with
app-owned correlation, one-turn decisions, redaction, and safe cancellation;
Milestone 9B adds the selectable expanded activity and bounded approval
interface over that contract.
Milestone 10A adds a fixed native read-only Git boundary, normalized status and
diff contracts, a responsive changed-file reviewer, and revalidated editor
handoff. Milestone 10B adds fixed stage, unstage, bounded revert/recovery, and
commit workflows behind native-held preview tokens, exact postconditions,
project concurrency, attachment scope, and secret review.
Milestone 11A adds the native managed-worktree foundation, strict inventory and
preview contracts, app-generated destinations, native-picker attachment, and
ordinary project registration without adding cleanup or concurrent execution.
Milestone 11B adds a four-task native conversation registry, independent
worktree execution and interruption, refresh recovery from normalized active
state, and aggregate activity/changed-file/conflict presentation without
adding destructive cleanup or automatic conflict resolution.
Milestone 11C adds opaque recovery for retained app-managed worktrees and
confirmed removal of clean, inactive, app-managed worktrees while preserving
their branches. Attached worktrees, force removal, generic prune, direct
directory deletion, and conflict resolution remain excluded.
Milestone 12 adds a bounded native PTY registry, controlled shell environment,
fresh project-cwd verification, tabbed xterm presentation, byte-preserving
input/output, resize, background-job ownership, and metadata-only restart
recovery without exposing raw paths or process identity to React.
Milestone 18 implements app-owned, policy-bounded model and reasoning selection
for the next turn. The current turn never replaces itself; Manual, Recommend,
and explicitly bounded Automatic ownership remain under visible user control.
Milestone 13A supplied the validated dynamic-tool lifecycle used by that
selector. Milestone 13B establishes live read-only integration discovery.
Milestone 14A establishes the confirmed native plugin and marketplace mutation
boundary. Milestone 14B adds the user-facing Integration Center over that
boundary without broadening it. Milestone 14C adds only reviewed connector/MCP
authorization, skill enablement, refresh, and connector prompt mentions. Later
Milestones 15A–15C complete the bounded local preview, conversation-image, and
desktop-integration surfaces. Milestone 16 completes the production static
website. Milestone 17A establishes read-only installed-plugin task-template
discovery; scheduling management and execution remain unsupported.

## Status

| Milestone | Scope                                                             | Size         | Status                                                                                                       |
| --------: | ----------------------------------------------------------------- | ------------ | ------------------------------------------------------------------------------------------------------------ |
|         0 | Existing project and feasibility discovery                        | Very large   | Complete; merged to `main`                                                                                   |
|         1 | QuireForge rename, move, GitHub migration, and governance closure | Medium       | Complete; merged to `main`                                                                                   |
|         2 | QuireForge brand and static website foundation                    | Large        | Complete; merged to `main`; deployed later through Milestone 16                                              |
|         3 | Desktop scaffold consolidation                                    | Large        | Complete; merged to `main`; not packaged                                                                     |
|         4 | Codex process adapter and contracts                               | Very large   | Complete; merged to `main`                                                                                   |
|         5 | Authentication and onboarding                                     | Medium       | Complete; merged to `main`                                                                                   |
|         6 | Projects and direct local-directory attachment                    | Very large   | Complete; merged to `main`                                                                                   |
|         7 | Conversation MVP                                                  | Very large   | Complete; merged to `main`                                                                                   |
|         8 | Session lifecycle and crash recovery                              | Large        | Complete; merged to `main`                                                                                   |
|         9 | Approvals and command presentation                                | Large        | Complete and verified; publication recorded in repository history                                            |
|        10 | Git status, diff review, and controlled mutations                 | Large        | Complete and verified; publication tracked by this milestone change                                          |
|        11 | Worktrees and parallel work                                       | Very large   | Complete through 11C and verified locally                                                                    |
|        12 | Integrated terminal                                               | Large        | Complete; merged to `main`; not packaged                                                                     |
|        13 | Integration discovery and compatibility                           | Very large   | Complete through 13B; verified locally                                                                       |
|        14 | Integration Center and installation workflows                     | Very large   | Complete through 14C; merged and verified on `main`                                                          |
|        15 | File previews and desktop integration                             | Large        | Complete through 15C; verified locally                                                                       |
|        16 | Complete Webuzo-hosted static website                             | Very large   | Complete through 16D; production and automatic origin TLS renewal active                                     |
|        17 | Scheduled tasks and advanced supported features                   | Medium–Large | Complete through 17A locally; management/execution deferred                                                  |
|        18 | Agent-directed model and reasoning selection                      | Large        | Complete and verified locally; not published                                                                 |
|        19 | Security, accessibility, and performance hardening                | Very large   | Complete and verified locally                                                                                |
|        20 | Packaging and release automation                                  | Large        | Complete and verified locally; not published                                                                 |
|        21 | Product readiness, beta publication, and download activation      | Very large   | 21A complete; 21B local preflight passed, publication approval-gated                                         |
|        23 | UI platform feasibility decision                                  | Medium       | Complete; ADR 0028 retains Tauri conditionally                                                               |
|       24A | Project-state contract                                            | Medium       | Complete; strict contract only, no ingestion, UI, or automation                                              |
|       24B | Repository-state reader                                           | Large        | Complete; attached-project-only read service, no UI or automation                                            |
|       24C | Project-state workspace                                           | Medium       | Complete; read-only presentation over the existing normalized reader                                         |
|        25 | Desktop visual polish                                             | Medium       | Complete; branded presentation refinement with fresh Ubuntu package gate                                     |
|        26 | Appearance themes                                                 | Medium       | Complete; eight closed local palettes with fresh Ubuntu package gate                                         |
|        27 | Unified Conversation Engine                                       | Large        | Complete; managed Chat/Codex boundary and fresh Ubuntu package gate                                          |
|        28 | Reference-only Advisor foundation                                 | Medium       | Complete; safe shell, confirmed temporary Project State projection, and fresh Ubuntu package gate            |
|        29 | Managed Advisor conversation foundation                           | Medium       | Complete; managed no-project Advisor turn with transient per-send safe context and fresh Ubuntu package gate |

## Milestone definitions

### 0 — Existing Project Audit and Feasibility

Inspect the installed Codex CLI, app-server, plugins, marketplaces, skills,
MCP, apps/connectors, policy, authentication, local cwd behavior, Linux/Tauri
prerequisites, GitHub, public DNS/TLS/site behavior, and the selected Cloudflare
Pages account through a separately approved method. Document production
constraints, previews, security, cutover, and rollback. Make no hosting, DNS,
repository-setting, or production change.

### 1 — QuireForge Rename, Move, and GitHub Migration

Verify and reconcile the already-completed intact local move, permanent
QuireForge identity, in-place GitHub repository rename, package/application
contracts, historical references, and user-data conclusion. Complete the
required governance baseline—license, contribution/security/conduct/support
policies, templates, dependency automation, and initial CI—without repeating or
discarding completed migration work.

### 2 — Brand Identity and Cloudflare Website Foundation

Develop the approved QuireForge vectors into consuming assets and scaffold the
Astro site, design tokens, themes, navigation, metadata, responsive layout, and
accessibility foundation. Confirm the audited document-root/deployment design.
Do not touch staging or production without separate approval.

Completed locally: the Astro static package, 15-page information architecture,
design tokens, original brand exports, themes, navigation, metadata, custom
404, Cloudflare headers, deterministic artifact validation, and desktop/mobile
accessibility checks. No Cloudflare project, custom domain, DNS record, preview,
or production deployment was created.

### 3 — Desktop Scaffold Consolidation

Install/verify prerequisites, scaffold Tauri 2 + React + TypeScript + Rust,
establish typed frontend/native IPC, shell layout, lint/format/test commands,
and CI.

Completed locally: the Tauri/React desktop package, exact executable and runtime
application identities, Linux icon exports, responsive light/dark shell, one
versioned bootstrap command, shared Rust/TypeScript contract fixture, empty
plugin-permission capability, strict frontend/native checks, desktop/mobile
axe-core coverage, unbundled release build, and GNOME Wayland launch. No Codex
process, directory attachment, persistence, package, push, or release was
created.

### 4 — Codex Process Adapter

Implement version/capability probing, process lifecycle, stable normalized
events, app-server stdio adapter, CLI fallbacks, mock backend, generated schema
fixtures, and contract tests.

Completed locally: fixed-command CLI version detection, a versioned
`CodexBackend` contract, serialized runtime probing, newline-delimited
app-server request correlation, bounded messages/timeouts, deterministic mock
and failure processes, normalized capability/model/error records, strict
Rust/TypeScript fixtures, selected generated initialize/model schemas, and an
honest desktop status. A non-billable live probe verified Codex CLI 0.144.6 and
left no child process. Authentication, threads, turns, project paths,
persistence, configuration writes, package, push, and release remain absent.

### 5 — Authentication and Onboarding

Implement Codex detection, account status, Codex-managed browser/device login,
logout, diagnostics, redaction, and failure recovery without owning secrets.

Completed locally: stable generated account schemas, normalized read-only
status, a single-owner pending-login process, allowlisted browser/device
handoffs, exact completion correlation, cancellation races, explicit two-step
logout, stable redacted errors, strict Rust/TypeScript fixtures, accessible
onboarding UI, and deterministic failure tests. A live non-mutating
`account/read` probe returned only normalized state and left no child process.
No real login, browser authorization, logout, token handling, project,
conversation, package, push, deployment, or release occurred.

### 6 — Projects and Direct Local-Directory Attachment

Implement the persistent multi-root-ready project schema, native picker,
selected/resolved identity, Git/worktree and project-instruction detection,
confirmation, missing/read-only/mount states, detach, relink, and per-task cwd
preflight.

Completed locally: the native core owns a migrated SQLite metadata store,
UUIDv7 project/association IDs, selected and resolved path identity, mount and
Git/worktree evidence, project-instruction detection, confirmation-time change
detection, detach/archive/relink metadata operations, and fail-closed cwd
preflight. Deterministic Rust tests cover symlink retargeting, linked worktrees,
read-only and missing directories, duplicate roots, storage permissions, and
the no-source-deletion boundary. A strict TypeScript contract rejects unknown
or path-bearing input, while the accessible project workspace provides native
selection, confirmation, missing/read-only states, preflight, relink, and
two-step detach/archive controls. Desktop/mobile browser checks, an unbundled
release build, and a native Wayland/D-Bus launch are verified. No source
directory, Codex-owned state, package, deployment, or release was changed.

### 7 — Conversation MVP

Start threads/turns in the verified attached directory, stream normalized
output, stop tasks, persist references, and expose model, reasoning, sandbox,
and approval controls from capabilities.

Milestone 7A native-runtime checkpoint implemented locally: one serialized
native owner validates the project association and re-resolves the exact
attached cwd before starting work; discovers the live model/reasoning catalog;
starts `thread/start` and `turn/start` with explicit sandbox and approval
controls; emits only bounded normalized lifecycle, message, reasoning-summary,
plan, and coarse-activity events; interrupts the exact owned turn; and stores
only Codex reference IDs and lifecycle metadata in QuireForge SQLite. Active
execution reserves the project against detach, archive, or relink races.
Approval requests block and close the task rather than being guessed or
auto-approved. Deterministic tests use a mock app-server and make no model call.

Milestone 7B adds an accessible responsive composer for a verified attached
project, with model and reasoning options taken from the normalized runtime
catalog, explicit filesystem and approval controls, pre-IPC rejection of the
unsafe unrestricted/no-approval combination, an ordered bounded event view,
stable terminal diagnostics, and exact app-owned conversation interruption.
Browser preview remains visibly non-interactive and never simulates a native
task. Session lifecycle is handled by Milestone 8; approval decisions, command
details, diffs, packaging, and deployment remain later milestones.

### 8 — Session Lifecycle

Resume, fork, archive, restore, title search, tabs, app grouping, and crash
recovery while keeping Codex authoritative.

Milestone 8A implements the native lifecycle and recovery boundary. Fixed Tauri
commands accept only app-owned UUIDv7 references and bounded prompts; Rust
reloads reference-only metadata, revalidates the exact attached cwd, reads the
owned thread, and invokes reviewed `thread/list`, `thread/read`,
`thread/resume`, `thread/fork`, `thread/archive`, and `thread/unarchive`
contracts. Fork lineage and archive timestamps are app metadata only. Startup
conservatively converts stale active rows to interrupted and clears active-turn
ownership. Deterministic tests prove exact-ID/cwd correlation, bounded listing,
no transcript/path exposure, no source or thread deletion, child cleanup, and
no live model use.

Milestone 8B adds a second bounded `thread/list` title-search projection after
complete reconciliation, then intersects both results with app-owned
references. React receives only transient normalized titles, app/project IDs,
parent-app lineage, controls, timestamps, and stable lifecycle states. Titles
are not persisted. The responsive interface groups sessions by project and
fork lineage, provides keyboard-accessible tabs, and wires resume, fork,
archive, and restore through exact app-owned IDs. Browser preview remains
honestly non-interactive, and archive never becomes deletion.

### 9 — Approvals and Command Presentation

Render exact scoped command, file, MCP/app, and permission requests; implement
decision handling, safe cancellation, terminal-control sanitization, redaction,
and recovery. Live activity rows must be selectable and expand in place to show
normalized real-time command/tool/file/process progress, comparable to Codex's
own disclosed activity presentation, without exposing raw protocol payloads,
credentials, unsafe terminal sequences, or unredacted private paths.

Milestone 9A implements the native security and contract checkpoint. The
serialized conversation owner recognizes only reviewed stable command, file,
and permission approval methods; correlates the exact native thread, turn,
request, and item; and exposes only app-owned UUIDv7 approval/activity IDs.
Approve, decline, and cancel are bounded decisions. Session-wide acceptance,
policy amendments, unstable write-root grants, and unsupported request types
remain unavailable. Turn-scoped permission profiles are strictly parsed, and
cancel resolves the request before interrupting the exact turn.

Activity schema version 2 provides stable IDs, safe titles/details, exit codes,
bounded command-output and MCP-progress deltas, and approval requested/resolved
events. Native presentation strips terminal and bidirectional controls, redacts
credential-shaped values, reduces paths to project-relative or
`[outside project]`, buffers output to line boundaries, and discards raw tool
arguments and file diffs. Pending approval remains ephemeral and uses existing
conservative crash recovery; no database migration or sensitive persistence is
added.

Milestone 9B aggregates activity lifecycle and bounded output deltas by stable
app activity ID. Each semantic button expands in place to show only normalized
kind, detail, live output, and exit status, retains its open state while polling
updates arrive, and caps the rendered activity/output history. A prominent
approval card displays the normalized reason and details and renders only the
approve, decline, or cancel choices advertised for the exact pending request.
Decision submission is single-flight, uses the fixed typed bridge, and pauses
polling so stale waiting snapshots cannot overwrite a completed decision.
Desktop and mobile fixtures verify keyboard semantics, accessibility, bounded
layout, exact app-ID submission, and duplicate-submission prevention.

### 10 — Git and Diff Review

Add status, branch, changed-file list, diff viewer, inline review context,
editor integration, and explicit stage/revert/commit workflows.

Milestone 10A implements the read-only checkpoint. Three fixed Tauri commands
accept only an app-owned project ID plus a normalized current-status path and
closed staged/worktree area. Native code revalidates the attachment on every
operation, runs shell-free Git with bounded environment/output/time, limits
status to the attached directory, discards object IDs and raw headers, and
rejects escaping, deceptive, symlink, conflicted, submodule, or stale targets.
The responsive interface presents branch divergence, changed files, staged and
working-tree selections, normalized line-numbered diffs, binary/truncated
states, refresh, and an explicit revalidated default-editor handoff. Browser
preview never simulates repository data, and no Git or diff state is persisted.

Milestone 10B implements explicit stage, unstage, revert, and commit workflows.
Preview accepts only a closed operation with an app-owned project ID and either
one normalized attachment-relative path or one bounded commit message. Rust
revalidates writable Git/worktree identity, reserves the project against Codex,
captures exact evidence, and retains the plan behind a five-minute in-memory
UUIDv7. Confirmation consumes only that token, revalidates the evidence, and
checks exact postconditions; React cannot resubmit paths or messages.

Stage/unstage preserve exact prior index entries for failure rollback. Revert
is limited to reviewed tracked regular-file modifications of at most one MiB
and offers a 30-minute single-use, process-local atomic recovery. Commit refuses
staged paths outside the attachment, conflicts, submodules, repository
operations in progress, missing repository-local identity, oversized content,
and high-confidence secrets in files, filenames, or the message. Git plumbing
creates the reviewed tree without hooks, signing, editors, prompts, or
global/system configuration, then updates `HEAD` with expected-old evidence and
checks the final reference/index state. Branch/worktree/remote mutation, push,
pull, reset, checkout, stash, arbitrary Git commands, packages, deployment, and
release remain separately gated. See
[ADR 0013](DECISIONS/0013-reviewed-git-mutation-boundary.md).

### 11 — Worktrees and Parallel Work

Create/attach isolated worktrees, run concurrent threads, display status,
detect conflicts, and make cleanup explicit and safe.

Milestone 11A implements the managed-worktree foundation. A fixed native
inventory command accepts only an app-owned project ID and normalizes
`git worktree list --porcelain -z` without exposing object IDs, raw stderr, or
Git configuration. Each managed or attached worktree is also an ordinary
QuireForge project linked to its canonical source by schema migration 4.
Externally discovered worktrees remain unselectable until the user chooses the
exact directory with the native picker.

Creation accepts only a bounded new branch name. Rust generates the destination
beneath private app storage, captures source repository identity and current
HEAD internally, disables hooks and configured checkout filters, and retains a
five-minute one-use confirmation. Confirmation reserves every app-owned project
in the source repository group, revalidates identity, HEAD, branch absence, and
destination, then uses one fixed shell-free `git worktree add` workflow.
Metadata registration is transactional. If Git succeeds and registration
fails, the worktree is reported as recoverable and deliberately left in place.

Milestone 11B replaces the single active-process slot with a bounded registry
of at most four independently locked conversations. Starts reserve their exact
project before process creation, duplicate work in one project fails closed,
and poll, approval, and interruption route only through an app-owned
conversation ID. A strict normalized registry lets the webview recover active
tasks after refresh without receiving Codex IDs, cwd, commands, process
metadata, or raw protocol messages.

React polls each active task independently and presents one aggregate worktree
monitor. Selecting a row opens the existing expandable live activity stream;
read-only Git snapshots supply only normalized changed-file and conflict
counts. Process ownership does not survive an application restart, so stale
active records follow the existing interrupted-state recovery rule. Milestone
11B performs no conflict resolution or Git mutation.

Milestone 11C adds separately gated recovery and cleanup. Native inventory
issues opaque recovery IDs only for unregistered linked worktrees inside the
exact private managed-storage slot. Recovery registers the retained checkout
without changing Git or files. Cleanup accepts only app-owned project IDs and
removes only a clean, unlocked, non-current `managed` checkout after repository-
group reservation and confirmation-time relation, identity, branch, `HEAD`,
and status revalidation. Git removal never uses force, preserves the branch,
and must satisfy explicit path/inventory/branch postconditions before a
transaction detaches and archives project metadata.

If Git succeeds but metadata retirement fails, the missing managed entry can be
reviewed again for metadata-only finalization; no filesystem mutation is
retried. Attached/external worktrees, direct directory deletion, branch
deletion, conflict resolution, arbitrary Git arguments, and repository-wide
`git worktree prune` remain unavailable. See
[ADR 0014](DECISIONS/0014-managed-worktree-foundation.md),
[ADR 0015](DECISIONS/0015-bounded-parallel-worktree-execution.md), and
[ADR 0016](DECISIONS/0016-safe-managed-worktree-cleanup.md).

### 12 — Integrated Terminal

Implement Rust PTY lifecycle, tabs, verified project cwd startup, resize/input,
background processes, environment handling, and terminal safety tests.

Implemented and verified: a dedicated Rust `portable-pty` service owns up to eight
app-generated UUIDv7 terminal sessions, starts only after project reservation
and cwd identity revalidation, clears and reconstructs a narrow noncredential
environment, transports bounded base64 bytes, applies typed resize/input, and
ends the complete owned Linux session through bounded HUP/TERM/KILL cleanup.
React uses stable xterm APIs with the DOM renderer, inaccessible browser-preview
controls, responsive tabs, explicit close confirmation, and a visible warning
that terminal commands run with the Linux account rather than Codex approval
policy. SQLite migration 5 persists only presentation state and marks stale
sessions interrupted; input, output, history, cwd, environment, TTY, and
process/session IDs are never stored or exposed. Closing a tab does not delete
project files. Daemons that deliberately create a new session, remote shells,
shell selection, process inspection, command approvals, and terminal content
recovery remain outside this milestone. See
[ADR 0017](DECISIONS/0017-native-integrated-terminal.md).

Publication completed through
[PR #27](https://github.com/codeframe78/quireforge/pull/27) and successful
pull-request and `main` repository checks. No package or release was produced.

### 13 — Integration Discovery and Compatibility Layer

Normalize apps/connectors, plugins, marketplaces, skills, MCP, policy, runtime
requirements, scopes, and health. Use stable routes and deterministic mock
catalogs; preserve unknown/blocked/degraded states.

Milestone 13A refreshes the installed Codex 0.145.0 schema evidence and accepts
the category-preserving `codex-integration-v1` contract. It distinguishes
upstream availability from QuireForge implementation, defines bounded scope,
permission, requirement, policy, and health states, and validates a documented
client-owned dynamic-tool lifecycle through `thread/start` and
`item/tool/call`. This checkpoint is contract-only: it does not expose a live
catalog, install or authorize integrations, register the selector tool, or add
an Integration Center UI. See
[ADR 0018](DECISIONS/0018-normalized-integration-contracts.md).

Milestone 13B implements the read-only native discovery/normalization service,
strict IPC, exact CLI-minor routing, bounded cache invalidation, and
deterministic partial-failure tests against these contracts. It uses supported
app-server methods for connector, skill, MCP, and policy reads and stable CLI
JSON commands for plugin and marketplace discovery; experimental plugin RPCs,
raw paths/URLs/configuration, account identity, and tool arguments do not cross
the native boundary. Mutation and the user-facing Integration Center remain
Milestone 14.

Milestone 13A publication completed through
[PR #32](https://github.com/James-Jennison/quireforge/pull/32), merge commit
`7bc5f5f`, and successful pull-request and `main` repository checks. This
checkpoint produced no live integration discovery, installation, authorization,
package, release, or deployment.

Milestone 13B publication completed through
[PR #34](https://github.com/James-Jennison/quireforge/pull/34), merge commit
`007f5b7`, and successful pull-request workflow
[`29890814046`](https://github.com/James-Jennison/quireforge/actions/runs/29890814046)
and post-merge `main` workflow
[`29890942589`](https://github.com/James-Jennison/quireforge/actions/runs/29890942589).
This checkpoint made no integration, account, package, release, deployment, or
hosting mutation.

### 14 — Integration Center and Installation Workflows

Implement browse/search/filter/details, permission review, CLI-backed plugin and
marketplace operations, supported connector/MCP authorization handoff,
enable/disable/update/remove where validated, health/troubleshooting, prompt
mentions, and supply-chain warnings.

Completion requires a supported test-plugin lifecycle and an honest limitation
when connector management is unavailable.

Milestone 14A implements the native plugin and marketplace lifecycle only. It
uses the reviewed stable CLI 0.145.x JSON commands for plugin install/remove
and marketplace add/remove/upgrade, never the under-development app-server
plugin-management RPCs. Every operation starts with a fresh normalized catalog
and policy read, resolves an opaque entry ID to native-held CLI evidence,
reviews the source class and normalized permissions/warnings, and creates a
five-minute one-use UUIDv7 confirmation. Confirmation serializes mutation,
rechecks the CLI minor, policy, normalized entry, and exact raw evidence, then
accepts only the closed documented JSON result and verifies the resulting
catalog state. Repository marketplace adds accept only `owner/repository` plus
a 40- or 64-hex pinned reference. Raw paths, URLs, CLI arguments/results,
configuration, and credentials never cross IPC.

The deterministic test suite uses temporary state, while the ignored explicit
real-CLI proof runs a local fixture marketplace and plugin under temporary
`CODEX_HOME` and `HOME`; it does not read or change personal Codex state. No
connector authorization, MCP configuration, skill configuration, plugin
enable/disable, generic command execution, Integration Center UI, package,
release, deployment, or personal integration mutation is included. See
[ADR 0019](DECISIONS/0019-confirmed-integration-mutations.md).

Milestone 14A publication completed through
[PR #36](https://github.com/James-Jennison/quireforge/pull/36), implementation
commit `e46cb5c`, merge commit `a20919f`, successful pull-request workflow
[`29893588842`](https://github.com/James-Jennison/quireforge/actions/runs/29893588842),
and post-merge `main` workflow
[`29893692681`](https://github.com/James-Jennison/quireforge/actions/runs/29893692681).
This checkpoint made no personal integration or account mutation and produced
no package, release, deployment, or hosting change.

Milestone 14B implements the user-facing browse/search/filter/details and
permission-review Integration Center over the normalized discovery and 14A
mutation contracts. It exposes only capability-ready fixed operations, uses a
pinned-reference form for repository marketplace adds, presents separate hook
trust and supply-chain warnings, and keeps unavailable management explicit.
Desktop/mobile, keyboard, overflow, and automated accessibility checks pass
locally and in hosted CI. Publication completed through
[PR #38](https://github.com/James-Jennison/quireforge/pull/38), implementation
commit `42cff70`, merge commit `93e585f`, successful pull-request workflow
[`29918268480`](https://github.com/James-Jennison/quireforge/actions/runs/29918268480),
and post-merge `main` workflow
[`29918513538`](https://github.com/James-Jennison/quireforge/actions/runs/29918513538).
No personal integration or account state was read or mutated, and no package,
release, deployment, or hosting change was made. A later separately gated
Milestone 14 checkpoint must handle only supported connector/MCP authorization,
enable/disable or update flows, health/troubleshooting, and prompt mentions;
unsupported management must remain visibly unavailable.

Milestone 14C implements the supported portion of that next gate. A closed
native preview/confirm service authorizes a connector only through the official
URL returned by Codex, starts MCP OAuth only through
`mcpServer/oauth/login`, and changes skill enablement only through
`skills/config/write` with an exact postcondition. Browser handoff URLs, raw
connector IDs/paths, MCP names, and skill manifest paths remain native-only.
The Integration Center exposes those controls only for capability-ready,
eligible rows; explicit refresh rebuilds normalized health/catalog state.

New conversation turns may select up to eight authorized, enabled, healthy
connectors by opaque catalog ID. Native code re-resolves callable state and
constructs the documented `mention` plus `app://` path; the webview cannot
supply a path or raw Codex identifier. Generic connector installation or
configuration, plugin enable/disable, MCP add/remove/logout/configuration,
arbitrary health repair, and generic config writes remain unavailable. Routine
tests use deterministic fixtures only and do not read or mutate personal Codex
or integration state. See
[ADR 0020](DECISIONS/0020-confirmed-integration-authorization-and-controls.md).
Publication completed through
[PR #41](https://github.com/James-Jennison/quireforge/pull/41), implementation
commit `86a114d`, merge commit `e4d8333`, successful pull-request workflow
[`29950963936`](https://github.com/James-Jennison/quireforge/actions/runs/29950963936),
and post-merge `main` workflow
[`29951143628`](https://github.com/James-Jennison/quireforge/actions/runs/29951143628).
No personal integration or account state was read or mutated, and no package,
release, deployment, or hosting change was made.

### 15 — File Previews and Desktop Integration

Split this milestone so each security/desktop boundary is independently
reviewable:

- **15A — safe project-file previews:** use a native picker and opaque project
  ID; revalidate attachment identity, containment, symlink/regular-file state,
  and opened file identity. Return only attachment-relative names and bounded
  normalized UTF-8 text, PNG/JPEG data, or metadata-only PDF state through a
  strict contract. Browser preview cannot select or read local files. See
  [ADR 0021](DECISIONS/0021-safe-project-file-previews.md).
- **15B — drag/drop and conversation attachments:** define source ownership,
  staging/retention, model-interface support, explicit send semantics, size and
  count limits, cancellation, and cleanup without turning drag/drop into a
  general path bridge. The implemented checkpoint accepts only PNG/JPEG,
  disables Tauri's default path-bearing drag/drop events, stages validated
  browser bytes or one-use native-captured Linux file drops in private app
  data, sends only native `localImage` paths, and retains each consumed copy
  until its turn is terminal. See
  [ADR 0022](DECISIONS/0022-bounded-conversation-image-attachments.md).
- **15C — desktop handoffs and Linux verification:** add notifications and
  reviewed editor/open-with behavior, then verify native picker/handoff behavior
  on supported Wayland and X11 sessions. External destinations stay visible and
  allowlisted; no generic opener or arbitrary command IPC is allowed. The code
  checkpoint uses native-held one-use preview actions, an explicit system-
  default-application review, and fixed privacy-safe background notifications;
  the completed final Linux display-session gate is recorded below. See
  [ADR 0023](DECISIONS/0023-reviewed-desktop-handoffs-and-notifications.md).

Milestones 15A–15C are implemented and verified locally. The 15C handoff and
notification checkpoint uses the official Tauri notification plugin, a Linux
binding already present in the Tauri stack, and no source-path persistence,
unrelated user-file access, billable model call, package, release, or
deployment. Its production native Wayland project/file/image picker, bounded-
preview, real Nautilus-drop, and fixed-copy notification evidence is complete
against disposable app data. Complete XWayland and true-X11 picker/preview/
default-application/attachment paths remain separately recorded. Milestone 17
is the next planned implementation milestone.

### 16 — Complete the Webuzo-Hosted Static Website

Milestone 16A reconciles Home, Features, Integrations, Downloads, Installation,
Documentation, Compatibility, Roadmap, Releases, Security/Privacy, Development,
FAQ, Troubleshooting, and About for a public site backed by private source. It
retains the approved design, removes private repository/activity links,
supersedes the unimplemented Cloudflare Pages plan, and produces a verified
Apache-compatible static artifact.

Milestone 16B created the isolated Webuzo origin and staged the reviewed
artifact without public DNS. Trusted origin TLS, route/header validation, and
rollback rehearsal passed. Milestone 16C separately activated the canonical
hostname after owner approval. Public DNS, Full (Strict), scoped HSTS, live
route/accessibility checks, 100/100/100/100 mobile and desktop Lighthouse
results, and post-launch recovery verification passed. Milestone 16D then
completed provider-managed automatic origin TLS and renewal validation. Private
provider identifiers and operational diagnostics remain outside source
control.

### 17 — Scheduled Tasks and Advanced Features

Implement only capabilities exposed through supported interfaces. Distinguish
local scheduling from hosted scheduling and defer unsupported features.

Milestone 17A implements the supported read-only portion. The native
integration service queries stable `plugin/read` only for installed, enabled
plugins already established by the CLI catalog. Raw marketplace roots and
lookup values remain native-only. Scheduled task names and prompts are treated
as untrusted plugin content, normalized into bounded inert previews, and paired
with a strict hourly/daily/weekdays/weekly schedule. The existing integration
catalog read/refresh IPC advances to schema version 2, and the Scheduled
workspace exposes no action controls.

The reviewed stable request set and plugin CLI provide no task create, edit,
enable, run, pause, or delete route. QuireForge therefore implements no local
scheduler, hosted scheduler, official-client automation, or private web
integration. Those capabilities remain deferred pending a separately reviewed
supported interface and explicit approval. See
[ADR 0025](DECISIONS/0025-read-only-scheduled-task-catalog.md).

### 18 — Agent-Directed Model and Reasoning Selection

Milestone 18 is implemented and verified locally. It adds a typed, app-owned
selector-control boundary that lets Codex inspect the normalized `model/list`
catalog, current effective choice, pending next-turn choice, and user policy.
Codex may request at most one model/reasoning change per completed turn with a
short rationale. Native code revalidates the request against a fresh advertised
catalog and the configured policy before applying it to the next `turn/start`;
the executing turn never claims to replace itself.

Expose explicit Manual, Recommend, and Automatic ownership modes. Automatic
mode requires deliberate user opt-in and an allowlist or model/reasoning
ceiling. A user lock or later manual choice always wins. The UI must distinguish
effective from pending selection and show that Codex requested the change and
why. Prevent repeated oscillation and silent cost escalation, and persist only
QuireForge policy and bounded provenance—not prompts, account identifiers,
credentials, or raw protocol payloads.

Use only documented Codex interfaces and normalized typed IPC. Do not automate
the ChatGPT/Codex web selector, call private endpoints, or edit Codex-owned
configuration behind the user's back. Validate the exact supported
request/response lifecycle against the installed app-server schemas. If a
stable or explicitly accepted experimental control path is unavailable,
degrade to visible recommendation-only behavior rather than fabricating
automatic control. Deterministic mocks must cover prompt-injection attempts,
stale/unadvertised models, unsupported efforts, manual locks, policy ceilings,
one-change-per-turn enforcement, restart behavior, and next-turn application.

The implementation registers the closed `quireforge_model_selector` dynamic
tool, keeps exact request/thread/turn/call correlation native, stages a valid
request only after successful turn completion, persists bounded policy and
provenance separately from the effective choice, and revalidates immediately
before resume. Strict schema-v3 conversation/session contracts and the
`model_selection_update` command expose only app-owned state. The responsive UI
shows effective versus pending selection, provenance and rationale, manual
override, recommendation acceptance/dismissal, automatic allowlists/ceilings,
and the user lock. Registration rejection produces an explicit
recommendation-only state. See
[ADR 0026](DECISIONS/0026-policy-bounded-next-turn-selection.md).

### 19 — Security, Accessibility, and Performance Hardening

Revisit the threat model; audit secret handling, injection, filesystem races,
integration supply chain, credentials, Tauri permissions/CSP, accessibility,
performance, reliability, and crash recovery.

Complete locally. The main capability remains Linux/window-scoped and
permission-empty; the global Tauri API and asset protocol are explicitly
disabled, unused plugin commands are removed from production builds, CSP and
response headers are narrowed, and repository validation rejects unpinned
Actions or direct frontend active-content/network primitives. High-severity
pnpm and warning-denying RustSec audits now run in CI with exact reviewed
Tauri/GTK3 exceptions. Keyboard skip/focus, reduced motion, forced colors,
terminal confirmation focus ownership, and raw-error-free reload recovery are
covered across desktop and website profiles. Separate startup, application,
and terminal chunks reduce the startup entry from 805,736 to 193,549 bytes and
the pre-terminal application path to 459,684 bytes, with an opaque startup
overlay and enforced generated-asset budgets. See the
[Milestone 19 hardening review](MILESTONE_19_HARDENING.md).

### 20 — Packaging and Release Automation

Produce AppImage and Debian packages on an appropriate baseline, checksums,
release workflows, install/upgrade/uninstall tests, and website download data.
Do not publish a release without approval.

Complete locally. The `0.1.0-beta.1` x86_64 AppImage and Debian candidates are
built inside a digest-pinned Ubuntu 22.04 container with Rust 1.95 and Node
22.22.1 inputs. Tauri's Linux helper downloads are checksum-pinned and verified
before use; normalized packages have canonical identities, deterministic
timestamps, an exact release manifest, and `SHA256SUMS`. Structural, offline
AppStream, GLIBC 2.35, visible X11 launch, install, upgrade, uninstall, data
preservation, and repeated-normalization checks pass. The release workflow is
manual-only, uses immutable Action revisions, uploads review artifacts, and
requires an exact tag, confirmation phrase, protected environment, clean source
manifest, attestation, and separately approved publish operation before it can
create a prerelease. Website download data remains explicitly unavailable.
See the [Milestone 20 packaging report](MILESTONE_20_PACKAGING.md) and
[release procedure](RELEASING.md).

### 21 — Product Readiness, Beta Publication, and Download Activation

Milestone 21A closes the remaining user-facing beta-readiness gaps before any
publication: remove internal milestone scaffolding from the product UI, require
a verified Codex account state before exposing Codex work surfaces, introduce
an original QuireForge home/workspace hierarchy informed by the approved visual
reference, and display read-only remaining Codex usage only when the documented
app-server rate-limit endpoint provides it. QuireForge must not scrape a
website, estimate quota, read Codex credential files, expose raw account
metadata, or redeem reset credits.

Milestone 21A is complete locally. The desktop now opens on a Codex-owned
authentication gate, starts workspace and account-data reads only after the
normalized account state grants access, and presents an original responsive
QuireForge home with project, recent-thread, quick-action, account, and
remaining-usage regions. Remaining percentages and reset times come only from
the documented `account/rateLimits/read` response and degrade to an honest
unavailable or not-metered state. Raw plan, balance, account, reset-credit, and
protocol metadata are discarded in Rust. User-facing milestone labels are no
longer rendered. See the
[Milestone 21A product-readiness report](MILESTONE_21A_PRODUCT_READINESS.md).

The compact sidebar summarizes only the exact 10,080-minute window of the
general upstream `codex` meter; model-specific meters remain detailed-only.
It never treats `primary`/`secondary` position or array order as evidence of a
weekly allowance, substitutes a short window, or calculates, estimates,
predicts, combines, or infers a quota.

Milestone 21B retains the external release boundary:

Run final package and supported-platform QA; confirm the approved distribution
location, release artifact, checksums, provenance, download data, and rollback;
then request beta-publication approval. Update the already hosted website only
with the approved package metadata and verify downloads, installation guidance,
known limitations, and checksums. Website updates and application release
publication remain independently approval-gated.

The local 21B preflight now passes for the clean pinned Ubuntu 22.04 candidate,
repeated byte-identical normalization, disposable lifecycle, current-host
AppImage and extracted-Debian launches, and the signed-out product pixels. The
dormant website publication path validates exact same-origin versioned
packages, hashes, sizes, and manifest/checksum URLs while the committed state
remains unavailable. A full-history disclosure audit found no credentials or
secrets, the accepted residual identity/path/log disclosures are documented,
and the repository is approved for public visibility with fork-origin code
excluded from persistent self-hosted runners. The public beta 1 GitHub
prerelease and attestations exposed a tilde-normalized Debian asset-name
mismatch; beta 2 corrects the outer name while preserving the internal Debian
prerelease version. Beta 1 remains immutable and superseded. Exact owner-hosted
package promotion, public retrieval checks, website activation, and deployment
remain separate terminal gates. See the
[Milestone 21B release-readiness report](MILESTONE_21B_RELEASE_READINESS.md).

### 22 — Routed Desktop Workspace and Account Settings

Complete locally. The persistent QuireForge shell now routes Home, New task,
Projects, Threads, Scheduled, Integrations, Files, Changes, Worktrees, and
Terminal into dedicated primary workspaces instead of stacking them in one
scrolling document. The typed hash route supports deep links and local
restoration, active navigation and route-change focus are explicit, and
existing stateful tool components remain mounted while inactive views are
hidden from layout and assistive technology.

The desktop frame now has a compact route-aware toolbar, independently
scrollable primary content, a meaningful collapsible/resizable contextual
inspector, a persistent status bar, compact navigation at medium widths, and a
scrollable off-canvas drawer at small widths. Inspector width and compact
sidebar choice remain local presentation preferences. No pane is rendered only
to imitate a reference.

The full account row now opens QuireForge Settings at Accounts & connections.
It uses only the existing normalized Codex refresh, remaining-usage, and
two-step logout controls. Direct ChatGPT account management is not a supported
interface, so the view states that boundary and exposes no fabricated account,
billing, credential, or private-page control. Appearance and About clearly
remain local QuireForge settings.

No frontend routing/pane dependency, backend capability, Tauri permission,
credential store, deployment, release, or hosting change was added. The local
acceptance gate passed 162 desktop and seven website unit/component tests, 178
runnable Rust tests with three deliberate live probes ignored, 38 desktop and
mobile Playwright scenarios, production asset budgets, and local Debian and
AppImage packaging. See the
[Milestone 22 completion report](MILESTONE_22_ROUTED_DESKTOP_WORKSPACE.md).

### 22B — Visual Workspace Refinement

Complete locally on `feat/milestone-22b-visual-workspace-refinement`. This was a
presentation and usability refinement of the existing routed desktop shell,
not a replacement for it: shared header and surface conventions, improved
route hierarchy, responsive behavior, and accessibility polish must preserve
the existing Tauri bridge, hash routing, safety boundaries, and workspace
actions. The current refinement includes the representative Scheduled,
Integrations, Files, and Settings surfaces and mobile-drawer evidence. Each
checkpoint remains under the 100 KiB production CSS budget. Full repository,
browser, native, and fresh Ubuntu 22.04-compatible Debian/AppImage validation
now close the work.

### 23 — UI Platform Feasibility Decision

Complete on `docs/milestone-23-ui-platform-feasibility`. The read-only evidence
maps the Tauri façade against reusable Rust services, defines a future
UI-neutral core/adapters boundary, and compares retaining Tauri, a full Qt 6
migration, and a deferred conditional migration. ADR 0028 accepts retaining
Tauri conditionally; Qt reconsideration requires measurable documented
triggers. See the [Milestone 23 report](MILESTONE_23_UI_PLATFORM_FEASIBILITY.md).

No code, dependencies, prototypes, migrations, package changes, or full
repository audit occurred.

### 26 — Appearance Themes

Complete presentation-only palette work. Forge remains the default local
appearance; the closed built-in set adds Midnight Atelier, Blueprint Terminal,
Signal Noir, Aurora Workbench, Obsidian & Copper, Monochrome Editorial, and
Pacific Night through Settings → Appearance. The work is limited to semantic
CSS tokens, local preference restoration, keyboard-accessible live preview,
and visual/accessibility regression coverage. It excludes custom themes,
layout/density/typography redesign, native or repository-state contracts,
automation, external branded assets, and Qt work. Fresh pinned Ubuntu 22.04
Debian/AppImage lifecycle and launch evidence passed from clean implementation
commit `0ae0de7995f10128728116b148d49f2cb5b2cf79`.

### 27 — Unified Conversation Engine

Complete on `feat/milestone-27-unified-conversation-engine`. The managed
ChatGPT-only native conversation boundary distinguishes Chat from Codex by real
capability policy, not styling: Chat has no attached project, terminal, Git,
worktree, tool, integration, approval, or API-key authority. It uses only the
documented Codex app-server browser-login path, never consumer ChatGPT APIs,
external tokens, cookies, or credentials. A Chat/Codex transition is explicit;
only a confirmed closed local preference persists, safely defaulting to Codex
when absent or invalid, with no project context or other capability-bearing
state transfer. Final implementation commit
`cc4d0cea7d28d275e5ad1c8aa9d7a2a4f0627d6c` and clean incremental
`0.1.0-beta.4` Ubuntu 22.04 Debian/AppImage evidence passed the complete
validation, lifecycle, visible-launch, and installed-host smoke gates. See the
[Milestone 27 report](MILESTONE_27_UNIFIED_CONVERSATION_ENGINE.md).

### 28 — Reference-Only Advisor Foundation

Complete on `main`. This first Advisor slice is
strictly local metadata and contract work: opaque Codex-thread references,
explicit closed context-reference kinds, separate trust/provenance/freshness,
and digest-only future dispatch proposals that always require a later explicit
user approval. Its fixed `#advisor` route receives only a safe summary of
app-owned reference metadata and presents no composer or action controls. One
user-confirmed source may invoke the existing M24B reader in fixed
local-only/metadata-only mode and return only a temporary safe projection;
project identity, paths, refs, source content, artifacts, and diagnostic text
remain excluded. It deliberately excludes model calls, arbitrary project
file/document/screenshot reading, transcript or prompt retention, dispatch
bridge, Python sidecar, watcher, automatic handoff, contradiction resolution,
and repository mutation. The clean `0.1.0-beta.5` Ubuntu 22.04 Debian/AppImage
set passed the pinned container lifecycle, desktop/icon, visible-launch, and
installed-host smoke gates. See the
[Milestone 28 report](MILESTONE_28_ADVISOR_FOUNDATION.md).

### 29 — Managed Advisor Conversation Foundation

Complete on `feat/milestone-29-advisor-conversation-foundation`. This adds a
separate Advisor-only managed Codex app-server conversation profile. It
uses only Codex-managed ChatGPT browser authentication and a fixed no-cwd,
no-tools, no-approval, read-only, no-network turn boundary. Advisor prompt and
response text remain transient in QuireForge; only opaque Codex thread metadata
is retained. A user may include the existing selected Project State safe
projection only after a second per-send confirmation. Approval/Dispatch and
Codex execution remain separate future work. The clean `0.1.0-beta.6` Ubuntu
22.04 Debian/AppImage set passed the pinned container lifecycle, desktop/icon,
visible-launch, and installed-host smoke gates. See the
[Milestone 29 report](MILESTONE_29_MANAGED_ADVISOR_CONVERSATION.md).

### 30 — Advisor Bounded Text/Data Content Ingestion and Reviewed Single-File Export

Complete one closed `text-data` Advisor Content Ingestion Registry entry for
one user-selected `.txt`, `.md`, `.csv`, `.json`, or `.py` file. The native
boundary accepts at most 512 KiB of normalized UTF-8, exposes only a typed
display manifest, hash, bounded projection, explicit per-send confirmation,
and one-send in-memory disposal behavior. The source path and raw content do
not enter React, SQLite, logs, project metadata, or later sessions.

Advisor may send the confirmed bounded text only as its existing no-project,
read-only app-server text input. It may offer a user-selected preview and one
new-file native Save-dialog export of visible text, Markdown, JSON, CSV, or
Python output. Exports may not overwrite files, create directories, write to a
project, execute content, or grant any Advisor capability.

The registry reserves distinct future `image`, `document`, `archive`, and
`static-binary` categories without a parser, picker, transport, UI, or
persistence implementation. Dynamic sandbox analysis is explicitly deferred.
This milestone excludes generic uploads, image/document/archive/binary
handling, API/provider alternatives, credential access, project browsing,
terminal/Git/repository authority, dispatch changes, and transcript retention.
The versioned package gate requires a fresh unique candidate, strict
Rust/TypeScript contract parity, accessibility/responsive coverage, production
bundle/Tauri/package contracts, and fresh Debian/AppImage lifecycle and visible
launch evidence.

Every future Advisor content-type entry below remains closed and type-specific:
there is no generic upload system. Each must use native validation and a bounded
projection, expose a path-free manifest/hash, require explicit per-send
confirmation, and dispose of source bytes and projections transiently. Advisor
never receives shell, Git, project-write, execution, credential, or provider
authority from an ingestion entry.

### 31 — Advisor Bounded PNG/JPEG Image Analysis

Extend the Advisor Content Ingestion Registry with one closed `image` entry for
one user-selected PNG or JPEG per send. Native code validates extension, magic
bytes, safe decode, dimensions, and resource limits before a path-free display
manifest, preview, hash, and explicit per-send confirmation are available.
Only the documented app-server `localImage` input may carry the confirmed image;
native staging remains transient through the terminal turn. This excludes generic
uploads, multi-image batches, image export, project access, execution, and all
other file types. The package gate requires a fresh `0.1.0-beta.19` candidate,
strict native/frontend parity, malformed-image and disposal coverage,
accessibility/responsive checks, and Debian/AppImage lifecycle and visible-launch
evidence.

### 32 — Advisor Conversation History Viewport and Mode Picker

Provide a bounded, scrollable in-memory viewport for the active transient
Advisor conversation and an accessible explicit mode picker: **Advisor — Create,
Learn, Explore** and **Codex — Build, Debug, and Ship**. A mode change that
affects capability, attached-project context, visible context, or permissions
requires confirmation and never inherits project context, approvals, dispatch,
or transcript data. This excludes persistent transcript/history storage,
provider/authentication changes, automatic dispatch, and any new execution
authority. The package gate requires a fresh `0.1.0-beta.20` candidate with
streaming/rendering, keyboard, screen-reader, zoom, desktop, and narrow-layout
coverage.

### 33 — Advisor Bounded PDF/Office Document Analysis

Add one closed `document` registry entry only after a type-specific native
projection design is reviewed. It may accept one supported PDF or Office document
and construct a bounded, safe native projection with a path-free manifest/hash
and explicit per-send confirmation. It excludes generic document upload,
raw-document transport, macros, embedded-object execution, document editing or
export, and project browsing. The package gate requires a fresh
`0.1.0-beta.26` lifecycle/transport corrective candidate, strict QuireForge-owned projection/resource-limit tests, no-raw-content
or path-retention tests, and full accessibility/package evidence.

### 34 — Workspace Selector and QuireForge Naming

Replace the page-level Conversation Mode selector with one compact accessible
workspace menu in the application/sidebar header: **Advisor — Create, learn,
and explore** and **QuireForge — Build, debug, and ship**. The menu invokes the
existing capability-boundary confirmation and clearing behavior; it adds no
authority, persistence, transfer, or authentication behavior. User-facing
workspace labels use QuireForge while technically accurate managed Codex
protocol, authentication, and runtime labels remain unchanged. The package gate
requires a fresh `0.1.0-beta.29` candidate and keyboard, focus-restoration,
screen-reader, desktop, and narrow-layout evidence.

### 35 — Advisor Archive Manifest-First Analysis

Add one closed `archive` registry entry for a single safely validated archive.
Advisor may receive only a bounded native-generated manifest of entry names and
aggregate safe metadata after explicit manifest/hash confirmation. It excludes
archive extraction into projects, recursive content analysis, embedded-file
handoff, password handling, symlink/path-traversal acceptance, and execution.
The package gate requires a fresh `0.1.0-beta.30` candidate with malformed,
zip-bomb, traversal, symlink, manifest-bound, disposal, and authority-regression
coverage.

### 36 — Advisor Static Binary/Executable Inspection

Add one closed `static-binary` registry entry for a single supported binary
format after an approved format and bounded-projection matrix. It may produce
only safe static metadata; it never loads, executes, debugs, emulates, or
detonates a file. Unsupported formats fail closed. The package gate requires a
fresh `0.1.0-beta.32` candidate with format-specific parser/resource-limit,
no-execution, retention, contract, accessibility, and package evidence.

### 37 — Advisor/Approval/Dispatch/Execution End-to-End Acceptance Gate

Run a separately approved human-in-the-loop acceptance gate over the existing
Advisor, digest-bound Approval/Dispatch controller, one-time managed Codex
execution handoff, and bounded completion-report flow. It uses deterministic
contracts plus explicitly user-participated managed-Codex session checks and
proves approval, invalidation, dispatch, completion, mode-switch, and recovery
behavior. It excludes new authority, automatic retry or redispatch, transcript
retention, and destructive project actions. A package version changes only if
implementation corrections are separately approved; otherwise this is a
release-acceptance gate with bounded evidence and cleanup.

**Complete (evidence-only):** deterministic approval, invalidation, one-time
dispatch, completion-report, mode-reset, and recovery contracts passed. The
user-authorized managed-Codex checks completed strict no-project Advisor turns,
an interruption followed by recovery, and one read-only/untrusted execution
profile turn in a disposable directory, with no authority request or project
modification. No implementation or package version changed. See
[Milestone 37 acceptance evidence](MILESTONE_37_END_TO_END_ACCEPTANCE.md).

### 38 — Dynamic Sandbox / Malware-Analysis Discovery Gate

Produce a decision-ready threat model and feasibility proposal for any future
dynamic analysis. It must define isolation, cleanup, platform support, legal and
operational constraints, network policy, and a clear go/no-go recommendation.
It adds no product code, package, sandbox, execution, or malware handling.

**Complete (decision-only):** the discovery gate closes with a no-go for
dynamic analysis in the current desktop architecture. Docker, bubblewrap, and
an ad-hoc QEMU process are not accepted as a hostile-binary boundary. Any future
proposal requires a separately approved KVM microVM architecture, zero-network
and no-host-mount policy, disposable lifecycle, typed bounded results, image
provenance, supported-host matrix, and legal/operational ownership. See
[Milestone 38 discovery record](MILESTONE_38_DYNAMIC_SANDBOX_DISCOVERY.md).

### 39 — Dynamic Sandbox Analysis Implementation

Add a separately installed, root-owned `quireforge-sandboxd` component for one
explicitly selected and confirmed static ELF64 x86_64 `ET_EXEC` or static-PIE
`ET_DYN` sample per run. It uses pinned Firecracker 1.15.1 and the matching
jailer only on supported Linux x86_64 KVM hosts; it has no guest network and no
host or project mounts. The immutable guest contains only a fixed,
non-interactive analysis agent and returns a bounded
`dynamic-analysis-result-v1` metadata result. Distribution remains Debian-only;
the panel shows unavailable state when the separately administered worker is
absent.

Only signature-validated ELF64 x86_64 `ET_EXEC` and `ET_DYN` samples with no
`PT_INTERP` are accepted. Dynamically linked samples fail closed with the broad
`unsupported-runtime` diagnostic. The worker never accepts generic uploads,
terminal output, guest-file export, Advisor input, project attachment, or
automatic execution. It excludes dynamic-loader/library support, other binary
formats, runtime interaction, network, release, and deployment. The package
gate requires fresh `0.1.0-beta.33` desktop and worker Debian evidence, a
Debian-only release contract, immutable guest asset provenance, ABI evidence,
benign-probe-only worker tests, and installed/visible desktop lifecycle gates.

### Post-M39 corrective checkpoint — Workspace Boundary Acknowledgement

Correct the repeated workspace-boundary confirmation without renumbering or
rewriting completed M34 history. The first ordinary Advisor/QuireForge switch
under `advisor-quireforge-boundary-v1` requires explicit confirmation.
Confirmation stores only the closed non-sensitive local record
`{ schemaVersion, boundaryPolicyVersion, acknowledged }`; ordinary later
switches under that exact current policy proceed without another dialog.
Missing, malformed, unknown, or stale data requires confirmation again, and a
material change to the capability/context boundary must increment the policy
version. Every completed switch still performs the existing transient clearing
and isolation actions. This adds no workspace authority, context transfer,
transcript persistence, project path, attachment, approval, dispatch, terminal,
Git, worktree, execution, credential, or provider state. The corrective package
gate requires a fresh `0.1.0-beta.34` Debian desktop and worker release set,
focused acknowledgement/focus/isolation tests, full validation, provenance/ABI,
lifecycle, installed-smoke, and visible-launch evidence.

### 40 — QuireForge Task Workbench Shell

Evolve the existing QuireForge route into a calm task-centred workbench without
creating a generic IDE or changing authority. The existing conversation remains
the primary surface. Add a compact keyboard-accessible **Actions** palette for
existing navigation, an optional closed-by-default workbench-context drawer
with honest Diff, Git, and Problems summaries, and an optional collapsed
managed-terminal dock that re-presents the existing terminal registry only.
The shell preserves workspace isolation, terminal ownership, approval binding,
responsive behavior, and focus restoration. It introduces no shell, PTY,
command-launch, project-write, dispatch, execution, provider, or transport
path. A future closed attachment-composer milestone may present one **Attach a
file** entry; this shell does not add upload, drag-and-drop, attachment
collection, or file-type behavior. The package gate requires a fresh
`0.1.0-beta.35` Debian desktop and worker release set, full keyboard,
screen-reader, desktop, narrow-layout, 200%-zoom, bundle-ceiling, provenance,
ABI, lifecycle, installed-smoke, and visible-launch evidence.

**Complete:** integrated at `98fa8fa26d740572095c2dcd9d4c1f579156817b` with
the clean `0.1.0-beta.35` Debian-only desktop and worker release set. The
source, keyboard/focus, accessibility, desktop/narrow-layout, bundle,
provenance/ABI, container lifecycle/smoke, and installed-host visible-launch
gates passed. See the [Milestone 40 report](MILESTONE_40_TASK_WORKBENCH_SHELL.md).

### 41 — Advisor Conversation Usability

Refine the existing bounded, transient Advisor conversation only: make its
transcript the independent scrolling region; preserve follow-latest while a
reader remains at the end; provide a keyboard-accessible **Jump to latest**
control when they scroll away; ensure the final reply remains reachable above
the anchored composer; and make the existing closed-by-default Advisor details
surface independently scrollable as a desktop drawer and narrow overlay. The
drawer may display only existing safe, transient Advisor context summaries and
capability information.

This changes no Advisor input, attachment, app-server transport, approval,
dispatch, project, terminal, Git, execution, connector, authority, retention,
or persistence contract. The package gate requires a fresh
`0.1.0-beta.36` Debian desktop and worker release set; deterministic unit,
desktop/narrow Playwright, 200%-zoom, keyboard, focus-restoration,
screen-reader, no-overflow, bundle-ceiling, provenance/ABI, lifecycle,
installed-smoke, and visible-launch evidence are required.

**Complete:** integrated at `eee6a9ac7e3393fd7dcd73a2c4304894c70839d4` with
the clean `0.1.0-beta.36` Debian-only desktop and worker release set. Source,
desktop/narrow accessibility and scrolling checks, `48` desktop and `8`
website Playwright checks, bundle validation, provenance/ABI, container
lifecycle/smoke, restricted installed-package validation, and installed-host
visible-launch gates passed. See the [Milestone 41 report](MILESTONE_41_ADVISOR_CONVERSATION_USABILITY.md).

### Post-M41 Packaging-Efficiency Corrective Checkpoint

Use a checksum-verified cache for immutable Linux-kernel and Firecracker
archives only inside the pinned Ubuntu 22.04 release container. Every cache
hit is revalidated before extraction; all guest outputs remain disposable
fresh builds, and release provenance, ABI, lifecycle, smoke, and visible-launch
validation remain fail-closed. This changes no application capability,
dependency, authority, release, or deployment behavior. Its package candidate
is `0.1.0-beta.37`; this reserves that unique version and shifts the later
provisional package sequence below.

**Complete:** integrated at `502e56e46131c64e7821fc98b16152142ac50eff` with
the clean `0.1.0-beta.37` Debian-only desktop and worker release set. Cache
reuse/tamper/unsafe-name/symlink rejection tests, source validation,
desktop/narrow and website Playwright, provenance/ABI, container lifecycle,
restricted installed-package, and installed-host visible-launch gates passed.
See the [packaging-efficiency checkpoint](PACKAGING_EFFICIENCY_CHECKPOINT.md).

### 42 — Shared Task Continuity Proposal

Produce a separately approved architecture decision for one explicit,
user-visible task envelope between Advisor and QuireForge. It may contain only
a task title, original user request, user-approved task brief/handoff,
explicitly selected safe attachment manifests or bounded representations, and
bounded execution/completion receipts. It must exclude default transcript
transfer, project/repository/worktree/terminal/Git/permission state, raw paths,
approval/dispatch internals, execution authority, and hidden synchronization.
This is a decision-only gate with no package or product change.

### 43 — Shared Task Continuity

After the M42 contract is approved, implement the explicit Advisor-to-QuireForge
reviewed brief and QuireForge-to-Advisor bounded completion-receipt flows. Every
handoff is user initiated, expiring, cancelable, isolated, path-free, and
auditable; it never transfers a full transcript or authority. The provisional
package candidate is `0.1.0-beta.38`, subject to revalidation as the next unique
version when implementation is approved.

**Complete:** source integrated at `6eb526bdb0b1705414f5507081dc37872358198c`
with the clean `0.1.0-beta.38` Debian-only desktop and worker release set. The
bounded envelope is one-use, expiring, cancelable, path-free, and native-memory
only; it transfers no attachment payload, transcript, project, terminal, Git,
approval, dispatch, execution, or authority state. Full validation,
provenance/ABI (`GLIBC_2.34`), lifecycle, installed-package, smoke, and visible
launch gates passed. An initial transient Xvfb window-probe failure was
diagnosed without changing the requirement; the unchanged official gate then
passed.

The initial `task-handoff-envelope-v1` carries only a bounded title, original
user request, reviewed brief, or bounded completion receipt. Safe attachment
manifest/projection transfer is intentionally deferred to a later, separately
approved type-specific collection contract.

### Post-M43 Temporary Bundle Construction-Envelope Checkpoint

Replace reactive small bundle-budget changes with one measured, temporary
construction-period envelope. The 256 KiB startup-entry ceiling remains
unchanged because its beta.38 194,943-byte baseline retains 34.5% headroom.
The 448 KiB application-shell, 1.5 MiB total-JavaScript, and 160 KiB CSS
ceilings use binary allocation boundaries to cover M44–M58: M44–M50 work plus,
if separately approved, M52 durable-task records, M54 local artifact/design
review, and M56 local templates. M51, M53, M55, M57, and M58 are decision-only
and add no shipped bundle by themselves. Every limit remains enforced, reported
with largest assets, and closed to automatic escalation. The preliminary
beta.39 release set was superseded locally before it was recorded as
authoritative package evidence because it covered only M44–M50. This corrected
packaging-only checkpoint is the next unique
`0.1.0-beta.40` candidate and shifts later provisional package identities by
one without changing product scope. The post-workbench permanent-budget
reconciliation remains mandatory.

**Complete locally:** the clean `0.1.0-beta.40` Debian set is bound to
`0fed7983a3f32aa79ea4d1feee9947535d370a9b`; checksum, provenance/ABI,
lifecycle, container/installed smoke, and visible-launch gates passed. It does
not add a product capability or alter the permanent-budget reconciliation gate.

**Permanent-budget reconciliation complete:** A clean current production build
measured 195,014-byte entry, 239,280-byte application shell, 1,103,448 bytes
of JavaScript, and 118,484 bytes of CSS. The closed permanent budget is now
256 KiB entry, 320 KiB application shell, 1.25 MiB JavaScript, and 144 KiB
CSS. It preserves lazy route/pane loading, sets no new product authority, and
requires fresh measured evidence and explicit approval for any future increase.

### 44 — Unified Single Attachment Entry

Present one **Attach a file** action that routes only to already supported,
type-specific native pickers and validators. Native code remains authoritative
for type determination. The UI gains a compact bounded attachment tray without
adding generic upload, drag-and-drop, a new file type, multi-attachment
collection, changed transport, or relaxed confirmation/disposal rules. The
provisional package candidate is `0.1.0-beta.41`.

**Complete:** implemented at `891abf6d953e3b7c0dd3f0d3bd03baeb29de40fb` with
the clean `0.1.0-beta.41` Debian and worker package set. The single entry
selects only existing closed text/data, PNG/JPEG, PDF, ZIP, or ELF native
handlers. Full source validation, bundle ceilings, provenance/ABI, lifecycle,
container and installed smoke, and visible-launch gates passed. See the
[Milestone 44 report](MILESTONE_44_UNIFIED_SINGLE_ATTACHMENT_ENTRY.md).

### 45 — Bounded Multi-Attachment Proposal

Verify documented managed app-server multi-input support and define an ordered
collection manifest, collection digest, aggregate limits, atomic confirmation,
failure, claim, and disposal behavior for already supported closed types. This
is a capability-boundary decision gate with no package change.

**Approved decision:** adopt the closed, at-most-three, at-most-one-image
`advisor-attachment-collection-v1` contract defined in the
[Milestone 45 proposal](MILESTONE_45_MULTI_ATTACHMENT_PROPOSAL.md). It uses
only existing typed handlers and the documented app-server input list; the
completed M46 implementation remained within that scope.

### 46 — Bounded Multi-Attachment

Only after M45 approval, add a closed ordered collection of approved existing
attachment types. Every member retains its own manifest, confirmation,
expiration, one-use claim, and disposal; any invalid member fails the whole
request without silent omission or partial reuse. This is not generic upload.
The provisional package candidate is `0.1.0-beta.42`.

**Complete:** implemented at `1bc2e787ab785016041d70845c97ca9c2c4f84db` with
the verified `0.1.0-beta.42` Debian and worker set. The closed collection is
limited to three existing typed entries and one image, with native 40 MiB
aggregate preflight and explicit collection confirmation. See the
[Milestone 46 report](MILESTONE_46_BOUNDED_MULTI_ATTACHMENT.md).

### 47 — Generated Artifact Workflow Proposal

Define a bounded transient artifact registry, multiple reviewed output cards,
type-specific validation, and an explicit native save boundary for `.txt`,
`.md`, `.json`, `.csv`, and `.py`. This decision gate must exclude automatic
save, execution, project writes, overwrite, Git actions, retained destination
paths, and content persistence. No package changes here.

**Decision-ready proposal:** [Milestone 47](MILESTONE_47_GENERATED_ARTIFACT_WORKFLOW_PROPOSAL.md)
recommends the closed five-entry, native-memory-only
`advisor-generated-artifact-registry-v1` with a 512 KiB per-artifact, 2 MiB
aggregate, 15-minute lifecycle and one-artifact native atomic no-replace Save
boundary. M48 remains blocked pending explicit approval; no package or product
behavior changed in this proposal milestone.

### 48 — Generated Artifacts and Explicit Save

After M47 approval, provide typed inline artifact cards and independent,
explicit native Save dialogs. Saving remains one artifact at a time and never
opens, runs, imports, writes to a project automatically, or transfers
authority. The provisional package candidate is `0.1.0-beta.43`.

**Complete:** `advisor-generated-artifact-registry-v1` is integrated at
`5d483d0c068c450bbc779ee07b048fe848c7e1f0` with verified `0.1.0-beta.43`
Debian and worker evidence. See the
[Milestone 48 report](MILESTONE_48_GENERATED_ARTIFACTS_AND_EXPLICIT_SAVE.md).

### 49 — QuireForge Review Panes

Extend the task workbench with user-controlled, lazily loaded Files, Diff,
Git, Preview, Activity, and Approval panes over existing typed native services.
Background events may badge but never steal focus; stale data invalidates on
project/worktree changes. No new Git, project-write, shell, terminal, dispatch,
execution, browser, or provider authority is added. The provisional package
candidate is `0.1.0-beta.44`.

**Complete:** implemented at `f1a44324859faa2ed43f24ab60db12b58e6c6836` with
verified `0.1.0-beta.44` Debian and worker evidence. See the
[Milestone 49 report](MILESTONE_49_REVIEW_PANES.md).

### 50 — QuireForge Workbench Layout Refinement

Add accessible resizable/collapsible presentation controls, bounded local
layout preferences, and managed-terminal-dock ergonomics over existing PTY
ownership. Preferences must exclude paths, transcripts, terminal output,
approvals, credentials, and capability state. This does not create a shell,
PTY, command-launch, execution, or context-transfer path. The provisional
package candidate is `0.1.0-beta.45`.

**Complete:** implemented at `1cc7c50ceed6d2b6c2f91274110471d71fe6292a`
with verified `0.1.0-beta.45` Debian and worker evidence. M51 behavior is not
included. See the [M50 report](MILESTONE_50_WORKBENCH_LAYOUT_REFINEMENT.md).

### 51 — Durable Task Records and Alternate-Plan Proposal

Produce a separate persistence, privacy, and lifecycle decision for minimal
durable task records, title/status search, archive/restore, and alternate-plan
branches. It must distinguish existing Codex-owned session history from the
transient Advisor transcript and the explicit M43 handoff envelope. It may not
make raw transcript transfer, project/repository/worktree/terminal/Git state,
permissions, credentials, or execution state shared by default; an alternate
plan never clones execution state or authority. This is a decision-only gate
with no package change.

**Approved proposal:** [M51 durable task records and alternate-plan
proposal](MILESTONE_51_DURABLE_TASK_RECORDS_ALTERNATE_PLAN_PROPOSAL.md)
selects a bounded local SQLite task catalogue, four non-authoritative plans per
task, explicit archive/restore/deletion, and no transient-state import. Its
closed contract was explicitly approved for M52.

### 52 — Durable Task Records and Alternate Plans

Only after M51 approval, implement the approved closed record schema and
lifecycle. Retention, deletion, search, archive/restore, and any handoff
reference must be explicit, bounded, testable, and separately scoped for
Advisor and QuireForge. The provisional package candidate is
`0.1.0-beta.46`.

**Complete:** source commit
`6df055999d2ad01d2385096a14bc71f8aada2a8c` implements migration 11, native
storage and transactions, strict Rust/Zod commands, accessible compact
workbench controls, and deterministic isolation/failure/capacity tests. Its
clean beta.46 desktop and worker Debian artifacts passed the authoritative
pinned Ubuntu 22.04 package, visible-launch, provenance, ABI, sandbox-worker,
artifact, and restricted installed-host gates. See the
[M52 report](MILESTONE_52_DURABLE_TASK_RECORDS.md).

### 53 — Local Artifact and Design Review Proposal

Define a bounded local artifact/design-review contract over approved task
briefs, plans, mockups, safe previews, validation evidence, and generated
artifacts. It must specify comparison, selection, annotations, review state,
and user-approved promotion into a QuireForge task. It excludes broad
repository scraping, automatic execution, external publishing, deployment,
and direct third-party connectors, including Figma. This is a decision-only
gate with no package change.

**M53-B core contract:** [Local review core contract](MILESTONE_53B_CORE_REVIEW_CONTRACT.md)
selects task-scoped private-SQLite collections of bounded copied text, static
image mockups, and typed evidence snapshots. **M53-C interaction contract:**
[Local review interaction contract](MILESTONE_53C_REVIEW_INTERACTION_CONTRACT.md)
selects the bounded Review-tab interaction, accessibility, state presentation,
and deterministic M54 test design. **Complete:** [M53 local artifact and
design review proposal](MILESTONE_53_LOCAL_ARTIFACT_DESIGN_REVIEW_PROPOSAL.md)
reconciles all four M53 phases and approves the bounded M54 target
`0.1.0-beta.47` / `0.1.0~beta.47`. M54 is not started and requires its single
explicit approval gate.

### 54 — Local Artifact and Design Review

Only after M53 approval, implement the approved local review surfaces by
reusing the M48 typed artifact boundary and M49 preview/review services.
Artifact lifecycle, selection, comparison, accessibility, and no-path/no-
persistence boundaries are deterministic.

**Complete:** M54 is closed at package/source commit
`c4c2752466f36f791fde47edbc5c6b02b0e21320`, tagged
`v0.1.0-beta.53`. The beta.53 Debian pair passed the pinned Ubuntu 22.04
package, lifecycle, smoke, visible-launch, and restricted installed-host gates;
headless completion returned `created`, then `existing`. All seven ratified
evidence sources are implemented with persisted-bytes-only previews. The former finalizer lost the
beta.47 normalized canonical set; only its recorded hashes and distinct raw
Tauri bundle remain, and no replacement evidence was fabricated. Beta.48 is a
validated but non-deployable release candidate because it lacked the production
unprivileged receipt bootstrap. Beta.49 is preserved, validated, and
non-promoted because its promotion correctly failed on an order-sensitive
finalizer checksum check. Beta.50 was source-bound and passed build/smoke, but
its promotion stopped because the separate release validator retained that
order-sensitive comparison. The beta.51 corrective follow-on centralizes the
closed checksum mapping across release generation, finalization, and validation,
then adds a
fixed headless completion branch that resolves the production metadata path,
one live attached project, and one migration-18 unprivileged predecessor;
it initializes no GUI surface and records only through the native controller;
it also begins preservation by archiving the coherent beta.48 release set.
Beta.52 remains a preserved failed installed-host candidate and is not released.
The four beta.53 draft-prerelease assets are byte-identical to the canonical
package set; publication and deployment remain separate actions.

### 55 — Research Reports and Inspectable Task Templates Proposal

Define separately the contracts for source-linked bounded research reports and
user-visible QuireForge task templates/specialists with inspectable
instructions and explicitly selected approved context. The template-only
decision must remain separate from any live browsing, source retrieval, or
provider integration. It excludes opaque personalization, hidden retrieval,
automatic browsing, and automatic actions. This is a decision-only gate with
no package change.

**Complete and published:** the earlier proposal's M55 deferral was replaced by
the ratified [M55 Durable Source Admission Implementation
Contract](MILESTONE_55_DURABLE_SOURCE_ADMISSION_CONTRACT.md). Beta.59 is
published at `v0.1.0-beta.59`, bound to
`d6967e8bfd82acbef7dfa0dc74f085720f8b0384`. It adds only explicit, local,
private textual admission and chooser fail-safe handling; retrieval, provider
traffic, context inclusion, connectors, browser authority, and MCP remain
excluded.

### 56 — Inspectable Local Task Templates

**Complete:** James separately approved the closed M55 template-only contract.
The foundation is complete at
`614b22c870dd5a45c88e1e8f59dedc51c4b1c671` (`feat: add task template
foundation`); private transactional storage is complete at
`693a7e95e911ac6b50a74734171e2305fbdddbc0` (`feat: add task template
storage`); and the native lifecycle service is complete at
`03163389d6fe69dbf0deedfd532f8bad2f7bde03` (`feat: add task template
lifecycle`). M56-G1 through G8 are source-complete: migration 21 owns local
templates, migration 22 owns bounded digest-only application reservations, and
four static built-ins remain outside SQLite. The closed lifecycle and strict
bridge, lazy management UI, digest-bound application workflow, accessibility,
and focused browser acceptance are implemented. Research reports, providers,
connectors, MCP, browser authority, credentials, hidden instructions,
import/export, automatic actions, approval, dispatch, and execution remain
excluded. `0.1.0-beta.54` / `0.1.0~beta.54` closed at package/source commit
`e2b084ed0bdf17fb6f4b0b47663cdf6952ec8e73`, annotated tag
`v0.1.0-beta.54`, and a `James-Jennison/quireforge` draft prerelease. G9 source
acceptance, G10 canonical pinned-Ubuntu-22.04 packaging, and G11 restricted
installed-host validation passed. The draft contains exactly the canonical
Debian pair, `SHA256SUMS`, and `release-manifest.json`: the application package
is 5,864,924 bytes with SHA-256
`643e6bc3caf9068f7ed521ecd949f9f3f5d38b9c6a82bcce19384370f644d131`, and the
sandbox package is 3,233,492 bytes with SHA-256
`bd9c0682c0e9dd7761b28f03eb2e801ab7a925e7c5f5587eefc68bd7578bd21f`. Both
beta.54 packages remain installed, headless completion returned `created`, then
`existing`, and no rollback was required.
The draft is not published and there has been no deployment. Beta.53 remains
preserved as the prior released rollback generation; beta.52 remains preserved
as an unreleased candidate that failed its installed-host gate. The next work
is a separate decision/planning boundary. M57 is now complete as a
decision-only governance record; M58 remains a future decision gate and does
not start automatically.

### 57 — Connector Governance and External Authority

**Current authoritative contract:** [M57 Connector Governance Contract and
Executable Implementation Plan](MILESTONE_57_CONNECTOR_GOVERNANCE_CONTRACT.md)
ratifies the provider-neutral connector identity, capability/grant, lifecycle,
one-use authorization, ambiguous-outcome, audit, M55/context/transmission, and
generic-MCP exclusion boundary. It superseded the earlier implementation
deferral and governed the completed beta.60 fictional/local-only vertical
slice. It does not authorize a real
connector, credentials, network, retrieval, external mutation, M58 browser
behavior, MCP dispatch, or automation.

**Beta.60 implementation:** migration 24 and the closed Tauri/React
fixture implement the approved fictional/local-only read and mutation exercise.
They persist only bounded project/task, descriptor-digest, operation,
authorization, lifecycle, terminal-outcome, and audit linkage; no connector
content or secret is retained. The fixture has no network or external side
effect. Its packaging, installation, release staging, and publication all
completed without broadening its authority.

**Published completion:** beta.60 is published at
`v0.1.0-beta.60`, source
`b8b807f256170e6a35ada22893b410cb4b0057b7`. Its fictional connector remains
strictly local-only; the historical candidate wording above records its
pre-publication evidence and does not defer M57 implementation.

**Decision complete:** [M57 connector governance](MILESTONE_57_CONNECTOR_GOVERNANCE.md)
defines the least-authority connector classification, authority ladder,
consent, credential, provenance, privacy, audit, failure, and revocation
requirements for any future external boundary. It preserves the separately
approved Codex-owned Integration Center as-is and grants no new connector,
provider, MCP, OAuth, credential, browser, retrieval, synchronization, or
mutation authority. Its recommendation is to require additional bounded
decision artifacts before implementation: no per-project connector foundation
is approved, and no standing write or autonomous mutation authority is
permitted.

**Prerequisite decisions complete:** [post-M57 connector foundation
prerequisites](MILESTONE_57_CONNECTOR_FOUNDATION_PREREQUISITES.md) resolve the
native ownership, opaque account/project/scope reference, lifecycle,
operation/result, mutation-safety, content-free provenance/audit, retention,
and static descriptor-trust contracts. They approve only a proposed, unstarted
local mock-only foundation with no network, credentials, OAuth, provider,
browser, mutation, background, generic MCP, or M55 source-manifest authority.
Any real connector remains deferred to later provider-specific decisions.

**Local mock-only foundation source-complete:**
[M57 local mock connector foundation](MILESTONE_57_LOCAL_MOCK_CONNECTOR_FOUNDATION.md)
implements the approved private native in-memory contracts and deterministic
mock adapter. It adds static digest/version-bound fictional descriptors, opaque
bindings and inert credential references, closed lifecycle/operation/result
models, one-use expiry/cancellation/replay protection, and content-free audit
records. It adds no persistence, Tauri command, bridge, UI, network, provider,
credential, OAuth, browser, external mutation, or M55 source-manifest
authority. Packaging and release work remain separate future goals.

**Historical source-only closure:** [M57 source acceptance and release
policy](MILESTONE_57_SOURCE_ACCEPTANCE_AND_RELEASE_POLICY.md) accepts
`a1d407469626e34cd5d4921abdb6c8d305895d7e` as source-complete and closes M57
without a package or version. Its beta.54 release statements are historical;
beta.60 is now the latest published generation. The M57 contract retains M58
as a separate runtime lane; M58 planning is now complete.

### Goal — Provider-Neutral AI Foundation

**Active long-term product goal:** [Provider-Neutral AI Foundation](GOAL_PROVIDER_NEUTRAL_AI_FOUNDATION.md)
organizes the related decision gates, future implementation milestones, and
future release checkpoints. It grants no implementation authority, package, or
version. M55, M57, and M58 remain separate authority boundaries. M59 is the
completed context/transmission decision contract; M60 is implemented as the
beta.63 governed fictional/local-only vertical slice.

**Completed decision gates:**

- [External Capability Taxonomy and Sequencing](EXTERNAL_CAPABILITY_TAXONOMY_AND_SEQUENCING.md)
  separates the external-capability lanes and their dependencies.
- [Provider-Neutral Capability Registry and Descriptor
  Governance](PROVIDER_NEUTRAL_CAPABILITY_REGISTRY_AND_DESCRIPTOR_GOVERNANCE.md)
  defines metadata-only descriptor identity, provenance, capability claims,
  lifecycle, extensions, and authority separation.
- [Canonical Provider-Neutral Interaction and Event
  Protocol](CANONICAL_PROVIDER_NEUTRAL_INTERACTION_AND_EVENT_PROTOCOL.md)
  defines communication-only interaction attempts, envelopes, lifecycle,
  streaming, continuation, opaque provider-session references, structured and
  multimodal events, tool proposals/results, grounding, usage, errors, and
  governed extensions.
- [Provider Adapter Lifecycle and Conformance
  Governance](PROVIDER_ADAPTER_LIFECYCLE_AND_CONFORMANCE_GOVERNANCE.md)
  defines adapter identity, compatibility, trust, lifecycle, upgrade, rollback,
  revocation, quarantine, capability mapping, protocol translation, conformance,
  extension handling, and failure-closed behavior.
- [Credential Broker and Account/Project/Scope
  Custody](CREDENTIAL_BROKER_AND_ACCOUNT_PROJECT_SCOPE_CUSTODY.md) defines
  native broker ownership, opaque references, account/project/scope bindings,
  non-secret credential classes, least-authority leases, lifecycle/recovery,
  no-ambient-authority rules, content-free audit, and failure-closed behavior.
- [Context Assembly and Transmission
  Manifests](CONTEXT_ASSEMBLY_AND_TRANSMISSION_MANIFESTS.md) defines native-owned
  selection, exact item/projection bindings, transformations/omissions,
  exclusions, destination-aware authorization, revalidation, continuation
  confinement, privacy/retention, and content-free audit evidence.
- [M59 Context Assembly and Transmission
  Contract](MILESTONE_59_CONTEXT_ASSEMBLY_AND_TRANSMISSION_CONTRACT.md)
  supersedes the earlier record where they differ and ratifies M60's exact
  deterministic selection, preparation, review, confirmation, fictional-sink,
  retention, and failure boundary.
- [Limited Provider Inference
  Boundary](LIMITED_PROVIDER_INFERENCE_BOUNDARY.md) defines exact attempt
  binding/revalidation, an initially text/local-projection-only future route,
  lifecycle and ambiguity handling, output/proposal confinement, disclosures,
  emergency stop, and a separate local-runtime variant.

**M60 implementation:** [Governed Context Assembly Vertical
Slice](MILESTONE_60_GOVERNED_CONTEXT_ASSEMBLY.md) adds explicit deterministic
selection, redacted private assembly, review, one-use fictional delivery, and
content-free audit records. It is local-only and does not select a provider.

All currently planned core architecture gates are complete. No implementation
milestone starts automatically.

**M61 decision-only completion:** [Credential Broker and Account Reference
Contract](MILESTONE_61_CREDENTIAL_BROKER_AND_ACCOUNT_REFERENCE_CONTRACT.md)
ratifies only future custody/runtime selection criteria, scoped opaque account
references, lifecycle, content-free audit, and adapter compatibility gates. It
does not select or implement a custodian, provider, local runtime, account,
credential, or successor milestone.

**M62 decision-only completion:** [Limited Provider Inference
Boundary](MILESTONE_62_LIMITED_PROVIDER_INFERENCE_BOUNDARY.md) ratifies only
the future M60 bundle, M61 reference, destination/model allowlist, typed
adapter, privacy, lifecycle, and fail-closed gates for a limited-inference
proposal. It does not select or implement a provider, local runtime, model,
credential, or successor milestone.

**M63 local candidate:**
[In-Process Credential-Free Local Runtime Adapter](MILESTONE_63_IN_PROCESS_LOCAL_RUNTIME_ADAPTER.md)
vendors a verified, static, CPU-only llama.cpp source boundary for the one
approved Qwen2.5-3B descriptor. The beta.64 local candidate binds one confirmed
reviewed M60 bundle to one in-process attempt with fixed input, output,
deadline, cancellation, and open-view-only result limits. The fresh clean-tree
beta.66 package pair from source commit
`822b6703968f4cea95ce4828f130739bc56e8a01` passes the pinned package,
lifecycle, visible-launch, and release-artifact gates; focused native, UI, and
loopback-only browser-fixture coverage also passes. It excludes the model and
did not start the runtime. It includes no model artifact, acquisition,
provider, credential, network, public runtime route, package publication,
release, or deployment authority. The governed review preflights a typed,
content-free local-runtime availability state so an absent supervisor-provided
model contract cannot consume its one-use authorization. End-to-end
installed Debian desktop acceptance remains required before a release-ready
claim. The focused host-native adapter gate has completed one bounded
local-only attempt without retaining a model location or generated output; it
is real-adapter evidence only, not the required governed-review desktop flow.
The clean-tree beta.67 Debian pair from source commit
`8f604e3b98394b8ba8d5170c82818f357d5d5a11` passes the authoritative pinned
Ubuntu 22.04 package, lifecycle, visible-launch, and release-artifact gates;
it excludes the model and did not start the runtime. Installed-host
governed-review desktop acceptance remains pending. A repeated clean-source
beta.67 package gate on 2026-08-13 reproduced the same package/lifecycle/
visible-launch/release-artifact result while again excluding the model and
never starting the runtime.
The beta.68 candidate extends the governed-review browser fixture through an
application reload, proving that a completed local result is not restored
outside its open view. Its clean-tree Debian pair from source commit
`5c4ca198f94553dd760f20734f765c8abb5a488e` passed the authoritative pinned
Ubuntu 22.04 package, lifecycle, visible-launch, and release-artifact gates;
it excluded the model and did not start the runtime. Installed Debian desktop
acceptance remains pending.

The beta.69 candidate keeps the governed-review local-only action disabled
until its typed content-free runtime-availability preflight completes, with an
explicit checking state. It preserves native authority, excludes the external
model from packages, and does not start the runtime in package gates.
Installed Debian desktop acceptance remains pending.

The beta.70 candidate resolves an IPC-level availability-preflight failure to
a bounded unavailable state, preserving the disabled one-time action and
unconsumed acknowledged bundle. Package promotion and installed Debian desktop
acceptance remain pending.

**First implementation milestone source-complete:** [Provider Capability
Registry Contracts](MILESTONE_PROVIDER_CAPABILITY_REGISTRY_CONTRACTS.md)
implements private static fictional registry contracts and focused safeguards
only. It has no persistence, bridge, UI, provider route, package, or release.

**Second implementation milestone source-complete:** [Provider Interaction/Event
Contracts and Deterministic Mock Adapter Conformance](MILESTONE_PROVIDER_INTERACTION_PROTOCOL_CONTRACTS.md)
implements private fictional interaction contracts and deterministic fixture
translation only. It has no persistence, bridge, UI, provider route, package,
or release.

**Third implementation milestone source-complete:** [Provider-Neutral Core
Foundation and Mock Inference Vertical Slice](MILESTONE_PROVIDER_NEUTRAL_MOCK_INFERENCE_VERTICAL_SLICE.md)
adds a user-visible but strictly fictional/in-memory task-bound mock workflow.
Its manifest, inert lease, authorization, events, usage, and evidence are
closed, bounded, and local only; it has no real provider, network, credential,
context-transmission, retrieval, native-operation, persistence, package, or
release authority.

**Source-complete local hardening milestone:** [Provider-Neutral Mock Workflow
Hardening and Release Readiness](MILESTONE_PROVIDER_NEUTRAL_MOCK_WORKFLOW_HARDENING.md)
retains the local fixture boundary while covering registry-backed destinations,
bounded lifecycle polling, cancellation confirmation, authority failure,
recovery, and representative browser acceptance. A real-provider readiness or
provider-selection decision does not begin automatically.

**Replacement release checkpoint:** beta.55 passed source, package,
installation, and native-receipt gates but is release-ineligible because the
installed New task route omitted the Task Catalog/New task UI. Its artifacts
and receipt remain immutable failed-candidate evidence. Beta.56 then proved
release-ineligible at installed-host acceptance because its Task Catalog created
an unbound default task. The strictly increasing `0.1.0-beta.57` replacement
restores only explicit named, project-bound task creation for the governed
durable-task-to-mock-workbench path and must pass the existing pinned Ubuntu
22.04 and restricted installed-host gates; beta.54 remains the rollback
generation unless it passes every gate.

**Later implementation milestones:** Routine reversible, local,
non-production post-M62 implementation proceeds under the autonomous operating
rule. It does not authorize credential or account handling, browser access,
real provider/runtime connection, network transmission, production deployment,
public release, destructive action, third-party commitments, or irreversible
product-direction selection. Release checkpoints are assigned only when
user-visible or operational behavior is implemented.

### 58 — Controlled Browser Verification Proposal

**Decision complete:** the
[M58 Controlled Browser Verification Contract](MILESTONE_58_CONTROLLED_BROWSER_VERIFICATION_CONTRACT.md)
ratifies a narrow, project/task-scoped, fictional/local-only, read-only
verification proposal. It separates browser session, target, navigation,
observation, evidence, M55 admission, context, provider transmission,
interaction, mutation, automation, generic MCP, credentials, and native tools.
It requires digest-bound one-use confirmation, bounded navigation and cleanup,
failure-closed ambiguity handling, and no automatic retry.

Published beta.62 implements only the contract's fictional,
deterministic, local-only, read-only slice using an ephemeral native-owned
WebKitGTK custom-scheme fixture. M58 is complete; no real browser target, profile,
credential, connector, provider, MCP, automation, or mutation authority is
introduced.

### 59 — Context Assembly and Transmission Contract

**Decision complete:** the
[M59 Context Assembly and Transmission Contract](MILESTONE_59_CONTEXT_ASSEMBLY_AND_TRANSMISSION_CONTRACT.md)
ratifies explicit item eligibility and selection, instruction/evidence
separation, deterministic assembly, minimization/redaction/bounds, immutable
prepared bundles, review, digest-bound expiring one-use confirmation,
fictional-sink ambiguity handling, retention, drift, recovery, and audit.
It implements no assembler, provider transmission, credentials, inference,
network, connector/browser authority, MCP, automation, or mutation.

M60 is complete and published as beta.63. The separately approved M61 contract
is decision-only; it authorizes no successor implementation.

### 61 — Credential Broker and Account Reference Contract

**Decision complete:** the
[M61 Credential Broker and Account Reference Contract](MILESTONE_61_CREDENTIAL_BROKER_AND_ACCOUNT_REFERENCE_CONTRACT.md)
ratifies the future selection, opaque reference, lifecycle, audit, and adapter
compatibility gates for any later custody route. It implements and selects
nothing: credential collection/storage, accounts, OAuth, provider/local-runtime
connections, networking, inference, tools, retrieval, browser/connectors/MCP,
automation, external mutation, migrations, runtime/UI work, packages, and
releases remain excluded from this decision record. Routine post-M62 local
implementation follows the autonomous operating rule above; its hard stops
remain unchanged.

### 62 — Limited Provider Inference Boundary

**Decision complete:** the
[M62 Limited Provider Inference Boundary](MILESTONE_62_LIMITED_PROVIDER_INFERENCE_BOUNDARY.md)
ratifies future limited-inference gates: exact M60 bundle and M61 prerequisite
binding, destination/model/capability allowlists, typed adapter containment,
privacy and payload limits, lifecycle, cancellation, revocation, recovery, and
content-free audit. It implements and selects nothing: providers/local runtimes,
credentials/accounts/OAuth, networking, inference, model configuration, tools,
retrieval, browser/connectors/MCP, automation, external mutation, runtime/UI
work, packages, and releases remain excluded from this decision record.
Routine post-M62 local implementation follows the autonomous operating rule
above; its hard stops remain unchanged.

### Deferred capability gates

Real in-app browsing for Codex-assisted research and verification is a deferred
capability gate. It must be designed as an isolated, governed browser surface
with explicit target/navigation scope, read-only observation and provenance
capture, and separately approved authentication, interaction, download, upload,
or external-mutation lanes. It must not reuse ambient browser sessions,
cookies, or credentials, and it does not follow from M58's fictional local-only
verification fixture.

New archive, Office, or binary formats; generic upload; executable loading or
execution beyond the separately installed M39 worker; dynamic loader support;
browser/provider expansion; direct third-party connectors (including Figma or
GitHub); MCP per-project enablement; browser verification agents; extension
marketplaces; and parallel/multi-agent execution each require their own
security, authority, transport, dependency, and retention proposal before any
implementation. The completed post-workbench bundle reconciliation established
evidence-based permanent JavaScript and CSS ceilings before any future
product-readiness or public-release approval. It measured route/chunk and
stylesheet costs, retained lazy pane loading, and set strict closed limits
rather than extending the temporary construction envelope. Future increases
still require fresh measured evidence and explicit approval.

### Advisor Approval/Dispatch Phases A–B3 (completed capability history)

The approved Phase A/B1 controller is an editable, transient draft and explicit,
expiring digest-only approval record. It binds the prompt, exact temporary safe
selected-context projection, target project, declared capability/profile manifest,
requested model and reasoning labels, timestamp, and decision. Approval
revalidates the complete transient binding and native target-project preflight
immediately before recording a decision. It has no dispatch command and no
reference to the Codex execution service; changing any bound input requires a
new approval. The integrated B2 slice hands one approved,
immediately revalidated request to the existing project-bound managed Codex
execution workspace. It supports only read-only/untrusted and
workspace-write/on-request profiles, consumes the approval once, stores only an
opaque start receipt, and never returns execution output to Advisor. The
integrated B3 slice returns only a bounded, correlated terminal completion
report; it never returns a terminal stream, raw transcript, repository data,
credentials, or new authority. Danger full access remains unavailable.

## Forecast policy

The initial whole-project estimate is several hundred active engineering hours
and many real-world weeks. Each milestone receives a refreshed range covering
inspection, implementation, builds, tests, debugging, visual verification,
documentation, review, and commit preparation before work begins. Forecasts
will be compared with measurable actuals in milestone completion reports.
