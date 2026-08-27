# M70 -- Typed Project Knowledge Foundation

Status: source validation complete in the `0.1.0-beta.97` candidate; desktop
E2E and package/installed-host acceptance remain required. M70 is the first
continuity slice; it creates no agent read/write path, context assembly,
provider transmission, execution authority, or external capability.

## Purpose

Create a private, project-scoped native Knowledge Ledger that records durable
project knowledge without changing M66's content-free governance projection.

## Closed record contract

Every record has an opaque UUIDv7 ID, schema version, project ID, optional
task/objective reference, closed kind, bounded title and body, provenance role
(`owner`, `agent`, or `system`), status, creation/update timestamps, optional
supersedes ID, and immutable event history. The closed kinds are:

- `owner-decision`, `constraint`, `observed-fact`,
  `verified-implementation`, `agent-claim`, `assumption`,
  `recommendation`, `rejected-approach`, and `unresolved-question`.

Records must not retain transcripts, raw logs, artifact bytes, credentials,
provider/session identifiers, paths, or execution authority. Unknown fields
and prohibited content fail closed.

## Lifecycle and binding

Non-binding kinds use:

`recorded -> active -> validated | disproven | resolved -> superseded | retired`.

`owner-decision` and `constraint` use:

`proposed -> pending-owner-binding -> active -> superseded | retired`.

Material changes create a successor record linked by `supersedes`; they never
rewrite the prior record. Only an explicit native owner-confirmation path may
activate an owner decision or constraint. M70's create/read paths are
owner/native only. Agent creation of non-binding records is deferred to M73,
where an acting-agent capability and objective boundary can scope it.

## Required evidence

1. Strict Rust and TypeScript contracts reject unknown fields and prohibited
   content.
2. Forward-only migration, restart, corruption, lifecycle, provenance,
   supersession, and atomic-binding tests pass.
3. Tests prove no path other than owner confirmation can activate a binding
   record.
4. Tests prove M66, M60, provider transmission, context assembly, execution,
   and credential boundaries are unchanged.
5. `pnpm validate`, applicable desktop E2E, and the documented Linux package
   gate pass before M70 is marked validated. Owner acceptance remains separate.
