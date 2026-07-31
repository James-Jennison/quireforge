# Milestone 57 — Connector Governance and External Authority

Status: complete decision. This is the authoritative M57 governance record. It
created no connector, provider client, retrieval, browser, MCP, OAuth,
credential, storage, command, UI, schema, package, or network behavior. The
later separately approved local mock-only foundation is source-only closed by
[M57 source acceptance and release policy](MILESTONE_57_SOURCE_ACCEPTANCE_AND_RELEASE_POLICY.md);
it does not expand this record into real external authority.

## Decision and current baseline

QuireForge remains a local application whose ordinary SQLite metadata is not a
credential store and whose task, plan, template, artifact, and review records
do not confer external authority. M52 owns local task and plan records; M56
owns only local templates and bounded, digest-only application reservations.
M55 research-report implementation remains deferred because durable
source-manifest authority is not approved.

The existing Integration Center and ADR 0020 are a distinct, supported
Codex-owned boundary: they normalize a catalog and permit only fixed native
authorization or enablement handoffs for capability-ready entries. Codex and
the operating system retain integration and credential state; opaque IDs and
short-lived native confirmations are not a general provider API or a
per-project connector grant. M57 neither expands, revokes, nor reclassifies
that existing boundary.

No new external authority is approved here. In particular, M57 does not
approve direct provider clients, generic MCP tools, connector installation,
OAuth/token exchange, credential custody, browser-mediated login, live
retrieval, synchronization, imports/exports, polling, webhooks, automatic
actions, approval, dispatch, or execution.

## Threat and authority model

An opaque record selector identifies a locally held record; it never proves
ownership, account access, provider scope, freshness, or permission. Provider
claims and connector observations are untrusted until a future native boundary
validates their closed shape, binding, and freshness. A future request must
separately establish all of these authorities:

| Authority                                                     | M57 disposition                                                                                                |
| ------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| Installation or registration                                  | Later supply-chain and transport decision; never implicit in authorization.                                    |
| Account authorization                                         | Potentially considered only through an approved, native-owned account-binding design.                          |
| Credential/token custody                                      | Deferred; never ordinary SQLite, task, plan, template, artifact, log, export, crash report, or prompt content. |
| Metadata discovery                                            | Possible future lowest-risk read class, but not approved for implementation.                                   |
| Read, search, or content fetch                                | Separate bounded operation and provenance contract required.                                                   |
| Write, mutation, or destructive action                        | No standing authority; each potential action requires a later closed, one-time confirmation design.            |
| Approval, dispatch, or execution                              | Categorically excluded from connector authority.                                                               |
| Background, recurring, cross-project, or cross-account access | Prohibited unless a later decision explicitly approves each distinct case.                                     |
| Browser-derived or ambient-browser authority                  | Prohibited; M58 is a separate unstarted decision gate.                                                         |

## Connector classification

Classes describe risk, not vendor approval. No provider, endpoint, plugin, or
MCP server is approved merely by fitting a class.

| Class | Exposure and side effects | Minimum future visible authority and provenance | Background / approval / dispatch |
| --- | --- | --- |
| Local filesystem or local-application source | Private local content and local mutation risk | Explicit source and project binding; local object identity and digest | No background; never approval or dispatch. |
| Read-only remote data source | Account-scoped metadata or content leaves/returns through a provider | Explicit account, requested scope, source identity/revision, retrieval time, and retention disclosure | No background by default; never approval or dispatch. |
| Remote mutation source | External data plus durable third-party side effects | Exact proposed operation, target, scope, and digest-bound one-time confirmation | No background mutation; never approval or dispatch. |
| Communication provider | Recipients, messages, attachments, and social consequences | Account, recipient/target class, content boundaries, and irreversible-side-effect disclosure | No background send; never approval or dispatch. |
| Project or issue tracker | Project metadata, issues, and potentially write actions | Explicit project/account binding and object revision | No background write; never approval or dispatch. |
| Source-control provider | Repository identity, source, and mutation risk | Explicit repository/account binding, revision, and exact proposed operation | No background write; never approval or dispatch. |
| Calendar or scheduling provider | Sensitive time, attendee, and future-action data | Explicit account, calendar, attendee/target class, and operation disclosure | No recurring access or action; never approval or dispatch. |
| Browser-mediated source | Cookies, rendered content, redirects, and ambient authority | Not eligible under M57; any evidence is only an unverified browser observation pending M58 | Prohibited. |
| Generic MCP or tool provider | Arbitrary tool schemas, data, and side effects | Not eligible for generic enablement; each tool class needs a closed future contract | Prohibited. |

Revocation must remove callable authority immediately, invalidate pending
previews/confirmations, prevent new operations, and show the affected state.
It does not silently erase independently retained local records; a future
retention decision must describe those records and their cleanup.

## Authority ladder

Future connector lifecycle is fail-closed and least-authority:

1. **Known but unavailable** — display-only normalized metadata; no operation.
2. **Installed but disconnected** — no account, read, or mutation authority.
3. **Connected but unauthorized** — connection observation only; no access.
4. **Authorized for metadata discovery** — explicitly bounded account/project
   metadata only, subject to a later contract.
5. **Authorized for bounded read** — one named source/object class, explicit
   binding and retention disclosure.
6. **Authorized for bounded search and fetch** — separately named search,
   result, and fetch limits with provenance for every retained result.
7. **Proposed narrowly scoped mutation** — native preview binds target,
   account, scope, exact proposed operation, and fresh evidence.
8. **Confirmed one-time mutation** — one digest-bound, expiring,
   replay-protected confirmation; native revalidation immediately precedes the
   side effect.
9. **Revoked, expired, degraded, or quarantined** — no operation; pending
   authority is consumed or invalidated and a closed reason is visible.

There is no standing write authority, autonomous mutation authority, or
authority inheritance from templates, plans, tasks, projects, advisors, or
generated content. Scope changes require fresh authorization; any changed
binding, account, target, provider revision, or draft requires a new preview.

## Consent and confirmation

A future implementation must require explicit connector and account selection,
then show human-readable requested scopes before authorization. Read authority
and write authority must be visibly distinct. A destructive or externally
mutating operation needs its own confirmation after an authoritative preview;
the confirmation is one-time, expiring, replay-protected, cancellable before
side effects, and binds the exact operation/target/draft by digest or an
equivalent canonical binding. The frontend may submit only opaque native
selectors/handles and user-authored bounded values. It may not assert scope,
ownership, account, expiry, provider state, capability, or completion.

No confirmation may silently expand scope, retry a mutation, or become valid
again after revocation, expiry, account/project change, or a failed
postcondition. A template, task, plan, project, Advisor item, or generated
content can describe a proposed operation but cannot authorize it.

## Credential governance

QuireForge must never receive or collect raw passwords. Secret values — OAuth
tokens, API keys, certificates, cookies, local-socket credentials, helper
credentials, authorization codes, and provider refresh material — must never
enter ordinary project data, SQLite metadata, task/plan/template text, logs,
exports, artifacts, crash reports, diagnostics, or generated prompts. They
must not be displayed, copied, or exported.

If a future supported route requires secret custody, the secret must remain in
an approved OS or external secret store or in the provider/Codex-owned custody
model; this milestone selects no technology. In-memory handling must minimize
duration and exposure, and all serializable error/event paths must redact
secret-bearing fields. If secure custody, rotation/expiry handling, revocation,
or disconnect cannot be supplied, the connector remains unavailable. Rotation,
expiration, revocation, and provider disconnect invalidate relevant authority
and never fall back to a less secure store.

## Provenance, privacy, and project binding

Every future retained external observation must have a bounded, inspectable
provenance envelope containing connector class/identity, provider identity when
applicable, a privacy-preserving account binding, project binding, retrieval
time, source-object identity, source revision/ETag/digest when available,
requested scope, actual operation, content treatment (fetched, summarized,
transformed, or referenced), local evidence digest, and stale, deleted,
unavailable, or unverifiable state. Missing or conflicting provenance makes
the item unavailable rather than inferred or repaired.

The envelope must distinguish provider claims, connector observations,
user-supplied facts, locally generated interpretations, verified local
artifacts, and browser-rendered observations. Browser-rendered content is not
authoritative. M57 does not grant M55’s durable source-manifest authority.

Future requests must bind exactly one active local project and one selected
account where applicable; cross-project and cross-account lookup is rejected.
Only the explicitly active context may display connector output. Users must be
told what data leaves the machine, why, and the intended retention boundary.
Data minimization is mandatory. Multi-user/shared-host operation is unavailable
unless a later decision defines OS-user isolation, account separation, and
retention safety. Connector revocation never grants continued access to local
task, plan, artifact, or template data.

## Audit, failures, and revocation

A future audit record may retain bounded lifecycle event type, connector class
and opaque identity, privacy-preserving account/project binding, requested and
actual operation classes, confirmation state, outcome, time, and an evidence
digest. It must make authorization/revocation, scope change, read/search/fetch,
proposed/confirmed mutation, provider failure, stale/replay rejection,
background-access attempt, policy denial, and retention cleanup inspectable.
Content, secrets, raw provider responses, URLs, paths, tokens, raw errors, and
private account identifiers must never be logged. Aggregate telemetry must be
content-free and opt-in under a later decision.

Network failure, outage, authentication expiry, scope reduction, token
revocation, account removal, uninstall, rate limit, schema change, redirect,
ambiguous response, provenance mismatch, local clock ambiguity, and degraded
provider state all fail closed. Partial fetches are labelled partial and never
presented as complete. Partial or ambiguous mutations require an unavailable
result and fresh user review; no automatic retry may create a duplicate side
effect. Provider postconditions must be independently revalidated where a
future supported route permits it. Integrity or provenance failure quarantines
the connector until an explicit later recovery rule exists.

## M58 boundary and exclusions

M57 may classify browser-derived material only as unverified observation. It
does not implement or approve a browser controller, DOM inspection,
screenshots, downloads, form submission, ambient browser authentication,
credential capture, cookie reuse, or browser automation. M58 remains a
separate, unstarted high-risk decision gate.

This milestone also excludes providers, retrieval, external API calls,
synchronization, imports/exports, polling, webhooks, background actions,
connector implementation, schema changes, package changes, release changes,
and deployment. Existing M56 exclusions remain intact.

## Future implementation envelope and recommendation

**Recommendation: require additional decision artifacts before implementation.**
The existing Codex Integration Center boundary establishes that normalized,
native-owned, closed handoffs are possible, but it does not define a durable
per-project/account binding, a source-provenance envelope, secret-custody
technology, retention/deletion lifecycle, or external-mutation postcondition
contract. A future narrowly bounded connector-foundation proposal must first
select one read-only connector class, one supported route, a closed account and
project binding, source admission/provenance fields, retention/removal rules,
native-only custody, strict command/bridge schemas, audit projections, and
adversarial tests. It must separately prove that no generic tool, browser, or
standing write authority is introduced.

Before any mutation-capable class, a second decision must define canonical
operation bindings, digest/expiry/replay semantics, preflight and postcondition
rules, ambiguity handling, and explicit destructive confirmation. Browser
verification remains M58 and must not be combined with either proposal.

Unresolved decisions are the approved secret-store technology (if custody is
ever necessary), privacy-preserving account-binding representation, retention
periods, permissible read-only class and route, source-manifest admission, and
whether any external mutation can ever meet the required verification bar.
Until they are separately approved, connector implementation remains deferred.

## Later acceptance criteria

A later implementation milestone must demonstrate a named approved class and
route; closed native request/response schemas; selector/account/project
revalidation; least-privilege lifecycle; explicit scope/retention disclosure;
redaction; provenance and audit inspection; revocation/expiry/replay handling;
failure/partial-result behavior; strict tests for cross-project/account and
authority escalation; and accessible user-visible state. It must prove that
secrets, raw diagnostics, browser state, automatic actions, approval, dispatch,
and execution cannot cross its boundary. It requires separate explicit approval
and may not infer authority from this decision.
