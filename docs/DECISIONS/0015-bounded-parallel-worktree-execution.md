# ADR 0015: Bounded parallel worktree execution

- Status: Accepted for Milestone 11B
- Date: 2026-07-20

## Context

Milestone 11A made every managed or attached worktree an ordinary QuireForge
project, but the conversation runtime still owned one global active process.
Replacing that slot with unconstrained process spawning would make task
identity, approvals, cancellation, project reservations, resource use, and
restart recovery ambiguous. React also needs aggregate status without receiving
raw app-server messages, native thread/turn IDs, working directories, or
process handles.

## Decision

`ConversationService` owns a bounded registry of at most four active
conversations. The registry is keyed by app-generated conversation UUIDv7 and
each entry has its own asynchronous lock. The registry lock is held only while
reserving capacity, locating an entry, inserting a started task, or removing a
terminal task on active-task paths; task app-server I/O occurs under only the
exact conversation lock. Existing all-session reconciliation remains
serialized while no task is active so lifecycle discovery cannot race a new
start.

The existing `ProjectService` reservation remains authoritative for project
ownership. One project may own at most one active task, while distinct attached
worktree projects may run concurrently. Start and resume count in-progress
starts against capacity, release their provisional slot on every failure, and
release the project reservation only after the owned child is shut down.
Polling, approvals, interruption, and terminal transitions first resolve the
app conversation ID to its exact native-owned entry. A per-conversation action
generation prevents an older frontend poll from overwriting a newer approval
or interruption result.

A strict versioned registry IPC returns at most four already-normalized
conversation snapshots with empty event batches. This lets a refreshed webview
recover active app IDs and resume independent polling without persisting
transcript content. React keeps bounded in-memory event views keyed by project,
and the worktree monitor combines those normalized states with the existing
read-only `GitService` snapshot for changed-file and unmerged-conflict counts.
Selecting a task opens the existing expandable normalized activity stream.

The monitor does not expose Codex thread/turn/item IDs, raw protocol messages,
working directories, executable paths, argument vectors, or raw Git output. It
does not resolve conflicts or perform worktree cleanup. A native application
restart cannot recover subprocess ownership, so existing database-open recovery
continues to mark stale active rows interrupted.

## Consequences

Slow output or an approval on one task does not hold the registry lock or block
unrelated worktrees. Four concurrent app-server children are a deliberate local
resource ceiling, not a claim that every machine should use all CPU threads.
Git conflict presentation is advisory and read-only; users must review and
resolve conflicts through separately authorized workflows. Completed task
events remain ephemeral, and only the existing reference metadata is stored.

Milestone 11C remains separately gated for worktree recovery, removal, pruning,
and filesystem cleanup.
