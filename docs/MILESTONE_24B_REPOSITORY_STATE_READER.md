# Milestone 24B — Repository State Reader

Status: complete on `feat/milestone-24b-repository-state-reader`.

## Scope

The reader observes one already attached QuireForge project and returns a
version-1 project-state contract plus diagnostics. It accepts no filesystem
path, arbitrary Git argument, or background-scan request.

## Initial reader boundary

Rust resolves the project through `ProjectService::review_root`, then uses the
existing closed, credential-free Git command environment. Local-only reads do
not inspect remote refs. Existing-tracking reads use only present refs. The
explicit `fetch-authorized` mode is the sole remote-refresh path and exists
only under James's approved boundary; it is never the default.

Document claims are reported evidence. The initial narrow document reader only
compares the documented current-state branch with verified Git evidence. It
does not rewrite documents or select a winner. Diagnostics remain separate from
contract truth and suggest only inferred next actions.

Missing, unsafe, oversized, and malformed current-state evidence produces a
stable diagnostic rather than an invented branch value.

The Git evidence now distinguishes staged, unstaged, and untracked changes,
detached HEAD, missing upstream, ahead/behind counts, shallow repositories, and
merge, rebase, cherry-pick, or bisect markers. Deterministic temporary Git
fixtures cover dirty state, detached HEAD, missing upstream, and an ahead local
branch. The fixture fingerprint records symbolic and detached HEAD state, local
and remote-tracking refs, porcelain status, index bytes, safe local config,
operation markers, `FETCH_HEAD`, and tracked/untracked fixture-file contents.

Local-only and existing-tracking reads preserve that complete fingerprint. The
existing-tracking test deliberately points `origin` at an unavailable local path
after recording its tracking ref, proving it reads existing refs without fetch
or remote contact. The separately authorized fetch mode uses only the closed
`git fetch --no-tags --no-write-fetch-head origin` operation: a controlled bare
remote may advance its intended tracking ref, while HEAD, branch, local refs,
index, worktree, configuration, operation markers, files, and `FETCH_HEAD`
remain unchanged. This is limited authorized mutation, not a mutation-free
mode; callers cannot select a remote or refspec.

## Supported evidence and freshness

The reader uses a closed registry for the Ubuntu package manifest and
`SHA256SUMS`, `target/validation-summary.json`, and approved handoff phrases in
`docs/CURRENT_STATE.md`. These are bounded, non-symlink, UTF-8 files only; the
reader neither scans arbitrary documents nor runs validation or package work.
Missing optional evidence returns a partial snapshot with diagnostics.

Package and validation claims receive commit-based `current`, `stale`, or
`unknown` freshness independently from trust. Git observations are verified,
machine-readable files are verified as file contents, Markdown claims remain
reported, and suggested actions remain inferred. Malformed or absent evidence
is never silently repaired or treated as completion.

The reader adapts the established release producer's version-1 manifest
(`schemaVersion`, release state/version, `source.commit`/`treeState`, pinned
Ubuntu builder metadata, and `format`/`filename` artifacts) as closed Rust
records. The producer's `appimage` input spelling is normalized to the stable
reader `app-image` value; emitted architecture remains `x86_64`. Validation
summaries are likewise parsed as closed records:
unknown fields, unsupported artifact/status values, invalid 40-character commit
IDs, invalid SHA-256 values, and unsafe repository-relative artifact paths are
rejected with diagnostics. The normal reader remains metadata-only and never
hashes or rebuilds package artifacts. The closed `verify-local-artifacts` mode
uses SHA-256 only for accepted manifest paths, rejects symlinks and non-files,
and reports missing, size, and checksum mismatches without repository mutation.

`SHA256SUMS` is parsed only in its closed double-space SHA-256 format. Accepted
records are reconciled with manifest paths without choosing a winner: malformed,
duplicate, orphaned, missing, and disagreeing records receive diagnostics, and
an accepted manifest/checksum disagreement is represented as conflicting
freshness.

Package manifests now require version `1` and clean-source metadata. Returned
artifact evidence keeps the manifest version, declared size, and optional local
presence distinct from checksum observations; absent optional package records
remain partial evidence rather than fabricated success.

The strict package envelope carries closed Ubuntu 22.04 platform and
architecture observations plus distinct manifest, checksum-file, and optional
local-verification results. Lifecycle, launch, desktop/icon, smoke, and GLIBC
results remain explicit package-gate evidence rather than invented manifest
fields.

Handoff phrases remain reported Markdown evidence. A pushed-checkpoint phrase
receives current or stale freshness only after the closed Git reader confirms a
valid local commit object and branch ancestry; otherwise it stays unknown with
a diagnostic. A local shallow-clone fixture proves shallow detection and full
fingerprint preservation for local-only and existing-tracking reads.

Validation summaries now require version `1`, a closed check family and status,
a full source commit, bounded operation ID, UTC-style timestamp, and safe
repository-relative evidence path. Optional malformed validation evidence is
diagnostic only and never causes the reader to execute validation.

## Final package evidence

The final implementation commit is
`bd4c428405425d78d4df439a600c7e02085a83fb`. From its clean tree, the pinned
`ubuntu:22.04@sha256:0e0a0fc6d18feda9db1590da249ac93e8d5abfea8f4c3c0c849ce512b5ef8982`
workflow produced ignored local artifacts at
`target/ubuntu-22.04/release/packages/` and passed its manifest/checksum,
desktop-entry, icon, GLIBC, disposable Debian install/upgrade/remove, and
visible Debian/AppImage launch-smoke gates.

- `quireforge_0.1.0.beta.2_amd64.deb` — 4,629,924 bytes; SHA-256
  `05c7036320be8fe900fd7b52cef1f4f8b54041244b3bf88a83e1d36a3080c293`.
- `QuireForge-0.1.0-beta.2-x86_64.AppImage` — 83,855,864 bytes; SHA-256
  `6db4cb5f1c585f252cf06bbc825b5ce61311b63ad6bd030a9546e1950990eb74`.

The source manifest is version 1, a clean `release-candidate` for that
implementation commit; both artifacts target Ubuntu 22.04 `x86_64`, and the
packaged executable's maximum observed GLIBC symbol is `2.34`, within the
Ubuntu 22.04 `2.35` baseline. The lifecycle gate used disposable `dpkg` roots;
the local installation command is `sudo apt install
./target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.2_amd64.deb` and
the AppImage launch command is `./target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.2-x86_64.AppImage`.

`target/validation-summary.json` records the successful package operation in
the reader's strict version-1 validation format. The reader's metadata-only
and local-artifact-verification paths accept the producer-compatible manifest,
checksum file, and local files without a source, size, or checksum conflict.

## Deferred work

No 24C UI, 24D handoff generation, contradiction resolution, watcher, or
autonomous repair exists. A project-state workspace is the next proposed
milestone and requires separate approval.
