# Context Assembly and Transmission Manifests

Status: complete decision-only architecture gate within the active
[Provider-Neutral AI Foundation](GOAL_PROVIDER_NEUTRAL_AI_FOUNDATION.md). This
record approves no context-assembly implementation, manifest schema, scanning,
indexing, retrieval, source admission, provider/model invocation, transport,
credential resolution, persistence, bridge, or UI. M55, M57, and M58 remain
separate; beta.54 remains the latest packaged generation.

## Purpose and native ownership boundary

QuireForge must own future context selection, projection, transformation,
authorization, transmission decision, and invalidation. A provider, model,
endpoint, adapter, credential reference, provider session, task, plan, template,
artifact, Advisor projection, or frontend selector cannot independently choose
what enters an interaction.

A future context manifest is a closed, native-issued authorization record for
one bounded proposed transmission. It is not a provider request, upload, file
lifecycle, session replay, retrieval result, credential lease, or model
invocation. Its existence does not permit bytes, text, files, images, audio,
video, documents, or references to leave QuireForge. A later Limited Provider
Inference Boundary must separately authorize any dispatch and immediately
revalidate the then-current manifest.

Projects and durable tasks remain authoritative. Every manifest is bound to its
project and, where the selected interaction is task-owned, its durable task;
provider-owned threads, sessions, cursors, jobs, and continuation references
remain opaque subordinate data.

## Canonical transmission manifest

Every future manifest must be versioned, digest-bound, native-issued, opaque to
the frontend, and bind at least:

- the QuireForge project identity/version and durable task identity/version when
  applicable;
- canonical interaction-attempt identity, version, and digest;
- provider, endpoint/deployment, model, adapter, and capability-profile
  identities, versions, and digests;
- privacy-preserving organization/account binding, opaque credential-reference
  binding, and effective scope-set digest;
- an ordered, exact, bounded item list with each item identity, provenance,
  authorization class, freshness, sensitivity, retention classification,
  source/projection digest, and transformation chain;
- explicit exclusions and the reason each excluded candidate is absent;
- destination class, permitted transport characteristics, and destination-aware
  disclosure classification;
- manifest policy version, creation time, expiry, authorization state, and
  canonical digest; and
- bounded audit/evidence references and invalidation reason where applicable.

Human-readable names, model strings, account labels, project titles, provider
session IDs, file names, paths, frontend selections, or manifest ID alone never
prove ownership, freshness, policy, scope, or transmission authority. Unknown,
missing, cross-project/task, stale, broadened, reordered, altered, or
unrecognized safety-relevant fields fail closed. This gate chooses no wire,
database, IPC, or serialization format.

## Context-item classes and provenance

The following are conceptual classes. Their inclusion always requires a later
explicit native selection and manifest authorization; no class is automatically
available simply because it exists locally.

| Context-item class                | Permitted future basis                                                               | Required distinction                                                         |
| --------------------------------- | ------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------- |
| User-authored input               | Exact selected user text or bounded structured value                                 | User-authored, revision/digest-bound; no hidden prompt expansion             |
| Explicit project/file projection  | A named, user-approved bounded projection from one project                           | Projection only, not repository scanning or path authority                   |
| Approved task records and plans   | A task-owned record selected under its existing native policy                        | Task/plan version, project/task ownership, and visibility binding            |
| Approved artifacts                | An explicitly admitted local artifact projection                                     | Artifact provenance/version/digest; not automatic artifact enumeration       |
| Prior interaction outputs         | A selected, bounded local projection from a prior authorized attempt                 | Derived output, source attempt, status, and retention classification         |
| Provider continuation reference   | Opaque provider state subordinate to a selected prior attempt                        | Reference only; never implicit prior context or authority                    |
| Tool-result projection            | A bounded projection of an already completed, separately authorized native operation | Result provenance; no tool execution or route creation                       |
| Transient Advisor-safe projection | A bounded, existing locally safe projection under Advisor policy                     | Transient status; no hidden Advisor context or authority inheritance         |
| Future admitted external source   | A future M55-approved source record or evidence projection                           | Durable source identity/admission, provenance, retention, and citation state |
| Retrieved but unadmitted content  | None                                                                                 | Explicitly excluded; it cannot enter a manifest or become a durable source   |

Each item must carry provenance class, native owner, authorizing record or
selection, revision/digest, freshness/observation state, sensitivity class,
retention class, size/modality bounds, and source-to-projection relationship.
The manifest must distinguish transient explicitly selected local context,
approved durable local artifacts, opaque provider-managed references, future
admitted external sources, and retrieved-but-unadmitted material. The final
class is unavailable by default and cannot be repaired with a summary,
citation, provider claim, or user-interface label.

M55 remains the only authority for durable source identity, admission,
provenance, retention, and citation mapping. A URL, provider-managed file,
retrieval observation, search result, connected-service object, browser-rendered
content, or artifact is not a research source merely because a manifest names
or excludes it.

## Inclusion, exclusion, and minimum necessity

Inclusion is explicit and item-specific. A future manifest may include only the
exact canonical item projections shown in its ordered item list. It does not
authorize a surrounding directory, repository, task collection, conversation,
artifact collection, account, provider workspace, or continuation history.

Exclusion is explicit and authoritative. The manifest must state excluded
candidate class or identity, exclusion reason, and whether the reason is policy,
absence of user selection, stale provenance, sensitivity, missing authority,
destination mismatch, unsupported modality, size bound, or unavailable state.
An omitted item cannot be silently inferred from a summary, reference, prior
provider session, template, tool proposal, or adapter extension. If an item is
required but cannot be safely projected, the manifest is unavailable rather than
broadened.

Native policy must minimize each manifest to the context needed for the declared
future operation. More context, a larger projection, or an extra modality is a
new authorization decision and requires a new manifest. A provider's claimed
context limit, capability, account plan, or prompt format never expands the
manifest independently.

## Transformations and loss accounting

Future native assembly may use only closed, declared transformations such as
bounded extraction, canonical normalization, structural redaction, deterministic
truncation, approved summarization, media projection, and schema conversion.
Every transformation chain must bind its input identities/digests, transformation
rule/version, output digest, bounds, and loss/omission record. It must preserve
source provenance rather than treating a projection, summary, conversion, or
redaction as a new original source.

- **Bounded extraction** identifies exactly which allowed region or record
  projection was selected; it does not scan adjacent content.
- **Normalization** changes representation only under a versioned canonical
  rule and records all semantic limitations.
- **Structural redaction** removes a declared sensitive class before any future
  transmission; it cannot be replaced by best-effort string filtering.
- **Truncation** records omitted range/count/size and never claims completeness.
- **Summarization** is derived content with a visible source relationship,
  incomplete/interpretive status, and separate future authority; it cannot hide
  unadmitted retrieved content or invent source provenance.
- **Media projection** records modality conversion, source digest, bounds, and
  unavailable or omitted portions; it does not grant camera, microphone, file,
  browser, or upload authority.
- **Schema conversion** records input/output schema references and validation
  result; it does not approve generic JSON passthrough.

Unknown transformation, rule drift, unavailable input, missing loss record,
unbounded output, or a transformation that would weaken sensitivity or
provenance fails closed. This decision creates no transformation implementation.

## Authorization, preview, and review

Before a future manifest could be authorized, QuireForge must prepare a bounded
preview that makes the selected provider/destination class, account binding,
attempt, exact item identities/projections, transformations, exclusions,
sensitivity/retention classifications, expiry, and canonical digest inspectable
to the user. A later UI decision must make that preview accessible; this gate
creates no UI.

Authorization is explicit, native-owned, one-purpose, expiring, and tied to the
exact digest. It is distinct from context selection, credential availability,
adapter compatibility, capability support, and provider invocation. The
frontend may submit only opaque native selectors/handles and bounded authored
values accepted by a later route; it cannot assert project/task ownership,
descriptor state, account scope, item freshness, transformation outcome,
authorization, expiry, or digest.

Cancellation consumes or invalidates a future pending manifest authorization
without transmitting anything. Confirmation of a manifest must not create,
modify, save, dispatch, approve, execute, or retry a provider operation. A
future dispatch boundary needs a separate explicit confirmation model where its
risk requires one.

## Invalidation and immediate revalidation

The manifest is invalidated by any relevant project or task change; user edit;
selected item/projection revision or digest change; artifact replacement;
descriptor/model/endpoint/deployment or adapter drift; capability-profile or
policy change; account, credential-reference, scope, or lease revocation;
provider continuation change; source freshness, availability, deletion, or
provenance change; transformation-rule change; destination change; expiry; or
quarantine/degradation.

Immediately before any future submission, native code must recompute the exact
manifest digest and revalidate every project/task/item binding, item freshness,
transformation result, exclusion, sensitivity/retention policy, descriptor and
adapter compatibility, account/credential/lease state, capability profile,
destination, expiry, and authorization state. Mismatch, stale data, uncertain
ordering, unknown state, or partial revalidation fails closed and requires a
fresh manifest; no automatic repair, context expansion, or submission retry is
permitted.

## Continuation and provider-managed state

A provider continuation, thread, session, cursor, or job reference is opaque,
subordinate, and bound only to its selected prior attempt, adapter, descriptor,
and future manifest policy. It does not automatically authorize the prior
context, a previous credential lease, a new account, an earlier provider
session's hidden state, or a future model invocation.

Any changed selected item, transformation, exclusion, destination, task/project
binding, account/credential state, capability profile, descriptor, adapter, or
policy requires a new manifest. A continuation reference may be named as a
reference item only if a later route permits it; it does not restore, append,
infer, or transmit unseen provider-held state. Provider-managed state remains
opaque and cannot replace QuireForge's own manifest/audit record.

## No ambient context

No future manifest may inherit, scan, or silently include an entire repository,
filesystem path, working tree, directory, clipboard, terminal history, shell
session, browser state/cookies/downloads, environment variable, existing Codex
session, unrelated task or project, hidden template/instruction, advisor hidden
state, connected service, provider workspace, generic MCP context, or host
process state. A user-visible title, a provider session identifier, a file name,
or an account label does not select its contents.

The context assembler must not inspect these sources merely to discover whether
they might be useful. Any future local projection or external source needs its
own explicit, bounded, separately authorized admission/selection route. No
ambient context source can be used as a fallback when an exact selected item is
unavailable.

## Privacy, retention, and destination boundaries

Every future manifest is destination-aware: native policy must classify the
specific provider, endpoint/deployment, account, adapter, and permitted
transport characteristics before deciding whether each selected projection is
eligible. A credential reference only identifies a potential custodian path; it
does not authorize content transmission. Sensitive, private, secret-bearing,
or destination-incompatible context must be excluded or structurally redacted.
Raw secrets are never context items or transformation inputs.

The future retention policy must distinguish ephemeral assembly workspace,
transient manifest authorization, durable local manifest metadata, and durable
evidence/artifact records. It must state retention period, deletion behavior,
revocation effects, and recovery semantics before persistence is approved.
Deletion or revocation invalidates future selection/transmission authority; it
does not claim that a provider erased data or that a historical local audit
record is itself secret-free without separate verified policy.

## Audit and evidence

A future audit/evidence projection may record the opaque manifest identity and
digest; project/task/attempt/descriptor/adapter/account/scope bindings;
item-class counts and opaque item/projection digests; transformation rule and
loss summaries; exclusion reasons/counts; destination/retention/sensitivity
classifications; authorization/expiry/invalidation/dispatch-decision state; and
closed error classification. It must distinguish authorized, prepared,
cancelled, invalidated, expired, and any later separately authorized submission
decision.

Audit must not duplicate raw context, credentials, full paths, task/template
content, provider payloads, provider session handles, browser state, terminal
history, environment values, or private diagnostics. It records what was
authorized and, once a future dispatch boundary exists, what was actually sent
only as bounded identity/digest/provenance evidence—not as a second sensitive
copy of the content.

## Relationship to completed decisions and separate lanes

The [capability registry](PROVIDER_NEUTRAL_CAPABILITY_REGISTRY_AND_DESCRIPTOR_GOVERNANCE.md)
continues to govern metadata, capability claims, limits, provenance, and
extensions—not context scope. The [interaction protocol](CANONICAL_PROVIDER_NEUTRAL_INTERACTION_AND_EVENT_PROTOCOL.md)
continues to define attempts and events—not selection or transmission. [Adapter
governance](PROVIDER_ADAPTER_LIFECYCLE_AND_CONFORMANCE_GOVERNANCE.md) keeps
adapters as intelligence-traffic translators; adapters cannot select, redact,
retain, or broaden context. [Credential custody](CREDENTIAL_BROKER_AND_ACCOUNT_PROJECT_SCOPE_CUSTODY.md)
keeps credential references and leases separate from context authorization.

M57 remains reusable least-authority evidence for bindings, digest checks,
revocation, replay resistance, and content-free audit; it does not authorize a
connector, external source, or generic context route. M58 remains an
independent, unstarted, verification-only browser lane. This decision grants no
browser access, cookie reuse, DOM inspection, screenshot, download, form
submission, OAuth, automation, or web research authority.

## Persistence and UI boundaries

This decision approves no assembler, projection, transformation, manifest
record, retention store, migration, command, bridge, UI, package, release, or
host change. A later implementation must separately define data model,
durability, deletion, recovery, strict native/bridge contracts, accessibility,
and adversarial tests. No current source contract implements this gate.

## Recommended next gate

The next recommended decision is **Limited Provider Inference Boundary**. It
must decide the first narrowly bounded model-invocation authority, user action,
pre-dispatch revalidation, cancellation, result lifecycle, provider-specific
extension constraints, and end-to-end proof that projects/tasks remain
authoritative. It must not authorize retrieval, durable source admission,
native tools, browser behavior, connected services, external mutation,
automation, or a selected provider implementation.

## Explicit exclusions

This decision grants no context-assembly implementation; manifest schema; file
scanning; repository indexing; retrieval; source admission; citation authority;
provider/model invocation; network access; credential resolution; provider
upload/file lifecycle; native tool execution; browser behavior; connector or
MCP implementation; persistence schema or migration; Rust or TypeScript
contract; Tauri command; bridge; UI; package; release; tag; installation; host
change; deployment; background work; automation; or multi-agent behavior.
