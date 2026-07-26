# Milestone 27 — Unified Conversation Engine

Status: complete.

## Objective

Introduce one native conversation boundary with materially distinct Chat and
Codex modes. Chat is a no-project conversation profile; Codex remains the
attached-project, approval-bound software-development profile. A mode is a
capability policy and persisted metadata, never a presentation-only label.

## Managed ChatGPT authentication feasibility gate

Chat requires the supported Codex app-server managed browser login with
`account/login/start` type `chatgpt`. The app-server owns the browser callback,
credentials, refresh, and logout. QuireForge accepts only a bounded status and
the reviewed official-host handoff URL.

- Passwords, API keys, access tokens, refresh tokens, cookies, browser storage,
  account identifiers, and raw authentication responses must never enter
  QuireForge storage, React state, diagnostics, fixtures, or package evidence.
- The experimental externally managed `chatgptAuthTokens` path is prohibited.
  QuireForge must never supply or refresh a token for Codex.
- OpenAI API/project-key access is separately managed and billed. It is not a
  fallback for the required managed ChatGPT-account path.
- This gate authorizes only documented Codex app-server conversations. It does
  not authorize use of consumer ChatGPT APIs, history, browser sessions, or
  branded product behavior.

The first implementation slice adds matching Rust/Zod readiness and capability
contracts, a version-7 bounded SQLite migration, and fixed native Chat
commands. Chat is ready only for normalized managed `chatgpt` authentication;
an API-key, managed-provider, missing, pending device-code, or unavailable
state cannot enable it. The fixed thread start has `cwd: null`, no environments,
no dynamic tools, read-only sandboxing, and `never` approvals. Any attempted
native tool or permission request blocks the Chat conversation rather than
asking the user to approve it. The only persisted Chat fields are opaque local
conversation and Codex-thread references with timestamps; no prompt, response,
project path, credential, or account identity is stored. Codex retains its
project-bound capability profile and existing records.

## Planned implementation phases

1. Establish managed-auth feasibility, strict mode contracts, the Settings
   foundation, and bounded mode-aware metadata. **Implemented.**
2. Add the no-project Chat profile only after native capability tests prove
   that its fixed app-server request cannot obtain project/native-action
   authority. **Implemented with a direct Chat workspace and bridge.**
3. Route the existing Codex conversation service through the native engine and
implement explicit confirmed mode transitions with no automatic context
transfer. **The initial confirmed selector is implemented. The confirmed mode
choice persists locally as the closed `chat` or `codex` preference and restores
safely to Codex if absent or invalid; no project context, attachment,
integration, approval, or transcript is persisted or transferred. Unified
history and continuation remain deferred.**
4. Close full Rust/TypeScript/desktop/browser validation and the pinned Ubuntu
   22.04 Debian/AppImage package gate. **Implemented.**

## Settings foundation

The existing Settings route now has stable destinations for General,
Appearance, Chat, Codex, Permissions & safety, Models & providers,
Integrations, Privacy & data, Keyboard shortcuts, and About & updates.
Unsupported controls remain absent rather than simulated. The former
`#settings/accounts` deep link resolves safely to General; M26 Appearance and
its local `quireforge-theme` preference remain unchanged.

## Boundaries

- No consumer ChatGPT API, token reuse, credential collection, browser
  scraping, automatic handoff, watcher, contradiction resolution, repair, or
  repository-state semantic change is included.
- A Chat/Codex switch must require confirmation whenever capabilities or project
  context would change. It must never transfer a repository, file attachment,
  approval, terminal, integration, or hidden transcript automatically.
- Codex remains authoritative for authentication and session data; QuireForge
  SQLite stores only bounded local metadata.
- No QuireForge-Qt work occurs.

## Final validation and package evidence

The final implementation commit is
`cc4d0cea7d28d275e5ad1c8aa9d7a2a4f0627d6c`. It was built from a clean source
tree as the unique incremental package version `0.1.0-beta.4` (Debian internal
version `0.1.0~beta.4`). The ignored, version-1 release manifest and checksum
file are at `target/ubuntu-22.04/release/packages/release-manifest.json` and
`target/ubuntu-22.04/release/packages/SHA256SUMS` respectively. They identify
that implementation commit and agree with the locally measured artifact sizes
and checksums.

| Artifact | Path | Size | SHA-256 |
| --- | --- | ---: | --- |
| Debian | `target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.4_amd64.deb` | 4,674,456 bytes | `3cdb4eda670a9b771efbb53b8ac84c70ea92189330a334a06788f262268cc9f7` |
| AppImage | `target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.4-x86_64.AppImage` | 83,905,016 bytes | `b527286c55565690b9b26f52fe18a8d7d4904f4466bec5ba07d909d580815ba9` |

The digest-pinned Ubuntu 22.04 builder contains `/usr/bin/xvfb-run`. Its
release validator passed manifest/checksum, desktop-entry, PNG icon, disposable
Debian install/upgrade/remove lifecycle, visible Debian and AppImage launches,
and representative smoke validation. The executable requires at most
`GLIBC_2.34`, within the Ubuntu 22.04 `GLIBC_2.35` baseline. The package gate
used its normal closed package workflow and then this explicit validation:

```bash
docker run --rm --init --user "$(id -u):$(id -g)" \
  --volume "$PWD:/workspace" --volume "$PWD/.cache/packaging:/cache" \
  --env HOME=/cache/home --env CARGO_TARGET_DIR=/workspace/target/ubuntu-22.04 \
  --env QUIRE_FORGE_PACKAGE_DIR=target/ubuntu-22.04/release/packages \
  --workdir /workspace quireforge-packaging:ubuntu-22.04 /bin/bash -c \
  'command -v xvfb-run && python3 scripts/validate_release_artifacts.py \
    --artifact-dir target/ubuntu-22.04/release/packages --lifecycle --smoke'
```

The installed host was upgraded with the following command and reports Debian
version `0.1.0~beta.4`:

```bash
sudo apt install --reinstall -y \
  /mnt/faststorage/quireforge/target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.4_amd64.deb
```

With the host's active display, visible installed-artifact smoke passed for
both `/usr/bin/quireforge` and the AppImage:

```bash
python3 scripts/smoke_linux_package.py --label 'Installed Debian beta.4' /usr/bin/quireforge
python3 scripts/smoke_linux_package.py --label 'Installed AppImage beta.4' \
  ./target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.4-x86_64.AppImage \
  --appimage-extract-and-run
```

The implementation gate passed `pnpm test` (203 desktop and 7 website tests),
desktop TypeScript and ESLint checks, `cargo fmt --all -- --check`,
warning-denying workspace Clippy, locked workspace Rust tests (200 passed, 3
ignored), repository validation, production builds, Tauri no-bundle
compilation, and the complete-chunk distribution budget. The final distribution
measured 861.71 KiB JavaScript and 99.80 KiB CSS against 865 KiB and 105 KiB
ceilings. The strict package-evidence reader's focused producer-format and
commit-freshness test passed; its evidence semantics were not changed. The
ignored version-1 validation summary records the package gate against the
implementation commit.

## Deferred work

Unified history/continuation presentation and migration/backfill coverage for
an already populated production database remain deferred to separately
approved work. No Advisor/Planner agent, Python sidecar, watcher, automatic
handoff, contradiction resolution, or autonomous repair was added. The desktop
distribution gate retains complete-chunk accounting with measured ceilings of
865 KiB JavaScript and 105 KiB CSS; it does not exclude the Chat/Codex contract
or any appearance palette.
