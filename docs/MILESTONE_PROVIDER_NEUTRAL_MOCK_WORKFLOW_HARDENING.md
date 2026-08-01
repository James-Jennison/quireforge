# Milestone — Provider-Neutral Mock Workflow Hardening and Release Readiness

Status: source-complete local hardening; beta.56 replacement-candidate
correction authorized.

Beta.55 passed its source, package, installation, and native-receipt gates but
is release-ineligible because the installed New task route omitted the required
Task Catalog/New task UI. Its immutable artifacts and receipt remain preserved.
Beta.56 restores only that governed durable-task-to-mock-workbench path;
packaging remains non-release evidence until the pinned-container and
installed-host gates pass.

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

Focused Rust lifecycle and registry tests cover the complete deterministic
state-machine taxonomy, sequencing, authority drift, isolation, and state-loss
rejection. Bridge-schema and component tests cover strict payloads, editable
binding invalidation, incremental rendering, cancellation confirmation, and
fresh-attempt presentation. Desktop/mobile Playwright acceptance covers the
live-workbench launch; input and destination invalidation; a streamed,
keyboard-operated cancellation flow; ambiguous-result messaging without an
automatic retry; fresh retry; close-focus restoration; and narrow,
reduced-height, effective-200%-zoom completion. This deliberately
keeps exhaustive fixture-enum checks below the browser layer. Repository
safeguards, formatting, type checking, focused build checks, and the Rust
workspace suite are recorded locally. Full packaging and release evidence are
intentionally outside this milestone.

## Release readiness

The mock slice is source-ready for a separate **Mock Vertical Slice Release
Decision and Packaging** checkpoint. That checkpoint must independently decide
whether a version, package, installed-host validation, or release action is
appropriate.
