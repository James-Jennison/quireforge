# Provider-Neutral AI Foundation completion sequence

Status: ratified planning. M55, M57, and M58 are complete; beta.62 is
published. No M59+ implementation is authorized by this document.

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
| M59 Context assembly and transmission contract | Decision-only: source selection, minimization/redaction, limits, attribution, digest/review, destination binding, retention and failure semantics. Exit: ratified executable contract. | Provider call, credential, context transmission, inference. |
| M60 Governed context assembly vertical slice | Local-only implementation of M59 with project/task, selected M55/review evidence and explicit browser/connector evidence selection. Exit: persisted review/one-use transmission authorization, UI, migration, package and installed-host gates. | Any provider destination or automatic inclusion/transmission. |
| M61 Credential broker and account reference contract | Decision-only secure-storage, scoped reference, rotation/revocation/expiry/audit and adapter compatibility contract. Exit: ratified provider/local-runtime selection criteria. | Credential collection, real account, OAuth, provider call. |
| M62 Limited inference runtime and response governance | Implementation: typed request/response, streaming/cancellation/timeout, unambiguous read-only retry only, subordinate sessions, normalized failures, usage/latency/cost evidence, review/adoption. Exit: fictional local adapter proves lifecycle. | Real provider, credentials, tools, retrieval, mutation. |
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

## Immediate next milestone

**M59 Context assembly and transmission contract** is next, decision-only. M55
admission, M57 connector references, M58 evidence boundaries and provider-neutral
protocol foundations are prerequisites; the unresolved authority is precisely
what context may reach which destination. A comprehensive approval may ratify
the contract and documentation only. It must stop before any provider,
credential, transmission, inference, deployment, or runtime implementation.
