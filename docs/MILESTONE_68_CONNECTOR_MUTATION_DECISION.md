# M68 Decision: Connector Mutation, Transfer, and Delivery

## Decision

No external mutation, download, upload, or delivery is authorized. A later
implementation must treat each as an independent destructive capability,
separate from connector read access.

## Required future gate

It must bind the reviewed payload digest, exact destination identity, operation
class, account identity, scope, expiry, single-use confirmation, idempotency
or ambiguity policy, cancellation, revocation, auditable outcome, and rollback
story. Prompt injection, target drift, partial completion, retry ambiguity, or
unavailable verification must fail closed with no silent retry.

No content bytes, provider context, credentials, paths, or live result may be
introduced by this decision. M57's local mock mutation scenario remains a test
fixture and cannot be connected to a real service.
