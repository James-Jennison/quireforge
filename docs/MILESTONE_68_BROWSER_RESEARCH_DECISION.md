# M68 Decision: Isolated Read-Only Browser Research

## Decision

No browser-research runtime is authorized. A future proposal must use an
isolated, native-owned profile with no ambient cookies, sessions, credentials,
extensions, downloads, uploads, or filesystem exposure.

## Required future gate

The user must approve an exact target/origin and read-only navigation scope
before launch. Captured provenance must identify the target, timestamp, content
digest, and observation limits without retaining credentials or unbounded page
content. Prompt injection, redirects, ambiguous identity, and origin drift
must stop observation and surface a content-free audit result.

Retention, revocation, cancellation, crash recovery, dependency review, and
package rollback must be specified before any implementation. M58's local
fixture is not browser authority and may not be generalized.
