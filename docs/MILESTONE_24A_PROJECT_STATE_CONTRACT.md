# Milestone 24A — Project State Contract

Status: active implementation on `feat/milestone-24a-project-state-contract`.

## Purpose

Version 1 defines a strict, machine-readable representation of project truth
without reading repositories, parsing Markdown, showing a workspace, generating
handoffs, or changing project state. Markdown remains evidence for people; it
is not the sole durable model.

## Canonical contract

Rust Serde types in `apps/desktop/src-tauri/src/project_state.rs` are the
authoritative wire contract. `apps/desktop/src/lib/projectState.ts` mirrors the
closed JSON shape with Zod using existing dependencies. Shared fixtures in
`apps/desktop/fixtures/project-state.json` prove Rust/TypeScript interpretation
of active, pushed, paused, completed, missing-evidence, and contradictory
states.

`schemaVersion: 1` is required. Unknown fields and unknown security-sensitive
enum values are rejected. A future version is rejected; a later migration owner
must explicitly add a compatible reader or migration. No generic migration
framework is introduced.

## Trust, provenance, and ownership

Every material aggregate carries provenance with `verified`, `reported`,
`inferred`, or `unknown` trust and optional source type/reference/commit,
observed/verified timestamps, and note. Verified Git facts cannot be replaced
by an agent report; inferred recommendations remain distinct from approvals.

- User-approved policy and boundaries are canonical policy, but their long-term
  persistence location is intentionally deferred by approved decision.
- Git, documentation, validation reports, and package manifests are derived
  evidence owned by their source systems.
- QuireForge application storage owns local project metadata only.
- Agent-session state is transient reported evidence.

No credentials, raw tokens, secret material, raw paths, or transcripts belong
in the contract.

## Deferred work

24B may read and normalize sources. 24C may display the contract. 24D may
generate handoffs and record contradictions. None is implemented here.

## Package boundary

The contract is library code compiled into the desktop application. Under the
standing installed-application policy, final 24A completion therefore requires
fresh package evidence from the final source commit. No package is built by
this contract checkpoint; package production and installed-app validation
remain the final closure work.
