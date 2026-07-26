# Milestone 25 — Desktop Visual Polish

Status: complete.

## Objective

Refine the existing QuireForge desktop presentation so its density, hierarchy,
and interaction polish feel familiar to modern native AI-workspace clients
while retaining QuireForge's own name, warm accent, local-first language, and
native behavior.

## Approved boundary

This milestone changes only the React presentation layer, shared CSS, and
visual/accessibility regression coverage. It refines the existing sidebar,
top bar, Home empty state, project entry affordances, and native conversation
composer. The composer continues to expose only existing attachment, model,
reasoning, filesystem, approval, and start/stop controls. No voice capability
exists in the reviewed native contract, so this milestone does not imply or
simulate one.

The presentation remains branded as QuireForge. It does not copy OpenAI or
ChatGPT logos, artwork, or branded content.

## Exclusions

- No Rust, Tauri command, backend, or repository-state contract change.
- No project-state reader, Git, package, validation, or persistence behavior.
- No watcher, background scan, autonomous action, handoff generation, or
  contradiction resolution.
- No QuireForge-Qt work, Qt migration, external asset import, release, or
  merge.

## Implementation

- A denser 244px desktop sidebar with compact 36px navigation targets and a
  clear inset active treatment preserves keyboard links and compact-rail use.
- The 58px top bar groups route context and existing actions on a consistent
  alignment line without introducing native-window controls or commands.
- Neutral layered dark surfaces, borders, and text hierarchy retain the
  QuireForge amber accent rather than copying another product's palette.
- Home prioritizes a centered task entry surface. The conversation route uses
  a calmer header, rounded composer and stream panels, a larger centered empty
  state, and a separated action row.
- Responsive drawer/rail rules, visible focus, forced-colors behavior, and
  reduced-motion handling continue to use the existing shell mechanisms.

## Verification

The focused browser regression checks desktop and mobile composer geometry,
active sidebar state, keyboard focus, axe results, overflow, and screenshots.
The full repository gate passed formatting, TypeScript, ESLint, 188 desktop and
7 website unit tests, production builds, distribution budgets, Rust check,
warning-denying Clippy, 193 Rust tests (3 documented manual probes ignored),
and the Tauri no-bundle compilation. The full browser suite passed 42 desktop
and 8 website Playwright scenarios, including axe, keyboard, responsive, and
overflow coverage.

Fresh ignored artifacts were built from clean implementation commit
`9ae07167448d81c18c8e6fb293ffe52a146b346b` with
`./scripts/run_linux_package_container.sh` in the digest-pinned Ubuntu 22.04
container. Its release-manifest and checksum records are:

- `target/ubuntu-22.04/release/packages/release-manifest.json`
- `target/ubuntu-22.04/release/packages/SHA256SUMS`

| Artifact | Path | Size | SHA-256 |
| --- | --- | ---: | --- |
| Debian | `target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.2_amd64.deb` | 4,636,464 bytes | `0592f36f6a00ec1e4e835fc52821cab205625dd3a80598f4dd8a12a5b4792b93` |
| AppImage | `target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.2-x86_64.AppImage` | 83,855,864 bytes | `240c4927aad116e6ab4fbb7d0d7ea86bd69dee3f84327bd6b149e7f3b48b487e` |

The artifacts are `x86_64` Ubuntu 22.04 candidates. The packaged executable's
maximum required GLIBC is `2.34`, within the Ubuntu 22.04 `2.35` policy
baseline. The container includes `/usr/bin/xvfb-run`; manifest/checksum,
desktop-entry, and PNG icon checks passed, followed by the disposable Debian
install/upgrade/remove lifecycle, visible Debian and AppImage launches, and
representative smoke validation. The gate used
`python3 scripts/validate_release_artifacts.py --artifact-dir
target/ubuntu-22.04/release/packages --lifecycle --smoke`. A local installation
would use `sudo apt install
./target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.2_amd64.deb`; the
AppImage launch command is
`./target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.2-x86_64.AppImage`.

## Deferred work

This milestone adds no new workspace, model control, voice control, native
windowing integration, agent behavior, or project-state automation.
