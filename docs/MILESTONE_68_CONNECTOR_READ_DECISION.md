# M68 Decision: User-Scoped Connector Read Access

## Decision

No real connector or credential path is authorized. A future read connector
requires a named identity boundary, explicit user-scoped account reference,
capability descriptor, destination-aware review, digest-bound confirmation,
short expiry, revocation, and content-free audit linkage.

## Required future gate

The proposal must state the exact remote data classes, filtering, retention,
redaction, provenance, injection handling, ambiguity response, and recovery
behavior. It must fail closed on descriptor change, account drift, scope
mismatch, expired confirmation, unavailable evidence, or incomplete audit.

M57's fictional connector records are a deterministic governance fixture only;
they do not authorize a network client, OAuth, connector discovery, or read
retrieval.
