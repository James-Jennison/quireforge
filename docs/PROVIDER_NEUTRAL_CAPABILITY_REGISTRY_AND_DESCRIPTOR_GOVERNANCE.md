# Provider-Neutral Capability Registry and Descriptor Governance

Status: complete decision-only architecture checkpoint. Following the
[External Capability Taxonomy and Sequencing](EXTERNAL_CAPABILITY_TAXONOMY_AND_SEQUENCING.md)
decision, this record grants no model invocation, provider access, networking,
credentials, retrieval, tool authority, browser authority, persistence, UI, or
implementation authority. M57 remains closed as source-only, M58 remains
separate and unstarted, and beta.54 remains the latest packaged generation.

## Purpose

A future provider-neutral registry may represent external intelligence
capabilities without granting authority; determine which controls and workflows
may be offered for a selected descriptor; validate that an adapter's claims
match approved capability semantics; preserve provider-specific behavior without
forcing every provider into a lowest-common-denominator chat abstraction; and
support later compatibility, deprecation, provenance, conformance, and policy
checks.

> A descriptor is metadata and identity, not authority, authentication,
> availability proof, or permission to invoke a provider.

This record selects neither a provider nor a transport. It does not create a
registry implementation, descriptor schema, adapter, fixture, persistence
record, command, bridge, or UI.

## Descriptor entities

The following entities remain distinct even when one product supplies several
of them.

### Provider descriptor

A provider descriptor represents the organization, platform, runtime family,
or service boundary responsible for endpoints or models. It may carry stable
identity, bounded display metadata, provider class, extension namespace,
documentation or provenance references, conceptually supported authentication
classes, and lifecycle status. It contains no credential or account binding.

### Endpoint or deployment descriptor

An endpoint or deployment descriptor represents a specific API family,
cloud deployment, region, base-endpoint class, or local-runtime endpoint. It is
not the provider or model identity. Public APIs, cloud-hosted deployments,
organization-specific endpoints, local loopback runtimes, and private-network
endpoints are conceptual examples only; none is selected here.

### Model descriptor

A model descriptor represents a model identity or family in its provider and
endpoint context. It may carry stable registry identity, provider model
identifier, display name, family, revision or release channel, capability
claims, context limits, lifecycle and compatibility state, and provider-specific
extension metadata. Equal marketing names do not make models globally identical
across endpoints.

### Local runtime descriptor

A local runtime descriptor represents a locally operated inference or
processing runtime. It remains distinct from its model, endpoint instance,
QuireForge native authority, terminal or process authority, and sandbox worker.

### Capability definition and claim

A capability definition gives a canonical, versioned meaning independent of a
provider. A capability claim records a model, endpoint, or runtime assertion
that the definition is supported under stated conditions. A claim is not an
authorization or an availability proof.

### Adapter descriptor

An adapter descriptor represents a future translation component that
understands one provider or endpoint protocol. It declares compatibility and
conformance claims, but registration grants it no credentials, invocation, or
native operation authority.

## Identity, versioning, and drift

Future provider, endpoint/deployment, model, local-runtime, adapter,
capability-definition, and capability-claim records require opaque native
identifiers consistent with the M57 model. Human-readable names are display
metadata, not identity; provider model strings are insufficient primary keys;
and identity must survive a display-name change. Endpoint-specific variants
must not collide.

Descriptors are versioned and digest-bound. Drift invalidates dependent
selections and cached compatibility conclusions until they are re-evaluated.
Aliases may assist a future migration but must never silently merge distinct
authority, account, transport, or endpoint boundaries. This decision selects no
particular UUID format or database schema.

## Provenance and trust classes

A later registry may distinguish these descriptor provenance classes:

- QuireForge built-in static descriptor;
- signed or packaged QuireForge descriptor update;
- adapter-supplied static descriptor;
- provider-discovered metadata;
- local-runtime-discovered metadata; and
- user-declared custom endpoint metadata.

Provenance affects trust: discovered metadata is observational and may be stale
or false; user-entered metadata is unverified; provider marketing claims are
not conformance evidence; and static built-in metadata can become stale. Trust
class must remain visible to native policy and any later UI. No descriptor
source grants native operation authority. Updates require version/digest
comparison, and unknown fields or extension claims fail closed whenever they
would affect safety or availability.

Automatic network discovery is not authorized by this decision.

## Canonical capability namespace

The future namespace is canonical and versioned, but this is not an exhaustive
or permanent capability list.

| Category                           | Canonical capability families                                                                                                                                                                                      |
| ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Input modalities                   | text, image, audio, video, document or file reference input                                                                                                                                                        |
| Output modalities                  | text, structured data, image, audio, video, embeddings or vector output                                                                                                                                            |
| Interaction                        | streaming, cancellation, continuation, provider-managed session reference, stateless request, batch request, realtime session                                                                                      |
| Reasoning and response controls    | reasoning-mode support, structured-schema conformance, deterministic or seed-like controls, output-length controls, sampling controls, provider-defined reasoning summaries or protected reasoning representations |
| Tool-related intelligence behavior | tool-call proposal, parallel tool-call proposal, structured tool arguments, tool-result continuation                                                                                                               |
| Information and source behavior    | citations, provider-grounded response metadata, provider-managed file references, provider-managed retrieval capability                                                                                            |
| Efficiency and lifecycle           | prompt/context caching, token counting, usage reporting, asynchronous job state, fine-tuning or customization references, model deprecation metadata                                                               |

Protected reasoning content is not presumed available. Tool-related capability
only describes model communication behavior; it does not authorize a QuireForge
operation or tool execution. Provider-managed retrieval capability does not
bypass M55 durable-source admission.

## Capability claim semantics and limits

A claim is more precise than a boolean. A future claim must conceptually bind a
capability identifier and version, support state, confidence, provenance,
qualifiers, limits, endpoint/deployment conditions, known account or plan
conditions, compatibility notes, observed time where applicable, expiry or
freshness, conformance state, and a provider-specific extension reference.

Support states distinguish **supported**, **unsupported**, **unknown**,
**conditionally supported**, **advertised but unverified**, **deprecated**,
**temporarily unavailable**, and **incompatible with the installed adapter**.
Unknown never means supported.

Qualifiers may express context and maximum-output size, accepted formats, media
duration or dimensions, file count/size, schema restrictions, tool-count limits,
parallelism, regional availability, rate-limit observations, and preview or
experimental status. Limits may differ by provider, endpoint, account,
deployment, and model revision. Raw provider-specific units must be preserved;
runtime-discovered limits cannot weaken stricter QuireForge policy; absent
limits are unknown rather than unlimited. Pricing is not a capability and stays
for a later usage/cost decision.

## Provider-specific extensions

Canonical capabilities cover shared semantics. Provider-specific extensions use
namespaced identifiers and may preserve unique controls or events. An unknown
extension cannot be treated as a known canonical capability and cannot grant
native authority. Extensions affecting context transmission, retention,
credentials, tool behavior, or external mutation require separate approved
policy support. A future UI may expose a provider-specific control only after an
approved adapter and capability mapping exist. Canonical behavior must not
discard extension data needed for accurate continuation or error reporting.

## Capability is not authority

```text
Capability claim
≠ account authorization
≠ credential access
≠ context-transmission approval
≠ native tool authority
≠ connected-service scope
≠ browser authority
≠ external mutation authority
```

For example, `tool-call-proposal` does not authorize a terminal command;
provider-managed retrieval does not authorize durable source admission;
image-input does not permit arbitrary project-image transfer; long-context does
not authorize broad repository transfer; a local runtime does not permit process
execution or host filesystem access; realtime-session does not permit microphone
or screen capture; provider-managed-session does not replace a QuireForge task;
and citations do not prove source validity.

## Static and discovered metadata

A future registry may support both reviewable, package- or source-bound static
descriptors and bounded observations from an approved adapter. Discovery itself
requires a later implementation decision. Reconciliation must preserve claimed
and observed states without silently replacing stable identity. A newly
discovered capability cannot enable itself, expand credentials, or authorize a
mutation. Disappeared, renamed, deprecated, and newly introduced models require
explicit lifecycle handling.

## Lifecycle and compatibility

Descriptors may be active, preview, deprecated, retired, unavailable,
quarantined, unsupported by the installed adapter, or subject to detected drift.
A descriptor may exist while unusable. Compatibility is a relation among
descriptor, adapter, canonical protocol, capability-definition, and QuireForge
policy versions; it is never inferred solely from a name or marketing claim.

## Adapter relationship and conformance

A future adapter translates provider-native requests and events, declares the
descriptor and protocol versions it understands, maps provider behavior to
canonical capabilities, preserves approved extensions, produces conformance
evidence, and classifies provider errors. It does not own QuireForge tasks,
receive credentials except through a future approved temporary-access mechanism,
execute native operations, bypass context manifests, or persist provider content
without later approval. The adapter ABI remains a later decision.

Conformance governance will require static fixtures for descriptor parsing and
canonicalization, deterministic digest/version behavior, capability-claim
validation, unknown-field and extension handling, lifecycle transitions,
adapter compatibility checks, advertised-versus-observed distinction, and
failure-closed treatment of safety-relevant claims. No live-provider conformance
test is authorized until networking and credentials are separately approved.
Conformance proves metadata and translation behavior, never provider reliability
or authority.

## Persistence and UI boundaries

This checkpoint approves neither persistence nor UI. A later implementation
decision must separately decide built-in descriptors, retention/deletion of
discovered or user-entered metadata, account-specific metadata, migration,
staleness, privacy, and import/export. It adds no SQLite schema.

A later UI may display provider, endpoint, model, capability, provenance,
lifecycle, compatibility, trust, limits, and deprecation warnings. Presentation
must not imply account connection, current availability, or permission.

## Relationship to current architecture

This decision preserves M57 descriptor digest/version concepts and opaque
project/account/scope references; project and task authority; the Advisor and
Codex separation; inspectable templates; artifact provenance; strict native
ownership; closed bridge contracts; and review/evidence patterns.

Existing Codex model or reasoning labels are not a universal registry. The
Integration Center does not become generic provider ownership. M57 `LocalMock`
is governance evidence, not a real provider descriptor. No current source
contract is declared to implement this decision.

## Later decision order

The dependency order is:

1. this capability-registry and descriptor-governance decision;
2. Canonical Provider-Neutral Interaction and Event Protocol;
3. provider-adapter lifecycle and conformance;
4. credential broker and account/project/scope custody;
5. context assembly and transmission manifests; and
6. limited inference.

Later evidence may justify revisiting order, but no implementation begins
automatically. M55 durable source admission remains separate, and M58 remains
independent.

## Recommended next decision

The next recommended decision is **Canonical Provider-Neutral Interaction and
Event Protocol**. Its future scope should cover inputs and outputs, streaming,
cancellation, continuation, provider-session references, structured data,
multimodal events, tool proposals and results as communication events only,
citations and provider-grounding metadata, usage events, errors and terminal
states, provider-specific extension envelopes, and the relation between provider
events and QuireForge tasks.

That decision must exclude provider implementation, credentials, networking,
native tool execution, retrieval authority, browser behavior, UI, and persistence
implementation. This record does not write it.

## Explicit exclusions

This decision grants no selected provider or vendor, provider implementation,
model invocation, network access, credentials or custody, OAuth, account
connection, endpoint/model discovery, inference, retrieval, source admission,
citation authority, browser behavior, connector implementation, MCP execution,
tool execution, terminal, shell, Git, filesystem, repository, or deployment
authority. It also grants no persistence, SQLite migration, Tauri command,
frontend bridge, UI, package, release, tag, automation, background activity, or
multi-agent behavior.
