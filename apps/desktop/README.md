# QuireForge Desktop

Status: desktop work through Milestone 15B is implemented and verified locally.
This package contains the Tauri shell, typed Rust/TypeScript boundary, supported
Codex app-server adapter, Codex-owned authentication handoff, direct local
project attachment, conversation/session/approval presentation, reviewed Git
status/diff/mutation, bounded parallel worktrees, retained-worktree recovery,
explicit clean managed-worktree removal, native PTY tabs, normalized/confirmed
integration workflows, safe project-file previews, and bounded PNG/JPEG
conversation attachments. Generic file attachment, worktree prune, force
cleanup, advanced remote Git operations, packaging, and release workflows
remain separately gated.

Run package checks from the repository root with `pnpm validate`. Start the
native development window with `pnpm desktop:dev` after installing the Linux
packages documented in [`docs/BUILDING.md`](../../docs/BUILDING.md).

`pnpm codex:schema` refreshes only the reviewed initialize and `model/list`
schema subset for the installed Codex CLI. Review the generated diff and update
the versioned adapter before accepting a new schema snapshot.
