# M55 Durable Source Admission Implementation Contract

Status: implementation contract ratified for `0.1.0-beta.58`. This document
supersedes the earlier M55 implementation deferral; it does not authorize
retrieval, provider transmission, context inclusion, connectors, browser
authority, generic MCP, or automation.

M55 admits one bounded local textual source through a native prepare/confirm
workflow. Admission creates a private QuireForge-owned byte copy with a fresh
immutable source ID, immutable project ID, optional immutable task ID, class,
SHA-256, byte size, deterministic line count, timestamps, and lifecycle. It
does not make the source available to a provider or context manifest.

## Admission classes and limits

Only these classes are permitted: `manual_text` (exact native-received UTF-8,
at most 32,768 Unicode code points, 128 KiB, and 2,000 lines),
`local_text_file` (one explicitly selected regular non-symlink UTF-8 file,
copied once, 128 KiB and 2,000 lines), and `reviewed_artifact_text` (an
explicitly selected current, accepted, usable M48/M54 text artifact, copied
into a separate M55 record, 128 KiB and 2,000 lines). No absolute original
path is persisted; file origin metadata is basename only.

Every prepare creates a short-lived, one-use ID and nonce plus bounded preview.
Confirmation repeats the digest and validates current project/task ownership,
expiry, use state, and staged bytes. Ambiguous mutation is never retried.
Identical content may be deliberately admitted again only through a fresh
preparation and confirmation, creating a fresh source ID.

Canonical bytes are SHA-256 hashed natively and stored under native-controlled
source identities in application-private storage, never in the attached
project repository. Staged writes are permission restricted and are cleaned on
expiry/startup; metadata insertion is fail-closed if the private final copy
cannot be established.

## Lifecycle and presentation

M55 v1 states are `active` and `deleted`. Deletion is another explicit,
one-use confirmed local mutation. It removes active private content and leaves
only the permitted metadata tombstone. Sources are not silently rebound,
restored, included in plans, conversations, reviews, provider sessions, or
context manifests. Project deletion must remain governed and must not gain an
unreviewed source cascade.

The project-scoped Durable Sources UI lists bounded active metadata and
previews, names the owning project/task, explains copy semantics and limits,
and provides keyboard-reachable review/confirmation/deletion controls without
hidden duplicate controls. It must preserve focus, constrained-layout behavior,
and bounded identifiers, diagnostics, and lists.

Native commands are closed and typed: prepare manual/file/reviewed-artifact,
confirm admission, list/read bounded metadata, and prepare/confirm deletion.
There is no arbitrary path read/write, SQL, source mutation, retrieval,
provider-facing source access, context-inclusion API, or MCP route.
