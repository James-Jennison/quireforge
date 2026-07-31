# Provider Adapter Lifecycle and Conformance Governance

Status: complete decision-only architecture gate within the active
[Provider-Neutral AI Foundation](GOAL_PROVIDER_NEUTRAL_AI_FOUNDATION.md). This
record selects no provider, adapter implementation, schema, transport,
credential route, context-transmission route, model invocation, persistence,
bridge, or UI. M55, M57, and M58 remain separate; beta.54 remains the latest
packaged generation.

## Purpose and authority

A future provider adapter is a bounded intelligence-traffic translator between
one approved provider or local-runtime protocol and QuireForge's approved
[capability registry](PROVIDER_NEUTRAL_CAPABILITY_REGISTRY_AND_DESCRIPTOR_GOVERNANCE.md)
and [canonical interaction and event protocol](CANONICAL_PROVIDER_NEUTRAL_INTERACTION_AND_EVENT_PROTOCOL.md).
It makes provider-specific behavior inspectable and testable without turning a
provider protocol, a generic tool interface, or an OpenAI-compatible chat shape
into native authority.

An adapter is neither a provider account nor an invocation route. Registration,
compatibility, conformance, discovery, or successful translation does not prove
availability, identity, authentication, context authorization, permission, or
permission to send or mutate anything.

## Adapter identity and compatibility

Every future adapter requires an opaque native adapter identity, adapter release
version, descriptor digest, adapter manifest digest, declared compatibility
range, trust class, lifecycle state, and a bounded canonical protocol mapping
version. Display names, module names, provider identifiers, endpoint strings,
or frontend selectors are never adapter authority.

Compatibility is a revalidated relationship among all of the following:

- the provider, endpoint/deployment, model, or local-runtime descriptor
  identity/version/digest;
- the selected capability-definition and claim versions;
- the canonical interaction/event protocol version and extension namespaces;
- the adapter identity, release, manifest digest, and conformance evidence; and
- the applicable QuireForge policy version.

An adapter may support multiple descriptor variants only through explicit,
digest-bound compatibility declarations. A provider marketing name, endpoint
family, model-family alias, or nominal protocol similarity cannot silently
broaden a compatibility claim. Descriptor, protocol, extension, adapter, or
policy drift invalidates cached compatibility and blocks a future route until it
is re-evaluated.

## Trust and installation-source classes

Future adapter provenance must remain visible to native policy and any later UI.
This gate defines these conceptual classes:

- **QuireForge built-in** — reviewed, source- or package-bound static adapter
  metadata.
- **QuireForge signed or packaged update** — a separately verified update with
  explicit version and digest transition.
- **Approved local development fixture** — deterministic conformance-only test
  material with no operational route.
- **Untrusted declaration** — user- or provider-supplied metadata that may be
  displayed only as unverified observation and cannot become callable.
- **Unknown or unverifiable source** — unavailable and quarantined by default.

No dynamic plugin loading, provider-delivered executable, downloaded code,
generic MCP server, script, command, shell hook, URL, browser session, or
ambient host process is an approved installation source. This decision does not
authorize adapter installation, loading, update retrieval, signature technology,
or a runtime ABI; it defines the conditions a later implementation must meet.

## Lifecycle and controlled transitions

The future adapter lifecycle is closed. A static record may be **known** but
unavailable; an installed approved adapter may be **candidate**; a fully
validated one may be **compatible**; and a currently unusable one may be
**incompatible**, **deprecated**, **retired**, **revoked**, **degraded**, or
**quarantined**. These states describe translation eligibility only. None
describe a connected account, a credential, a dispatched request, or a real
provider action.

Only native validation may perform these transitions:

```text
known -> candidate -> compatible
candidate/compatible -> incompatible | deprecated | degraded | quarantined
deprecated -> retired | quarantined
any non-terminal state -> revoked | quarantined
retired/revoked/quarantined -> known only through a separately approved,
                         explicit replacement or recovery contract
```

Unknown compatibility, changed manifest digest, failed safety conformance,
unsupported protocol version, unrecognized safety-relevant extension, missing
required evidence, or ambiguous upgrade state fails closed to unavailable or
quarantined. A quarantined adapter cannot translate future traffic or supply a
compatibility conclusion. Revocation invalidates all dependent compatibility
conclusions and any future pending attempt binding; it does not erase bounded
historical evidence under a later approved retention policy.

## Upgrade, rollback, revocation, and deprecation

An upgrade is a new adapter release and manifest digest, not an in-place trust
assumption. Before it could become compatible, a later implementation must
revalidate its descriptor mappings, canonical protocol range, extension
handling, conformance evidence, and policy compatibility. It must not inherit a
prior adapter's compatibility conclusion merely because its display name or
provider family matches.

Rollback means selecting a previously retained, independently compatible
adapter release through an explicit native decision. It must revalidate against
the current descriptor, protocol, policy, and extension set; it cannot restore
revoked, quarantined, or expired authority. An interrupted upgrade, incompatible
rollback, or unverifiable release remains unavailable rather than silently
choosing another adapter.

Deprecation gives a bounded warning state and a future migration deadline, but
does not create a replacement selection or context transfer. Retirement blocks
new attempts. Revocation is an immediate safety or trust invalidation. Neither
status authorizes automatic retries, continuation, account movement, context
reuse, or provider fallback.

## Capability mapping

An adapter must map a provider- or runtime-specific behavior to a versioned
canonical capability definition and claim. Each mapping identifies the source
descriptor/adapter release, canonical capability and version, support state,
qualifiers and limits, trust/provenance, conformance status, and any governed
extension namespace. Mapping is directional: an advertised provider feature is
not canonical support until it has an approved mapping and required conformance
evidence.

Mappings must preserve the registry distinction among supported, unsupported,
unknown, conditionally supported, advertised-but-unverified, deprecated,
temporarily unavailable, and adapter-incompatible. Missing limits are unknown,
not unlimited. A mapping cannot turn a capability claim into account authority,
credential access, context approval, retrieval permission, native tool
authority, browser authority, connected-service scope, or mutation authority.

## Canonical event translation

The adapter may later translate approved provider-native communication into the
closed canonical attempt, input, event, and terminal envelopes. Translation
must preserve native attempt correlation, monotonic ordering, modality,
replacement-versus-delta semantics, cancellation/interruption distinction,
outcome-unknown state, provider-session opacity, usage-unit provenance, and
safe closed error classification.

Provider sessions, thread IDs, request IDs, stream IDs, jobs, cursor tokens,
and continuation handles remain opaque subordinate references. They cannot
select a different project or durable task, substitute a descriptor, revive a
terminal attempt, or transfer an account, context, or authority. QuireForge
projects and durable tasks remain authoritative over every future interaction;
an adapter cannot create, replace, or infer their binding.

Provider tool proposals and tool-result events are communication events only.
The adapter must represent them as the protocol permits, but cannot dispatch a
native operation, execute a shell or terminal action, use Git, access files,
invoke MCP, or treat a model proposal as approval. Provider citations,
grounding, retrieval observations, and provider-managed file references remain
claims under M55 and do not become durable sources or evidence by translation.

## Extension governance

Provider-specific controls, event fields, lifecycle detail, structured forms,
multimodal behavior, reasoning summaries, grounding metadata, and errors must
use the registry's namespaced, versioned extension rules. An extension mapping
binds its source adapter release and descriptor compatibility claim; unknown or
malformed safety-relevant extensions fail closed.

An extension cannot silently alter canonical ordering, terminality, context
selection, credential semantics, retention, tool behavior, retrieval, browser
behavior, native operations, or mutation policy. An extension that needs any of
those effects requires separate explicit policy support before an adapter could
offer it. Protected reasoning is unavailable by default and cannot be coerced
into a canonical requirement.

## Conformance evidence and deterministic validation

Conformance is deterministic evidence that an adapter's declared translation
matches approved metadata and protocol semantics. It does not prove a provider
is available, safe, authenticated, reliable, authorized, or operational.

Before a future adapter could become compatible, its conformance suite must use
static, sanitized fixtures and prove at least:

- identity, release, manifest digest, descriptor compatibility, and canonical
  protocol-version determinism;
- closed descriptor and capability mapping validation, including unknown-field,
  unknown-enum, alias-collision, limit, and trust-class rejection;
- advertised-versus-observed capability distinction and failure-closed unknown
  support, limits, and safety-relevant extensions;
- translation of input modality/role, output shape, deltas, snapshots,
  structured and multimodal events, usage units, citations/grounding claims,
  cancellation, continuation, errors, and terminal states;
- monotonic ordering, duplicate/late/post-terminal rejection, descriptor and
  adapter drift invalidation, and opaque session-reference confinement;
- extension namespace/version handling and rejection of unapproved authority
  effects; and
- structural redaction of credentials, private content, raw diagnostics, paths,
  URLs, and provider handles from bounded conformance outputs.

Fixtures must be local, deterministic, content-bounded, non-secret, and
non-executable. No live-provider test, DNS, HTTP client, provider SDK, account,
credential, browser, shell, process, filesystem mutation, or network discovery
is authorized by conformance. A passing fixture suite cannot promote an
adapter's trust class or make it callable without each later gate's authority.

## Error classification and failure-closed behavior

A future adapter may report only closed, sanitized classifications such as
local validation failure, descriptor drift, incompatible protocol, unsupported
or unknown capability, malformed provider event, extension violation,
translation failure, cancellation acknowledgement missing, timeout,
provider-declared failure, outcome unknown, degraded state, trust failure, and
quarantine. It must not emit raw provider diagnostics, credentials, request
payloads, paths, URLs, private account identities, or opaque provider handles
as user-visible errors.

Failures never cause automatic adapter substitution, upgrade, rollback,
continuation, retry, dispatch, context expansion, account selection, or native
operation. A translation failure cannot reinterpret a provider payload as a
successful terminal event. Any ambiguous external result remains
outcome-unknown and requires a separately approved future route and fresh user
review; it is never retried automatically.

## Authority boundary

An adapter receives no implicit credential custody, secret access, account
selection, context-selection or transmission authority, inference authority,
native tool execution, shell, terminal, Git, filesystem, browser, connector,
MCP, cloud, deployment, approval, dispatch, external mutation, scheduling, or
background authority. It cannot inherit authority from a project, task, plan,
template, artifact, Advisor item, generated content, environment, browser
session, Codex session, or host process.

Credential Broker and Account/Project/Scope Custody remains the required next
separate decision before any real account or credential route can be considered.
Context Assembly and Transmission Manifests remains separately required before
any data leaves a QuireForge-controlled boundary. Limited Provider Inference
remains separately required before any model invocation. This record grants
none of them.

## Relationship to M55, M57, and M58

M55 remains the authority for durable source identity, provenance, retention,
and citation mapping; adapter-translated retrieval or citations cannot bypass
it. M57 remains reusable evidence for least-authority lifecycle, digest/version
binding, revocation, confirmation, replay resistance, and content-free audit
models, but it does not turn an adapter into a connector or provider route.

M58 remains an independent, unstarted, verification-only browser decision lane.
This gate grants no browser transport, browser credential, cookie, rendered
content, DOM, screenshot, download, form-submission, automation, OAuth, or web
research authority. Adapter governance must not be used to begin or broaden
M58.

## Persistence and UI boundaries

This decision approves no adapter persistence, installation record, manifest
store, migration, command, bridge, UI, package, or release. A future
implementation decision must separately define any durable retention, deletion,
recovery, and user-visible projection. Until then, no current source contract
implements this gate.

## Recommended next gate

The next recommended decision is **Credential Broker and Account/Project/Scope
Custody**. It must decide secret-custody responsibility, opaque credential
references, account and project bindings, scope grants, leases, expiry,
rotation, revocation, isolation, and unavailable-custody failure behavior. It
must not select a credential technology, implement storage, accept secrets,
connect an account, invoke a provider, transmit context, or begin inference.

## Explicit exclusions

This decision grants no adapter implementation or installation, provider/model
selection, network access, provider discovery, credentials, account connection,
OAuth, context transmission, model invocation or inference, retrieval, source
admission, citation authority, native tool execution, shell, terminal, Git,
filesystem, browser behavior, connector or MCP execution, cloud or deployment
authority, external mutation, persistence, migration, Tauri command, bridge,
UI, package, release, tag, host change, background work, automation, or
multi-agent behavior.
