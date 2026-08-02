# M59 — Context Assembly and Transmission Contract

Status: ratified decision-only contract within the active
[Provider-Neutral AI Foundation](GOAL_PROVIDER_NEUTRAL_AI_FOUNDATION.md).
It authorizes no assembler, transmission, provider call, credential use,
inference, network activity, migration, bridge, UI, package, or release. It
supersedes the planning role of
[Context Assembly and Transmission Manifests](CONTEXT_ASSEMBLY_AND_TRANSMISSION_MANIFESTS.md)
where the two records differ; the earlier record remains historical architecture
evidence.

## Purpose and invariants

M60 must make the exact local material selected for one prospective intelligence
operation inspectable before any future destination receives it. Projects and
durable tasks remain authoritative. Plans, durable sources, reviews, connector
and browser evidence, provider sessions, and provider artifacts are subordinate
records. Eligibility is not selection; selection is not inclusion; inclusion is
not transmission; a provider capability or credential reference is not
authority.

Content from any source, page, connector, review, model, or provider is
untrusted data. It cannot become QuireForge or user instruction, change policy,
select itself, invoke a tool, or grant filesystem, terminal, shell, Git,
browser, connector, MCP, automation, deployment, credential, or mutation
authority. M55 admission, M57 grants, M58 verification, provider context, and
provider transmission remain separate decisions.

## Typed records and ownership

The later native implementation owns opaque IDs for `ContextItemSelection`,
`PreparedContextBundle`, `ContextReview`, `TransmissionAuthorization`,
`TransmissionAttempt`, and `ContextAuditEvent`. A prepared bundle binds one
immutable project and, when chosen, one immutable task; every selected item
must have the same project ownership and compatible task ownership. Provider
sessions/threads are opaque subordinate references and never reconstruct hidden
prior context.

A prepared bundle contains: bundle/project/task IDs; purpose; prospective
adapter/provider-target class and permitted model/capability class; selected
item identities, lifecycle revisions, content digests, exact ranges, source
classes, provenance, ordering, and redaction/truncation results; exclusion
reasons; canonical assembled private bytes or an app-private immutable
reference; item and bundle digests; policy and assembler versions; creation and
expiry; review/confirmation/attempt references; and content-free audit linkage.
Those fields are immutable. Later lifecycle and audit records reference the
bundle rather than changing it.

## Eligibility, selection, and ordering

Nothing is included by default. The native policy evaluates ownership,
lifecycle, deletion/revocation/quarantine/incompatibility, byte availability,
encoding, freshness, and sensitivity before a user can explicitly select an
item. Each selection records its exact source revision and allowed projection.
The review lists every requested exclusion and its reason; unavailable required
material fails preparation rather than being silently substituted.

| Class | Eligible basis | M60 support | Default |
| --- | --- | --- | --- |
| Project/task metadata | bounded title/status/identity projection owned by the selected scope | yes | excluded |
| Primary or explicit alternate plan | approved current plan owned by scope | yes | excluded |
| M55 manual text, local text file, reviewed artifact text | active, project/task-compatible private source | yes | excluded |
| Approved local review/package evidence | active bounded local evidence | yes | excluded |
| User-authored instruction | exact bounded value supplied for this bundle | yes | excluded |
| Conversation, provider response, session/thread | separately selected subordinate record | no | excluded |
| M57 fictional connector and M58 browser evidence | active evidence selected through their own future content-selection policy | metadata-only, no content in M60 | excluded |
| Future real connector/browser/provider evidence | separately ratified provenance and retention policy | no | excluded |
| Error/diagnostic evidence and provider extensions | sanitized bounded evidence, if later approved | no | excluded |

Stable order is: governing application instructions; explicit user instructions;
project/task metadata; plan; durable sources by selection order then immutable
ID; review evidence; and future evidence classes by their separately ratified
order. Duplicate byte-identical projections remain distinct attributed items
unless the user removes one; no deduplication silently changes provenance.

## Instructions, evidence, and canonical assembly

Only QuireForge governing policy and the explicit user instruction field are
instructions. Task goals and plans are labeled task/plan evidence; all sources,
reviews, connector/browser evidence, quoted content, and provider metadata are
labeled untrusted evidence. Serialization uses typed structural fields,
provenance labels, escaping, and length-delimited values—not delimiters alone.
Conflicting evidence is displayed as conflict, never resolved by content.

For identical selected bytes, ranges, policy, and target, assembly is
deterministic: UTF-8 only; Unicode NFC; LF line endings; canonical field order;
explicit empty values; versioned escaping; SHA-256 digests over canonical UTF-8
bytes. Binary, invalid encoding, missing bytes, unknown transformation, or
conflicting metadata fails closed. Per-item source digest and subset digest bind
the exact transmitted subset; the bundle digest additionally binds ordering,
policy, redaction, bounds, target, and assembler version.

## Bounds, minimization, truncation, and redaction

M60 has hard limits of 16 selected items, 24 KiB canonical bytes per item,
96 KiB total canonical bytes, 24,576 estimated tokens, 8 KiB user instruction,
12 KiB plan material, 16 KiB per durable source, and 8 KiB per review evidence.
Estimated tokens are advisory and deterministic (UTF-8 byte count divided by
four, rounded up); they never expand a hard byte limit. Limits are QuireForge
policy, not provider context-window claims.

Only the declared leading range after normalization may be partially included;
the review records byte/character ranges and omitted counts. Required material
that cannot fit fails preparation. Truncated material cannot prove a complete
assertion. Selection count, size, and excluded/truncated summaries are always
visible; no silent omission or replacement is allowed.

Credentials, keys, tokens, cookies, authorization headers, private keys,
passwords, connection strings, password-manager data, and credential references
are categorically prohibited. Structural redaction runs before storage or
review on configured secret patterns and bounded sensitive classes (personal,
health, financial, internal path/host/network, Git remote, and unsafe
diagnostic data). It produces fixed replacement markers and a content-free
reason/count record. M60 permits no override; a possible false negative is
warned, and a false positive may only be resolved by removing the item or a new
review after future policy change. Original redacted values never appear in UI,
audit, or support output.

## Closed lifecycle and authorization

The lifecycle is `proposed -> selected -> assembling -> prepared -> awaiting_review
-> awaiting_confirmation -> authorized -> dispatching -> accepted_delivery |
rejected_delivery | cancelled | denied | expired | revoked | drifted | timed_out |
ambiguous | failed -> closed`. Normalization, redaction, and budgeting occur
inside `assembling`; no bytes are final before `prepared`. Every terminal state
emits a content-free audit event. Invalid transitions fail closed.

Preparation and review have no transmission authority. Review may cancel;
denial, cancellation, expiry, revocation, drift, missing private bytes,
quarantine, incompatibility, target/model/policy change, project/task/plan
change, source deletion/revision, evidence revocation, or redaction/assembler
version change invalidates confirmation and blocks dispatch. Confirmation binds
the bundle digest, project/task, purpose, target class, model/capability class,
and policy version; it is expiring and one-use. It is consumed atomically when
the native controller changes `authorized` to `dispatching`, immediately before
the attempt reaches any sink. Replay, duplicate UI events, remounts, and restart
recovery cannot create another attempt.

`accepted_delivery` requires complete durable attempt/audit linkage. A definite
pre-dispatch failure leaves the authorization unconsumed only when the native
controller proves no sink was reached; otherwise it becomes consumed. Timeout,
process interruption, or any uncertain acknowledgement is `ambiguous`, never a
success and never automatically retried. Recovery closes an in-flight attempt
as ambiguous unless an idempotent future adapter can prove a terminal result;
M60 has no such exception.

| State | Entry and permitted next states | Bytes / authorization / audit / recovery |
| --- | --- | --- |
| `proposed` | native request has a valid project and optional task; may select or cancel | no finalized bytes or authority; proposal audit; restart closes it |
| `selected` | all requested identities are explicit; may assemble, alter selection, or cancel | no finalized bytes; selection audit; changed ownership returns to proposed |
| `assembling` | selected inputs validate; may prepare or fail/cancel | private transient bytes only; no review/authority; interruption removes staging |
| `prepared` | canonical digest, bounds, redaction, and immutable record validate; may review, expire, revoke, drift, or cancel | bytes finalized privately; preparation audit; restart preserves only valid TTL record |
| `awaiting_review` | user opens exact prepared record; may await confirmation, deny, cancel, expire, revoke, or drift | review required; no authority; review audit; restart returns to prepared |
| `awaiting_confirmation` | complete review is presented; may authorize, deny, cancel, expire, revoke, or drift | confirmation unused; audit; restart returns to awaiting review |
| `authorized` | matching explicit confirmation succeeds; may dispatch, expire, revoke, drift, or cancel | one-use authorization usable only here; audit; restart blocks it unless atomically proved unused |
| `dispatching` | native consumes matching authorization and begins only the permitted sink attempt | confirmation consumed; attempt audit; interruption/unknown acknowledgement becomes ambiguous |
| `accepted_delivery` | sink supplies complete definite acceptance; may close | immutable attempt evidence required; audit; recovery closes only after durable finalization |
| `rejected_delivery` / `failed` | sink definitely rejects or a named local failure occurs; may close | no success evidence; authorization state is recorded; audit and no retry |
| `cancelled` / `denied` / `expired` / `revoked` / `drifted` | matching terminal cause; may close only | no dispatch permitted; invalidates authorization; terminal audit and private-byte cleanup |
| `timed_out` / `ambiguous` | deadline or uncertain completion after possible dispatch; may close only | no success claim and no retry; consumed authorization; audit and recovery quarantine if needed |
| `closed` | every terminal record and required cleanup finalizes | no usable authority or live bytes beyond retention; closure/cleanup audit |

`quarantined` and `incompatible` are blocking policy classifications that may
be entered from selection through dispatching; they invalidate preparation or
block dispatch, create a named audit event, and reach `closed` without a retry.
No terminal state can transition back to a usable authorization.

## Review and transmission boundary

The M60 UI must show project/task, purpose, prospective fictional target,
every included item and provenance, ranges/sizes, redaction and truncation,
requested exclusions and reasons, total size, advisory token estimate, expiry,
and the statement that confirmation authorizes only one exact fictional
delivery. It uses accessible bounded scrollable views and never reveals secret
originals or raw internal IDs as ordinary UI content.

M60's typed sink receives only canonical bundle bytes and the opaque attempt/
target class; it is deterministic, fictional, local-only, and non-networked.
It has no credential, provider session, inference, connector dispatch, browser,
MCP, automation, or external mutation path. Future M62–M64 must separately
ratify wire metadata, credential leases, session references, streaming,
cancellation, rate limits, real provider rejection, usage, and response
handling. No automatic retry follows ambiguity.

## Persistence, retention, drift, and failure

M60 must use forward-only transactional migrations. Prepared bytes are stored
only app-private, mode-restricted, atomically staged/finalized, and retained for
at most 30 minutes when unconfirmed; confirmation/terminal closure deletes
unneeded assembled bytes. Immutable metadata/audit tombstones retain only
ownership, digests, lifecycle, and reason. Count and byte ceilings are enforced
before write. Deleting a source after preparation invalidates it; it does not
rewrite the prepared record or silently retain/reconstruct content.

Assembly, redaction, storage, audit, or cleanup failure; invalid encoding;
required-content overflow; target incompatibility; cancellation; timeout; and
restart all return bounded named failures with no false success, implicit retry,
or orphan authority. Audit failure blocks success finalization. Quarantine
blocks future selection and dispatch. Cleanup failure is itself audited and
closes safely.

## Security, testing, and M60 completion

Threats include prompt injection/instruction smuggling, source impersonation,
secret/path leakage, Unicode confusables, semantic truncation, denial through
oversize inputs, cross-project/task leakage, stale/deleted evidence, digest or
target substitution, replay/races, audit tampering, ambiguous completion, and
provider retention/training differences. Page, connector, source, review, and
provider content remains untrusted evidence throughout.

M60 must test fresh/upgrade migrations; ownership; eligibility/default
exclusion/selection; ordering, normalization, duplicates, bounds, truncation,
redaction, prohibited secrets, attribution, and determinism; immutable prepared
bundles and accurate review; confirmation/cancel/deny/expiry/revoke/replay/
duplicate/remount/restart/drift; fictional success, definite failure and
ambiguity without retry; audit/retention/cleanup; cross-project isolation; and
no M55 admission, M57/M58 authority, credentials, real provider/network,
inference, MCP, automation, or mutation. It must also pass accessibility,
viewport/scaling, M55–M58 regression, packaging, and installed-host acceptance.
Completion requires those gates plus a fictional local-only sink and no real
transmission.
