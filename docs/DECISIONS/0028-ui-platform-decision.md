# ADR 0028: UI Platform Decision

- Status: Proposed
- Date: 2026-07-23

## Context

QuireForge currently uses a Tauri + React + TypeScript interface over a Rust
service layer. The desired long-term direction may instead be a non-WebView,
cross-platform native desktop interface.

## Decision to make

After feasibility evidence, either retain and evolve the Tauri prototype or
begin a controlled Qt 6 migration. This ADR does not select Qt or retire Tauri
yet.

## Constraints

- Preserve working Tauri functionality, Git history, and rollback capability.
- Reuse the Rust service layer where practical.
- Do not develop Tauri and Qt feature implementations in parallel.
- Keep Linux first while planning Windows and macOS portability.

## Required evidence before acceptance

- A Tauri façade versus reusable-core boundary map.
- Qt prototype feasibility and the smallest safe core/facade boundary.
- Packaging and platform-adapter implications.
- Accessibility, terminal, file-picker, and notification considerations.
- CI implications, migration risks, and estimated scope.

## Consequences while proposed

Tauri remains the active prototype. No Qt migration begins, and installed Qt
tooling alone does not constitute a platform decision.

## Decision trigger

Complete Milestone 23 and obtain explicit user approval.
