# Milestone 26 — Appearance Themes

Status: implementation complete; fresh package evidence pending.

## Objective

Add a small, palette-only local appearance system without changing QuireForge's
layout, typography, native bridge, repository-state contract, or product
behavior. Forge remains the default QuireForge appearance. The closed built-in
palette also includes Midnight Atelier, Blueprint Terminal, Signal Noir,
Aurora Workbench, Obsidian & Copper, Monochrome Editorial, and Pacific Night.

## Implementation

- `appearanceThemes.ts` owns the closed built-in registry, default, legacy
  `dark`/`light` migration, local-storage key, and DOM application helper.
  An absent or unsupported stored value safely falls back to Forge; the former
  light preference maps to Aurora Workbench.
- Startup applies the restored local selection before React renders, then the
  application persists only the selected palette identifier under
  `quireforge-theme`. No system-theme, account, Codex, repository, or native
  setting is read or written.
- Settings → Appearance provides a labelled radio group with visible palette
  samples, pointer/focus live preview, click selection, and arrow-key cycling.
  Leaving the group restores the saved selection unless a choice was made.
  The existing top-bar and signed-out appearance controls are shortcuts to the
  same closed local behavior.
- CSS semantic tokens cover shell, workspace, composer, terminal, focus,
  hover, disabled, warning, success, and error surfaces. Terminal colors read
  the active token values when a terminal starts or the selected palette
  changes. Palette preview swatches are intentionally static illustrations,
  not copied product artwork.

## Accessibility and responsive behavior

All palette choices retain visible focus, reduced-motion, forced-colors, and
existing compact/mobile rules. Direct browser coverage selects each palette on
desktop and mobile, checks text, muted text, accent, success, warning, and
error contrast against its active background, validates the settings route
with axe, verifies keyboard selection and persistence, checks overflow, and
captures visual evidence.

## Boundaries

- No custom theme authoring, import, marketplace, font, density, or layout
  redesign is included.
- No Rust, Tauri command, backend, repository-state, Codex configuration, or
  native persistence contract changed. Theme selection is local presentation
  preference only.
- No watcher, background scan, autonomous action, generated handoff,
  contradiction resolution, or repair behavior was added.
- No OpenAI or ChatGPT logo, artwork, or branded content was copied. No
  QuireForge-Qt work occurred.

## Verification and package gate

Implementation validation covers TypeScript, direct appearance/App/terminal
tests, browser accessibility/visual evidence, formatting, linting, production
build, and the repository validation gate. Because themes change the installed
desktop application, final completion still requires fresh, clean pinned Ubuntu
22.04 Debian and AppImage candidates, lifecycle and launch checks, reader-safe
package evidence, and a separate documentation-evidence commit.

## Deferred work

Per-user authored themes, operating-system preference synchronization, remote
profile synchronization, palette marketplace/import, layout-density changes,
and all new product capabilities are intentionally deferred.
