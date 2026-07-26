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
contracts, a version-7 bounded SQLite migration, and fixed native Chat
commands. Chat is ready only for normalized managed `chatgpt` authentication;
an API-key, managed-provider, missing, pending device-code, or unavailable
state cannot enable it. The fixed thread start has `cwd: null`, no environments,
no dynamic tools, read-only sandboxing, and `never` approvals. Any attempted
native tool or permission request blocks the Chat conversation rather than
asking the user to approve it. The only persisted Chat fields are opaque local
conversation and Codex-thread references with timestamps; no prompt, response,
project path, credential, or account identity is stored. Codex retains its
project-bound capability profile and existing records.

## Planned implementation phases

1. Establish managed-auth feasibility, strict mode contracts, the Settings
   foundation, and bounded mode-aware metadata. **Implemented.**
2. Add the no-project Chat profile only after native capability tests prove
   that its fixed app-server request cannot obtain project/native-action
   authority. **Native bridge implemented; user workspace wiring remains.**
3. Route the existing Codex conversation service through the native engine and
   implement explicit confirmed mode transitions with no automatic context
   transfer.
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

The user-visible Chat/Codex selector with a confirmed transition, unified
workspace event presentation, migration/backfill coverage, desktop/browser
accessibility coverage, complete regression suite, and final pinned Ubuntu
22.04 package/evidence gate remain.
