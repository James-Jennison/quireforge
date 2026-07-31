# Credential Broker and Account/Project/Scope Custody

Status: complete decision-only architecture gate within the active
[Provider-Neutral AI Foundation](GOAL_PROVIDER_NEUTRAL_AI_FOUNDATION.md). This
record approves no credential broker implementation, secret store, provider,
account connection, transport, context-transmission route, model invocation,
persistence, bridge, or UI. M55, M57, and M58 remain separate; beta.54 remains
the latest packaged generation.

## Purpose and native ownership boundary

A future credential broker is QuireForge's native-owned authority boundary for
requesting and validating the availability of a narrowly scoped, non-exportable
credential reference. It separates credential custody from provider adapters,
projects, durable tasks, context selection, and invocation. The broker does not
make an adapter callable, authorize context transmission, select a provider, or
approve an interaction attempt.

The broker's future role is limited to closed identity, binding, lifecycle,
lease, revocation, and content-free audit decisions. A separately approved
secret custodian may hold secret material, but QuireForge's ordinary metadata
store is never that custodian. This gate selects no OS service, keyring, wallet,
file format, database, provider mechanism, or broker ABI.

Projects and durable tasks remain authoritative over any future interaction.
Credential presence can make a future route eligible for additional native
review; it cannot create a project/task binding, replace one, choose context,
or turn a provider session into an authority record.

## Credential reference identity

The future broker may issue only opaque, native-owned credential-reference and
lease identifiers. A reference must bind, at minimum, to:

- its credential class and non-secret custodian class;
- one provider descriptor and digest, and the compatible adapter identity and
  manifest digest;
- one endpoint/deployment identity and digest where applicable;
- a privacy-preserving organization and account reference;
- one QuireForge project and explicit effective scope-set digest;
- lifecycle, issuance/observation, expiry, revocation, and quarantine state;
- a reference version/digest; and
- bounded conformance and availability classification.

Human-readable account labels, provider account IDs, provider endpoint strings,
browser-session labels, frontend selectors, and a credential-reference ID alone
are not sufficient authority. A reference never resolves to a password, token,
key, certificate, cookie, session value, private key, or secret-bearing error.

Raw secrets must never enter project, task, plan, review, artifact, template,
provider descriptor, adapter descriptor, canonical interaction envelope,
context manifest, log, audit record, diagnostic, export, crash report, prompt,
or frontend state. Redaction is structural: serializable reference, lease,
availability, error, and audit shapes contain no secret-bearing field by design,
rather than relying on best-effort string filtering.

## Conceptual credential classes

The following classes describe future custody requirements; none is enabled or
implemented here.

| Credential class                             | Future reference may describe                                                 | This decision does not permit                                                             |
| -------------------------------------------- | ----------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| API key                                      | provider/endpoint binding, bounded label, scope class, lifecycle              | receiving, storing, displaying, copying, or exporting the key                             |
| OAuth access and refresh material            | authorization family, grant status, account binding, expiry/revocation state  | OAuth redirect, authorization code, token exchange, refresh, cookie use, or token custody |
| Cloud credential chain or assumed role       | cloud authority class, role/reference identity, requested scope, lease expiry | CLI/profile lookup, role assumption, metadata-service access, or cloud API use            |
| Local endpoint authentication                | endpoint class, authentication family, local-runtime binding, availability    | local socket, file, process, keyring, or password access                                  |
| Provider account session                     | opaque provider-session class and account reference                           | session creation, session reuse, cookie import, provider login, or browser handoff        |
| Platform keyring or secret-service reference | custodian class, opaque external reference, availability and lease state      | keyring/Secret Service/wallet integration or secret resolution                            |

Each actual custody technology, account family, and authorization flow needs a
later explicit implementation decision. A class label is not proof that a
reference exists, that its secret is valid, or that a provider can be contacted.

## Account, organization, project, endpoint, and scope binding

Every future authority-bearing reference must bind exactly one opaque provider,
organization, account, project, and effective scope set. It also binds the
selected endpoint/deployment where transport identity affects authorization,
region, model, or policy. Provider and organization descriptions remain
distinct from account identity; account identity remains privacy-preserving and
non-secret.

An account may be represented in several projects only through separate,
explicit per-project grants. A project may refer to several accounts for one
provider only when a future approved flow selects one account for one operation.
No task, plan, template, artifact, review, generated text, imported content,
provider session, or adapter can create, clone, broaden, restore, or choose a
binding. Discovery metadata may be global only when it conveys no account,
scope, credential, or operation authority.

Scopes are closed, versioned vocabulary owned by future native policy. A scope
set records requested, granted, and effective values separately. Unknown,
broadened, missing, expired, cross-provider, cross-endpoint, cross-account, or
cross-project scope claims fail closed. A provider-advertised scope, account
plan, or model capability does not alter QuireForge policy.

Project archive, project removal, account disconnect, organization change,
endpoint/deployment drift, adapter incompatibility, scope change, descriptor
drift, or provider-object deletion invalidates dependent references and leases.
Restoring a project never revives a credential reference or grant. Local
historical records may retain only later-approved content-free evidence; they
cannot regain external access.

## Custody lifecycle

The future lifecycle is closed and native-validated:

```text
unknown -> enrollment-requested -> enrollment-pending -> reference-observed
reference-observed -> validation-pending -> available | unavailable | quarantined
available -> lease-pending -> leased -> expired | revoked | degraded | quarantined
unavailable/degraded -> validation-pending only through a later approved action
available/leased -> rotation-pending -> available | unavailable | quarantined
any non-terminal state -> revoked | deleted | quarantined
```

These states describe local knowledge of a reference and its bounded usability,
not a secret value or a remote account fact. Enrollment, validation, renewal,
rotation, and deletion require future separately approved routes and explicit
user-visible consent. This document neither establishes those routes nor
permits any external call.

`available` means only that a later approved custodian has supplied fresh,
bounded non-secret evidence under its own contract. It does not mean the user
has approved context transmission, model invocation, retrieval, tool execution,
or external mutation. `leased` does not dispatch anything. `deleted` removes
future local reference authority but cannot claim secure erasure from a custodian
without that custodian's separate verified contract.

## Least-authority leases

A future broker lease is native-issued, opaque, non-exportable, short-lived,
one-purpose authority to request access from the approved custodian. Before a
lease can be issued, the broker must revalidate the exact:

- compatible adapter identity/release/manifest digest;
- provider and endpoint/deployment identity/digest;
- privacy-preserving organization and account reference;
- QuireForge project identity and effective scope-set digest;
- closed operation class and future policy classification;
- requesting canonical interaction-attempt identity, version, and digest; and
- lease version, issuance time, expiry, and reference lifecycle state.

A lease cannot be transferred to another adapter, provider, endpoint,
deployment, account, organization, project, scope, operation, attempt,
continuation, process, or user interface. It cannot be reconstructed from a
reference ID, frontend value, task, plan, template, provider session, or
provider response. A changed descriptor, adapter, scope, context authorization,
attempt binding, expiry, revocation, quarantine, or policy makes it invalid.

Leases do not carry context bytes or selection authority. A future Context
Assembly and Transmission Manifests gate must separately bind exactly what may
leave QuireForge. Leases also do not grant invocation: a future Limited Provider
Inference Boundary must separately validate the attempt, context manifest,
capability, adapter compatibility, and user-visible action immediately before
any allowed dispatch.

## Availability, authorization, and invocation are distinct

The following propositions are intentionally independent:

```text
credential reference observed or available
≠ account/project/scope grant
≠ adapter compatibility
≠ context-transmission authorization
≠ provider invocation approval
≠ retrieval authority
≠ native tool authority
≠ external mutation authority
```

For example, an API-key reference may be available while the selected adapter
is incompatible; a project grant may exist while no context is authorized; an
interaction attempt may be prepared while no invocation route exists; and a
provider session reference may exist while the account reference is revoked.
Every later route must revalidate all required current facts rather than infer
authority from any one of them.

## Revocation, expiry, renewal, rotation, and recovery

Revocation immediately invalidates dependent account/project/scope bindings,
leases, compatible-adapter conclusions, continuation bindings, pending
submissions, and future adapter access. It cannot be undone by a frontend retry,
project restore, provider session, model selection, or adapter rollback. An
expired reference or lease is unavailable until a future explicit renewal route
finishes with fresh, bounded validation.

Rotation creates a new reference version and invalidates prior leases and
dependent cached conclusions. Renewal never silently broadens scopes, endpoint,
account, project, adapter, or operation binding. A failed, interrupted, or
ambiguous rotation leaves the prior reference unavailable or quarantined unless
a future custodian contract independently proves the old reference remains
valid; it never guesses or falls back to ambient authority.

On application, broker, helper, host, or custodian restart, every pending
enrollment, validation, renewal, rotation, deletion, and lease becomes
unavailable until a later approved recovery route revalidates it. Recovery never
replays a credential action, reuses a secret, resumes a provider request, or
converts an ambiguous state into availability.

## No ambient authority

Future broker requests must not inherit authority from environment variables,
shell sessions, terminal processes, browser cookies or sessions, existing Codex
credentials, cloud CLI profiles, desktop keychains, inherited process state,
agent sessions, OS login state, or provider SDK defaults. These sources may be
classified by a future custodian decision but remain unavailable unless an
explicit user-authorized enrollment route, a compatible opaque reference, and a
fresh least-authority lease exist.

In particular, a local environment token, browser cookie, existing Codex login,
or cloud CLI session cannot be discovered, copied, proxied, imported, or treated
as a QuireForge credential reference. The broker never requests a password or
handles a raw secret in a project-facing, adapter-facing, bridge-facing, or
diagnostic-facing path.

## Audit and diagnostics

A future content-free audit projection may record opaque reference/lease,
adapter/provider/endpoint/account/project/scope bindings; lifecycle transition;
non-secret custodian class; requested operation class; attempt correlation;
policy outcome; timestamp; expiry; and a closed result/error classification.
It may distinguish enrollment requested, validation pending, availability,
lease issued/expired, rotation pending/completed, revocation, deletion,
quarantine, and recovery rejection.

Audit and diagnostic records must never contain raw secrets, provider account
identifiers, tokens, authorization codes, passwords, key material, cookies,
certificate data, provider payloads, context contents, task/plan/template
content, raw diagnostics, filesystem paths, URLs, shell text, environment
values, browser state, or opaque custodian handles. Aggregate telemetry, if
ever approved, remains opt-in, bounded, and content-free.

## Ambiguous failure handling

Enrollment, validation, refresh, renewal, rotation, revocation, deletion, and
custodian recovery failures are closed classifications, not reasons to assume
continued authority. Network loss, provider outage, custodian unavailability,
schema drift, unknown scope, clock ambiguity, partial success, cancellation,
timeout, or ambiguous remote acknowledgement yields unavailable, degraded, or
quarantined state as appropriate. No automatic retry may create an account
connection, duplicate a remote grant, rotate a secret twice, revive a revoked
reference, or dispatch a provider request.

The user must receive a bounded actionable state through a future approved UI;
this gate creates no UI or diagnostic surface. A later route may offer a fresh
explicit enrollment or validation attempt only after its own consent, transport,
and custody rules are approved.

## Relationship to completed decisions and separate lanes

The [capability registry](PROVIDER_NEUTRAL_CAPABILITY_REGISTRY_AND_DESCRIPTOR_GOVERNANCE.md)
continues to describe metadata and capability claims, not credentials or
availability. The [interaction protocol](CANONICAL_PROVIDER_NEUTRAL_INTERACTION_AND_EVENT_PROTOCOL.md)
continues to describe communication envelopes, not secret transfer or
invocation. [Adapter governance](PROVIDER_ADAPTER_LIFECYCLE_AND_CONFORMANCE_GOVERNANCE.md)
continues to restrict adapters to intelligence-traffic translation; an adapter
receives only a later approved temporary broker-mediated access path, never
credential custody.

M55 still governs durable source admission, provenance, retention, and
citations. M57 still provides least-authority binding, lifecycle, revocation,
confirmation, replay resistance, and content-free audit patterns without
creating a real connector route. M58 remains an independent, unstarted,
verification-only browser lane; this decision grants no OAuth, browser cookie,
browser session, DOM, screenshot, download, form-submission, automation, or web
research authority.

## Persistence and UI boundaries

This decision approves no secret storage, reference persistence, schema,
migration, command, bridge, UI, package, release, or host change. A later
implementation must separately choose custody technology, durable retention and
deletion rules, recovery mechanics, user-visible projections, and adversarial
tests. No current source contract implements this gate.

## Recommended next gate

The next recommended decision is **Context Assembly and Transmission
Manifests**. It must define selected context identity, source, transformation,
bounds, user-visible review, digest binding, transmission authorization,
retention, invalidation, and audit projection. It must not select a provider,
resolve a credential, transmit context, invoke a model, retrieve sources, or
execute a native operation.

## Explicit exclusions

This decision grants no credential implementation or storage; keyring, Secret
Service, wallet, database, file, environment-variable, browser-cookie, OAuth,
cloud CLI, provider-session, or secret-custodian integration; real account
connection; provider selection; networking; model invocation or inference;
context transmission; retrieval; native tool execution; browser behavior;
connector or MCP implementation; persistence schema or migration; Rust or
TypeScript contract; Tauri command; frontend bridge; UI; package; release; tag;
installation; host change; deployment; background work; automation; or
multi-agent behavior.
