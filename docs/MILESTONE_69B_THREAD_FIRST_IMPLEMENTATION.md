# M69B — Thread-First Agent Workspace Shell

## Purpose

M69B changes QuireForge's information architecture from peer Chat, Work, and
Code lanes to a Threads-first shell. It is a local presentation and navigation
slice only. It must make ordinary conversation feel primary while preserving
every existing Work and Code surface for migration and later conversational
invocation.

M69B does not implement local chat runtime turns (M69A), the Action Card or
authority gateway (M69C), project attachment (M70), durable project memory
(M71), model selection (M72), providers (M73), browser/connectors (M74), or
automation/collaboration (M75).

## User-visible result

The sidebar's primary destination is **Threads**. It shows a compact tree of
recent no-project conversations and existing project-associated conversations.
A user can create, select, search, and understand a thread without first
choosing a product lane.

The selected thread fills the main workspace. Existing Work/Code features
remain reachable through contextual thread content or the existing migration
entry points; this phase does not delete, move, or reinterpret their underlying
data or authority.

## Navigation model

Top-level navigation contains:

1. **Threads** — the default route and thread tree.
2. **Projects** — durable project inventory and project-scoped thread grouping.
3. **Settings** — application preferences and account/provider configuration
   that already exists; it is not a permission or ledger destination.

Legacy Work and Code route hashes remain accepted. When selected, they retain
their current UI and show a compact migration affordance that explains that
their capabilities will become thread-scoped; no capability is hidden or
removed in this phase.

## Thread tree and status model

A thread row contains a bounded title, optional current-project label, and one
status dot. The shared model is deliberately closed:

- `unread`: a filled dot for a new/updated existing thread state that the user
  has not viewed in this app session;
- `needsDecision`: a hollow emphasized dot only when an already-existing native
  state reports that a user decision is genuinely pending;
- no dot: neither state applies.

Project folders aggregate their children's status: `needsDecision` takes
precedence over `unread`. A collapsed project must never hide a real pending
decision.

M69B must not fabricate `needsDecision` from route, draft, loading, or
optimistic UI state. Until M69C introduces the Action Card gateway and a real
pending-decision event, a status source that cannot prove `needsDecision` must
return no such state.

## Main workspace composition

The selected-thread header displays only currently known local metadata:
thread title, a compact scope summary when available, and an explicit project
association when one already exists. It must not claim a local runtime, source,
provider, tool, or authority that is not actually active.

The existing conversation surface stays central. A local, collapsible activity
region may show existing bounded task/evidence metadata only when it exists.
It must remain absent or collapsed while idle. Existing full review, diff,
artifact, terminal, and project surfaces remain detail destinations, not new
permanent peer lanes.

## Migration and compatibility

- Preserve all legacy hash routes and all current selected-view behavior.
- Do not migrate, rewrite, create, archive, or delete thread/project records.
- Show a one-time local migration explanation when an existing user first sees
  Threads: Work and Code are becoming capabilities of a conversation; nothing
  has been removed or granted additional authority.
- The explanation is local presentation state only and can be dismissed.
- Preserve keyboard navigation, narrow-layout behavior, and existing
  accessibility labels.

## Native/TypeScript contracts

M69B may introduce presentation-only schemas for a thread tree and closed
status enum. The inputs must use only existing bounded conversation/project
metadata. They may not contain raw paths, source bytes, prompt/transcript
content, credentials, model details, ledger contents, provider identifiers, or
tool authority.

No new native command may start a model, attach a project, read a source,
execute a terminal/Git action, or transmit data.

## Required tests

1. Threads is the default accessible navigation destination.
2. Existing Chat, Work, and Code routes remain navigable by their prior hashes.
3. Thread rows and project folders render each closed status state correctly.
4. Folder aggregation preserves a real `needsDecision` state while collapsed.
5. Unknown/unsupported status values fail closed to no dot.
6. No status is fabricated from loading, drafts, or route state.
7. The migration affordance is dismissible, local-only, and does not alter
   thread/project metadata.
8. Desktop, narrow desktop, and mobile E2E coverage proves no existing surface
   becomes unreachable.
9. Native/TypeScript contract tests prove the thread-tree projection excludes
   paths, source content, credentials, provider/model data, and authority.

## Completion evidence

Run `pnpm validate` and `pnpm test:e2e`; package and installed-host acceptance
are required only if M69B changes native package behavior. The milestone report
must state explicitly that M69B introduced no local-runtime turn, Action Card,
project attachment, provider, source, tool, terminal, Git, browser, connector,
or automation capability.
