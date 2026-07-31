# Milestone 56 — Inspectable Local Task Templates

Status: approved and in progress. The template-only contract in the approved
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

No beta.54 source acceptance, package preparation, package, installation,
tag, or release exists.

## Remaining M56 sequence

- G5: complete — native digest-bound application preview, reservation, and
  atomic confirmation are implemented and covered by focused Rust tests.
- G6: complete — closed Tauri lifecycle/application commands and a strict
  TypeScript/Zod bridge validate only user text, opaque selectors, opaque
  native mutation handles, and explicit confirmation values. No UI is added.
- G7: complete — lazy native-backed template catalog, detail inspection, and
  local-template lifecycle UI are implemented with focused component coverage.
- G8: application UI and accessibility/real-browser acceptance (next).
- G9: full source acceptance and beta.54 preparation.
- G10: canonical beta.54 packaging.
- G11: guarded installed-host validation.
- G12: tag, draft prerelease, and final M56 closure.
