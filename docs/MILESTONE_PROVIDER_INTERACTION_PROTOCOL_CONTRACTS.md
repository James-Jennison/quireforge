# Provider Interaction/Event Contracts and Deterministic Mock Adapter Conformance

Status: source-complete implementation milestone within the active
[Provider-Neutral AI Foundation](GOAL_PROVIDER_NEUTRAL_AI_FOUNDATION.md). This
milestone adds private, local, fictional interaction contracts and deterministic
fixture translation only. It has no route to a provider, network, credential,
context transmission, persistence, command, bridge, UI, package, or release.

## Implemented boundary

`apps/desktop/src-tauri/src/provider_interaction_protocol.rs` is private to the
native desktop crate. It defines strict, versioned interaction-attempt bindings
for opaque project, durable-task, provider, endpoint, model, adapter, protocol,
capability-profile, and account-class identities. Provider-owned session,
continuation, response, run, file, cache, realtime-session, and batch-job
references stay opaque and are scoped to the exact binding.

Canonical Serde contracts model ordered envelopes, correlations, bounded text,
structured values, projected media/document/artifact/citation references,
stream deltas, replacements, usage units, tool proposals/results, citation and
grounding metadata, terminal states, failures, and namespaced fictional
extensions. SHA-256
digests, UUIDv7 identities, strict unknown-field rejection, exact sequencing,
and closed lifecycle transitions fail closed. Cancellation requested and
confirmed remain distinct; terminal attempts cannot stream again; transport
closure and timeout remain ambiguous/non-success outcomes rather than proof of
no submission.

The nested deterministic mock adapter accepts only predeclared fictional
registry identities. It maps a bounded canonical input fixture into an inert
fictional representation and translates fictional fixture events back into
canonical envelopes. It validates protocol and adapter compatibility, preserves
approved namespaced extensions, and classifies fictional outcomes without any
transport or side effect.

Tool proposals are untrusted communication events, not native operation
identifiers. Tool-result receipts are fictional and bounded; ambiguous receipts
cannot claim success. Citation and grounding events remain explicitly
unadmitted/provider-claimed metadata and create no M55 durable source authority.
Usage preserves raw fictional provider units and whether they are reported or
estimated.

## Safeguards and evidence

Focused Rust tests cover digest determinism, strict parsing, duplicate and
out-of-order rejection, terminal lifecycle closure, cancellation/timeout/
transport distinctions, recovery-path identity, provider-reference scoping,
tool and citation confinement, structured/usage states, extension containment,
adapter-aware event identity, and deterministic fixture translation.

`scripts/validate_repository.py` adds a separate narrow guard for this module.
It rejects process, network, socket, environment, filesystem, Tauri/bridge,
persistence, native-dispatch, browser/MCP, and known vendor authority markers
while requiring the static attempt, adapter, envelope-validation, transition,
and strict-Serde markers. Existing M57 and capability-registry guards remain
unchanged.

## Persistence, exposure, and release policy

There is no SQLite migration, persistence, Tauri command, TypeScript/Zod
bridge, or UI. The existing M57 LocalMock connector foundation and the first
capability-registry implementation remain unchanged. Beta.54 remains the latest
packaged generation. This is a source-only, unexposed milestone; version
selection, packaging, installed-host validation, tag, release, publication, and
deployment remain separate future decisions.

## Deferred boundaries

No real provider or vendor, network call, model invocation, inference,
credential custody, account connection, context assembly/transmission,
retrieval, citation admission, indexing, native tool mapping/execution, shell,
terminal, filesystem, Git, browser/M58, connector, MCP, cloud/deployment,
mutation, automation, or multi-agent behavior exists here.

## Next recommendation

The next separately approved implementation milestone is **Credential Broker
Foundation Contracts**. It must first remain local and contract-only, with no
secret custody integration, account connection, provider invocation, context
transmission, or native authority.
