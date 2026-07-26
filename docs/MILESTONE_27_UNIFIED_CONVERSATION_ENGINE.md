# Milestone 27 — Unified Conversation Engine

Status: in progress.

## Objective

Introduce one native conversation boundary with materially distinct Chat and
Codex modes. Chat is a no-project conversation profile; Codex remains the
attached-project, approval-bound software-development profile. A mode is a
capability policy and persisted metadata, never a presentation-only label.

## Managed ChatGPT authentication feasibility gate

Chat requires the supported Codex app-server managed browser login with
`account/login/start` type `chatgpt`. The app-server owns the browser callback,
credentials, refresh, and logout. QuireForge accepts only a bounded status and
the reviewed official-host handoff URL.

- Passwords, API keys, access tokens, refresh tokens, cookies, browser storage,
  account identifiers, and raw authentication responses must never enter
  QuireForge storage, React state, diagnostics, fixtures, or package evidence.
- The experimental externally managed `chatgptAuthTokens` path is prohibited.
  QuireForge must never supply or refresh a token for Codex.
- OpenAI API/project-key access is separately managed and billed. It is not a
  fallback for the required managed ChatGPT-account path.
- This gate authorizes only documented Codex app-server conversations. It does
  not authorize use of consumer ChatGPT APIs, history, browser sessions, or
  branded product behavior.

The first implementation slice adds matching Rust/Zod readiness and capability
contracts. Chat is ready only for normalized managed `chatgpt` authentication;
an API-key, managed-provider, missing, pending device-code, or unavailable
state cannot enable it. The closed Chat policy has no attached project, native
actions, terminal, Git, worktree, or integration capability. Codex retains
those capabilities and attached-project requirements.

## Planned implementation phases

1. Establish managed-auth feasibility, strict mode contracts, and the Settings
   foundation.
2. Add transactional mode-aware conversation metadata while preserving existing
   Codex reference rows and M26 local appearance preferences.
3. Route the existing Codex conversation service through the native engine and
   implement explicit confirmed mode transitions with no automatic context
   transfer.
4. Add the no-project Chat profile only after native capability tests prove
   that its fixed app-server request cannot obtain project/native-action
   authority.
5. Close full Rust/TypeScript/desktop/browser validation and the pinned Ubuntu
   22.04 Debian/AppImage package gate.

## Settings foundation

The existing Settings route now has stable destinations for General,
Appearance, Chat, Codex, Permissions & safety, Models & providers,
Integrations, Privacy & data, Keyboard shortcuts, and About & updates.
Unsupported controls remain absent rather than simulated. The former
`#settings/accounts` deep link resolves safely to General; M26 Appearance and
its local `quireforge-theme` preference remain unchanged.

## Boundaries

- No consumer ChatGPT API, token reuse, credential collection, browser
  scraping, automatic handoff, watcher, contradiction resolution, repair, or
  repository-state semantic change is included.
- A Chat/Codex switch must require confirmation whenever capabilities or project
  context would change. It must never transfer a repository, file attachment,
  approval, terminal, integration, or hidden transcript automatically.
- Codex remains authoritative for authentication and session data; QuireForge
  SQLite stores only bounded local metadata.
- No QuireForge-Qt work occurs.

## Remaining work

Mode-aware persistence, native engine routing, no-project Chat request and
deny-by-default event handling, explicit mode-transition UI, direct migration
coverage, desktop/browser accessibility coverage, and the final pinned Ubuntu
22.04 package/evidence gate remain.
