# ADR 0028: UI Platform Decision

- Status: Accepted
- Date: 2026-07-25

## Context

QuireForge currently uses a Tauri + React + TypeScript interface over a Rust
service layer. The desired long-term direction may instead be a non-WebView,
cross-platform native desktop interface.

## Decision to make

Retain Tauri conditionally and reconsider Qt 6 only when the measurable
reconsideration triggers in
[Milestone 23](../MILESTONE_23_UI_PLATFORM_FEASIBILITY.md) occur. This accepts
neither an immediate Qt migration nor an unconditional permanent Tauri choice.

## Constraints

- Preserve working Tauri functionality, Git history, and rollback capability.
- Reuse the Rust service layer where practical.
- Do not develop Tauri and Qt feature implementations in parallel.
- Keep Linux first while planning Windows and macOS portability.

## Required evidence before acceptance

- A Tauri façade versus reusable-core boundary map.
- A current Tauri façade versus reusable-core boundary map and the smallest safe
  future core/facade boundary.
- Packaging and platform-adapter implications.
- Accessibility, terminal, file-picker, and notification considerations.
- CI implications, migration risks, and estimated scope.

## Decision and consequences

Tauri + React + TypeScript remains the active prototype. The existing Rust
services, closed typed contracts, and native safety boundaries are preserved.
No Qt migration, dependency, prototype, packaging change, or parallel frontend
development begins from this ADR.

The evidence found no measured QuireForge limitation that justifies replacing
the verified presentation, test, distribution, and Tauri façade layers. Qt 6
has credible documented desktop capabilities, but its QML presentation and any
Rust bridge would require a new façade, lifecycle integration, automation,
accessibility verification, packaging workflow, and feature-parity work.

The full comparison, confidence, external primary sources, estimate ranges,
unknowns, and triggers are recorded in the
[Milestone 23 feasibility report](../MILESTONE_23_UI_PLATFORM_FEASIBILITY.md).

## Reconsideration

Re-open this ADR only on documented trigger evidence: a material unresolved
Tauri/WebView Linux limitation; a repeatable performance failure against an
agreed target; a funded cross-platform requirement Tauri cannot meet; an
unmitigable support/security burden; or a separately approved narrow Qt
prototype that demonstrates the required parity and a materially improved
migration case. Host-installed Qt tooling or unmeasured preference is not
sufficient evidence.
