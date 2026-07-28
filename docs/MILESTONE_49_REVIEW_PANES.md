# Milestone 49 — QuireForge Review Panes

Status: implementation in progress for `0.1.0-beta.44`.

M49 adds a closed-by-default review-shell overlay to the dominant task
conversation. The shell itself is lazy loaded; each Files, Diff, Git, Preview,
Activity, and Approval implementation is its own lazy module. Closing the
shell unmounts the active pane and restores focus to its trigger. It creates no
timer, subscription, polling loop, durable record, or new native command.

| Pane | Existing typed source | Read-only boundary |
| --- | --- | --- |
| Files | `FilePreviewSnapshot` | Existing safe selected-file metadata only; no picker, opener, read, or write. |
| Diff | `git_status` then `git_diff` | One existing bounded changed-file diff after explicit pane open; no mutation. |
| Git | `git_status` | Existing branch/status summary after explicit pane open; no refresh loop or mutation. |
| Preview | M48 `advisor_generated_artifact_snapshot` and preview | Bounded generated text only after explicit pane open/selection. |
| Activity | existing normalized `ConversationEvent` state | Existing bounded task activity only. |
| Approval | existing `ConversationSnapshot.pendingApproval` | Existing proposal details only; no decision or dispatch action. |

Unavailable, empty, loading, failure, and truncation outcomes are explicitly
rendered. The shell uses semantic tabs and a labelled tabpanel, keeps focus
inside ordinary keyboard flow, and contains no live region beyond short
operation status text. Narrow layouts make the overlay full-width with bounded
insets. No pane imports terminal, worktree, browser, connector, Advisor
dispatch, approval-decision, Git-mutation, or project-write capabilities.
