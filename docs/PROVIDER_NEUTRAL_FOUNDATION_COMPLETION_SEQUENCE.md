# Provider-Neutral AI Foundation completion sequence

Status: ratified planning. M55, M57, M58, and M60 are complete; beta.63 is
published. M59 is complete as the decision-only
[Context Assembly and Transmission Contract](MILESTONE_59_CONTEXT_ASSEMBLY_AND_TRANSMISSION_CONTRACT.md),
and M61 is complete as the decision-only
[Credential Broker and Account Reference Contract](MILESTONE_61_CREDENTIAL_BROKER_AND_ACCOUNT_REFERENCE_CONTRACT.md),
and M62 is complete as the decision-only
[Limited Provider Inference Boundary](MILESTONE_62_LIMITED_PROVIDER_INFERENCE_BOUNDARY.md).
Routine reversible, local, non-production M63+ implementation is autonomous
under the roadmap's post-M62 operating rule. That rule does not authorize
credentials or account/browser access, real provider/runtime connection,
network transmission, deployment, public release, destructive action,
third-party commitments, or irreversible product-direction selection.

## Completion definition

The foundation is complete when QuireForge can perform one governed,
provider-neutral intelligence operation while projects/tasks remain authoritative:
reviewed minimized context, destination-bound transmission authorization,
credential lifecycle where needed, typed request/response with cancellation and
bounded failure, subordinate session references, usage evidence, governed
response review/adoption, one approved adapter, and end-to-end installed-host
acceptance. This is not provider availability in general, production readiness,
connector/browser/MCP/automation availability, or overall product completion.

## Ordered milestones

| Milestone | Scope and exit criteria | Not granted |
| --- | --- | --- |
| M59 Context assembly and transmission contract | Complete decision-only contract: explicit selection, minimization/redaction, bounds, attribution, digest/review, destination binding, retention and failure semantics. | Provider call, credential, context transmission, inference. |
| M60 Governed context assembly vertical slice | Complete and published local-only implementation of M59 with project/task, selected M55/review evidence, deterministic fictional delivery, migration, package, and installed-host gates. | Any provider destination or automatic inclusion/transmission. |
| M61 Credential broker and account reference contract | Complete decision-only secure-custody selection criteria, scoped opaque reference, rotation/revocation/expiry/audit, and adapter compatibility contract. Exit: ratified provider/local-runtime selection criteria. | Credential collection, real account, OAuth, provider call. |
| M62 Limited provider inference boundary | Complete decision-only M60 bundle/M61 reference binding, destination/model allowlist, typed request/response and lifecycle constraints, cancellation, privacy, audit, and local-runtime compatibility contract. Exit: ratified limited-inference gates. | Provider/local-runtime implementation, credentials, tools, retrieval, mutation. |
| M63 First adapter selection and bounded adapter | Decision plus implementation: select a credential-free local runtime if it meets M62 conformance; otherwise select one real provider only after its credential scope is ratified. Exit: conformance, drift/quarantine, installed-host acceptance. | Additional providers, connector/browser authority, native tools. |
| M64 Credentialed provider enablement (conditional) | Only if M63 selected a real provider: broker implementation, one provider-scoped account reference and explicit reviewed transmission. Exit: revocation/rotation, cost/usage, failure and installed-host proof. | Other providers, OAuth expansion, automation. |
| M65 Foundation end-to-end reconciliation | Fresh/upgrade migration, context/inference/cancellation/drift/recovery, M55–M58 separation, package/ABI/provenance and installed-host acceptance. Exit: foundation-complete declaration. | Production deployment or external mutation. |
| M66 Product-readiness reconciliation | Bundle ceilings, security/privacy review, release-operability and remaining deferred-lane audit. Exit: production-readiness decision only. | Deployment, automation, multi-agent work. |

M59–M65 are required for foundation completion; M64 is conditional on a
credentialed first adapter. M66 is production-readiness work. Real connectors,
generic MCP, browser agents, browser expansion, automation, multi-agent work,
additional providers, extension marketplaces, and deployment are optional
post-foundation lanes and each remains separately governed.

## Dependency decisions

Context policy precedes transmission; M60 precedes any adapter so context never
becomes implicit. Credential-free runtime conformance is preferred before
credentialed provider selection; credentials precede only a selected real
provider. Streaming belongs in the first runtime because cancellation and usage
evidence otherwise cannot be validated end-to-end. MCP requires M57 plus its
own dispatch contract; browser agents require M58 plus governed inference;
automation/multi-agent work requires explicit scheduling, budgets, delegation,
recovery, tool grants, confirmations and loop prevention.

## Current sequence status

M60 is complete and published as the local-only M59 slice. M61 and M62 are
complete decision-only contracts for future custody/runtime and
limited-inference gates. Codex may autonomously continue routine reversible,
local, non-production M63+ implementation. It must stop before credential or
account/browser handling, real provider/runtime connection, network
transmission, production deployment, public release, destructive action,
third-party commitments, or an irreversible product-direction decision.
