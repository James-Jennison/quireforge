# Post-M57 Connector Foundation Prerequisite Decisions

Status: complete decision checkpoint. A narrowly bounded connector-foundation
implementation is approved as a future, unstarted milestone only. This record
creates no connector, provider client, network access, OAuth, credential store,
schema, command, bridge, UI, package, or runtime behavior.

## Prior authority and scope

[M57](MILESTONE_57_CONNECTOR_GOVERNANCE.md) remains the governing authority
ladder and exclusion set. The existing Codex-owned Integration Center and
[ADR 0020](DECISIONS/0020-confirmed-integration-authorization-and-controls.md)
remain unchanged: they are supported fixed Codex handoffs, not a generic
provider or per-project connector boundary. M55 keeps research-report and
durable source-manifest implementation deferred. M56 local task templates do
not carry connector, provider, credential, approval, dispatch, or execution
authority. M58 browser verification remains separate and unstarted.

This checkpoint resolves M57's prerequisites only sufficiently to approve a
future **local, no-network connector-foundation** proposal. It does not approve
an external connector implementation, provider selection, live read, search,
fetch, mutation, browser handoff, or secret custody.

## Prerequisite inventory and disposition

| M57 prerequisite                            | Why it blocked implementation                                                               | Affected classes/states                | Decision here                                                                                                                                       |
| ------------------------------------------- | ------------------------------------------------------------------------------------------- | -------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Execution and ownership boundary            | A generic caller could inherit shell, browser, Codex, or provider authority.                | All classes; every callable state.     | Resolved for the foundation: a future native connector authority service owns lifecycle validation only; no process executes an external operation. |
| Secret-store technology                     | Ordinary SQLite and content records are prohibited secret stores.                           | All authorized external classes.       | Deferred for real connectivity; not needed by the no-credential foundation. Any custody model requires a separate approved broker decision.         |
| Privacy-preserving account identity         | Raw provider account identity would leak private data or create cross-account confusion.    | Remote classes and authorized states.  | Resolved as an opaque, native-issued account reference contract with bounded display label; real account admission remains deferred.                |
| Project/account/scope binding and lifecycle | Selectors alone cannot prove ownership, current scope, or revocation.                       | Metadata through mutation states.      | Resolved as an explicit single-project, single-account, single-scope binding contract.                                                              |
| Read-only class and route                   | A general tool or arbitrary route would create uncontrolled data/transport authority.       | Metadata/read/search/fetch.            | Resolved only for static local descriptors and mock adapters; no remote route or provider is selected.                                              |
| Retention and provenance model              | Unbounded fetched data or logs would defeat privacy and M55 evidence boundaries.            | Read/search/fetch and audit states.    | Resolved for the foundation as no retained provider content; bounded metadata-only audit/provenance contracts only.                                 |
| M55 source admission                        | Connector output cannot become a research source by implication.                            | Fetched content and generated reports. | Deliberately unresolved and not a foundation dependency; M55 remains deferred.                                                                      |
| External mutation finality                  | Ambiguous side effects cannot safely be retried or represented without a provider contract. | Mutation states.                       | Resolved as a required contract shape, but external mutation remains deferred pending a separate provider-specific decision.                        |
| Descriptor registration and trust           | Mutable or executable descriptors could smuggle authority.                                  | Known/installed states.                | Resolved for the foundation as static, inspectable, non-executable local descriptors only.                                                          |

The risks of deciding these incorrectly are confused-deputy execution, secret
exposure, cross-project/account disclosure, duplicate irreversible mutations,
and treating unverified provider material as durable evidence. Deferral leaves
connectors unavailable; it does not weaken existing local features.

## Process, execution, and credential custody

The future foundation has one native authority owner: a bounded Rust connector
authority service behind strict typed commands. It may create and validate
opaque lifecycle, project, account, scope, descriptor, and operation references
using local fixture/mock data only. React is presentation only and supplies no
identity, scope, lifecycle, provider claim, expiry, or authority fact.

No component executes an external connector operation in the approved
foundation. QuireForge may not directly invoke a provider, helper, MCP tool, or
browser. It may not inherit ambient authority from the shell, environment,
browser session, existing Codex session, installed plugin, or provider process.
The operating system/Codex/provider may remain owners of their existing state,
but that state is not callable through the foundation.

For future real connectivity, a separately approved native credential-broker
contract is mandatory. It must identify a credential owner and make
registration, account binding, scope grant, revocation, operation execution,
and audit-result reporting separately authoritative. QuireForge may receive
only an opaque credential reference, never a secret or secret-bearing error.
The reference is bound natively to connector descriptor, provider, opaque
account reference, project, effective scope digest, lifecycle state, and
optional expiry. It exposes at most a bounded non-secret availability state and
display label. A missing, expired, revoked, scope-changed, or unavailable
reference makes the operation unavailable; it never falls back to SQLite,
project data, prompts, logs, or another credential source.

Raw passwords, OAuth tokens/codes, API keys, certificates, cookies, socket
credentials, helper credentials, and provider refresh material remain
prohibited from SQLite, project data, tasks, plans, templates, artifacts,
prompts, diagnostics, exports, crash reports, and audit records. Codex-owned,
OS-owned, provider-owned, and external secret stores each require a separate
custody decision before use.

## Binding, lifecycle, and recovery contract

A future binding has opaque native-issued references for descriptor, provider,
account, project, scope, credential (if any), and lifecycle record. Discovery
may be global display metadata only; every authority-bearing action binds one
active project and one selected account. One account may be represented in
multiple projects only by distinct per-project grants; grants cannot be shared,
inferred, or cloned. A project may have multiple accounts for a provider only
when the user explicitly selects one per operation. Account inspection uses a
native privacy-preserving reference plus a bounded user-visible label, never a
raw provider identifier.

No task, plan, template, review, artifact, generated instruction, or imported
content can create, broaden, restore, or select a binding. Project archive
invalidates callable authority; restore does not revive it. Disconnect,
descriptor removal, credential expiry, provider-object deletion, scope change,
or revocation invalidates pending operations and requires a fresh grant. Local
records retain their existing local evidence but cannot regain external access.

The required lifecycle is: known/unavailable; installed/disconnected;
connected/unauthorized; metadata-authorized; bounded-read authorized;
bounded-search/fetch authorized; pending mutation; confirmed/not dispatched;
dispatched; completed; expired; revoked; degraded; quarantined; and removed.
Only explicit, revalidated transitions may advance authority. Restart, host
restart, local restore, network loss, provider outage, token expiry, scope
reduction, descriptor change, or schema change returns the affected binding to
unavailable, degraded, revoked, or quarantined; pending authority never
silently survives or becomes reusable.

## Closed operation and result contract

No generic tool invocation is approved. A later provider-specific proposal may
instantiate only these named operation classes: discover availability; list
authorized accounts; read metadata; search; fetch content; fetch an attachment
or artifact; propose/confirm/execute one mutation; report/reconcile an outcome;
and revoke/disconnect.

Every instance must declare required authority level, one project/account/scope
binding, credential reference where applicable, bounded user-authored input,
timeout/cancellation semantics, retention rule, and a closed provenance/result
projection. It must never serialize secrets, raw provider payloads, URLs,
paths, unnecessary source content, or raw diagnostics. Metadata authorization
is distinct from content access; search is distinct from fetch; attachment or
artifact fetch requires a separate explicit authority. Bulk retrieval is
prohibited by default.

The foundation may model only unavailable, ready, rejected, cancelled, expired,
revoked, degraded, quarantined, and mock-completed results. It performs no
dispatch. A real contract must additionally distinguish succeeded, dispatched
outcome-unknown, partially completed, externally duplicated, externally rolled
back, and irreversible results without guessing from a timeout.

## Mutation safety, reads, retention, provenance, and audit

All future external mutations require an explicit one-time confirmation after a
native preview. Its canonical binding includes descriptor, provider, account,
project, operation type, target object reference, proposed-payload digest,
effective-scope digest, expiry, and a native nonce. Destructive operations need
a separately labelled destructive confirmation. Confirmation is cancellable
before dispatch, expires, is consumed once, and is replay-protected. Providers
that support idempotency require a native-held idempotency key; providers that
do not support it may not receive automatic retry after dispatch. An ambiguous
or partial dispatch is unavailable for automatic repair and requires fresh
user review under a provider-specific decision.

The foundation retains no external content, snippets, attachments, artifacts,
summaries, or research-source material. A future read/search/fetch decision
must set strict count/size/time/scope limits, visible fetch action, staleness,
deletion, and retention rules. Retained evidence remains historical local data
only when its later contract says so; revocation/access loss makes it stale or
unavailable and never refreshes it. Summaries remain visibly distinct from
source content. Nothing here admits a M55 source manifest or research report.

The minimum future inspectable audit/provenance projection is connector class
and descriptor version/digest; provider identity; privacy-preserving account
and project references; operation class; requested/effective scope digests;
opaque credential-reference state; request/dispatch/completion times; source
object and revision/digest/ETag where available; mutation confirmation digest;
closed result/error classification; retention state; and revocation/quarantine
state. Durable storage, if later approved, may hold only those bounded
non-secret fields and local evidence digests. Aggregate-only data may cover
content-free counts/timings. Raw payloads, content, secrets, URLs, paths,
private account identifiers, tokens, raw errors, browser state, and provider
request identifiers unless separately proven safe must never be logged.

## Descriptor registration and trust

The first foundation may know only static, repository-defined, non-executable
local descriptors used with fixtures/mocks. Each descriptor has a stable
identifier, class, provider label, semantic version, canonical digest,
compatibility declaration, closed operation declarations, and declared scope
classes. Descriptors contain no executable instructions, endpoints, secrets,
arbitrary tool schema, or provider-supplied authority text. A descriptor change
invalidates its bindings and requires a new review; it cannot silently preserve
account authorization.

Built-in descriptors and any future locally installed, signed,
helper-advertised, or provider-supplied descriptor each require separate
supply-chain, transport, compatibility, and renewal decisions. Generic MCP,
arbitrary tools, and executable descriptor registration remain excluded.

## Rejected alternatives and explicit exclusions

Rejected alternatives are direct QuireForge provider calls; ordinary SQLite
secrets; global account grants; reusable/standing write authority; provider
payload logging; automatic refresh/retry; ambient browser/Codex/shell authority;
generic MCP/tool contracts; and treating connector reads as M55 research
sources. They are rejected because they collapse distinct trust boundaries or
make side effects and provenance uninspectable.

M58 is untouched: no browser control, cookie reuse, credential extraction, DOM
inspection, screenshots, downloads, form submission, automation, or rendered
content verification is approved. This checkpoint also excludes OAuth,
providers, network access, retrieval, synchronization, import/export, polling,
webhooks, background jobs, schema changes, commands, bridges, UI, packages,
versions, installation, tags, releases, publication, and deployment.

## Future implementation acceptance and recommendation

**Recommendation: approve one proposed, unstarted local connector-foundation
milestone.** Its exact envelope is static inspectable non-executable local
descriptors; native-only opaque descriptor/project/account/scope/credential-
reference lifecycle contracts; closed operation, confirmation, result,
provenance, and content-free audit models; fail-closed transition/recovery
validation; and deterministic in-memory fixtures/mock adapters. It must have
no network calls, real credentials, OAuth, secret storage, provider SDKs,
browser authority, external mutations, background activity, generic MCP
execution, fetched-content retention, or M55 source-manifest behavior.

The future milestone remains unstarted and needs its own explicit approval.
Real read/search/fetch requires a later named provider/class/route, source
admission, retention, and credential-broker decision. Any mutation requires a
second provider-specific finality/postcondition decision. Remaining unresolved
items are secret-store technology if real custody is ever needed, the first
remote provider/class/route, durable retention periods, M55 source admission,
and whether an external mutation can meet the required verification bar.

Acceptance requires strict native/bridge contracts with unknown-field denial,
adversarial tests for binding/revocation/replay/recovery/redaction, inspectable
closed lifecycle/audit projections, and proof that no excluded authority can
cross the boundary. M55 and M58 remain independently gated.
