# M69 — Chat-First Local Workspace

## Product intent

QuireForge is a native Linux AI workspace whose primary interaction is an
ordinary conversation. Chat is not a renamed governed-source review: a user
starts by typing a message, receives a bounded response in the conversation,
and chooses other workspace capabilities only when they are useful.

The experience may draw on the useful interaction patterns of leading AI
products, while retaining QuireForge's local ownership, provider neutrality,
and inspectable authority boundaries.

## First implementation slice

The default Chat route becomes a local-only, no-project conversation surface.
It sends only the text entered in its composer to the approved local runtime
and shows the bounded result inline. Each turn uses M63's CPU-only limits: one
attempt, at most 4,096 input tokens, 512 output tokens, 60 seconds, and the
supervisor's fixed 6 GiB cgroup limit.

Chat has no source selection, project attachment, repository access, terminal,
Git, browser, connector, credential, or external-provider authority. Its live
conversation is ephemeral by default; persistence, export, or promotion of a
message must be a separate explicit action with a visible destination and
receipt.

## Workspace actions

Conversation may offer clearly named actions such as “use a source”, “open a
project”, “draft an artifact”, or “work with code.” Choosing one opens the
existing governed surface and states what information, if any, will cross the
boundary. A chat message never silently becomes project context, selected
source content, a tool instruction, or a provider payload.

## Provider neutrality

The local runtime is the first Chat destination. Managed ChatGPT/Codex and any
future third-party provider remain optional, separately selected destinations
under M61, M62, and M67; they do not gain local Chat history, source content,
or authority by being displayed in the same application.

## Completion evidence

The first slice requires native and UI tests for no-project conversation,
fixed M63 bounds, cancellation, unavailable-runtime recovery, and proof that
Chat does not attach or retain project/source/tool authority. Installed-host
acceptance uses the existing supervised local-runtime launcher and preserves
content-free evidence rules.
