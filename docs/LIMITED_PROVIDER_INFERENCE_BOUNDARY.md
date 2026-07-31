# Limited Provider Inference Boundary

Status: complete decision-only architecture gate within the active
[Provider-Neutral AI Foundation](GOAL_PROVIDER_NEUTRAL_AI_FOUNDATION.md). This
record selects no provider, model, endpoint, adapter, credential custodian, or
local runtime and approves no implementation, network access, credential
resolution, context transmission, model invocation, persistence, bridge, or UI.
M55, M57, and M58 remain separate; beta.54 remains the latest packaged
generation.

## Purpose and minimum future authority

This decision defines the smallest future authority under which QuireForge could
invoke one separately approved provider or local runtime. That future route is a
bounded, user-visible, native-owned interaction submission—not a general chat
integration, provider session product, connector, background service, or tool
authority.

QuireForge projects and durable tasks remain authoritative. A provider session,
thread, request, stream, job, response ID, or continuation reference is opaque
and subordinate to a native interaction attempt. Provider adapters translate
intelligence traffic only; they do not select context, resolve credentials,
approve submission, own task state, or execute operations.

This document does not authorize a submission. A later explicitly approved
implementation milestone must select one route, prove every boundary below, and
receive its own implementation/release authority before any provider or runtime
is invoked.

## Exact attempt binding and pre-submission proof

Every future inference submission must bind one exact, native-issued:

- QuireForge project and durable task, including current ownership/version where
  the interaction is task-owned;
- canonical interaction attempt, attempt version, and attempt digest;
- provider, endpoint/deployment, model, and adapter identities, releases, and
  digests;
- compatible adapter manifest and canonical capability profile, including
  limits, support state, and governed extensions;
- privacy-preserving organization/account binding, credential-reference, and
  short-lived least-authority lease;
- context-transmission manifest, exact item/transformation/exclusion set,
  destination classification, expiry, authorization state, and digest;
- closed operation class of **limited inference** and bounded user-visible
  submission intent; and
- native policy version, request idempotency classification, cost/usage guard
  state, issuance time, and expiry.

Immediately before any future submission, native code must revalidate every
binding above: project/task ownership and state; descriptor/model/endpoint and
adapter compatibility; capability profile; account/project/scope binding;
credential-reference and lease lifecycle; context-manifest digest, item
freshness, transformations, exclusions, destination and expiry; policy;
emergency-stop state; and exact user-visible intent. Unknown, stale, revoked,
degraded, quarantined, incompatible, expired, mismatched, broadened, reordered,
or partially revalidated facts fail closed. The result is a new reviewable
attempt or no submission—not a repair, fallback, context expansion, silent
model change, credential substitution, or automatic retry.

## Initial scope and context limits

The initial future inference route may accept only explicitly authorized text
and already-approved bounded local projections named by the context manifest.
It does not include retrieval, search, provider-managed file lifecycle, source
admission, citations as verified evidence, repository scanning, indexing,
ambient sessions, browser state, attachments by implication, or arbitrary
multimodal transfer. Any future image, audio, video, document, structured, or
other modality needs its own approved capability/profile, projection,
transformation, destination, and manifest review.

M55 remains the authority for durable source identity, admission, provenance,
retention, and citation mapping. Retrieved but unadmitted content cannot enter
the initial route, directly or through a summary. Explicitly selected transient
local input remains distinct from durable local artifacts, opaque provider
references, and future M55-admitted external sources.

## Submission, acknowledgement, and event lifecycle

A future submission has separate native-correlated facts:

1. **Prepared** — prerequisites and user-visible preview exist; no dispatch.
2. **Authorized for one submission** — the exact attempt/manifest/lease binding
   passed immediate revalidation; still no provider claim of receipt.
3. **Submission acknowledgement** — an adapter translates only an unambiguous,
   safe provider or local-runtime acknowledgement into the canonical protocol.
4. **Receiving** — ordered canonical events may be translated under the
   interaction-protocol rules.
5. **Terminal** — succeeded, refused, failed, cancelled, interrupted, timed
   out, outcome-unknown, degraded, or quarantined.

An acknowledgement is not a completed response, a retention guarantee, a cost
settlement, or an authorization for another request. A provider timeout,
transport interruption, missing acknowledgement, or ambiguous reply does not
prove no work occurred. It produces a closed outcome-unknown or appropriate
terminal classification rather than an inferred failure/success.

Streaming, usage, structured output, multimodal events, provider grounding,
citations, tool proposals/results, errors, cancellation, continuation, and
terminal events use the canonical interaction/event protocol. The adapter must
preserve ordering, replacement-versus-delta behavior, opaque session reference,
and safe error classification. It cannot turn an event into context selection,
credential authority, a native operation, a connected-service action, or a
provider dispatch for another attempt.

## Cancellation, timeout, interruption, and continuation

Cancellation is an explicit native request against one active attempt. A cancel
request, a provider/local-runtime acknowledgement, and a terminal cancelled
outcome are separate facts. If the eventual external state is ambiguous, the
attempt remains outcome-unknown rather than repaired by a new submission.

Timeout and interruption preserve the same distinction: they stop local waiting
or mark a route unavailable; they do not establish that the provider did no work
or incurred no usage. A later approved user action may create a fresh attempt
after review, but it cannot silently reuse a prior submission binding.

Continuation is a new native interaction attempt with an explicit predecessor
reference. It requires fresh task/project, descriptor/adapter, account/lease,
context-manifest, policy, expiry, and user-intent validation. A provider
continuation/session reference never reauthorizes prior context, credentials,
model selection, tool proposals, output retention, or transmission. Changed
context always requires a new manifest.

## Idempotency, retry, and regeneration

Every future route must declare whether it has reliable provider/local-runtime
idempotency evidence for the exact operation and target. Where reliable
idempotency is absent, an ambiguous acknowledgement, timeout, interruption,
transport failure, or unknown cost/outcome prohibits automatic retry. Provider
timeout is never evidence that work did not occur.

A retry is a new explicitly reviewed interaction attempt, not a replay of the
old one. A regeneration is likewise a distinct attempt, even if user intent,
model, and selected context appear identical. Each receives a new canonical
attempt ID, manifest authorization, credential lease, expiry, cost guard review,
and audit relation to its predecessor. Neither may overwrite the original
outcome or hide a duplicate provider-side result.

Reliable idempotency support, if a future chosen route provides it, may prevent
duplicate submission only after native policy binds its key or equivalent to the
exact provider/endpoint/account/attempt/manifest/operation. It does not create
standing retry authority, context reuse, or automatic regeneration.

## Output handling and proposal boundary

Provider outputs are untrusted provider claims represented through the canonical
event protocol. A later implementation may show bounded output projections or
create proposals or bounded artifacts only under separately approved local
artifact, review, retention, and admission policies. Output never automatically
modifies a task, plan, project, template, artifact, review, credential, account,
context manifest, or provider binding.

Tool proposals and tool-result continuations are communication events only.
They cannot execute native tools, shell commands, terminals, Git, filesystem
actions, browser activity, MCP, connectors, cloud actions, approval, dispatch,
or mutation. A proposed action needs the relevant existing or future closed
native-operation route and its own revalidation/confirmation before it could
ever occur.

Provider citations, grounding metadata, provider-managed retrieval, and
provider-managed file references remain unverified observations. They do not
admit a source under M55 or authorize retrieval, retention, citation, or report
generation.

## Provider policy, privacy, and usage disclosures

Before a real provider is selected for a future implementation, its specific
route must have inspectable, current, destination-aware disclosures covering at
least provider-side retention, logging, training or secondary-use policy, data
residency/processing location where material, account/organization terms,
endpoint/deployment distinctions, incident/deletion implications, supported
security characteristics, and applicable content restrictions. Unknown,
contradictory, stale, or non-reviewable policy makes the route unavailable.

Usage and spending guardrails are future native policy inputs, not provider
promises. A future implementation must preserve raw provider-specific usage
units, measurement scope, observation time, confidence, and any attribution
limitations. It must define bounded per-attempt, project, account, and time
window limits; user-visible warnings; stop conditions; and behavior when usage
or cost is delayed or unknown. This decision selects no price, currency, budget,
account plan, vendor, or billing integration. Unknown cost cannot authorize
unbounded use.

## Revocation and emergency stop

An emergency stop is a native policy state that blocks new preparation,
authorization, submission, continuation, and retry for the affected route. It
may be triggered by credential/account revocation, lease expiry, adapter or
descriptor incompatibility/revocation/quarantine, provider-policy drift,
security incident, project removal, manifest invalidation, or a later
user-visible stop action. It must be scoped and auditable without storing
secrets or raw context.

For an in-flight attempt, native code must request cancellation where a future
approved route supports it, then preserve the actual known state: cancellation
requested, cancelled, interrupted, timed out, failed, or outcome-unknown. It
cannot claim remote cancellation, erase a provider result, or retry/continue
automatically. Credential, account, adapter, descriptor, or manifest revocation
invalidates all dependent future submissions and continuations immediately.

## Local-runtime variant

A local runtime is an intelligence-traffic destination with separately bound
runtime, endpoint/deployment, model, adapter, capability, context-manifest, and
policy identities. It may not require a cloud credential, but still requires a
project/task binding, explicit context authorization, immediate revalidation,
privacy/residency disclosure, usage/resource guard, cancellation/outcome
handling, and the same no-ambient-authority rules.

Local-runtime classification grants no filesystem, process, terminal, shell,
Git, sandbox-worker, socket, device, browser, environment, or host authority.
Runtime discovery, launching, process management, loopback access, resource
allocation, and local endpoint authentication remain separate future decisions.

## Relationship to completed gates and separate lanes

The [capability registry](PROVIDER_NEUTRAL_CAPABILITY_REGISTRY_AND_DESCRIPTOR_GOVERNANCE.md)
governs descriptor/capability identity and claims, not availability or dispatch.
The [canonical protocol](CANONICAL_PROVIDER_NEUTRAL_INTERACTION_AND_EVENT_PROTOCOL.md)
governs attempts/events, not invocation permission. [Adapter governance](PROVIDER_ADAPTER_LIFECYCLE_AND_CONFORMANCE_GOVERNANCE.md)
keeps adapters translation-only. [Credential custody](CREDENTIAL_BROKER_AND_ACCOUNT_PROJECT_SCOPE_CUSTODY.md)
governs opaque references and leases, not context or invocation. [Context
manifests](CONTEXT_ASSEMBLY_AND_TRANSMISSION_MANIFESTS.md) govern exact selected
projections and authorization, not transmission or provider request creation.

M57 remains reusable evidence for least-authority binding, lifecycle, revocation,
digest checks, replay resistance, one-time confirmation, and content-free audit;
it does not create a real connector or connected-service route. M55 remains
separate for durable source admission/research. M58 remains unstarted and
separate; this decision grants no browser session, cookie, DOM, screenshot,
download, form-submission, OAuth, automation, or web research authority.

## Implementation and release boundary

No current source contract implements this decision. A future **Implementation
Readiness and First Milestone Selection** checkpoint must select whether any
implementation is warranted, then define one bounded route, one authority
surface, deterministic fixtures, adverse-case tests, strict native/bridge
contracts, persistence decision, UI/accessible review needs, source-acceptance
requirements, and any package/release policy. It is not an automatic
implementation authorization and must not select a provider merely by starting.

## Explicit exclusions

This decision grants no provider/vendor selection; provider adapter
implementation; model invocation; network access; credential implementation or
storage; context transmission; retrieval, citations, indexing, or source
admission; native tool execution; browser behavior; connector or MCP
implementation; persistence schema; Rust or TypeScript contracts; Tauri
commands; bridge; UI; package; version; release; tag; installation; host change;
deployment; background work; automation; external mutation; or multi-agent
behavior.
