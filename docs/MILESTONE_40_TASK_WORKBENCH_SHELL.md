# Milestone 40 — QuireForge Task Workbench Shell

Milestone 40 is a presentation and interaction refinement of the existing
QuireForge workspace. It keeps the task conversation central while adding a
closed-by-default workbench context drawer, a compact keyboard-accessible
**Actions** palette, and a collapsed terminal dock that re-presents the
existing managed terminal registry.

## Closed scope

- The drawer exposes only current Diff, Git, and Problems summaries. The
  Problems view uses an honest empty state when no native problem feed exists.
- The Actions palette invokes existing safe navigation and existing drawer/dock
  presentation controls only.
- The terminal dock does not create a terminal, shell, PTY, command launcher,
  execution profile, or project attachment. It uses the already owned terminal
  workspace when the user opens it.
- The drawer is closed by default, remains independently scrollable, and turns
  into the existing narrow-width overlay pattern.

## Exclusions

M40 does not add generic upload, drag-and-drop, a new file type, attachment
collection, task continuity, generated artifacts, project write, Git write,
dispatch, provider, terminal, or execution authority. The future attachment
composer may present a single **Attach a file** action only after its separate
closed type-specific proposal is approved.

## Evidence

The beta.35 candidate requires App-shell accessibility tests, all desktop and
repository validation, production bundle enforcement, and the Debian-only
Ubuntu 22.04 provenance/ABI, lifecycle, installed-smoke, and visible-launch
release set. Package evidence is recorded only after the clean implementation
commit exists.
