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

The clean implementation commit is
`98fa8fa26d740572095c2dcd9d4c1f579156817b`. App-shell unit coverage passed
(`37` tests); complete frontend coverage passed (`266` desktop unit tests and
`46` desktop Playwright checks alongside `8` website Playwright checks); and
the repository, formatting, lint, type, Rust check/clippy/test, production
build, and distribution gates passed.

The pinned Ubuntu 22.04 Debian-only release set is bound to that exact clean
commit and `0.1.0-beta.35`:

- `quireforge_0.1.0.beta.35_amd64.deb` — SHA-256
  `430bc262807f0ba39e1cab41946ac78c9373414456fac1997fdc50eb567b8204`;
- `quireforge-sandboxd_0.1.0.beta.35_amd64.deb` — SHA-256
  `929e1256d404486ce75f7d902bdeea623bab97a249428fbade5d7fe11bb6fb7a`.

The release manifest records clean pinned-container provenance and highest
shipped `GLIBC_2.34`, within the Ubuntu 22.04 `GLIBC_2.35` ceiling. Checksums,
metadata, ABI evidence, disposable lifecycle/installed smoke, and a host
installed-Debian visible X11 launch all passed. No release or deployment was
performed.
