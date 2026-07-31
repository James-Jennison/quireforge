# Milestone 56 — Inspectable Local Task Templates

Status: complete. The template-only contract in the approved
[M55 proposal](MILESTONE_55_RESEARCH_REPORTS_AND_INSPECTABLE_TASK_TEMPLATES_PROPOSAL.md)
remains authoritative. James's separate explicit M56 approval satisfied the
implementation gate; it did not authorize research reports, retrieval,
providers, connectors, MCP, OAuth, cookies, browser access, hidden
instructions, or automatic actions.

## Completed checkpoints

- Foundation: `614b22c870dd5a45c88e1e8f59dedc51c4b1c671` — `feat: add task
template foundation`. It establishes the canonical native template model,
  digest rules, migration 21, and four static in-memory built-ins.
- Storage: `693a7e95e911ac6b50a74734171e2305fbdddbc0` — `feat: add task
template storage`. It adds private transactional local-template storage
  primitives and focused validation.
- G4 lifecycle service: `03163389d6fe69dbf0deedfd532f8bad2f7bde03` — `feat:
add task template lifecycle`. It is a bounded native service only; it adds no
  command, bridge, UI, application, reservation, package, or release claim.

M56-G1 through G8 are source-complete. Migration 21 owns local templates and
migration 22 owns bounded digest-only application reservations. Four static
built-ins remain outside SQLite. The lifecycle, strict bridge, lazy management
UI, digest-bound application workflow, accessibility, and focused browser
acceptance are implemented. Research, providers, connectors, MCP, browser
authority, credentials, hidden instructions, import/export, automatic actions,
approval, dispatch, and execution remain excluded.

## Release closure

M56 closed at package/source commit
`e2b084ed0bdf17fb6f4b0b47663cdf6952ec8e73`, annotated tag
`v0.1.0-beta.54`, and a `James-Jennison/quireforge` GitHub draft prerelease.
G9 source acceptance passed; G10 produced and independently validated the
existing canonical pinned-Ubuntu 22.04 package set without rebuilding during
closure; and G11 passed restricted installed-host validation after the operator
performed the authorized two-package installation. Both `0.1.0~beta.54`
packages remain installed; integrity and restricted-boundary checks passed,
headless completion returned `created`, then `existing`, and no rollback was
required.

The draft prerelease contains exactly four canonical assets:

- `quireforge_0.1.0.beta.54_amd64.deb` — 5,864,924 bytes, SHA-256
  `643e6bc3caf9068f7ed521ecd949f9f3f5d38b9c6a82bcce19384370f644d131`.
- `quireforge-sandboxd_0.1.0.beta.54_amd64.deb` — 3,233,492 bytes, SHA-256
  `bd9c0682c0e9dd7761b28f03eb2e801ab7a925e7c5f5587eefc68bd7578bd21f`.
- `SHA256SUMS`.
- `release-manifest.json`.

The prerelease remains draft and has not been published or deployed. Beta.53
remains preserved as the prior released rollback generation, while beta.52
remains an unreleased candidate that failed its installed-host gate. M55
research-report implementation remains deferred because durable source-manifest
authority is not approved. M57 connector governance and M58 browser
verification remain future decision gates; the next action is a separate
decision/planning goal, not an automatic start of either milestone.

## M56 delivery sequence

- G5: complete — native digest-bound application preview, reservation, and
  atomic confirmation are implemented and covered by focused Rust tests.
- G6: complete — closed Tauri lifecycle/application commands and a strict
  TypeScript/Zod bridge validate only user text, opaque selectors, opaque
  native mutation handles, and explicit confirmation values. No UI is added.
- G7: complete — lazy native-backed template catalog, detail inspection, and
  local-template lifecycle UI are implemented with focused component coverage.
- G8: complete — explicit native-reservation application UI and focused
  accessibility/browser acceptance are implemented.
- G9: complete — definitive source acceptance and beta.54 preparation passed.
- G10: complete — canonical beta.54 packaging and independent artifact
  validation passed.
- G11: complete — guarded installed-host validation passed with `created`, then
  `existing`, and no rollback.
- G12: complete — annotated tag, four-asset draft prerelease, and final M56
  documentation closure.
