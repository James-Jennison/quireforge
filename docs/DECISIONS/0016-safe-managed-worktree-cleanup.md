# ADR 0016: Safe managed-worktree cleanup and recovery

- Status: Accepted for Milestone 11C
- Date: 2026-07-20

## Context

Milestone 11A deliberately retained an app-created worktree when project
registration failed. It also distinguished worktrees QuireForge created from
worktrees the user merely attached. Milestone 11C needs to recover retained
worktrees and remove obsolete managed checkouts without turning a stale path,
database failure, active task, dirty checkout, or repository-configured helper
into data loss.

`git worktree prune` cannot target one worktree. Exposing it would let one UI
action remove unrelated stale administrative records. Force removal can discard
untracked or modified files. Neither operation is compatible with the required
app-owned target and reviewed-effect boundary.

## Decision

Worktree IPC schema version 2 adds two fixed previews. Recovery accepts an
app-owned project ID and an opaque recovery ID issued by the latest native
inventory. Removal accepts the currently selected app-owned project ID and the
target worktree project ID. The webview never supplies a path, branch, working
directory, executable, or Git argument.

A recovery ID is issued only for an unregistered linked worktree whose exact
canonical directory is one level beneath the source project's private
QuireForge worktree directory. Preview consumes that ID, binds the complete
directory and repository identity behind a new five-minute confirmation, and
confirmation revalidates it. Recovery registers the existing checkout as a
managed project without changing files, Git state, or the branch.

Filesystem cleanup is available only when the stored relation is `managed`.
Source, selected, attached, external, locked, and prunable worktrees are not
eligible. Preview and confirmation both verify the source relation, exact
private-storage shape, canonical directory identity, common Git directory,
linked-worktree inventory entry, branch, and current `HEAD`. Confirmation
reserves every QuireForge project in the repository group and refuses tracked,
untracked, conflicted, or submodule changes.

The native service invokes only fixed, shell-free Git commands with its existing
sanitized environment, timeout, and output bounds. Repository-configured
checkout/status filters are replaced with an app-controlled identity transform;
hooks, configured process filters, prompts, and global/system configuration stay
disabled. Removal uses `git worktree remove` without `--force`, never deletes
the branch, and verifies that the directory and inventory entry are gone while
the branch remains before changing app metadata.

After Git succeeds, one SQLite transaction detaches the directory association
and archives the worktree project while retaining its relation and conversation
references for history. If that transaction fails, inventory reports a missing
managed worktree. A second reviewed removal preview becomes metadata-only and
can finalize the transaction only while the path and Git inventory entry remain
absent. QuireForge never retries filesystem deletion automatically.

Generic prune, force removal, branch deletion, direct directory deletion,
attached/external worktree deletion, conflict resolution, and arbitrary Git
commands remain unavailable.

## Consequences

Committed work remains reachable through the preserved branch, and any
uncommitted state blocks cleanup. A local actor who changes a path, relation,
branch, `HEAD`, lock, or cleanliness after preview causes confirmation to fail
closed. Metadata failure after Git removal is visible and recoverable without a
second filesystem mutation.

Stale Git administrative entries that require repository-wide pruning remain a
manual Git responsibility. This is intentional: Git does not offer a safe
single-target prune primitive, and QuireForge does not trade unrelated worktree
safety for automatic cleanup.
