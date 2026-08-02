# QuireForge Repository Guidance

## Project state and authority

QuireForge is an unofficial native Linux workspace for planning and supervising AI-assisted development with Codex. It is built with Tauri, Rust, React, TypeScript, and Vite. `docs/CURRENT_STATE.md`, `docs/ROADMAP.md`, relevant ADRs, and the active milestone define current scope; do not claim a feature, package, website, or release exists until evidence and acceptance prove it.

`quireforge` remains the authoritative production website source. `quireforge-website-next` is a private redesign workspace and does not replace this authority until an explicitly approved cutover.

## Non-negotiable boundaries

- Work against selected project directories in place; do not substitute copied content or expand writable roots silently.
- Keep metadata separate from Codex authentication, configuration, sessions, and connector credentials. Never scrape ChatGPT/Codex UIs, use private endpoints, or fabricate documented interfaces.
- Task records, alternate plans, Advisor content, and review artifacts are local, bounded, and non-executing. Do not turn them into an executor or transmit provider context without the documented explicit, expiring, digest-bound dispatch flow.
- Keep provider boundaries neutral. Credentials, provider configuration, and live API calls remain outside versioned files; no provider is silently authoritative.
- Detach, archive, and remove are not filesystem deletion. Never commit credentials, private diagnostics, .env, support bundles, or personal Codex data.

## Change and validation workflow

Read the active milestone, current state, relevant ADR, and scoped subsystem files. Preserve existing work, keep changes milestone-scoped, and update roadmap/changelog evidence when project state or user-visible behavior changes. Run `pnpm validate`; add `pnpm test:e2e` when desktop/browser behavior is affected. Use documented package-validation gates before compatibility or installed-host claims.

Tauri is the active product implementation. QuireForge-Qt is a separate, fixture-first feasibility project; do not develop parallel product features or start a Qt migration without explicitly approved roadmap authority.
