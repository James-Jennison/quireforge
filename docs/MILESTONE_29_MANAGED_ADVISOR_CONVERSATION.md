# Milestone 29 — Managed Advisor Conversation Foundation

Status: complete.

## Objective

Add the smallest real Advisor conversation capability through the documented,
locally supervised Codex app-server while preserving the separate Advisor,
Approval/Dispatch, and Codex Execution boundaries.

## Implemented scope

- A dedicated Advisor conversation service owns one transient native turn at a
  time. It uses the existing managed ChatGPT browser-login state reported by
  Codex; it accepts no password, API key, browser cookie, external token, or
  consumer ChatGPT endpoint.
- The fixed profile has no cwd, environments, dynamic tools, integration,
  approval path, network access, terminal, Git, worktree, or project-write
  authority. Any unexpected tool, activity, plan, or permission request is
  blocked and reported as a bounded diagnostic.
- QuireForge persists only an opaque Codex thread reference and timestamps in
  the existing Advisor metadata table. Prompt text, response text, transcripts,
  credentials, project paths, project identity, and model/reasoning choices are
  not persisted by QuireForge.
- A selected Project State summary remains temporary. Including it in a message
  requires a second, per-send confirmation; Rust re-reads only the existing
  fixed local-only/metadata-only safe projection and sends only its closed
  trust/freshness/worktree/diagnostic-count summary.

## Explicit exclusions

This milestone does not add an Approval/Dispatch controller, editable Codex
prompt generation, a Codex dispatch bridge, stored Advisor transcripts,
model/reasoning selection, arbitrary project browsing, images, screenshots,
terminal access, Git actions, watchers, automatic retry, automatic handoff,
contradiction resolution, API-provider configuration, or repository mutation.

## Verification plan

Rust tests cover the fixed no-project app-server wire profile, managed-account
gate, bounded metadata, and safe selected-state input. TypeScript/Zod tests
reject thread identifiers and malformed requests at the bridge. Advisor UI and
shell tests cover the composer, managed-account fallback, explicit snapshot
selection, per-send context confirmation, and absence of execution controls.

The desktop bundle retains all shipped chunks in its budget calculation. The
reviewed JavaScript ceiling is 875 KiB for this milestone; the production M29
build measures 874.69 KiB. The final repository validation also covers Rust
formatting, locked checks/tests, warning-denying Clippy, TypeScript, ESLint,
Vitest, production builds, distribution checks, package-contract tests, the
unbundled Tauri build, and desktop/mobile Playwright accessibility coverage.

## Final validation and package evidence

The clean implementation commit is
`45a6d5f2219a5531cf336ed27f0cf7f389d984be`, built as unique incremental
version `0.1.0-beta.6` (Debian internal version `0.1.0~beta.6`). The ignored
version-1 evidence is at
`target/ubuntu-22.04/release/packages/release-manifest.json`,
`target/ubuntu-22.04/release/packages/SHA256SUMS`, and
`target/validation-summary.json`.

| Artifact | Path | Size | SHA-256 |
| --- | --- | ---: | --- |
| Debian | `target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.6_amd64.deb` | 4,736,756 bytes | `f5d543d69066508f94780987020223512370c90bb1ac9374673f8ec9d9e42dda` |
| AppImage | `target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.6-x86_64.AppImage` | 83,995,128 bytes | `999771b32b4120d2fc07872264b8f320e828360439b4ab8b80f551a5b024982e` |

`pnpm validate`, the focused desktop/mobile Advisor browser coverage, and the
unbundled Tauri build passed. The digest-pinned Ubuntu 22.04 builder includes
`/usr/bin/xvfb-run`; it passed manifest/checksum agreement, desktop-entry,
icon, AppStream, maximum required `GLIBC_2.34` against the Ubuntu 22.04
`GLIBC_2.35` baseline, disposable Debian install/upgrade/remove lifecycle, and
visible Debian/AppImage smoke. The installed host upgraded from beta.5 to
beta.6 and confirmed the installed Debian and AppImage launch/smoke path.

The exact installed-host upgrade command was:

```bash
sudo apt install --reinstall -y \
  /mnt/faststorage/quireforge/target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.6_amd64.deb
```

The installed Debian executable is `/usr/bin/quireforge`; the AppImage launch
command is:

```bash
./target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.6-x86_64.AppImage \
  --appimage-extract-and-run
```

The designated package directory retains only the complete beta.6 release set;
no installed package, source file, Git history, remote release, or required
evidence was removed.

## Post-completion action-row hotfix

The scoped `0.1.0-beta.7` frontend hotfix separates the fixed read-only
capability notice from the Advisor action row. It adds an explicit disabled
reason for managed sign-in, an empty message, or a pending native action. A
selected Project State snapshot remains optional; its existing explicit
selection and per-send confirmation are unchanged. Desktop and narrow-width
browser coverage verifies focus order, no visual overlap or horizontal
clipping, screen-reader status/note semantics, and a captured action-row
visual.

The clean hotfix implementation commit is
`bcf796bcca9f61d594e094af4a6c556774ed45d8`. Its unique package version is
`0.1.0-beta.7` (Debian internal version `0.1.0~beta.7`).

| Artifact | Path | Size | SHA-256 |
| --- | --- | ---: | --- |
| Debian | `target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.7_amd64.deb` | 4,736,968 bytes | `514387667e703cc0bf85455f084f1a3bef1cfc380c5e1cd130756054ed97a280` |
| AppImage | `target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.7-x86_64.AppImage` | 83,995,128 bytes | `bfef9b795aea79c1a64bad5f01d24c47008533f1b459d919e245013cb9507bb4` |

The digest-pinned Ubuntu 22.04 builder includes `/usr/bin/xvfb-run` and passed
checksum/manifest agreement, maximum required `GLIBC_2.34` against the Ubuntu
22.04 `GLIBC_2.35` baseline, desktop/icon/AppStream checks, disposable Debian
install/upgrade/remove lifecycle, and visible Debian/AppImage smoke. The host
installed `0.1.0~beta.7`; the user approved the installed desktop visual check
for the readable Advisor notice and action row. The beta.6 output set was
moved to local trash only after the complete beta.7 set passed validation.

## Deferred work

Approval/Dispatch, user-editable Codex prompt generation, a Codex dispatch
bridge, stored Advisor transcripts, model/reasoning selection, arbitrary
project browsing, image or screenshot context, terminal/Git actions, watchers,
automatic retry, handoff generation, contradiction resolution, provider
configuration, and repository mutation remain separate approval-gated work.
