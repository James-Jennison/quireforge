# Milestone 22B — Visual Workspace Refinement

Status: complete locally on `feat/milestone-22b-visual-workspace-refinement`.

## Objective

Refine the established routed Linux desktop workspace into a calmer, more
consistent operations surface without replacing routing, changing native
capabilities, or inventing remote account controls.

## Implemented checkpoints

- Consolidated repeated responsive shell rules and retained reduced-motion
  coverage within the enforced 100 KiB production CSS budget.
- Refined the persistent shell hierarchy: navigation, toolbar, and workspace
  surfaces remain the existing route-aware controls.
- Added Home's current-workspace section using existing attached-project state;
  it distinguishes verified Git and local workspaces and links only to existing
  New task, project-management, or attachment actions.
- Consolidated equivalent workspace-header rules for New task, Threads,
  Projects, Changes, Worktrees, and Terminal. Route-specific width and action
  constraints remain explicit.
- Aligned Scheduled, Integrations, Files, and Settings around the same calm
  surface, heading, status, and responsive conventions without flattening their
  read-only catalog, review, preview, or local-preference structures. The
  scheduled-card semantic heading now receives its intended shared presentation
  rule.
- Expanded the ignored Playwright visual evidence with reproducible mobile
  drawer captures for Scheduled, Integrations, Files, and Settings. The suite
  keeps axe and horizontal-overflow coverage for the exercised native fixtures.

## Boundaries

This work changes presentation only. Hash routing, mounted-workspace state,
Tauri commands, Codex protocol handling, authentication ownership, project and
Git safety controls, terminal behavior, usage reporting, and confirmation
requirements remain unchanged. No fixture data, account credentials, or new
remote management capability is introduced.

## Completion validation

The final repository gate passes TypeScript, ESLint, repository formatting, 33
desktop test files / 172 tests, 38 desktop/mobile Playwright scenarios
(including axe, overflow, and mobile-drawer evidence), production build, and
distribution-budget validation. The output is 189.01 KiB entry JavaScript,
289.75 KiB application JavaScript, 820.61 KiB total JavaScript, and 99.03 KiB
total CSS. Rust formatting, warning-denying Clippy, Rust tests, and Tauri
compilation also pass.

The pinned Ubuntu 22.04 workflow produces fresh `.deb` and AppImage candidates
with manifest, SHA-256, GLIBC-baseline, desktop-entry, icon, disposable Debian
lifecycle, and visible installed/AppImage launch evidence. No release,
deployment, merge, or remote-account action is part of this completion.
