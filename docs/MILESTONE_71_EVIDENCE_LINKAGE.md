# M71 — Evidence Linkage

## Outcome

M71 adds a private native, metadata-only claim → evidence → conclusion chain to
the M70 Knowledge Ledger. It links existing M48 artifact references, M52 typed
local-review evidence, package-validation summaries, and a bounded owner-trial
receipt to one knowledge record at a time.

## Boundary

- Evidence links retain only opaque source IDs, closed source classes, SHA-256
  digests, timestamps, and the closed owner-trial outcome vocabulary. They do
  not copy artifact bytes, paths, commands, screenshots, diagnostics, or
  provider/session data.
- Source ownership is resolved natively against the record's project. Callers
  cannot supply a digest, class, project, task, or package fact.
- Conclusions are separate immutable owner actions (`supports`, `contradicts`,
  or `inconclusive`). They never change a knowledge record's lifecycle status.
  A lifecycle transition remains an explicit M70 action.
- M71 is local and non-executing. It does not alter M60 context transfer, add
  agent access, dispatch a provider, or change M69C's Action Card boundary.

## Acceptance

- Migration 30 creates immutable evidence-link and conclusion tables with
  closed SQLite constraints and project-scoped reads.
- The native/Tauri/TypeScript bridge rejects unknown fields and invalid
  source/owner-trial combinations.
- Existing M48, M52, and package-validation records are resolved internally by
  immutable ID and digest; an owner trial stores a bounded receipt only.
- Tests prove a conclusion cannot silently change a knowledge record status and
  that evidence links cannot be modified after capture.
- Source validation, desktop E2E, and the clean-tree pinned Linux package gate
  must pass before installed-host claims are made.
