# Milestone 57 — Local Mock Connector Foundation

Status: source-complete. This source milestone implements only the local mock-only
foundation approved by the post-M57 prerequisite decisions. It is not a real
connector or external-authority implementation.

## Approved envelope

The foundation provides static, inspectable, non-executable descriptors with
canonical version/digest identity; native in-memory lifecycle and explicit
project/account/scope bindings; inert opaque credential-reference contracts;
closed operation proposals/results; one-time digest-bound mock mutation
confirmation; content-free provenance/audit records; and deterministic mock
adapter behavior. Native tests are the inspection surface; no Tauri command,
frontend bridge, or UI is needed because exposing mock controls would imply
operational connector support.

Persistence is deliberately deferred. Descriptors are compiled in and all
bindings, proposals, confirmations, results, and audit records are process
local. This is sufficient to prove expiry, replay, revocation, and recovery
invalidation without inventing a migration before a real retention decision.

## Exclusions

There are no network/DNS/HTTP calls, provider SDKs, providers, credentials,
OAuth, secret storage, browser authority, shell/process execution, ambient
authority, external reads/searches/fetches/mutations, background activity,
generic MCP/tools, executable descriptors, fetched-content retention, M55
source admission, approval/dispatch authority, import/export, package, or
release behavior.

## Internal checkpoints

1. Static descriptor, lifecycle, binding, credential-reference, proposal,
   confirmation, mock-result, and content-free audit contracts.
2. Focused adversarial native tests and a repository validator guard scoped to
   the new module.
3. Source validation and documentation closure.

## Implemented contracts and validation

The private native `connector_foundation` module registers one static fictional
local-mock descriptor with canonical SHA-256/version identity, closed operation
and scope enumerations, and no executable/transport fields. Its in-memory
service issues opaque UUIDv7 bindings and inert mock credential references,
enforces lifecycle transitions, project/account/scope matching, proposal and
confirmation expiry/cancellation/consumption, descriptor drift, revocation,
quarantine, and content-free mock audit records. Dispatch is deterministic and
local only; an unknown/duplicate/partial/irreversible outcome is represented
without retry or external side effect.

Focused Rust tests cover descriptor drift, invalid lifecycle progression,
cross-project/account/scope rejection, expired credential authority,
revocation, quarantine, stale/cancelled/replayed confirmations, payload/target
mismatch, mock completion, ambiguous dispatch, duplicate outcome, and
content-free/mock-only audit output. The repository validator additionally
rejects process, network, environment, URL, and command imports in this module
and requires local-only contract markers. Focused tests, clippy with warnings
denied, formatting, and repository validation passed.

## Completion criteria

The native model must reject malformed/unknown descriptors and operations,
cross-project/account/scope binding, revoked/expired/quarantined authority,
stale/replayed/mismatched confirmations, descriptor drift, and automatic retry
after ambiguous dispatch. Records must be visibly mock/local and structurally
exclude secrets and content. M55 research implementation remains deferred and
M58 remains unstarted. Packaging, installed-host validation, and release work
are separate future goals.
