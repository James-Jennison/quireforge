# Goal — Provider-Neutral AI Foundation

Type: active long-term product goal.
Status: active; M55, M57, M58, and M60 implementation slices are complete
through the published beta.63 release. M59 Context Assembly and Transmission
Contract and M61 Credential Broker and Account Reference Contract are completed
decision records. M62 Limited Provider Inference Boundary is also a completed
decision record.
Packaging status: beta.55 and beta.56 remain preserved failed candidates;
beta.59 (M55), beta.60 (M57), beta.62 (M58), and beta.63 (M60) are published
prereleases. Latest published release: `0.1.0-beta.63` / `0.1.0~beta.63`.
Implementation authority: none granted by establishing this goal or completing
M61 or M62.
Parent vision: native, project-centered, all-in-one Linux AI workspace.

> The goal organizes related decisions and future implementation work. It does
> not itself authorize any capability.

## Purpose

QuireForge must eventually support multiple cloud and local intelligence
providers without organizing the application around provider-owned chat
products, allowing provider sessions to replace projects or tasks, granting
providers native tool or execution authority, inheriting a lowest-common-
denominator OpenAI-compatible abstraction, or mixing capability metadata,
credentials, context transmission, inference, retrieval, tools, and browser
behavior into one integration boundary.

This goal supplies one coherent product destination for the bounded decision,
implementation, and release work required to avoid those failures.

## Desired outcome

At goal completion, QuireForge has a provider-neutral foundation in which:

- projects and durable tasks remain authoritative;
- providers and local runtimes are selected capabilities within tasks;
- governed extensions preserve provider-specific behavior;
- provider adapters translate intelligence traffic only;
- credentials are separately governed;
- context transmission is explicit and digest-bound;
- native operations remain closed, typed, and QuireForge-owned;
- provider sessions are opaque subordinate references; and
- limited inference can exist without automatically granting retrieval, tools,
  browser access, or external mutation.

## Goal completion criteria

The goal is not complete merely because its decision documents exist. Completion
requires separately approved and validated implementation of at least:

1. provider-neutral capability-registry contracts;
2. canonical interaction and event contracts;
3. a provider-adapter lifecycle and conformance boundary;
4. credential broker and scoped account binding;
5. context assembly and transmission manifests;
6. a limited inference runtime;
7. one separately approved bounded provider or local-runtime adapter;
8. end-to-end proof that project/task authority is preserved;
9. proof that no implicit native tool, retrieval, browser, connector, or
   mutation authority exists; and
10. appropriate packaging, installed-host validation, and release evidence for
    the first user-visible capability.

Later decisions may refine these criteria but may not weaken their authority
boundaries without explicit approval.

## Completed decision gates

### External Capability Taxonomy and Sequencing

Status: complete, decision-only. The
[taxonomy and sequencing decision](EXTERNAL_CAPABILITY_TAXONOMY_AND_SEQUENCING.md)
separates inference, retrieval, connected services, local runtimes, execution,
credentials, browser verification, and automation; establishes dependency
ordering; and preserves M55, M57, and M58 boundaries.

### Provider-Neutral Capability Registry and Descriptor Governance

Status: complete, decision-only. The
[registry and descriptor-governance decision](PROVIDER_NEUTRAL_CAPABILITY_REGISTRY_AND_DESCRIPTOR_GOVERNANCE.md)
establishes provider, endpoint, model, runtime, adapter, capability, and claim
identity; versioning, digest binding, provenance, trust, lifecycle,
compatibility, limits, and governed extensions; and the separation of capability
metadata from authority.

These are completed gates within this goal, not completed product capabilities.

### Canonical Provider-Neutral Interaction and Event Protocol

Status: complete, decision-only. The
[interaction and event protocol decision](CANONICAL_PROVIDER_NEUTRAL_INTERACTION_AND_EVENT_PROTOCOL.md)
defines native-owned interaction attempts, closed envelopes, inputs/outputs,
streaming, cancellation, continuation, opaque provider-session references,
structured and multimodal events, tool communication, grounding/usage/errors,
terminal states, and governed extensions. It grants no transport, invocation,
credential, context, retrieval, tool, browser, persistence, or UI authority.

### Provider Adapter Lifecycle and Conformance Governance

Status: complete, decision-only. The
[adapter lifecycle and conformance decision](PROVIDER_ADAPTER_LIFECYCLE_AND_CONFORMANCE_GOVERNANCE.md)
defines adapter identity, compatibility, trust, lifecycle, upgrade, rollback,
revocation, quarantine, capability mapping, protocol translation, deterministic
conformance, extension handling, and failure-closed behavior. It grants no
adapter implementation, credential, transport, context, native-operation, or
provider authority.

### Credential Broker and Account/Project/Scope Custody

Status: complete, decision-only. The
[credential broker and custody decision](CREDENTIAL_BROKER_AND_ACCOUNT_PROJECT_SCOPE_CUSTODY.md)
defines native ownership, opaque credential references, account/organization/
project/endpoint/deployment/scope bindings, non-secret credential classes,
least-authority leases, lifecycle and recovery, no-ambient-authority rules,
content-free audit, and failure-closed behavior. It grants no secret custody,
account connection, context transmission, invocation, or native authority.

### Context Assembly and Transmission Manifests

Status: complete, decision-only. The
[context assembly and transmission decision](CONTEXT_ASSEMBLY_AND_TRANSMISSION_MANIFESTS.md)
defines native-owned selection, exact item/projection bindings, transformations
and omissions, exclusions, destination-aware authorization, revalidation,
continuation confinement, privacy/retention, and content-free audit evidence.
It grants no assembly implementation, transmission, invocation, retrieval,
credential resolution, or native authority.

### Limited Provider Inference Boundary

Status: complete, decision-only. The
[limited provider inference decision](LIMITED_PROVIDER_INFERENCE_BOUNDARY.md)
defines exact attempt binding, immediate revalidation, an initially text/local-
projection-only route, acknowledgement/streaming/cancellation/continuation,
idempotency and ambiguity rules, output/proposal confinement, policy/usage
disclosures, emergency stop, and a separate local-runtime variant. It grants no
provider selection, implementation, transmission, invocation, retrieval, tool,
browser, connector, or native authority.

## Current completion sequence

The authoritative ordered completion record is
[Provider-Neutral Foundation Completion Sequence](PROVIDER_NEUTRAL_FOUNDATION_COMPLETION_SEQUENCE.md).
M59's [Context Assembly and Transmission Contract](MILESTONE_59_CONTEXT_ASSEMBLY_AND_TRANSMISSION_CONTRACT.md)
governed the completed M60 slice. M61's
[Credential Broker and Account Reference Contract](MILESTONE_61_CREDENTIAL_BROKER_AND_ACCOUNT_REFERENCE_CONTRACT.md)
defines future custody/runtime selection gates, and M62's
[Limited Provider Inference Boundary](MILESTONE_62_LIMITED_PROVIDER_INFERENCE_BOUNDARY.md)
defines future limited-inference gates. Completing a decision gate never starts
an implementation milestone automatically.

### Capability Registry Contracts Only

Status: source-complete, source-only. The
[Provider Capability Registry Contracts](MILESTONE_PROVIDER_CAPABILITY_REGISTRY_CONTRACTS.md)
milestone implements private static fictional registry contracts and focused
validation only. It has no persistence, bridge, UI, package, or operational
provider behavior and does not complete the goal's later implementation criteria.

The next separately approved implementation milestone is **Canonical
Interaction/Event Contracts and Deterministic Mock Adapter Conformance**.

### Canonical Interaction/Event Contracts and Deterministic Mock Adapter Conformance

Status: source-complete, source-only. The [Provider Interaction/Event Contracts
and Deterministic Mock Adapter Conformance](MILESTONE_PROVIDER_INTERACTION_PROTOCOL_CONTRACTS.md)
milestone implements private fictional attempts, canonical envelopes, closed
lifecycle validation, and deterministic fixture translation only. It has no
persistence, bridge, UI, provider route, package, or operational behavior.

The next separately approved implementation milestone is **Credential Broker
Foundation Contracts**.

### Provider-Neutral Core Foundation and Mock Inference Vertical Slice

Status: source-complete, source-only. The [Mock Inference Vertical
Slice](MILESTONE_PROVIDER_NEUTRAL_MOCK_INFERENCE_VERTICAL_SLICE.md) provides a
small user-visible local fixture workflow: explicit task-bound preparation,
bounded authored-text manifest review, inert opaque lease, one-use authorization,
deterministic canonical events, and content-free evidence. It remains
in-memory, fictional, non-networked, and non-operational; it adds no real
provider, credential, retrieval, context transmission, native operation, or
external authority.

The source-complete [**Provider-Neutral Mock Workflow Hardening and Release
Readiness**](MILESTONE_PROVIDER_NEUTRAL_MOCK_WORKFLOW_HARDENING.md) milestone
extends this local fixture with registry-backed destination selection, bounded
polling, explicit cancellation confirmation, authority-failure fixtures, and
focused browser acceptance. It remains local, ephemeral, and non-operational;
no real-provider readiness decision begins automatically.

The separately approved **Mock Vertical Slice Release Decision and Packaging**
checkpoint evaluates the `0.1.0-beta.55` candidate. It packages only the
fictional local fixture after release gates pass; beta.54 remains the installed
rollback generation until restricted installed-host validation succeeds.

## Separate goals and lanes

### M55 durable source admission

M55 is the bounded local Durable Source Admission slice defined in
[its implementation contract](MILESTONE_55_DURABLE_SOURCE_ADMISSION_CONTRACT.md).
It owns explicit project-bound copies of manual text, one selected local UTF-8
file, and eligible reviewed text artifacts. Admission is not retrieval,
provider transmission, context-manifest inclusion, or filesystem authority
beyond the one approved native intake. Retrieval, cited research, indexing, and
any later context inclusion remain separate decisions.

### M57 connector governance

M57 is complete and published as beta.60. The
[M57 Connector Governance Contract and Executable Implementation
Plan](MILESTONE_57_CONNECTOR_GOVERNANCE_CONTRACT.md) remains authoritative for
the implemented fictional/local-only slice. It grants no real connector,
provider, credential, network, browser, MCP, automation, or external authority.

### M58 browser verification

M58 is complete and published as beta.62. Its fictional/local-only read-only
WebKitGTK fixture is not general browser automation, OAuth, web research,
provider transport, or external action.

### M59 context assembly and transmission

M59 is complete decision-only work. Its ratified contract defines the explicit
selection, deterministic assembly, redaction, bounded review, one-use
confirmation, fictional-sink, retention, drift, ambiguity, and audit decisions
needed by M60. It added no runtime context assembly or provider transmission.

### Connected services

GitHub, Gmail, calendars, cloud systems, messaging, storage, and deployment
systems remain individually governed connected-service work.

### Automation and multi-agent behavior

Scheduling, conditions, background work, parallel agents, multi-agent
coordination, and unattended execution remain later goals or lanes after their
underlying authorities exist.

## Goal-level invariants

- Projects and durable tasks remain authoritative.
- Provider threads and sessions remain subordinate opaque references.
- Provider adapters translate intelligence traffic only.
- Capability metadata does not grant authority.
- Credentials, context, inference, retrieval, tools, connected services,
  browser behavior, and automation remain separate authorities.
- No provider receives implicit filesystem, terminal, Git, browser, connector,
  credential, cloud, or deployment access.
- Native operations remain closed and typed.
- Mutations require immediate revalidation and digest-bound authorization.
- Ambiguous mutation dispatch is never automatically retried.
- Generic MCP cannot bypass lane-specific governance.
- Governed extensions preserve provider-specific behavior.
- Inspectable templates remain distinct from hidden agents and authority-bearing
  plugins.
- No goal gate automatically starts another gate.

## Goal reporting

Future [CURRENT_STATE](CURRENT_STATE.md) and [ROADMAP](ROADMAP.md) updates must
report progress through this hierarchy:

```text
Product vision
└── Product goals
    ├── Decision and architecture gates
    ├── Implementation milestones
    └── Release checkpoints

Goal status
├── Completed gates
├── Active gate
├── Pending gates
├── Approved implementation milestone
└── Latest packaged/released capability
```

Do not present every decision artifact as a new top-level product destination.
When a gate completes, record it as progress under this goal.

## Current gate status

M60 is complete and published as the [Governed Context Assembly Vertical
Slice](MILESTONE_60_GOVERNED_CONTEXT_ASSEMBLY.md), limited to M59's
fictional/local-only sink. M61 is complete as the decision-only
[Credential Broker and Account Reference Contract](MILESTONE_61_CREDENTIAL_BROKER_AND_ACCOUNT_REFERENCE_CONTRACT.md).
M62 is complete as the decision-only
[Limited Provider Inference Boundary](MILESTONE_62_LIMITED_PROVIDER_INFERENCE_BOUNDARY.md).
No provider, credential, network transmission, inference, or post-M62
implementation milestone has started.

## Package and release policy

Establishing or advancing this goal through decision-only gates requires no
package or version. M60 altered user-visible and operational behavior and its
beta.63 release is published; M61 and M62 add no package or release behavior.

## Explicit exclusions

This organizational checkpoint grants no provider or vendor selection; provider,
descriptor, protocol, or adapter implementation; model invocation; networking;
credentials; account connection; OAuth; context transmission; inference;
retrieval; source admission; citations; browser behavior; connector
implementation; MCP execution; native tool execution; filesystem, terminal,
Git, repository, cloud, or deployment authority; persistence; migration; Tauri
command; bridge; UI; package; version; release; tag; background work;
automation; or multi-agent behavior.
