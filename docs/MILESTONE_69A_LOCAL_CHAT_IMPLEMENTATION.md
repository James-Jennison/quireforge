# M69A — Threads-First Local Chat

## Purpose

M69A establishes the first ordinary QuireForge conversation: a local-only,
no-project chat turn from text entered in the composer to a bounded result
rendered inline. It makes no request to use a source, project, artifact, Git,
terminal, browser, connector, credential, provider, or external network.

M69A is deliberately separate from M60 governed-context delivery. M60 controls
reviewed source-derived payloads. M69A accepts only the user's bounded typed
chat text and must not gain a way to carry M60/M48/M55 data.

## Native service

Implement `LocalChatService` as a dedicated Rust module and typed IPC boundary.
Its request contains only a bounded opaque user-text value and its response
contains a bounded local-result lifecycle suitable for the current view. Its
public request/response types must not reference project, source, artifact,
reviewed-context, ledger, provider, tool, filesystem, or credential types.

`LocalChatService` reuses the verified local inference primitive only behind
that narrow type boundary. It applies the fixed M63 execution limits to every
turn:

- CPU-only execution;
- one attempt per submitted turn;
- at most 4,096 input tokens;
- at most 512 output tokens;
- a 60-second deadline;
- the supervisor-enforced 6 GiB cgroup limit.

The service reports stable, content-free outcome categories for unavailable
runtime, rejected input, timeout, cancellation, and completed turn. It must not
expose a model location, model bytes, loader output, filesystem observation,
or raw diagnostic content.

## Conversation UI

Add a local-chat conversation route that presents:

- a familiar multiline composer and send control;
- one visible local response area per completed turn;
- cancel while a turn is active;
- a concise, actionable unavailable/timeout/cancelled status;
- an explicit retry only after a completed unsuccessful turn;
- a compact initial scope summary: `Local runtime · No project · Ephemeral`.

The initial conversation is renderer-local and ephemeral. M69A does not create
durable thread records, search, migration, project association, project memory,
artifacts, tasks, Action Cards, or a pending-decision model. M69B adds the
Threads navigation and M69C adds the shared authority grammar.

The UI contains no source selector, attach-project affordance, provider picker,
model picker, terminal/Git action, file picker, or tool button in this phase.
Those capabilities must not be hidden behind developer-only UI or IPC paths.

## Cancellation and concurrency

- One local turn may be active for the view at a time.
- Submitting while active is refused without starting a second runtime attempt.
- Cancel is an explicit user action and leaves the composer usable after the
  native service reaches its terminal cancelled state.
- Closing/resetting the view disposes only its local ephemeral presentation
  state and does not attempt to recover model data or generated output.

## Required tests

1. Rust contract tests prove `LocalChatService` request/response types cannot
   carry project/source/artifact/review/ledger/provider/tool authority.
2. Native tests cover empty/NUL/oversize input refusal, one-attempt admission,
   cancellation, deadline, unavailable runtime, and content-free diagnostics.
3. UI tests cover send, busy refusal, cancel, retry after terminal failure,
   response rendering, and no-project scope presentation.
4. UI tests prove the M69A composer presents none of the future capability
   controls listed above.
5. Regression tests prove M60 governed review remains unchanged and cannot
   dispatch through `LocalChatService`.
6. Desktop and narrow/mobile E2E coverage proves an ordinary local chat turn
   is usable without a project, source, provider, or settings flow.
7. Installed-host acceptance uses the supervised launcher and retains only
   content-free package/lifecycle evidence.

## Completion

Run `pnpm validate`, `pnpm test:e2e`, the pinned package/release-artifact gate,
and fresh supervised installed-host acceptance. Update M69 status only after a
new packaged candidate completes those gates. No cloud, connector, browser,
tool, provider, project, persistence, or Action Card capability may be claimed
as part of M69A.

## Candidate implementation

The beta.90 candidate retains the typed `LocalChatService`, dedicated
`local_chat_run`/`local_chat_cancel` IPC, and the ephemeral Chat composer. Its
native request is only a bounded message string; it uses a local-chat-specific
runtime reservation and prompt, preserving the M60 reviewed-request prompt and
contract. Enter submits a normal turn while Shift+Enter remains multiline; the
visible transcript retains only the typed message and bounded result for the
open renderer session. Direct date/time questions return the host local clock
without a model attempt, while ordinary references to time remain ordinary
bounded local-runtime requests. Desktop and narrow/mobile E2E cover a normal
no-project chat turn. The candidate remains incomplete until the package,
installation, and supervised installed-host acceptance gates have fresh
evidence.
