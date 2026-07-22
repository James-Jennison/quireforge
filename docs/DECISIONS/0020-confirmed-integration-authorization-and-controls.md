# ADR 0020: Confirmed integration authorization and controls

- Status: Accepted
- Date: 2026-07-22

## Context

Milestones 13A–14B established a normalized integration catalog, fixed plugin
and marketplace mutations, and the Integration Center. The reviewed Codex
0.145.x interfaces also support MCP OAuth, skill enablement changes, connector
authorization through a URL returned by Codex, and connector prompt mentions.
They do not establish a general connector-install RPC, a stable generic
configuration writer, or a production-ready plugin-management RPC.

Authorization URLs, connector paths, skill manifest paths, MCP names, OAuth
completion details, and configuration evidence are sensitive native state. A
generic bridge or a frontend-supplied URL/path would create a credential,
confused-deputy, and arbitrary-configuration boundary.

## Decision

Milestone 14C introduces `IntegrationControlService` as the native owner of the
supported authorization and enablement controls:

- React can request only connector authorization, MCP authorization, skill
  enable, or skill disable for one opaque normalized catalog ID. It cannot
  supply a URL, path, MCP name, config key/value, app path, protocol method, or
  arbitrary JSON.
- Preview requires a ready normalized capability and eligible current catalog
  state, resolves fresh native evidence through the reviewed Codex 0.145.x
  app-server routes, and stores that evidence behind a five-minute one-use
  UUIDv7 confirmation. At most 16 confirmations can remain pending and only one
  control or browser handoff can be active.
- Confirmation consumes the token, rechecks the catalog and exact evidence,
  then uses only `skills/config/write`, `mcpServer/oauth/login`, or the
  connector URL already returned by `app/list`. Skill success requires the
  exact effective-enabled response plus a fresh `skills/list` postcondition.
- Authorization URLs allow credential-free HTTPS, plus HTTP only for loopback
  callback handoffs. They are length bounded, cannot contain URL credentials or
  fragments, remain native-only, and are opened by Tauri from an opaque action
  ID. OAuth codes, tokens, raw errors, and URLs never enter serialized React
  state or QuireForge SQLite.
- MCP completion must match the exact native-held server name and successful
  completion notification. Connector completion is inferred only from fresh
  supported accessibility state. Either result triggers normalized catalog
  refresh.
- Conversation start accepts up to eight unique normalized connector entry
  IDs. The native service re-resolves each as accessible, enabled, callable,
  and safely identified, then constructs the documented `mention` item and
  `app://` path. React never supplies or receives that path.
- Explicit catalog refresh is fixed-purpose and non-destructive. Unsupported
  plugin enablement, generic connector installation/configuration, MCP
  add/remove/logout/configuration, arbitrary health repair, and generic config
  editing remain unavailable.

## Test-state isolation

Routine tests use deterministic shell app-server fixtures and sanitized shared
Rust/TypeScript fixtures. They verify exact routes, one-use confirmations,
postconditions, URL rejection, OAuth completion correlation, connector mention
construction, and raw-field exclusion. They do not inspect or change personal
Codex configuration, authentication, connectors, skills, MCP servers, or
account state and make no model call or third-party authorization request.

## Consequences

- Codex remains the credential and integration-state owner; QuireForge stores
  only short-lived process-local evidence and opaque action IDs.
- A CLI-minor change, malformed response, stale evidence, replayed token,
  unsafe URL, mismatched OAuth completion, unavailable connector, or failed
  skill postcondition fails closed with a stable diagnostic.
- Connector and MCP authorization share a consistent user review and external-
  browser handoff without pretending that an unsupported generic install API
  exists.
- Later integration work must receive a new route and policy review; it cannot
  reuse this service as a generic configuration or command bridge.
