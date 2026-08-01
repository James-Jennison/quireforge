# Milestone — Provider-Neutral Mock Workflow Hardening and Release Readiness

Status: source-complete local hardening; release decision remains separate.

## Outcome

The fictional, in-memory mock workflow now derives its selectable Lantern and
Ember destinations from the private static capability registry. Each attempt
remains project/task/input/destination/manifest/lease/policy bound. Polling
adds at most one canonical fixture event; cancellation enters `cancelling` and
is confirmed as `cancelled` only by a later poll.

Deterministic local profiles cover streamed and structured success, refusal,
failure, timeout, interruption, ambiguity, manifest expiry, lease expiry,
revocation, quarantine, explicit invalidation, descriptor drift, and adapter
incompatibility. Authority failure and missing in-memory state fail closed and
require a fresh review. Retry/regeneration retains only bounded prior evidence
and never reuses an authorization, lease, manifest, event sequence, or result.

## Boundary

This remains an ephemeral local fixture. It adds no provider route, endpoint
URL, network activity, credential material, context transmission, retrieval,
browser authority, connector, MCP, tool execution, persistence, package,
release, publication, or deployment.

## Acceptance

Focused Rust lifecycle and registry tests, bridge-schema/component tests, and
desktop/mobile Playwright invalidation acceptance cover the bounded surface.
Repository safeguards, formatting, type checking, and focused build checks are
run before the local follow-up commit. Full packaging and release evidence are
intentionally outside this milestone.

## Release readiness

The mock slice is source-ready for a separate **Mock Vertical Slice Release
Decision and Packaging** checkpoint. That checkpoint must independently decide
whether a version, package, installed-host validation, or release action is
appropriate.
