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

ADR 0029 accepts a serialized, interleaved next-delivery order: M70 typed
project knowledge, M71 evidence linkage, M76 isolated read-only browser
research, M72 objective-scoped authority, M77 connector read access, M73
agent-neutral context assembly, M78 scheduled/background work, M74 three-part
completion, M79 connector mutation/delivery, and M75 cross-agent handoff and
recovery. M70 is approved for implementation as a new private native service;
it does not alter the existing M66 content-free ledger or make QuireForge's
own ADR/CURRENT_STATE process ledger-backed.

The `0.1.0-beta.97` M70 candidate adds the private native Knowledge Ledger:
closed record kinds, bounded storage, immutable event history, owner-only
decision/constraint activation, and strict native/TypeScript contracts. Source
validation has passed; desktop E2E and package/installed-host acceptance remain
required before M70 is marked validated.

M65's metadata-only artifact-reference vertical slice is implemented in the
unreleased `0.1.0-beta.75` candidate. Migration 27 persists only a confirmed
project/task association to an opaque M48 artifact UUID, digest, closed class,
and bounded label. The Studio shows whether the independently transient
original is currently available, without retaining or recovering it.

M66 is complete as a project-scoped, read-only Work-lane ledger over existing
M55 durable-source, M57 fictional-connector, M58 controlled-browser, M60
context-bundle, and M65 artifact-reference governance metadata. Its native and
TypeScript contracts expose only bounded content-free identity, lifecycle,
digest, timestamp, expiry, item-count, and audit-outcome fields. It cannot
transfer content or authority. M63 activity remains represented by its M60
receipt; M64 introduces no separate ledger record.

M67 is complete as a deterministic local-fixture Work adapter catalogue. A
choice is ephemeral UI state only and cannot connect, authenticate, transmit,
retain input, or execute. M68 is complete as decision records for isolated
browser research, connector read, connector mutation/delivery, and scheduled
work; no external capability was implemented or authorized.

The unreleased `0.1.0-beta.79` accessibility candidate from source commit
`f7c79ea3a53a57dcb09bc2b5cfbea8f5780118b2` preserves the M66/M67 lazy Work
routes and their content-free/local-fixture boundaries while restoring the
complete H1-to-H2 hierarchy. Its local Debian candidate is intentionally
uninstalled and unreleased. The pinned Ubuntu 22.04 lifecycle, smoke, and
release-artifact checks passed, alongside `pnpm validate`, 78 desktop E2E
tests, 441 Rust tests (four ignored), and two sandbox tests. Its manifest is a
clean-tree `release-candidate` with SHA-256
`0cf4db1b38c3c9eac2105fed8740cf91033c71d1fd80181a1621f243c4f01303`; it is
immutable local evidence and must not be overwritten by a later package.

The unreleased `0.1.0-beta.80` M63 installed-host acceptance candidate from
source commit `43b655008e506f2f840ebf194e1ba0a10c14e590` passed the pinned
Ubuntu 22.04 package, lifecycle, smoke, and final artifact gates. Its desktop
Debian artifact has SHA-256
`edf17af3fb688136cc98caf1701f0532e5d46eb9556314d1993cdfe1178e37c7` and was
installed with the matching sandbox package. The governed local review then
failed closed before review: project-only preparation returned a zero-item,
zero-byte snapshot. No reviewed bundle, availability recheck, model attempt,
output, or retry occurred. This is immutable failed acceptance evidence and
does not support a release-ready claim; any later package requires a new unused
beta version.

The unreleased `0.1.0-beta.81` M63 replacement candidate fixes that
project-only preparation path: an empty review-evidence selection no longer
depends on optional repository storage, while every non-empty selection still
fails closed if that storage is unavailable. From clean source commit
`513a7806e01a8bfe4dd7cac4e901fe83a2267c08`, the pinned Ubuntu 22.04 package,
lifecycle, smoke, and final artifact gates passed. Its desktop Debian artifact
has SHA-256
`5dddfc36874ecd5d4b97e0ad3245bbfac603b03bf28ef3184ff4984dc55542bc`; the
matching sandbox package has SHA-256
`b6fce951b6849316239aaa6026c82f87ca797ee496e86a774d6bbfdcef215480`.
Both were installed locally with verified matching identities. The attached
project-only, no-evidence review prepared, opened for review without sink
dispatch, and remained unacknowledged; after closing and relaunching, neither
that prepared state nor its synthetic instruction was restored. Full
installed-host governed-review acceptance, including a separately authorized
local-model attempt, remains required; beta.81 is not a release-ready claim.

The unreleased `0.1.0-beta.82` replacement candidate from clean source commit
`a02f4d47cd8914be7c9b49713f0a93890a19fbe8` restores the opaque authorization
ID required to enable the exact one-use action after review acknowledgement.
Its pinned Ubuntu 22.04 package, lifecycle, smoke, and final artifact gates
passed, and the matching local pair was installed. One explicit project-only,
no-evidence local attempt then failed boundedly as `model-unavailable`; no
output or retry occurred. Closing and relaunching restored neither the review,
instruction, nor result. Beta.82 is immutable failed acceptance evidence and
not release-ready; a later retry needs a fresh unused beta version.

The `0.1.0-beta.83` candidate from clean source commit
`625638fa36aa698ca3c7709659a2b02944d34639` passed the pinned package,
lifecycle, smoke, final-artifact, and local installation identity gates. Its
dedicated supervised installed-host launcher carried the bound in-memory
contract through native revalidation, then the native loader failed before
output and without retry. Beta.83 is immutable failed acceptance evidence.

The `0.1.0-beta.84` candidate from clean source commit
`815385b8396b0bb14507d9d21a35655e5fd52bdc` passed the pinned package,
lifecycle, smoke, final-artifact, and local-installation identity gates. Its
dedicated supervised installed-host launcher failed closed during native
availability admission, before acknowledgement, bundle consumption, output, or
retry. The UI exposed no model location, content, loader output, or filesystem
observation. Beta.84 is immutable failed acceptance evidence.

The `0.1.0-beta.85` candidate from clean source commit
`5063882820e9094bd3ce4942b1b19711710c9eda` passed the pinned package,
lifecycle, smoke, final-artifact, local-installation identity, and desktop E2E
gates. Its supervised installed-host availability admission failed closed
before review acknowledgement, consumption, output, or retry. The original
bounded-memory category was not conclusive because it matched routine loader
status containing the word `memory`; beta.85 is immutable failed acceptance
evidence.

The `0.1.0-beta.86` candidate passed the pinned package, final-artifact,
staging, local-installation identity, and supervised-launch gates, but native
availability failed closed before review acknowledgement, consumption, output,
or retry. It is immutable failed acceptance evidence.

The beta.87 candidate passed package, installation, and supervised-launch
gates, but its cgroup admission lookup read the mount root and failed closed
before review acknowledgement, consumption, output, or retry. It is immutable
failed acceptance evidence.

The beta.88 candidate resolves the memory limit from the process's exact
cgroup v2 path. Its fresh pinned package passed the release-artifact gate,
trusted root-owned staging, daemon installation, supervised launch, and one
explicit governed local-only attempt. The evidence record remains content-free:
no model location, content, loader output, filesystem observation, or generated
response is retained. M48, M55, and M60 remain preserved.

The `0.1.0-beta.93` M69C Threads recovery candidate makes the chat-first path
discoverable from Threads and represents reconciliation misses honestly as
unavailable threads rather than fabricated untitled conversations. A stale
local reference remains intact and cannot open a substitute thread; no Codex
record, transcript, path, or app-owned metadata is deleted. Full validation,
package, installation, and supervised acceptance passed. The candidate passed
424 desktop tests, 455 Rust tests, package-contract checks, lint/format/build/
dist gates, 80 desktop E2E tests, and eight website E2E tests. Its pinned
Ubuntu package and release-artifact gates passed, followed by root-owned
staging, installer-daemon installation, supervised launch, and canonical
installed-host validation. The immutable receipt reports beta.93, two
artifacts, `passed`, and completed validation.

The `0.1.0-beta.94` Chat-first entry candidate redirects only the implicit
persisted Threads landing to Local Chat when the user has selected Chat mode.
An explicit Threads deep link remains a Threads request, and Codex-mode
navigation is unchanged. It preserves beta.93's unavailable-reference
representation and does not alter, recover, delete, or substitute any Codex
thread or transcript. Full validation, desktop and website E2E, the pinned
Ubuntu package/release-artifact gate, root-owned staging, installer-daemon
installation, supervised launch, and canonical installed-host validation
passed. The immutable receipt reports beta.94, two artifacts, `passed`, and
completed validation.

The `0.1.0-beta.95` candidate corrects the Local Chat/project mismatch without
creating ambient context transfer: Local Chat states that project context is
not attached, and its explicit linked-project action opens the pre-existing
managed Codex project conversation. The transient Local Chat transcript is not
transferred. Full validation, package, installation, and supervised acceptance
evidence is recorded only after the new candidate passes every gate.

The `0.1.0-beta.92` M69C Action Card candidate adds the accessible Local Chat
renderer to the already-accepted non-executing Action Card foundation. Its
closed Actions menu prepares only one of four visible proposal classes, and its
card approves or revokes only an opaque card ID. It explicitly states that no
project, source, artifact, code, provider, or tool data is selected and that
approval runs no action. It has no capability-specific receipt consumer. Full
validation passed: 424 desktop tests, 455 Rust tests, package-contract checks,
lint/format/build/dist gates, 80 desktop E2E tests, and eight website E2E
tests. The pinned Ubuntu package and release-artifact gates, root-owned staging,
installer-daemon installation, supervised launch, and canonical installed-host
validation passed. The immutable receipt reports beta.92, two artifacts,
`passed` installed-host state, and completed validation.

The `0.1.0-beta.91` M69C Action Card candidate completes the non-executing
native foundation: a closed prepare, approve, revoke, and opaque content-free
receipt lifecycle. Every card states that no data scope or execution is
authorized. It cannot attach a project, read/admit a source, claim/create an
artifact, run code, reserve the local runtime, or use a provider, filesystem,
terminal, Git, browser, connector, or network. `pnpm validate`, desktop/mobile
E2E, the pinned package/release-artifact gate, root-owned staging, daemon
installation, supervised launch, and canonical installed-host validation
passed. The immutable receipt reports beta.91, two artifacts, passed
installed-host state, and completed validation. Its renderer and all
capability-specific receipt consumers remain future work.

The `0.1.0-beta.90` M69B Threads-first candidate completed source, package,
staging, daemon-installation, supervised-launch, and canonical installed-host
validation. It combines
the M69A dedicated local-only IPC service and ephemeral Chat composer with a
presentation-only Threads-first shell. The thread tree uses existing bounded
thread/project labels, session-local unread state, and a closed fail-closed
status model; it does not fabricate pending decisions. Local Chat remains
no-project, source, artifact, ledger, provider, tool, filesystem, or credential
free, and direct local date/time questions use only the host clock. It reuses
only the bounded M63 primitive and does not alter M48, M55, or M60 governed
review. `pnpm validate`, desktop/mobile E2E, the pinned package/lifecycle/
visible-launch and release-artifact gates, root-owned staging, daemon
installation, and the supervised 6 GiB launch gate have passed. The initially
unavailable native receipt was resolved by a non-destructive relink of the
live attached project to the beta.90 candidate; the canonical headless gate
then returned `created`. Its immutable content-free record reports beta.90,
two artifacts, passed installed-host state, and completed validation. No
metadata was edited and no receipt was fabricated.

### Current provider-neutral completion record

This record supersedes the older historical candidate and draft-release
statements retained below. M55 Durable Source Admission is complete and
published as beta.59 at
`d6967e8bfd82acbef7dfa0dc74f085720f8b0384`. Its governed manual-text,
local-text-file, and reviewed-artifact-text paths remain private-copy,
project/task-bound admission only; they grant no retrieval, connector, context,
or provider authority.

M57 Fictional Connector Governance is complete and published as beta.60 at
`b8b807f256170e6a35ada22893b410cb4b0057b7`, tag
`v0.1.0-beta.60`. Migration 24 implements only the deterministic local-only
fictional connector lifecycle: separately declared/granted read and mutation
operations, digest-bound one-use review/confirmation, revocation, drift,
incompatibility, quarantine, ambiguity-without-retry, and content-free audit
linkage. It adds no real connector, credential, network, retrieval, browser,
MCP, automation, provider transmission, or external mutation authority.

M58 Controlled Browser Verification is complete and published as beta.62; its
contract is at
[MILESTONE_58_CONTROLLED_BROWSER_VERIFICATION_CONTRACT.md](MILESTONE_58_CONTROLLED_BROWSER_VERIFICATION_CONTRACT.md).
Its fictional/local-only, read-only implementation uses only a native-owned
ephemeral WebKitGTK custom-scheme fixture and passed installed-host acceptance;
it adds no real browser target,
profile/session reuse, credential, connector, provider, MCP, automation, or
external mutation authority.

M59 Context Assembly and Transmission Contract is complete decision-only work
at [MILESTONE_59_CONTEXT_ASSEMBLY_AND_TRANSMISSION_CONTRACT.md](MILESTONE_59_CONTEXT_ASSEMBLY_AND_TRANSMISSION_CONTRACT.md).
Its beta.63 M60 implementation is documented in
[MILESTONE_60_GOVERNED_CONTEXT_ASSEMBLY.md](MILESTONE_60_GOVERNED_CONTEXT_ASSEMBLY.md):
explicit source selection, deterministic bounded private assembly, structural
redaction, review, digest-bound one-use fictional local-only delivery, recovery,
and content-free audit linkage. It adds no provider, credential, network
transmission, inference, connector, browser, MCP, automation, or mutation.

M61 Credential Broker and Account Reference Contract is complete as an approved
decision-only milestone at
[MILESTONE_61_CREDENTIAL_BROKER_AND_ACCOUNT_REFERENCE_CONTRACT.md](MILESTONE_61_CREDENTIAL_BROKER_AND_ACCOUNT_REFERENCE_CONTRACT.md).
It ratifies future custody/runtime selection criteria, opaque scoped references,
lifecycle, audit, and adapter-compatibility gates only; it authorizes no
credential handling, provider/local-runtime connection, or successor
implementation.

M62 Limited Provider Inference Boundary is complete as an approved decision-only
milestone at
[MILESTONE_62_LIMITED_PROVIDER_INFERENCE_BOUNDARY.md](MILESTONE_62_LIMITED_PROVIDER_INFERENCE_BOUNDARY.md).
It ratifies only the future M60 bundle, M61 reference, destination/model
allowlist, typed-adapter, privacy, lifecycle, and fail-closed gates for a later
limited-inference proposal; it authorizes no provider or local-runtime activity.

M63's approved credential-free local-runtime direction is limited to an
in-process, CPU-only llama.cpp static-library build for the fixed Qwen2.5-3B
descriptor. The separately approved offline acquisition is recorded with its
upstream revision and SHA-256 in
[MILESTONE_63_IN_PROCESS_LOCAL_RUNTIME_ADAPTER.md](MILESTONE_63_IN_PROCESS_LOCAL_RUNTIME_ADAPTER.md).
The model remains outside the repository and un-packaged. The clean
`0.1.0-beta.64` Ubuntu candidate passed package/lifecycle/visible-launch
validation; its local-only adapter consumes one confirmed reviewed bundle once
and retains a bounded result only in the open local view. A clean-tree
`0.1.0-beta.65` Debian pair remains prior package evidence from source commit
`6f77d0c4d73cefe5ed335898830872ca63ad203a`. The clean-tree
`0.1.0-beta.66` Debian pair from source commit
`822b6703968f4cea95ce4828f130739bc56e8a01` passed the authoritative pinned
Ubuntu 22.04 package, lifecycle, visible-launch, and release-artifact gates.
It excludes the model and did not start the runtime. The focused host-native
adapter gate subsequently completed one bounded local-only attempt through the
supervisor-owned read-only model contract, retaining no model location or
generated output. The clean-tree `0.1.0-beta.67` Debian pair from source commit
`8f604e3b98394b8ba8d5170c82818f357d5d5a11` subsequently passed the
authoritative pinned Ubuntu 22.04 package, lifecycle, visible-launch, and
release-artifact gates. It excludes the model and did not start the runtime.
The repeated clean-source beta.67 package gate on 2026-08-13 reproduced those
results, again excluding the model and never starting the runtime.
Installed Debian desktop acceptance of the explicit governed review flow
remains required before any release-ready claim.

The beta.68 M63 candidate extends the governed-review browser fixture through
an application reload, proving that an open-view-only completed result is not
restored after relaunch. Its clean-tree Debian pair from source commit
`5c4ca198f94553dd760f20734f765c8abb5a488e` passed the authoritative pinned
Ubuntu 22.04 package, lifecycle, visible-launch, and release-artifact gates;
the model stays external and is never started by package gates. Installed
Debian desktop acceptance remains pending.

The beta.69 M63 candidate keeps the explicit one-time local action disabled
until the content-free local-runtime availability preflight completes and
shows its checking state in the governed review. This preserves native
authority and leaves the model external; package gates do not start it.
Installed Debian desktop acceptance remains pending.

The beta.70 M63 candidate resolves an IPC-level availability-preflight failure
to a bounded unavailable state in the governed review. It leaves the one-time
action disabled and the acknowledged bundle unconsumed; package promotion and
installed Debian desktop acceptance remain pending.

The beta.71 M63 candidate lets the open governed review explicitly recheck its
content-free local-runtime availability after an unavailable result. Rechecking
preserves the acknowledged bundle and cannot start an attempt; the one-time
action stays disabled unless native availability succeeds. Package promotion
and installed Debian desktop acceptance remain pending.

The beta.72 M63 candidate visibly confirms a successful content-free,
local-only availability recheck while preserving the separate explicit
one-time reviewed action. Its governed desktop fixture covers that recovery
flow. Package promotion and installed Debian desktop acceptance remain pending.

The beta.73 M63 candidate recovers the exact acknowledged review when native
revalidation finds the local model unavailable after a successful preflight.
It makes clear that no attempt started, requires an explicit availability
recheck before a later manual run, and keeps the model external. Package
promotion and installed Debian desktop acceptance remain pending.

M54 Local Review is complete at package/source commit
`c4c2752466f36f791fde47edbc5c6b02b0e21320`, tagged
`v0.1.0-beta.53`. Its beta.53 Debian pair passed pinned-container package,
lifecycle, smoke, visible-launch, and restricted installed-host validation;
headless completion returned `created`, then `existing`, and beta.53 is
installed. All seven evidence captures use closed native authority and canonical
persisted-bytes-only previews. Migration 19 is the native Activity ownership
ledger; migration 20 immutably binds Advisor-dispatched tasks. Beta.52 remains
preserved as an unreleased failed installed-host candidate.

M55 Durable Source Admission is complete and published as
`0.1.0-beta.59` / `0.1.0~beta.59`, tagged `v0.1.0-beta.59` at
`d6967e8bfd82acbef7dfa0dc74f085720f8b0384`. It admits only governed manual
text, one selected local UTF-8 file, and eligible reviewed artifact text as
private project/task-bound copies. Beta.59 corrects the installed-host
chooser/GVFS fail-safe: cancellation, unavailability, timeout, or ambiguity
returns bounded control and creates neither a durable source nor a private copy.
M55 does not authorize retrieval, connector access, context inclusion, or
provider transmission. M56 is complete at
package/source commit `e2b084ed0bdf17fb6f4b0b47663cdf6952ec8e73`, annotated tag
`v0.1.0-beta.54`, and a `James-Jennison/quireforge` draft prerelease. Migration
21 owns local templates,
migration 22 owns bounded digest-only application reservations, and four static
built-ins remain outside SQLite. The closed lifecycle, strict bridge, lazy
management UI, digest-bound application workflow, accessibility, and focused
browser acceptance passed source acceptance; canonical pinned-Ubuntu-22.04
packaging and restricted installed-host validation also passed.

The draft prerelease contains exactly the canonical beta.54 Debian pair,
`SHA256SUMS`, and `release-manifest.json`. The application package is 5,864,924
bytes with SHA-256
`643e6bc3caf9068f7ed521ecd949f9f3f5d38b9c6a82bcce19384370f644d131`; the sandbox
package is 3,233,492 bytes with SHA-256
`bd9c0682c0e9dd7761b28f03eb2e801ab7a925e7c5f5587eefc68bd7578bd21f`. Both
`0.1.0~beta.54` packages remain installed; headless completion returned
`created`, then `existing`, with no rollback. The draft is not published and
there has been no deployment. Beta.53 remains preserved as the prior released
rollback generation, and beta.52 remains an unreleased failed installed-host
candidate. Research, retrieval, providers, connectors, MCP, OAuth, browser
authority, credentials, import/export, hidden instructions, automatic actions,
approval, dispatch, and execution remain excluded. M57 connector governance is
now complete as a decision-only record: it preserves the existing
Codex-owned Integration Center boundary, grants no new external authority, and
requires further bounded decisions before any real connector implementation.
This beta.54-era account is historical: M58 later completed its separate
fictional/local-only implementation and published beta.62.

The post-M57 prerequisite decision checkpoint is complete. It approves only a
future, unstarted local mock-only connector foundation: static non-executable
descriptors and opaque native lifecycle/binding/operation/audit contracts may
be proposed, while network calls, real credentials, OAuth, providers, browser
authority, external mutation, background activity, generic MCP execution, and
M55 source-manifest authority remain deferred. It does not change the existing
Codex-owned Integration Center boundary or beta.54 release state.

The M57 local mock-only connector foundation remains preserved source-only
evidence at `a1d407469626e34cd5d4921abdb6c8d305895d7e`. Its later ratified
[M57 Connector Governance Contract](MILESTONE_57_CONNECTOR_GOVERNANCE_CONTRACT.md)
was implemented and published as beta.60 through migration 24: bounded
project/task connector binding, declared-versus-granted read/mutation
capabilities, deterministic local outcomes, one-use review/confirmation,
revocation, drift, incompatibility, quarantine, ambiguity without retry, and
content-free audit linkage. It grants no real connector, credential, network,
MCP dispatch, browser/M58 runtime, automation, retrieval, context, provider
transmission, or external mutation.

The **External Capability Taxonomy and Sequencing** decision checkpoint is
complete. It grants no implementation authority, separates inference,
retrieval, connected services, local runtimes, execution, credential, browser,
and automation lanes, and records their dependency ordering. Its earlier
capability-registry recommendation is complete. M58 later completed its
independent fictional/local-only runtime as published beta.62.

The **Provider-Neutral Capability Registry and Descriptor Governance** decision
is complete as a non-authorizing architecture artifact. It defines descriptor
entities, opaque identity, provenance, capability claims, lifecycle, extensions,
and authority separation; it grants no provider implementation or authority.
Canonical Provider-Neutral Interaction and Event Protocol is complete. M58
later completed its independent fictional/local-only runtime as published
beta.62.

The **Provider-Neutral AI Foundation** is an active long-term product goal. Its
taxonomy, capability-registry, interaction-protocol, adapter-governance,
credential-custody, context-manifest, and Limited Provider Inference Boundary
decisions are completed gates. M55, M57, and M58 remain separate; beta.62 is
the latest published generation, and M59 is the completed context/transmission
decision contract.

The first Provider-Neutral AI Foundation implementation milestone is
source-complete: private native capability-registry contracts provide static
fictional descriptors, closed claims, deterministic digest/version validation,
Serde unknown-field denial, endpoint-aware model identity, focused tests, and a
narrow repository safeguard. It has no persistence, Tauri/bridge/UI exposure,
provider route, or package. Canonical Interaction/Event Contracts and
Deterministic Mock Adapter Conformance remains historical source-complete
evidence; M60 is the next roadmap boundary, while M55, M57, and M58 remain
separate.

The second Provider-Neutral AI Foundation implementation milestone is
source-complete: private native interaction/event contracts bind fictional
attempts and canonical envelopes to opaque project/task/provider/endpoint/model/
adapter/protocol identities. Strict sequencing, lifecycle, reference, extension,
tool-result, citation, structured-output, usage, and deterministic mock-adapter
checks remain local and unexposed. It has no persistence, Tauri/bridge/UI
surface, provider route, or package. Credential Broker Foundation Contracts is
historical source-complete evidence; M60 is the next roadmap boundary, and M55,
M57, and M58 remain separate.

The third Provider-Neutral AI Foundation implementation milestone is
source-complete: the [local mock inference vertical
slice](MILESTONE_PROVIDER_NEUTRAL_MOCK_INFERENCE_VERTICAL_SLICE.md) is a
user-visible but fictional/in-memory task-bound workflow. It uses an explicit
bounded authored-text manifest review, inert opaque lease, digest-bound one-use
authorization, explicit one-event polling, deterministic canonical event
fixtures, and content-free evidence. Its destination is a validated static
registry projection; timeout is distinct from interruption; and the UI clears a
review when its task, destination, or input changes. It introduces no real
provider, credentials, network route, context transmission, retrieval, native
tool authority, persistence, or external action. Existing release policy leaves
it source-only: beta.54 and its draft prerelease remain unchanged. M55, M57,
and M58 remain separate.

The source-complete [mock workflow hardening milestone](MILESTONE_PROVIDER_NEUTRAL_MOCK_WORKFLOW_HARDENING.md)
keeps that boundary intact while exposing the workbench on the live QuireForge
conversation surface. It projects distinct fictional Lantern and Ember
destinations from the private registry, makes cancellation request and
confirmation separately observable, and requires a fresh review after local
authority failure or state loss. Its report distinguishes exhaustive native and
component evidence from representative browser acceptance.

Beta.55 passed source, packaging, installation, and native-receipt gates, but
is release-ineligible because the installed New task workspace omitted the
required Task Catalog/New task UI. Its packages, checksums, manifest, receipt,
and logs remain immutable failed-candidate evidence. Beta.56 then proved
release-ineligible at installed-host acceptance because its Task Catalog created
an unbound default task. The authorized `0.1.0-beta.57` replacement candidate
adds only explicit named, project-bound durable task creation to the local mock
workbench path. It does not change the beta.54 rollback baseline or imply a
real provider, credential, network, retrieval, browser, connector, tool, or
external authority.

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

## Release status

M60's beta.63 release is complete and published. It remains limited to the ratified
fictional/local-only sink; no provider, credential, network transmission,
inference, connector, browser, MCP, automation, external mutation, deployment,
or later milestone is implied.

M61 and M62 are complete as decision-only contracts. Routine reversible, local,
non-production post-M62 implementation is autonomous: Codex may inspect,
implement, test, commit, and push the highest-value safe task. Credentials or
account/browser access, production deployment, public release, destructive
actions, third-party commitments, and genuinely irreversible product-direction
decisions remain human-only stops.

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
source-closed at `8900995ad3645ebc5a95c6959de8be4e75f24ae8`; package and
installed-host closure passed at `c4c2752466f36f791fde47edbc5c6b02b0e21320`
for beta.53. Beta.52 remains an unreleased failed installed-host candidate.
Beta.48
is a validated but non-deployable candidate because the installed host still
lacked the migration-18 unprivileged receipt bootstrap. Beta.49 is a preserved,
validated but non-promoted candidate: its promotion correctly failed on an
order-sensitive finalizer checksum check. Beta.50 was source-bound and passed
build/smoke, but remained non-promoted because the separate release validator
retained that order-sensitive comparison. Beta.51 centralizes checksum semantics
across release generation, finalization, and validation. The beta.47 normalized
canonical set was
lost by the former finalizer; only its recorded hashes and distinct raw Tauri
bundle remain, and no replacement beta.47 evidence was fabricated. Restricted installed-host
validation now derives the installed version from the compiled executable and
binds the installed-host phase to the current unprivileged receipt. Beta.48,
beta.49, beta.50, beta.51, and beta.52 histories remain preserved without
fabricating beta.47 evidence. The beta.53 four-asset draft prerelease is
byte-identical to the canonical set; no publication or deployment occurred. See
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
