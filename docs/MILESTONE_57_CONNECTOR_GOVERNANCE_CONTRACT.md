# M57 — Connector Governance Contract and Executable Implementation Plan

Status: ratified architecture and implementation contract; beta.60 fictional
local-only candidate implementation. This is the authoritative current M57
contract. It supersedes the *implementation deferral* in the earlier [M57
governance record](MILESTONE_57_CONNECTOR_GOVERNANCE.md) and
prerequisite/source-only closure records, while preserving their historical
private local-mock foundation unchanged. The separately approved beta.60 slice
implements this contract without broadening it; installation, release mutation,
publication, and deployment remain unapproved.

M55 Durable Source Admission is complete and published as `0.1.0-beta.59`.
M58 Controlled Browser Verification remains unstarted and out of scope.

## Ratified boundary

Projects and durable tasks are authoritative. A connector, account, credential
reference, remote resource, remote session, result, or provider claim is a
subordinate external reference and cannot create, select, broaden, restore, or
inherit QuireForge authority.

This contract governs provider-neutral connected-service references. It is not:

- a provider intelligence adapter, which translates only separately authorized
  intelligence traffic;
- generic MCP, which has no authority unless a later closed connector operation
  maps it into this contract;
- a native tool, terminal, shell, filesystem, Git, cloud, package, deployment,
  or credential-store authority; or
- retrieval, M55 durable-source admission, context inclusion, provider
  transmission, M58 browser behavior, or automation.

No category implies another. Connector retrieval does not admit an M55 source;
M55 admission does not retrieve through a connector; neither includes context
nor transmits it to a provider.

## Identity, descriptor, account, and scope

Each connector descriptor is immutable, native-validated metadata with a stable
opaque ID, connector class, semantic version, canonical digest, compatibility
declaration, closed capability declarations, and provenance/trust class.
Display labels are not identity or authority. Descriptors contain no endpoint,
executable instruction, arbitrary tool schema, secret, browser state, or
provider-supplied authority text.

| Class | Meaning | M57 status |
| --- | --- | --- |
| `local_mock` | Deterministic local fixture with no transport or secret | Eligible for beta.60 slice |
| `remote_read` | Named remote metadata/read/search/fetch class | Contracted only |
| `remote_mutation` | Named remote typed side effect | Contracted only |

Browser-mediated sources, generic MCP servers/tools, arbitrary HTTP APIs,
provider inference, retrieval engines, filesystem/terminal/Git/cloud tools,
automation targets, and deployment systems are excluded descriptor classes.

A connector binding is a native-issued opaque tuple of descriptor, project,
optional task, opaque account reference, effective-scope digest, optional opaque
credential reference, descriptor digest, lifecycle state, and expiry. It is
single-project and single-account. Descriptor drift, project archive, task
removal where task-bound, scope change, expiry, disconnect, revocation,
incompatibility, or quarantine invalidates it.

Credentials are availability references only. They do not grant connection,
read, mutation, context inclusion, provider transmission, browser, or
automation authority. Raw passwords, tokens, OAuth codes, API keys,
certificates, cookies, environment values, and secret-bearing diagnostics are
prohibited from metadata, source records, logs, audit, UI, and fixtures. A real
credential owner/custody route requires separate approval.

## Capability declaration and grants

A descriptor declaration is neither availability nor authority. Native policy
grants each operation separately, against the exact binding and descriptor
digest:

| Operation | Separate grant requirement |
| --- | --- |
| List/discover metadata | Explicit metadata grant |
| Read one named resource | Explicit read grant |
| Search | Explicit search grant with query/result bounds |
| Retrieve/fetch content | Explicit fetch grant, distinct from search/read |
| Create, update, delete, or action | Exact typed mutation grant and confirmation |

Grants declare locality: `local_only`, `network_read`, or `external_mutation`.
The first implementation may use `local_only` only. Read, search, list, and
retrieve never imply create/update/delete/action; a mutation grant never
silently broadens to another target or operation.

## Lifecycle, health, and recovery

Closed states are `unavailable`, `disconnected`, `unauthorized`, `ready`,
`prepared`, `confirmed`, `dispatched`, `succeeded`, `cancelled`, `expired`,
`revoked`, `degraded`, `quarantined`, `incompatible`, and `outcome_unknown`.
Only native code changes state. Health is an observation, not a grant.
Unavailable, expired, revoked, quarantined, incompatible, descriptor-drift,
and outcome-unknown states are non-callable and invalidate pending grants.
Recovery requires fresh native review; it never revives or replays a grant.

## Request, confirmation, dispatch, and outcome

Every operation follows this closed sequence:

1. **Prepare:** native validates descriptor/binding/lifecycle and produces a
   bounded typed request with project/task ownership, authority class,
   operation, target reference, bounded user input, limits, expiry, and digest.
2. **Review:** the UI shows a non-secret bounded projection: connector/class,
   project/task, requested authority, operation/target summary, limits, expiry,
   and digest. It submits only opaque handles and bounded authored choices.
3. **Confirm:** native consumes one expiring replay-protected authorization
   bound to that exact digest. Cancellation invalidates it without operation.
4. **Dispatch:** only a separately approved route may dispatch, after immediate
   revalidation. The beta.60 slice simulates locally and never opens a network.
5. **Observe/finalize:** native records a closed terminal result. Exit, timeout,
   or transport loss is never proof of an external result.

Mutation confirmation binds descriptor/version/digest, project/task,
account/scope, operation, target, proposed-payload digest, expiry, and nonce.
It is one-use even if dispatch fails. A future native-held idempotency key may
correlate a provider-supported attempt; it cannot transform ambiguity into
success or authorize a retry. `outcome_unknown`, partial completion, duplicate
indication, or post-dispatch timeout prohibit automatic mutation retry and
require a fresh user review.

## Bounds, audit, admission, and transmission

Every later list/search/fetch definition must set exact page-size, total-result,
byte, timeout, pagination-token, and diagnostic bounds. Pagination is an
explicit request, not background enumeration. Rate limiting and partial data
remain distinct from empty/success.

Durable audit records may retain only opaque connector/binding/project/task
references; descriptor version/digest; requested/effective authority/scope
digests; prepared/reviewed/confirmed/dispatched/terminal timestamps; closed
outcome; retention state; and evidence digest. They exclude raw request/response
content, URLs, paths, account IDs, provider request IDs, tokens, secrets,
browser state, and diagnostics. Disconnect/revocation removes callable binding
and credential reference, consumes pending authorization, and retains only
approved content-free audit metadata. It never claims a provider deleted data.

Connector data is not authoritative. It cannot enter M55, a context manifest,
or provider transmission without separate native decisions for retrieval
provenance, M55 admission, and context/transmission review.

## Generic MCP and excluded authority

Existing Codex-owned Integration Center MCP discovery/authorization remains
separate. Generic MCP cannot receive a connector binding, credential reference,
grant, mutation confirmation, source-admission route, or automatic dispatch
merely because it is callable elsewhere.

Connector state cannot open a browser, reuse browser state, start OAuth, access
filesystem paths, execute terminal/Git/cloud/deployment actions, select or
transmit provider context, invoke inference, schedule work, or automate action.
M58 remains wholly unstarted.

## Required fictional/local-only beta.60 vertical slice

The separately approved **Provider-Neutral AI Foundation — M57 Fictional
Connector Governance Vertical Slice** targets `0.1.0-beta.60` because beta.59
is the published generation. It is limited to:

- static non-executable `local_mock` descriptor/capability contracts and a
  deterministic fictional fixture;
- project/task-scoped opaque connector reference, inert credential availability
  reference, separate read/mutation declarations, and explicit grants;
- closed Tauri commands and strict Rust/TypeScript bridge validation for bounded
  state, prepare/review/confirm/cancel, and local observation;
- deterministic local read, deterministic mutation simulation, and an ambiguous
  mutation outcome with no automatic retry;
- drift, expiry, revocation, incompatibility, quarantine, replay,
  cross-project/task/scope mismatch, cancellation, and process-exit cleanup;
- accessible connector-governance UI: keyboard/focus, narrow layout, high zoom,
  bounded lists, and non-secret diagnostics; and
- fictional/local conformance fixtures plus source, browser, Rust, worker,
  packaging, restricted installed-host, lifecycle, smoke, provenance, checksum,
  ABI, and final-artifact gates.

It adds no network/DNS/HTTP, real connector/provider, credential/OAuth flow,
secret store, external read/write, browser/M58, MCP dispatch, automation,
background work, filesystem/shell/Git/cloud/deployment authority, retrieval,
M55 admission, context inclusion, provider transmission, or automatic retry.

## Separate approval remains required

The exact beta.60 fictional/local-only slice above has been separately approved
and implemented as a candidate. Real network read still requires named
provider/class/route, credential custody, retention/provenance, and retrieval
decisions. Real mutation additionally requires provider-specific
finality/postcondition approval. M58, generic MCP dispatch, browser behavior,
automation, and excluded native authority remain independent approvals. Beta.60
installation and installed-host validation require a separate bounded plan.
