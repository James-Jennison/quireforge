# Milestone 56 — Inspectable Local Task Templates

Status: source-complete; beta.54 source candidate prepared. The template-only
contract in the approved
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

Beta.54 is a prepared source candidate only. Canonical packaging,
installed-host validation, tag, draft prerelease, publication, and deployment
have not occurred. Beta.53 remains the installed and released M54 baseline
until beta.54 passes later gates.

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
- G9: complete — definitive source acceptance and beta.54 preparation.
- G10: canonical beta.54 packaging (next).
- G11: guarded installed-host validation.
- G12: tag, draft prerelease, and final M56 closure.
