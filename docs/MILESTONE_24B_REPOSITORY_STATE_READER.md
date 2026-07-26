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

Missing, unsafe, oversized, and malformed current-state evidence produces a
stable diagnostic rather than an invented branch value.

The Git evidence now distinguishes staged, unstaged, and untracked changes,
detached HEAD, missing upstream, ahead/behind counts, shallow repositories, and
merge, rebase, cherry-pick, or bisect markers. Deterministic temporary Git
fixtures cover dirty state, detached HEAD, missing upstream, and an ahead local
branch. The fixture fingerprint records symbolic and detached HEAD state, local
and remote-tracking refs, porcelain status, index bytes, safe local config,
operation markers, `FETCH_HEAD`, and tracked/untracked fixture-file contents.

Local-only and existing-tracking reads preserve that complete fingerprint. The
existing-tracking test deliberately points `origin` at an unavailable local path
after recording its tracking ref, proving it reads existing refs without fetch
or remote contact. The separately authorized fetch mode uses only the closed
`git fetch --no-tags --no-write-fetch-head origin` operation: a controlled bare
remote may advance its intended tracking ref, while HEAD, branch, local refs,
index, worktree, configuration, operation markers, files, and `FETCH_HEAD`
remain unchanged. This is limited authorized mutation, not a mutation-free
mode; callers cannot select a remote or refspec.

## Supported evidence and freshness

The reader uses a closed registry for the Ubuntu package manifest and
`SHA256SUMS`, `target/validation-summary.json`, and approved handoff phrases in
`docs/CURRENT_STATE.md`. These are bounded, non-symlink, UTF-8 files only; the
reader neither scans arbitrary documents nor runs validation or package work.
Missing optional evidence returns a partial snapshot with diagnostics.

Package and validation claims receive commit-based `current`, `stale`, or
`unknown` freshness independently from trust. Git observations are verified,
machine-readable files are verified as file contents, Markdown claims remain
reported, and suggested actions remain inferred. Malformed or absent evidence
is never silently repaired or treated as completion.

## Deferred work

Artifact checksum verification, broader validation-report formats, shallow
fixture coverage, and final package validation remain 24B work. No 24C UI, 24D
handoff generation, contradiction resolution, watcher, or autonomous repair
exists.
