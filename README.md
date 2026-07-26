# QuireForge

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/brand/quireforge-lockup-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="assets/brand/quireforge-lockup.svg">
  <img alt="QuireForge — Build boldly. Work locally." src="assets/brand/quireforge-lockup.svg" width="620">
</picture>

> **Build boldly. Work locally.**

> [!IMPORTANT]
> QuireForge is an unofficial community project. It is not made, endorsed,
> supported, or distributed by OpenAI.

QuireForge is a native Linux workspace for planning and supervising
AI-assisted software development with Codex. It keeps user-selected projects
in place, uses documented interfaces, and keeps QuireForge metadata separate
from Codex authentication, configuration, and session data.

The canonical desktop identifier is `io.github.codeframe78.QuireForge`; the
static project site is [quireforge.jamesjennison.net](https://quireforge.jamesjennison.net).

## What QuireForge provides

- A Tauri desktop workspace built with Rust, React, TypeScript, and Vite.
- Managed Codex account onboarding and bounded native conversation workflows.
- User-selected project attachment, reviewed Git and worktree workflows, and a
  native integrated terminal with explicit controls.
- Read-only normalized Project State evidence and a managed, transient Advisor
  workspace with explicitly confirmed temporary context.
- Accessible routed workspaces, local appearance preferences, and reproducible
  Ubuntu 22.04 Debian/AppImage candidate validation.

The current implementation status, completed milestones, known limitations,
and next approved work are maintained in [Current State](docs/CURRENT_STATE.md).

## Safety boundaries

- QuireForge does not collect passwords, browser cookies, session tokens, or
  API keys for Codex or ChatGPT authentication.
- The frontend uses narrow typed native commands; it does not spawn arbitrary
  processes or consume raw Codex protocol messages.
- Project operations are scoped to attached directories and explicit user
  actions. Detach, archive, and remove actions are not filesystem deletion.
- Advisor remains non-executable. Its Phase A draft controller can record an
  explicit, expiring digest-only approval, but cannot dispatch, start Codex,
  or change a project; optional Project State context still requires explicit
  selection and per-send confirmation.
- The project does not scrape ChatGPT, reverse engineer private protocols, or
  imply OpenAI endorsement.

See the [architecture](docs/ARCHITECTURE.md), [threat model](docs/THREAT-MODEL.md),
and [Codex integration findings](docs/CODEX-INTEGRATION.md) for the detailed
boundaries.

## Quick start

Use a Linux host with the supported Node, pnpm, Rust, Tauri, Python, and Git
prerequisites described in [Building](docs/BUILDING.md).

```bash
pnpm install --frozen-lockfile
pnpm validate
pnpm test:e2e
pnpm desktop:dev
```

`pnpm validate` is the full non-browser quality gate. `pnpm test:e2e` adds the
desktop browser and accessibility coverage. See [Testing](docs/TESTING.md) for
targeted checks and [Building](docs/BUILDING.md) for the pinned Ubuntu 22.04
package workflow.

## Repository layout

| Path | Purpose |
| --- | --- |
| `apps/desktop/` | Tauri desktop application, Rust core, React UI, and desktop tests |
| `apps/website/` | Static Astro website |
| `docs/` | Architecture, decisions, milestone evidence, and operating guidance |
| `scripts/` | Repository, package, and release-validation utilities |
| `assets/brand/` | QuireForge brand sources and usage guidance |

## Documentation

Start with the [documentation index](docs/INDEX.md). The most common references
are:

- [Current State](docs/CURRENT_STATE.md) and [Roadmap](docs/ROADMAP.md)
- [Architecture](docs/ARCHITECTURE.md) and [decisions](docs/DECISIONS/)
- [Building](docs/BUILDING.md), [Testing](docs/TESTING.md), and
  [Releasing](docs/RELEASING.md)
- [Compatibility](docs/COMPATIBILITY.md), [Website](docs/WEBSITE.md), and
  [Changelog](CHANGELOG.md)
- [Milestone evidence](docs/), [forecasts](docs/MILESTONE-FORECASTS.md), and
  [time ledger](docs/MILESTONE_TIME_LEDGER.md)

## Contributing and support

QuireForge is licensed under the [Apache License 2.0](LICENSE). Read
[Contributing](CONTRIBUTING.md), the [Code of Conduct](CODE_OF_CONDUCT.md), and
the [security policy](SECURITY.md) before opening a contribution or report.
For project-status expectations and sanitized diagnostics guidance, see
[Support](SUPPORT.md).
