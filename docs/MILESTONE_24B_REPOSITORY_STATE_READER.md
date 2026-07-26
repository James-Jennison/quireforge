# Milestone 24B — Repository State Reader

Status: active implementation on `feat/milestone-24b-repository-state-reader`.

## Scope

The reader observes one already attached QuireForge project and returns a
version-1 project-state contract plus diagnostics. It accepts no filesystem
path, arbitrary Git argument, or background-scan request.

## Initial reader boundary

Rust resolves the project through `ProjectService::review_root`, then uses the
existing closed, credential-free Git command environment. Local-only reads do
not inspect remote refs. Existing-tracking reads use only present refs. The
explicit `fetch-authorized` mode is the sole remote-refresh path and exists
only under James's approved boundary; it is never the default.

Document claims are reported evidence. The initial narrow document reader only
compares the documented current-state branch with verified Git evidence. It
does not rewrite documents or select a winner. Diagnostics remain separate from
contract truth and suggest only inferred next actions.

## Deferred work

Fixture repositories, package/validation/handoff readers, complete freshness
coverage, and mutation-safety integration tests remain 24B work. No 24C UI,
24D handoff generation, contradiction resolution, watcher, or autonomous
repair exists.
