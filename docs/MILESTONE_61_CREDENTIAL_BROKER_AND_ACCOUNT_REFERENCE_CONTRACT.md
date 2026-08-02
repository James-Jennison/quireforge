# M61 — Credential Broker and Account Reference Contract

Status: ratified decision-only contract within the active
[Provider-Neutral AI Foundation](GOAL_PROVIDER_NEUTRAL_AI_FOUNDATION.md). M61
defines the gates for a future credential-broker proposal; it does not select,
configure, contact, or implement a custodian, provider, local runtime, account,
or adapter. It supersedes the M61 planning role in the
[Provider-Neutral Foundation completion sequence](PROVIDER_NEUTRAL_FOUNDATION_COMPLETION_SEQUENCE.md)
where they differ.

M60 / `v0.1.0-beta.63` remains the latest completed published slice. M61 adds
no release, package, migration, runtime, or user-visible behavior.

## Ratified boundary

Projects and durable tasks remain authoritative. A future credential broker is
only a native-owned policy boundary for opaque non-secret references and
least-authority eligibility decisions. It is distinct from:

- QuireForge's ordinary metadata store, which is never a secret custodian;
- an operating-system or other future secure custodian, which may hold secret
  material only after a separately approved implementation decision;
- a future provider or local runtime, which cannot receive, select, or retain a
  credential merely because an opaque reference is compatible; and
- context assembly, transmission authorization, inference, retrieval, tools,
  browser, connector, MCP, automation, and mutation lanes.

Credential availability is not account selection, project or task authority,
adapter compatibility, context-transmission authorization, invocation approval,
or external authority. No category implies another.

## Future selection criteria

Before any custody or runtime implementation can be proposed, a separately
approved selection record must evaluate candidate approaches without collecting
or resolving credentials. It must demonstrate all of the following:

| Concern | Required criterion |
| --- | --- |
| Custody separation | Secret material is inaccessible to the metadata store, frontend, adapters, diagnostics, logs, exports, and ordinary application state. |
| Native mediation | Only a closed native policy surface can request a non-exportable opaque reference or lease; no ambient process, environment, browser, or SDK state is inherited. |
| Scope binding | A reference can bind exact provider-or-local-runtime class, adapter manifest, endpoint/deployment class where relevant, opaque organization/account, project, task when required, scope digest, operation class, and expiry. |
| Revocation and recovery | The custodian can support evidence-based revocation, expiry, invalidation, and fail-closed restart recovery without claiming deletion or validity it cannot prove. |
| Auditability | It can emit bounded content-free lifecycle evidence without exposing secret values, account identifiers, custodian handles, paths, URLs, or payloads. |
| Adapter containment | A compatible adapter can receive only a temporary, non-exportable, purpose-bound mediation result; it cannot enumerate, copy, select, refresh, or persist credentials. |
| Local-runtime parity | A credential-free local runtime remains selectable without introducing a credential broker dependency, while any authenticated local runtime must meet the same isolation rules. |

This contract selects no OS keyring, Secret Service, wallet, file format,
database, SDK, provider, endpoint, local runtime, protocol, or broker ABI.

## Opaque references and ownership

A future broker may expose only native-owned opaque identifiers. A non-secret
reference record may contain only its version/digest, custodian class,
credential-family class, compatible descriptor/adapter digests, bounded
availability classification, opaque organization/account/project/task/scope
references, lifecycle state, issued/observed/expiry timestamps, and
content-free audit linkage. Display labels are not identity or authority.

Every reference must be isolated to one explicit project and one effective
scope-set digest. Task binding is required whenever a future operation is
task-scoped. Cross-project, cross-task, cross-account, cross-organization,
cross-provider, cross-endpoint, cross-adapter, broadened-scope, unknown, stale,
revoked, quarantined, or incompatible use fails closed. Project restore,
template application, session recovery, provider metadata, or adapter output
cannot recreate or broaden a reference.

Raw passwords, tokens, API keys, authorization codes, refresh material,
certificates, cookies, private keys, connection strings, browser state,
environment values, provider account identifiers, and secret-bearing errors
are prohibited from references, leases, metadata, context, artifacts, plans,
reviews, logs, diagnostics, audit, exports, crash reports, fixtures, UI, and
source control.

## Lifecycle, invalidation, and recovery

The future lifecycle is closed and native-validated:

```text
unknown -> enrollment_requested -> reference_observed -> validation_pending
validation_pending -> available | unavailable | quarantined
available -> lease_pending -> leased -> expired | revoked | degraded | quarantined
available | leased -> rotation_pending -> available | unavailable | quarantined
any state -> revoked | deleted | quarantined
```

These states describe only local, bounded knowledge of an opaque reference.
They never prove a secret value, account login, provider reachability, context
authorization, or invocation outcome. Enrollment, validation, renewal,
rotation, deletion, and any custodian interaction require distinct future
approval and explicit user-visible consent.

Descriptor or adapter drift, scope change, project/task removal or archive,
account/organization change, endpoint/deployment change, expiry, revocation,
quarantine, failed validation, interrupted rotation, unknown clock state, or
restart invalidates dependent leases and compatibility conclusions. Recovery
starts unavailable and requires fresh, later-approved evidence; it never
replays an action, reuses a secret, retries an ambiguous operation, or promotes
an unknown state to available.

## Leases, adapters, and future compatibility

A future lease is native-issued, opaque, non-exportable, short-lived, and
single-purpose. It may be issued only after immediate revalidation of the exact
reference version, custodian class, compatible adapter manifest, descriptor and
endpoint/deployment class, opaque account/organization/project/task bindings,
effective scope digest, operation class, canonical interaction-attempt digest,
policy version, and expiry.

A future provider or local-runtime adapter is compatible only when it declares
and validates a closed descriptor/version/manifest, required custody class,
supported scope and operation vocabulary, endpoint/deployment constraints,
lease-consumption semantics, revocation/expiry behavior, bounded error classes,
and no-ambient-authority conformance. Compatibility is eligibility only. It
does not select a provider, connect a runtime, resolve a secret, transmit
context, invoke inference, or permit a tool, retrieval, browser, connector,
MCP, automation, or mutation action.

An adapter cannot receive raw material, inspect custodian state, hold a durable
lease, reconstruct a lease from an ID, fall back to environment variables or
browser/Codex/cloud sessions, or treat an ambiguous result as success. A future
invocation route must separately revalidate context authorization and the
canonical interaction attempt immediately before any dispatch.

## Audit and failure semantics

Future audit and diagnostics may retain only opaque reference/lease and
descriptor/adapter/project/task/scope correlations, lifecycle transition,
non-secret custodian class, bounded operation class, policy outcome, timestamps,
expiry, and closed error classification. Audit failure blocks a success claim.

Custodian unavailability, schema drift, scope ambiguity, timeout, cancellation,
partial completion, unknown acknowledgement, or restart produces unavailable,
degraded, or quarantined state. No automatic retry may enroll an account,
rotate a credential, restore a revoked reference, dispatch a provider request,
or inherit ambient authority. A later approved UI may offer a fresh explicit
operation; M61 creates no UI.

## Ratification evidence and exit criteria

M61 is complete only when its record:

1. preserves native ownership, opaque project/task-scoped references, and the
   separation of metadata, custody, adapter compatibility, context, and
   invocation;
2. defines non-selecting storage/runtime criteria and a closed lifecycle with
   revocation, expiry, invalidation, recovery, audit, and fail-closed semantics;
3. defines future adapter compatibility without naming or implementing a
   provider, local runtime, custodian, account, or protocol; and
4. is reviewed for the explicit exclusions below and linked from the current
   state, roadmap, goal, and completion sequence.

Any successor—whether a fictional M62 runtime contract or an implementation
proposal—requires a new, specific owner approval after this ratification. It
must present its scope, selected authority boundary, validation plan, retention
model, failure handling, and all external effects. M61 grants no automatic
successor.

## Explicit exclusions

M61 authorizes no credential collection, secret storage, keyring/Secret
Service/wallet integration, real account or provider selection, login, OAuth,
networking, context transmission, inference, retrieval, tool use, browser,
connector, MCP, automation, external mutation, persistence schema or migration,
native/frontend/runtime code, package, installation, deployment, tag, release,
or push.
