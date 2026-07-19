# Testing QuireForge

Status: Milestone 2 establishes repository and website tests. Desktop, Rust,
Tauri, PTY, Git fixture, directory attachment, database migration, and Codex
adapter suites arrive with the milestones that introduce those systems.

## Repository and website checks

Run these commands from the repository root after installing locked
dependencies:

```bash
python3 scripts/validate_repository.py
pnpm check
pnpm lint
pnpm format:check
pnpm test
pnpm build
pnpm validate:dist
```

`pnpm validate` runs that non-browser sequence as one command. The checks cover
required repository files, secret-like tracked files, local documentation
links, QuireForge identity values, Astro and TypeScript correctness, linting,
formatting, content-model unit tests, the production build, routes, generated
assets, internal links, canonical URLs, the unofficial disclaimer, inline-code
restrictions, and version-controlled security headers.

## Responsive browser and accessibility checks

Install the Playwright Chromium browser once, then run the suite:

```bash
pnpm --filter @quireforge/website exec playwright install chromium
pnpm test:e2e
```

On a Linux workstation with an already installed compatible Chromium, avoid a
download by setting its executable only for the test command:

```bash
PLAYWRIGHT_CHROMIUM_EXECUTABLE=/path/to/chromium pnpm test:e2e
```

The suite exercises desktop and mobile viewports, every public route,
horizontal overflow, semantic page structure, light/dark theme persistence,
and axe-core checks on the home and integration pages. Automated accessibility
checks complement rather than replace keyboard, screen-reader, zoom, and visual
review.

GitHub Actions installs its own isolated Chromium and runs the same suite. It
does not deploy the site and receives no Cloudflare credentials.

## Manual Milestone 2 checklist

- Inspect Home and Integrations in light and dark themes.
- Inspect at desktop and narrow mobile widths.
- Navigate the header, theme control, page content, and footer by keyboard.
- Confirm focus remains visible and reduced motion is honored.
- Confirm no clipped text, horizontal scroll, stale identity, or broken asset.
- Confirm Downloads and Installation do not claim an unreleased package.
- Confirm the unofficial-project disclaimer remains visible.
- Confirm the built `_headers`, `robots.txt`, sitemap, manifest, icons, and 404
  page are present.

Production-origin Lighthouse and live-header measurements are deferred until a
separately approved Cloudflare preview or production deployment exists. Any
miss against the published quality targets must be recorded with remediation.
