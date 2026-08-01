# External Capability Taxonomy and Sequencing

Status: complete decision-only architectural ordering checkpoint. This record
grants no implementation authority and creates no package, version, tag,
release, migration, bridge, command, UI, provider, connector, browser,
credential, or network behavior.

## Authority and purpose

M57 is complete and published as beta.60 at
`b8b807f256170e6a35ada22893b410cb4b0057b7`; its deterministic fictional
connector is local-only and grants no real external authority. M58 Controlled
Browser Verification planning is complete through its ratified contract;
runtime remains unstarted. The all-in-one workspace direction is
non-authorizing north-star guidance, not approval for any external capability.

This checkpoint orders future decisions without selecting a provider or an
implementation. “External capability” is too broad to govern as one category:
intelligence transport, information retrieval, connected-service access, local
runtime use, credential custody, browser verification, and autonomous behavior
have materially different authority, privacy, transport, persistence, audit,
failure, mutation, retention, recovery, and consent requirements.

The M57 local mock contract is reusable architectural evidence for closed
identity, lifecycle, binding, confirmation, and audit shapes. It does not mean
that every future capability is an ordinary connector, or that replacing its
`LocalMock` adapter with a network adapter would be authorized.

## Capability lanes

The following lanes are independent authority categories. Fitting a capability
into a lane does not approve it.

### A. Inference providers

Inference providers are services or runtimes that receive explicitly authorized
context and produce model output. They may eventually encompass remote cloud
inference, model-hosting platforms, provider-managed sessions, structured
output, and multimodal model input or output. This lane does not inherently
include retrieval, native tool execution, connected-service authority,
credential custody, browser control, or durable source admission.

### B. Retrieval providers and source systems

Retrieval capabilities locate, fetch, search, rank, or return external or
indexed information. Conceptual categories include web retrieval, provider file
search, local knowledge indexes, vector search, reranking, and connected
document sources. This lane remains subject to M55’s durable source identity,
provenance, retention, and citation boundary. Inference never implies retrieval.

### C. Connected services

Connected services are named external systems with account, communication,
project, storage, infrastructure, or business operations. Source control, mail,
calendar, cloud infrastructure, messaging, storage, and deployment services
are examples of classes, not selected vendors. Every real service requires a
provider-specific decision and a closed operation catalog. No generic connector
or MCP interface grants standing authority across this lane.

### D. Local runtimes

Local runtimes are locally hosted model or processing runtimes that may not
need a cloud credential. They remain separate from QuireForge native authority,
shell or terminal execution, sandbox workers, and external inference providers.
A local model endpoint is intelligence transport, never automatic host
authority.

### E. Execution targets

Execution targets can change local or remote state: filesystem mutation,
terminal or process execution, Git operations, build systems, deployments, and
cloud-resource mutation are conceptual examples. Execution remains owned by
QuireForge native services, with closed typed project-bound operations and
confirmation. Inference providers and connected services do not inherit it.

### F. Credential authorities

Credential authorities are native or platform-backed systems responsible for
credential custody, issuance, leasing, rotation, expiry, and revocation. They
may eventually address API-key references, OAuth tokens, cloud credential
chains, local endpoint authentication, and provider account bindings. Existing
Codex-owned credentials, browser cookies, environment variables, and ambient
host sessions are not QuireForge credential authority.

### G. Browser verification

Browser verification is a separately governed, verification-only browser
surface. Its M58 decision contract is complete; runtime remains unstarted and
is distinct from
general browsing, research retrieval, OAuth, connected-service sessions,
browser automation, web agents, external mutation, and inference transport.

### H. Automation and autonomous coordination

Automation covers scheduled, conditional, parallel, multi-agent, or unattended
behavior. It amplifies every other authority and remains last in the dependency
sequence. It cannot be authorized indirectly through inference, connectors,
MCP, browser behavior, templates, or background services.

## Cross-lane invariants

- Projects and durable tasks remain authoritative over provider sessions or
  threads.
- Provider adapters translate intelligence traffic only; they receive no
  implicit native tool authority.
- Retrieval results do not become durable sources without approved source
  admission.
- Local runtimes receive no implicit filesystem, terminal, Git, browser,
  credential, connector, or deployment authority.
- Connected services expose named, closed operations only.
- Execution remains owned by QuireForge native services.
- Credentials are resolved only through a separately approved custody model.
- Browser state, cookies, and ambient sessions are never implied credentials.
- MCP is not a generic authority layer and cannot bypass lane-specific
  governance.
- Mutations require digest-bound authorization and immediate revalidation;
  ambiguous dispatch is never automatically retried.
- A canonical superset with governed provider-specific extensions preserves
  provider behavior; a lowest-common-denominator API must not erase it.
- Inspectable templates remain distinct from hidden agents, instructions, and
  authority-bearing plugins.
- Audit records distinguish proposals, authorization, dispatch, outcome,
  failure, revocation, and ambiguity.
- No lane automatically authorizes another lane.

## Dependency map

### Provider-neutral inference

Before real remote inference, later decisions must establish:

1. capability registry and descriptor governance;
2. a canonical interaction and event protocol;
3. provider-adapter lifecycle and conformance;
4. credential broker and account binding;
5. context assembly and transmission manifests;
6. retention, deletion, privacy, and provenance policy; and
7. a limited-inference boundary.

Durable retrieval and source admission are not prerequisites for an initial
limited-inference boundary only where input is explicitly selected, transient,
user-authored, or already an approved local projection.

### Retrieval and cited research

Before durable retrieval or cited research, later decisions must establish
M55-compatible durable source identity and admission, retrieval-result
provenance, retention and deletion, citation mapping, staleness and refresh
rules, and a provider- or connector-specific transport decision.

### Connected services

Before a real connected service, later decisions must establish credential
custody, named account/project/scope binding, a closed operation catalog, read
versus mutation classification, confirmation and ambiguous-dispatch policy,
content-retention and audit policy, and a provider-specific lifecycle decision.

### Local runtimes

Before a local runtime, later decisions must establish runtime classification
and discovery policy, capability metadata, transport and process isolation,
resource limits, context authorization, and separation from native execution
authority. A local unauthenticated runtime may not need cloud credential
custody, but still needs identity, context, transport, privacy, and capability
decisions.

### Browser verification

M58 runtime remains an independent security implementation decision. It is not required before
text-only provider inference, and becomes a prerequisite only for a capability
that concretely depends on verification-only browser evidence.

### Automation

Automation requires every invoked lane to be approved first, plus separately
approved scheduling or condition authority, bounded recurrence, revocation,
notification policy, failure and recovery rules, unattended-operation limits,
and cumulative cost limits.

## M55, M57, and M58 boundaries

### M55 source admission

M55 remains the controlling boundary for durable research and source reports.
Temporary explicitly selected input is distinct from durable retrieved
evidence. Limited inference may precede durable source admission, but web
retrieval, provider file search, indexed knowledge, retained fetched content,
and citations remain outside it. M58 does not resolve source identity or
citation authority. Retrieval must not be hidden inside an inference adapter as
an undocumented provider feature.

### M57 connector governance

M57 governs common least-authority lifecycle, binding, confirmation, result,
revocation, and audit concepts. Its mock-only foundation does not select the
transport or authority model for every lane. Future lanes may reuse its concepts
while defining their own descriptors, capabilities, operations, retention,
adapter lifecycle, evidence, and failure semantics. No real capability may be
implemented by substituting a network adapter for `LocalMock`.

### M58 controlled browser verification

M58 is separately named and its decision contract is complete. Its
fictional/local-only, read-only runtime is neither automatically started nor
cancelled, is independent of the first provider-neutral decision family, and is
not a prerequisite for text-only limited inference. Nothing in this taxonomy
expands its scope.

## Current next authority boundary

The earlier capability-registry decision is complete. M58 planning now ratifies
the next separately approvable work: a comprehensive M58 implementation goal
for its fictional/local-only, read-only verification slice. It receives no
package or version merely by being ratified and must not start implicitly.

## Provisional dependency sequence

The following sequence is non-authorizing and may be refined only by later
approved decisions. No stage may silently absorb authority from another lane.

1. External capability taxonomy and sequencing.
2. Provider-neutral capability registry and descriptor governance.
3. Canonical interaction and event protocol.
4. Provider-adapter lifecycle and conformance.
5. Credential broker and account/project/scope custody.
6. Context assembly and transmission manifests.
7. Limited provider inference.
8. Native tool proposals and closed operation routing.
9. Durable source admission, provenance, and citation authority.
10. Retrieval and bounded research.
11. Named connected-service decisions.
12. Controlled browser verification through its separate M58 lane.
13. Media and realtime capability decisions.
14. Scheduled and conditional automation.
15. Parallel, multi-agent, or unattended execution.

## Explicit exclusions

This decision grants no provider implementation, model invocation, model or
vendor selection, network access, real connector, credentials, credential
storage, OAuth, inference, retrieval, citations, source admission, browser
behavior, browser automation, MCP execution, external mutation, terminal,
shell, Git, repository, or deployment authority. It also grants no persistence
schema, SQLite migration, Tauri command, bridge, frontend UI, package, release,
tag, background activity, automation, or multi-agent behavior.
