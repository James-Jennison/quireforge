# M69 — Chat-First Local Workspace

## Product intent

QuireForge is a native Linux agent workspace whose primary interaction is an
ordinary, persistent conversation. Chat is not a renamed governed-source
review: a user starts by typing a message, receives a bounded response in the
conversation, and gives the agent explicit capability only when it is useful.

The experience may draw on the useful interaction patterns of leading AI
products, while retaining QuireForge's local ownership, provider neutrality,
and inspectable authority boundaries. The conversation carries intent,
narration, decisions, and compact evidence; it is the substrate for both casual
talk and project work.

An authorized implementation request remains active through inspection,
diagnosis, change, validation, and its approved local acceptance path.
Inspection is progress evidence, never a terminal state. The agent may stop
only after the requested outcome is proven, or when it names a specific
external blocker or human-only decision that prevents the next action.

Work and Code are capability states of a thread, not peer top-level product
lanes. A thread may begin as local-only chat and later attach an approved
project, source, artifact, or coding capability through a visible boundary
crossing. That crossing remains revocable and recorded without making the user
leave the conversation.

## First implementation slice

The default route becomes Threads: a local-only, no-project conversation
surface. It sends only the text entered in its composer to the approved local
runtime and shows the bounded result inline. Each turn uses M63's CPU-only
limits: one attempt, at most 4,096 input tokens, 512 output tokens, 60 seconds,
and the supervisor's fixed 6 GiB cgroup limit.

Chat has no source selection, project attachment, repository access, terminal,
Git, browser, connector, credential, or external-provider authority. Its live
conversation is ephemeral by default; persistence, export, or promotion of a
message must be a separate explicit action with a visible destination and
receipt. Project-attached threads persist by default because their work is
stateful; no-project threads are ephemeral by default and can be explicitly
promoted to persistent local storage.

The header shows a compact current scope, such as “No project · Local runtime ·
Ephemeral.” The header expands to the thread's content-free boundary history;
it is not a separate governance screen.

## Workspace actions

Conversation may offer clearly named contextual actions such as “use a source”,
“open a project”, “draft an artifact”, or “work with code.” Choosing one emits
a shared Action Card that states the intended capability, what will be used,
what will happen, the data crossing, scope, cancellation/revocation choice, and
expandable detail. A chat message never silently becomes project context,
selected source content, a tool instruction, or a provider payload.

The main conversation contains intent, narration, decisions, compact evidence,
and approval cards. A collapsible side panel contains only live work activity,
current project state, and scope. Full diffs, logs, artifacts, and ledger
history open in a detail view rather than overwhelming the conversation. First
use of a boundary type receives an explanatory card; later uses in the same
thread collapse to a compact confirmation.

## Provider neutrality

The local runtime is the first Chat destination. Managed ChatGPT/Codex and any
future third-party provider remain optional, separately selected destinations
under M61, M62, and M67; they do not gain local Chat history, source content,
or authority by being displayed in the same application. The first M69 slice
does not add provider configuration, credentials, browser, connector, tools,
or cloud transmission.

## Native and data architecture

`LocalChatService` is structurally separate from the M60 reviewed-context path:
its input is only bounded chat text and M63 turn limits, with no
project/source/ledger types in its IPC contract. A native authority gateway is
the sole executor of every later boundary crossing; it classifies the action's
approval tier, records its content-free receipt, and refuses execution before
the required decision. React presentation cannot bypass this gateway.

Local thread storage uses a bounded thread record with an ephemeral flag and
optional project reference. Project memory is an inspectable, editable local
record, not an implicit reconstruction of chat history. Promoting a message to
project memory uses an explicit Chat-to-Project Action Card and receipt.

## Delivery order

1. Establish the Threads-first shell, the scoped local conversation, and a
   shared Action Card contract.
2. Add the thread scope header, local persistence promotion, and a compact
   chronological work timeline over existing evidence/task metadata.
3. Make project attachment and governed-source admission conversational
   invocations while preserving their existing native contracts.
4. Only after those flows are stable, add optional per-thread provider
   destinations through their individual approved capability gates.

The thread-first information-architecture implementation is separately scoped
as [M69B — Thread-First Agent Workspace Shell](MILESTONE_69B_THREAD_FIRST_IMPLEMENTATION.md).
The prerequisite local-chat implementation is separately scoped as
[M69A — Threads-First Local Chat](MILESTONE_69A_LOCAL_CHAT_IMPLEMENTATION.md).

## Completion evidence

The first slice requires native and UI tests for no-project conversation, fixed
M63 bounds, cancellation, unavailable-runtime recovery, no ambient
project/source/tool authority, Action Card consistency, scope-header accuracy,
and migration from the existing lanes without lost functionality. Installed-host
acceptance uses the existing supervised local-runtime launcher and preserves
content-free evidence rules.
