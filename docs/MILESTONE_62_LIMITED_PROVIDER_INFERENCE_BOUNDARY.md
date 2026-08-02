# M62 — Limited Provider Inference Boundary

Status: ratified decision-only contract within the active
[Provider-Neutral AI Foundation](GOAL_PROVIDER_NEUTRAL_AI_FOUNDATION.md). M62
defines the conditions for a future limited-inference proposal; it neither
implements nor enables an adapter, provider, local runtime, model, account,
credential, or inference route. It supersedes the M62 planning role in
[Limited Provider Inference Boundary](LIMITED_PROVIDER_INFERENCE_BOUNDARY.md)
and the [Provider-Neutral Foundation completion sequence](PROVIDER_NEUTRAL_FOUNDATION_COMPLETION_SEQUENCE.md)
where they differ.

M60 / `v0.1.0-beta.63` remains the latest completed published slice. M61
remains the decision-only custody/reference gate. M62 adds no runtime, package,
release, or user-visible behavior.

## Intended future boundary

A future inference route is one explicitly reviewed, native-owned, task-bound
submission. It is not a general-purpose assistant integration, unrestricted
model access, provider-session product, background service, connector, or
native-operation authority. Projects and durable tasks remain authoritative;
provider/local-runtime sessions, requests, streams, jobs, responses, and
continuations are opaque subordinate references.

The only eligible future input is an immutable M60 prepared bundle whose exact
digest, item set, transformations, exclusions, target classification, review,
one-use authorization, expiry, and audit linkage remain valid. An adapter may
receive only the canonical bytes and narrow typed metadata that a separately
approved route binds to that exact attempt. It cannot receive unreviewed project
content, hidden transcript/history, filesystem state, environment values,
browser state, credentials, account identity, provider defaults, or an
unbounded prompt.

M60 selection is not inference permission. M61 reference availability is not
inference permission. A future inference authorization requires each gate to
remain valid immediately before one submission.

## Future authorization gates

Before any implementation or onboarding proposal, a new owner approval must
name one prospective route and establish all of the following:

| Gate | Required proof |
| --- | --- |
| Destination selection | A provider or credential-free local-runtime class satisfies the closed descriptor, provenance, compatibility, privacy, residency, retention, and failure criteria; M62 selects none. |
| Model and capability allowlist | Native policy binds a finite model/capability profile, limits, supported operation class, adapter manifest digest, endpoint/deployment class where relevant, and governed extensions. No ambient/default model is eligible. |
| M60 context | One immutable prepared bundle has a matching project/task, destination class, review, expiring one-use authorization, and no drift, revocation, or redaction/policy change. |
| M61 account prerequisite | Where authentication is required, an opaque account/reference and short-lived lease are valid, compatible, scoped, non-exportable, and freshly revalidated. Credential-free local runtimes must instead prove the same project/task, policy, and no-ambient-authority constraints. |
| Attempt authorization | A canonical interaction-attempt digest, user-visible purpose, policy version, expiry, emergency-stop state, usage/cost guard state, and idempotency classification are all current. |
| Owner gates | Separate approvals are required for implementation planning, any provider or local-runtime onboarding, package/release work, and every external or production mutation. |

Unknown, stale, revoked, expired, quarantined, incompatible, broadened,
ambiguous, mismatched, or partially revalidated facts fail closed. They cannot
select a fallback model, substitute a credential, expand context, reconnect a
runtime, or retry a submission.

## Payload, privacy, and lifecycle requirements

M60's canonical item, byte, redaction, and expiry bounds are the upper boundary
for any future payload. A later route may only impose stricter fixed limits; it
cannot add an implicit source, modality, attachment, transcript, retrieval
result, provider-managed file, or hidden instruction. Invalid encoding, missing
bytes, redaction uncertainty, excess size, policy drift, or mismatched digest
blocks preparation and submission.

Future lifecycle states must remain distinct: prepared; authorized for one
submission; acknowledgement observed; receiving ordered canonical events; and
terminal succeeded, refused, failed, cancelled, interrupted, timed out,
outcome-unknown, degraded, or quarantined. An acknowledgement is not a response,
cost settlement, retention guarantee, or new authorization.

Cancellation is a native request for one attempt; request, acknowledgement, and
terminal outcome are separate facts. Timeout, interruption, missing
acknowledgement, or ambiguous completion never proves that no provider/local
runtime work occurred. It closes as outcome-unknown or another named terminal
state and prohibits automatic retry. Continuation, retry, and regeneration each
require a new reviewed attempt, fresh M60 authorization, M61 prerequisites when
applicable, expiry, policy check, and content-free audit relation.

Revocation of a project/task, prepared bundle, transmission authorization,
account reference, lease, descriptor, adapter, model profile, or policy blocks
future submissions immediately. Restart recovery begins with no active
submission authority unless a later approved route can prove a terminal state.
Emergency stop blocks preparation, authorization, submission, continuation, and
retry for its exact scope without claiming remote cancellation it cannot prove.

## Future adapter constraints

A future adapter may translate only closed typed request, acknowledgement,
ordered event, cancellation, usage, and terminal-outcome shapes for the exact
bound attempt. It must preserve event ordering, opaque session references,
replacement-versus-delta meaning, bounded error classification, request
idempotency classification, and content-free audit correlations.

An adapter may not select context, resolve or inspect credentials, retain a
lease, choose a model, expand a capability profile, create a second request,
or convert output into authority. It cannot expose arbitrary tool use,
retrieval, browser behavior, connectors, MCP, automation, filesystem, shell,
terminal, Git, cloud, deployment, or external mutation. Provider output,
grounding, citations, tool proposals, and provider-managed references remain
untrusted observations; they cannot modify QuireForge state or admit content
without independently approved local routes.

A local-runtime alternative must meet the same typed attempt, M60 context,
allowlist, policy, cancellation, expiry, audit, privacy/residency disclosure,
resource/usage guard, and no-ambient-authority requirements. Local classification
does not grant process launch, sockets, filesystem, environment, device,
terminal, shell, Git, or sandbox-worker authority.

## Audit, disclosure, and failure handling

Future content-free audit may retain only opaque attempt, bundle,
descriptor/adapter/model-profile, account-reference/lease where applicable,
project/task/scope, lifecycle, policy, expiry, bounded usage observation, and
terminal classification. It must exclude payload bytes, secrets, account
identifiers, provider request IDs, URLs, paths, diagnostics, transcripts,
browser state, and provider output.

Before a real provider can be proposed, a separate review must provide current,
inspectable destination-aware disclosure of retention, logging, secondary use,
processing/residency where material, account/organization terms,
endpoint/deployment distinctions, incident/deletion implications, content
restrictions, and supported security characteristics. Unknown, contradictory,
or stale disclosure makes that destination unavailable. This contract selects no
vendor, price, budget, currency, account, provider policy, or billing system.

## Ratification evidence and exit criteria

M62 is complete only when its record:

1. binds future inference to one exact M60 immutable prepared bundle and a
   native canonical attempt, rather than general assistant or ambient access;
2. requires M61 opaque-reference/lease prerequisites where authentication is
   needed without inspecting, collecting, storing, rotating, or revoking a
   credential;
3. defines destination, model, capability, privacy, payload, lifecycle,
   cancellation, revocation, recovery, audit, and fail-closed gates;
4. constrains adapters and local-runtime alternatives to typed
   intelligence-traffic translation with no tool or uncontrolled data path; and
5. is reviewed for the exclusions below and linked from the current state,
   roadmap, goal, and completion sequence.

M63 or any later implementation-planning or implementation milestone requires
a new, specific owner approval. It must name its authority surface, proposed
destination class, validation and adversarial-test plan, retention model,
external effects, and package/release implications. M62 grants no automatic
successor.

## Explicit exclusions

M62 authorizes no credential collection, storage, inspection, rotation, or
revocation; account, OAuth, API key, provider SDK, provider/local-runtime call,
networking, inference, model selection/configuration, tool, retrieval, browser,
connector, MCP, automation, external mutation, runtime code, migration,
package, installation, deployment, tag, release, or push.
