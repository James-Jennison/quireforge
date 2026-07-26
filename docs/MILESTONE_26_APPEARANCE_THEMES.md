# Milestone 26 — Appearance Themes

Status: hotfix candidate validated; host reinstall verification pending.

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

The original implementation commit is
`0ae0de7995f10128728116b148d49f2cb5b2cf79`. The selector-hotfix commit is
`8f7b505f24a489d468f02e82d0e6197606a83abe`: it adds direct Settings →
Appearance, restart persistence, and invalid-preference fallback regression
coverage, while documenting forced replacement of candidates that share the
same Debian version.
The full repository gate passed repository validation, package-contract tests,
TypeScript/Astro checking, ESLint, Prettier and Rust formatting checks, 192
desktop and 7 website unit tests, production builds and distribution budgets,
locked workspace Rust check, warning-denying Clippy, and locked Rust tests.
The Tauri no-bundle compilation passed. Full browser coverage passed the
desktop and website suites; the focused appearance scenario covers desktop and
mobile axe, keyboard, overflow, visual screenshots, and computed semantic
contrast checks across all eight palettes.

Fresh ignored release candidates were built from the clean hotfix commit with
`./scripts/run_linux_package_container.sh`. The digest-pinned
Ubuntu 22.04 builder contains `/usr/bin/xvfb-run`, and its manifest/checksum,
desktop-entry, PNG icon, disposable Debian lifecycle, visible Debian/AppImage
launches, and representative smoke checks passed with:

```bash
docker run --rm --init --user "$(id -u):$(id -g)" \
  --volume "$PWD:/workspace" --env HOME=/tmp --workdir /workspace \
  quireforge-packaging:ubuntu-22.04 /bin/bash -c \
  'command -v xvfb-run && python3 scripts/validate_release_artifacts.py \
    --artifact-dir target/ubuntu-22.04/release/packages --lifecycle --smoke'
```

| Artifact | Path                                                                           |             Size | SHA-256                                                            |
| -------- | ------------------------------------------------------------------------------ | ---------------: | ------------------------------------------------------------------ |
| Debian   | `target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.2_amd64.deb`       |  4,638,180 bytes | `772ce4bf25345ad70d11e30f31593ad0034e25be84e4389ed23fdf276df8ed7a` |
| AppImage | `target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.2-x86_64.AppImage` | 83,859,960 bytes | `d1b7cd4df1145d01e7c3b7cf50ac3cb87a07dfde5f703b2b0e96ddc5828ad380` |

The version-1 `release-manifest.json` and `SHA256SUMS` records under
`target/ubuntu-22.04/release/packages/` agree with both locally measured sizes
and SHA-256 values. The target is Ubuntu 22.04 `x86_64`; its maximum required
GLIBC is `2.34`, within the Ubuntu 22.04 `2.35` baseline.

The lifecycle gate creates a disposable lower-version Debian package, installs
it, upgrades it with the candidate, and removes it through the validator's
closed `dpkg --root=<temporary-root> --force-not-root --force-depends
--force-script-chrootless --install <package>` and `--remove quireforge`
operations. It proves that the executable and desktop entry are removed while
an attached-project file and local QuireForge metadata remain intact. The
actual visible launch commands are:

```bash
xvfb-run --auto-servernum python3 scripts/smoke_linux_package.py \
  --label 'Debian package' <extracted>/usr/bin/quireforge
xvfb-run --auto-servernum python3 scripts/smoke_linux_package.py \
  --label AppImage \
  ./target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.2-x86_64.AppImage \
  --appimage-extract-and-run
```

A local Debian installation would use:

```bash
sudo apt install --reinstall ./target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.2_amd64.deb
```

Candidate rebuilds deliberately retain the same prerelease package version.
Use `--reinstall` when replacing an already installed candidate so APT does not
retain an older package with the same Debian version and stale bundled assets.
The prior host installation was verified to have a different executable SHA-256
than this candidate, establishing that the missing selector was a stale-package
installation issue rather than an unreachable or conditionally hidden picker.
The candidate's container lifecycle and visible launch passed; an interactive
host reinstall and visible selector confirmation remain pending because this
automation environment has no sudo credential.

The bounded 24B reader's producer-compatible package parsing and both closed
metadata-only and local-artifact-verification contract paths are covered by its
strict Rust/TypeScript fixture suite. The manifest and checksum pair above
conform to that accepted producer format; reading them neither builds packages
nor changes the inspected repository.

## Deferred work

Per-user authored themes, operating-system preference synchronization, remote
profile synchronization, palette marketplace/import, layout-density changes,
and all new product capabilities are intentionally deferred.
