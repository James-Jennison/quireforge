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

The Git evidence now distinguishes staged, unstaged, and untracked changes,
detached HEAD, missing upstream, ahead/behind counts, shallow repositories, and
merge, rebase, cherry-pick, or bisect markers. A controlled fixture proves that
local-only inspection preserves HEAD, index status, worktree content, and
untracked files. Existing-tracking inspection has the same non-mutation rule.
The separately authorized fetch mode may update remote-tracking refs, but never
accepts a caller-selected remote or refspec and must not modify HEAD, branch,
index, worktree, tracked files, or untracked files.

## Deferred work

Fixture repositories, package/validation/handoff readers, complete freshness
coverage, and mutation-safety integration tests remain 24B work. No 24C UI,
24D handoff generation, contradiction resolution, watcher, or autonomous
repair exists.
