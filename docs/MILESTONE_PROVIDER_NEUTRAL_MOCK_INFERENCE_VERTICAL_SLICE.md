# Milestone — Provider-Neutral Core Foundation and Mock Inference Vertical Slice

Status: source-complete, source-only local fixture milestone.

## Scope

This milestone adds the first user-visible Provider-Neutral AI Foundation
workflow without creating provider authority. It is a deterministic local mock
exercise: a user selects a durable task, fictional static destination, and
bounded authored text; reviews the native context-manifest summary and inert
opaque lease; explicitly authorizes a one-use digest-bound mock submission; and
inspects its canonical mock events, bounded usage, and content-free evidence.

Projects and durable tasks remain authoritative. A fictional provider-session
reference is subordinate metadata only.

## Implemented boundary

`mock_inference.rs` owns private, in-memory records for static fictional
profiles, attempts, manifests, inert credential references and leases,
authorization, deterministic outcomes, and evidence. Native project metadata
resolves only the exact project/task identity; no task text, plans, files, or
ambient context is sent into the fixture service.

The closed Tauri and Zod bridge exposes catalog, prepare, authorize, submit,
cancel, and retry requests only. The workbench clearly labels every destination
and result as fictional/local mock behavior. It presents manifest and
authorization digests, explicit exclusions, ordered canonical events, bounded
usage, and content-free evidence.

There is no SQLite migration or persistence. The durable task is the durable
correlation; attempts and leases are deliberately in-memory and cannot revive
after application restart.

## Authority and lifecycle

An attempt is task/project/destination/profile/lease/manifest/policy bound.
Prepare creates a fresh attempt; authorization is explicit; submit consumes the
authorization exactly once. Cross-task use, replay, stale or invalidated
manifests, unavailable tasks, and terminal transitions fail closed. Retry is a
new attempt and never an automatic resend. Cancellation is explicit and final.

Fixture outcomes are deterministic: streamed text, structured output, refusal,
failure, timeout/interruption, and ambiguous outcome. An ambiguous outcome
states that no automatic retry occurs.

## Validation

Focused native and workbench tests cover ordered events, explicit authorization,
replay rejection, cross-task rejection, cancellation, deterministic failure
taxonomy, mock labeling, and the task-catalog entry. The repository validator
adds a narrow guard against authority-bearing imports and real provider or
destination identifiers in this local fixture boundary.

## Exclusions

This milestone adds no real provider or vendor, endpoint URL, network activity,
model invocation, credential material or custody, OAuth, account connection,
context transmission, retrieval, source admission, citation authority, native
tool execution, shell, terminal, Git, browser, connector, MCP, cloud,
deployment, automation, multi-agent behavior, or external mutation. It does
not add a package, version, release, installation, or deployment.

## Release policy and next work

Although the fixture workbench is user-visible, the Provider-Neutral AI
Foundation release policy requires a separately approved release decision
before assigning a version or packaging it. Beta.54 remains the latest packaged
generation and its draft prerelease is unchanged.

The recommended next separately approved milestone is **Additional Mock
Hardening**: bounded recovery/expiry fixture coverage and acceptance hardening
before any real-provider readiness or provider-selection decision.
