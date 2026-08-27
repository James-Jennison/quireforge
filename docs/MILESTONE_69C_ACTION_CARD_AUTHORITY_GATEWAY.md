# M69C — Action Card Authority Gateway

## Purpose

M69C establishes the one native-owned grammar by which a chat thread may ask
for a later capability. An Action Card is a visible proposal, not a hidden
context transfer and not an executor. Its initial vertical slices are strictly
non-executing.

## Closed initial contract

The native `ActionCardService` accepts exactly one closed action class:

1. `attach-project`;
2. `use-source`;
3. `draft-artifact`; or
4. `work-with-code`.

The prepare request contains only that enum. An approval or revocation request
contains only an opaque card ID. The snapshot contains only the opaque card ID,
closed action/state, expiry, and an opaque receipt ID after approval. It states
`dataScope: none` and `execution: not-authorized` on every lifecycle state.

No request or result type may contain user text, a path, project identity,
source/artifact reference, reviewed context, provider, credential, model,
tool, terminal/Git argument, or execution instruction. Unknown JSON fields and
unknown enum values fail closed.

## Lifecycle

`prepared` → `approved` or `revoked`; a prepared card may become `expired`.
All terminal states reject further decisions. Approval issues one opaque,
process-local, content-free receipt and performs no action. Revocation and
expiry issue no receipt. Cards are bounded in count and expire after five
minutes.

This receipt is deliberately not a project/source or execution authorization.
A later capability-specific native service must define its own compatible
receipt-consumption rule before it may attach a project, admit a source, create
an artifact, or expose code work.

## Boundary preservation

- **M48:** Action Cards neither claim nor save generated artifacts.
- **M55:** they neither select, read, copy, retain, nor admit a source.
- **M60:** they neither assemble, acknowledge, transmit, nor execute reviewed
  context.
- **M63:** they do not reserve or run the local runtime.

The service has no filesystem, network, process-launch, provider, browser,
connector, terminal, Git, or database dependency.

## Required evidence

1. Rust tests prove approval is one-use, revocation is terminal, and unknown
   request fields are rejected.
2. TypeScript/Zod tests prove the bridge rejects paths, prompts, and any
   snapshot that claims data scope or execution.
3. The full validation, desktop/browser E2E, pinned package gate, trusted
   staging, daemon install, and supervised installed-host acceptance run on a
   fresh candidate before M69C is claimed complete.

## Candidate implementation

The beta.91 candidate added the native service, strict Tauri commands, and
TypeScript bridge contracts. The beta.92 candidate adds the accessible Local
Chat renderer: a closed Actions menu can prepare one of the four visible
proposals, and the card can approve or revoke the same opaque lifecycle. The
renderer explicitly states that no project, source, artifact, code, provider,
or tool data is selected or used, and that approval runs no action. It does not
add a receipt consumer or any data/execution capability.

The beta.92 candidate passed full validation (424 desktop tests, 455 Rust
tests, package-contract checks, lint/format/build/dist gates), 80 desktop E2E
tests, and eight website E2E tests. Its fresh pinned Ubuntu package passed the
release-artifact gate and was installed through root-owned staging and the
installer daemon. The supervised installed-host launcher was active, running,
and successful under the fixed 6 GiB boundary; the canonical immutable receipt
records beta.92, two artifacts, `passed`, and completed validation.

The beta.93 candidate makes this normal Chat path discoverable from the
Threads-first shell. It also labels a reconciliation miss as an unavailable
thread instead of inventing an untitled conversation. It neither changes nor
deletes a local reference, and it does not create a substitute Codex thread.
The beta.93 candidate passed full validation, desktop and website E2E, the
pinned Ubuntu package/release-artifact gate, root-owned staging,
installer-daemon installation, supervised launch, and canonical installed-host
validation. Its immutable receipt records beta.93, two artifacts, `passed`,
and completed validation.
