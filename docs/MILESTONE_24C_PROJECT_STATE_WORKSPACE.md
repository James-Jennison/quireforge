# Milestone 24C — Project State Workspace

Status: complete on `feat/milestone-24c-project-state-workspace`.

## Objective

Present the existing Milestone 24B repository-state snapshot as a clear,
read-only workspace for the currently attached project. The route makes
repository, milestone, policy, validation, package, handoff, and diagnostic
evidence visible without changing its source or interpreting reported evidence
as approval.

## Approved boundary

The workspace consumes only the existing version-1
`repository_state_read` response. It requests:

- the already selected QuireForge project ID;
- `local-only` remote behavior; and
- `metadata-only` artifact behavior.

It is demand-driven when the route opens and may be refreshed explicitly by the
user. It does not fetch, watch, scan in the background, accept a path, run
validation, inspect local package artifacts, write project metadata, modify Git,
approve policy, resolve diagnostics, generate a handoff, or repair evidence.

Milestone 24D remains responsible for any separately approved handoff
generation or operational consistency rules. Canonical policy and persistence
ownership are unchanged.

## Presentation

The persistent desktop shell adds a **Project state** destination under the
Workspace navigation group. Its content:

- identifies the active project and normalized worktree state;
- shows current branch or detached state, local HEAD, tracking counts, and
  observed trust;
- presents the active milestone and the recorded owner, merge, and release
  approval decisions without offering approval controls;
- inventories validation, package, and reported handoff evidence;
- renders reader diagnostics and their inferred next actions without resolving
  them; and
- identifies browser preview, loading, missing-project, native, and read-error
  states honestly.

The existing context inspector displays only snapshot provenance, trust, and
diagnostic count. The workspace uses established responsive route components
and shell navigation instead of introducing another router or visual system.

## Contract and security

Rust remains the owner of attached-project identity, filesystem access, Git
inspection, evidence parsing, provenance, and diagnostics. TypeScript validates
the returned closed snapshot before presentation. React receives no arbitrary
path or Git argument and exposes no remote or mutation request.

Reported Markdown remains reported. Verified Git evidence remains verified.
Freshness remains independent from trust. The UI does not reinterpret unknown,
stale, or conflicting evidence and does not expose credential, token, raw
transcript, or arbitrary-file surfaces.

## Verification plan

The implementation gate requires:

- focused React route, component, bridge-request, navigation, and state tests;
- desktop and mobile Playwright evidence;
- axe accessibility and overflow checks;
- repository formatting, ESLint, TypeScript, and complete frontend tests;
- production frontend build and unchanged bundle-budget enforcement;
- Rust formatting, warning-denying Clippy, locked workspace tests, and Tauri
  compilation because the route ships in the desktop application;
- repository/document validation; and
- fresh pinned Ubuntu 22.04 Debian and AppImage lifecycle, launch, and smoke
  evidence from the clean implementation commit.

## Implementation evidence

The implementation gate passes:

- TypeScript and ESLint;
- 183 frontend unit/component tests across 36 files;
- 40 desktop/mobile Playwright scenarios, including the focused Project state
  route, axe, keyboard, responsive drawer, visual capture, and overflow checks;
- production frontend build and distribution budgets: 189.16 KiB entry
  JavaScript, 231.58 KiB application JavaScript, 845.34 KiB total JavaScript,
  and 99.03 KiB total CSS against the unchanged 100 KiB CSS limit;
- repository and package-contract validation plus the full repository formatter;
- warning-denying workspace Clippy;
- locked Rust workspace tests: 192 passed and 3 deliberate live probes ignored;
  and
- the unbundled optimized Tauri application build.

## Final package evidence

The final implementation commit is
`8a17703fd0c4d8ddf4ea55c121992202ce58b1c4`. Its source tree was clean when the
digest-pinned Ubuntu 22.04 workflow ran:

```text
./scripts/run_linux_package_container.sh
```

The ignored manifest and checksum records are:

- `target/ubuntu-22.04/release/packages/release-manifest.json`
- `target/ubuntu-22.04/release/packages/SHA256SUMS`
- `target/validation-summary.json`

The version-1 manifest records that exact implementation commit, clean source,
the pinned
`ubuntu:22.04@sha256:0e0a0fc6d18feda9db1590da249ac93e8d5abfea8f4c3c0c849ce512b5ef8982`
builder, Ubuntu 22.04, and `x86_64`.

- `target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.2_amd64.deb`
  is 4,635,632 bytes with SHA-256
  `0ecdc02ed9f7c85e77fbdc232237d10c3e55a7a0c7e5a95155bae254cbf528cc`.
- `target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.2-x86_64.AppImage`
  is 83,855,864 bytes with SHA-256
  `4e3d03d4c8a72c9b2c2fb6f0e8f9b005b33e8ec357ace914c260389a7201f78d`.

Manifest/checksum validation, canonical desktop-entry and icon validation, and
both artifact structures pass. The highest required GLIBC symbol is `2.34`,
within Ubuntu 22.04's `2.35` baseline.

The disposable Debian lifecycle passed initial installation, upgrade to
`0.1.0~beta.2`, removal, executable/desktop-entry cleanup, and preservation of
an attached-project fixture plus application metadata. The gate used:

```text
dpkg --root="$root" --force-not-root --force-depends --force-script-chrootless --install "$previous"
dpkg --root="$root" --force-not-root --force-depends --force-script-chrootless --install "$package"
dpkg --root="$root" --force-not-root --force-depends --force-script-chrootless --remove quireforge
```

A local installation would use:

```text
sudo apt install ./target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.2_amd64.deb
```

The container includes `xvfb-run`. Visible Debian and AppImage launch-smoke
checks passed with:

```text
xvfb-run --auto-servernum python3 scripts/smoke_linux_package.py --label "Debian package" "$debian_binary"
xvfb-run --auto-servernum python3 scripts/smoke_linux_package.py --label "AppImage" ./target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.2-x86_64.AppImage --appimage-extract-and-run
```

Direct AppImage launch uses:

```text
./target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.2-x86_64.AppImage
```

The strict package validator accepts the final manifest, checksum file, local
sizes, and local hashes without a source, size, checksum, version, or path
disagreement. The reader's producer-compatible metadata-only and
verify-local-artifacts fixture paths remain covered by the passing Rust
repository-state tests; normal inspection did not rebuild or mutate evidence.

## Deferred work

This milestone adds no repository reader behavior, fetch UI, background scan,
watcher, project-state edit, automatic approval, generated handoff,
contradiction resolution, autonomous repair, persistence migration, Qt work,
merge, release, or publication.
