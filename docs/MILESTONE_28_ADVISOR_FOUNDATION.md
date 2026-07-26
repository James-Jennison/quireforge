# Milestone 28 — Reference-Only Advisor Foundation

Status: complete; reference-only foundation and package gate closed.

## Objective

Establish the smallest safe local foundation for an Advisor workspace and a
future Approval/Dispatch controller without adding a model turn, arbitrary
repository reader, project attachment, prompt dispatch, or autonomous action.

## Implemented checkpoint

The foundation introduces a strict version-1 Rust/TypeScript contract for:

- opaque Advisor conversation references owned by the supported Codex
  app-server;
- user-selected, closed context-reference kinds only: project-state, roadmap,
  current-state, and execution-report;
- trust, provenance, and freshness kept as separate values;
- reference-only dispatch proposals with prompt and context-manifest digests;
- explicit-approval-required drafts, approvals, and rejections.

The active shell adds one fixed-purpose, no-argument Tauri read command and an
`#advisor` route. Rust validates the existing version-1 contract before
deriving a smaller strict Rust/Zod safe-summary projection for presentation. It
has no composer or controls for project selection, context reading, models,
approvals, or dispatch; opaque IDs, digests, model requests, and project IDs
are not serialized to the route.

The accompanying SQLite migration creates only bounded metadata tables. They
contain opaque references, closed source labels, commits, timestamps, and
SHA-256 digests. They deliberately contain no prompt body, response,
transcript, credential, token, arbitrary project path, or browser/session data.
The read command executes no model, Git, terminal, filesystem, project-context,
or network operation. It accepts no path, project ID, prompt, model, Git
argument, or other caller input and does not mutate SQLite.

The Selected Project-State Snapshot Reader adds one separate, explicit native
read after the user confirms the currently selected attached project as an
Advisor source. It hard-codes the established M24B reader to `local-only` and
`metadata-only`; it accepts only an application-owned UUIDv7 project ID and
cannot receive a path, Git argument, remote mode, artifact-verification mode,
or document name. Rust derives a second strict safe projection containing only
closed trust/freshness/provenance labels, coarse worktree state, and a bounded
diagnostic count. It excludes
project identity, repository, branch, commit, path, source document, artifact,
diagnostic text, image, and arbitrary file content.

This selection is in-memory only. It is cleared when removed, when the selected
project changes, and at restart; it creates no Advisor SQLite context record.
The Advisor route requires a visible confirmation before the read and has no
automatic refresh or background scan.

## Boundaries

- Codex remains authoritative for account state, browser authentication,
  transcripts, and threads. QuireForge does not collect or retain credentials,
  cookies, API keys, or external tokens.
- A future UI may hold editable prompt text transiently, but this foundation
  persists only its SHA-256 digest. A decision to retain verbatim local Advisor
  text requires separately approved privacy, storage, and migration work.
- Context is represented as an explicit user selection. This checkpoint reads
  only the existing normalized M24B Project State snapshot after confirmation;
  it does not read project files, screenshot images, attached-project paths,
  raw documentation, or arbitrary source content, and it never transfers Codex
  attachment or execution authority into Advisor.
- A dispatch proposal is not a dispatch. No text is parsed for `Approve`,
  `Proceed`, or `Confirmed`; a later controller must require an explicit user
  action and separately confirm any model or reasoning change.
- Milestone 24D operational handoff/consistency behavior remains deferred.

## Verification

The shared `apps/desktop/fixtures/advisor-foundation.json` fixture is parsed by
strict Rust/Serde and TypeScript/Zod contracts. Tests reject unknown fields,
unsafe path-like references, non-digest proposal records, and a proposal that
does not require explicit approval. SQLite migration tests assert that the
application-owned schema excludes credential, session, prompt, transcript, and
content columns.

The selected Project State reader is covered by Rust projection/request tests,
strict TypeScript bridge tests, App explicit-confirmation and reset tests, and
desktop/mobile Advisor accessibility regression coverage. It preserves the M24B
reader's normalized evidence semantics rather than introducing a second
evidence parser.

## Final validation and package evidence

The final package-source commit is
`73eaf14f4294790683469c03dbcd07efc92b1642`, built from a clean tree as the
unique incremental version `0.1.0-beta.5` (Debian internal version
`0.1.0~beta.5`). The ignored version-1 evidence is at
`target/ubuntu-22.04/release/packages/release-manifest.json`,
`target/ubuntu-22.04/release/packages/SHA256SUMS`, and
`target/validation-summary.json`.

| Artifact | Path | Size | SHA-256 |
| --- | --- | ---: | --- |
| Debian | `target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.5_amd64.deb` | 4,687,808 bytes | `e9b8dcbba0d73a086365e986824452dce795ebbdce2b60b87fb0693cf53a3c9f` |
| AppImage | `target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.5-x86_64.AppImage` | 83,917,304 bytes | `c818304448d6229594be19207fd8816a9ba089c1a6703abad444302afdbd0ad5` |

The digest-pinned Ubuntu 22.04 builder includes `/usr/bin/xvfb-run`. Its
validator passed manifest/checksum agreement, desktop-entry and icon checks,
maximum required `GLIBC_2.34` against Ubuntu 22.04's `GLIBC_2.35` baseline,
the disposable Debian install/upgrade/remove lifecycle, and visible Debian and
AppImage smoke. The installed host upgraded from beta.4 to beta.5 and passed
visible launch smoke for `/usr/bin/quireforge` and the AppImage. The exact
commands were:

```bash
sudo apt install --reinstall -y \
  /mnt/faststorage/quireforge/target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.5_amd64.deb
python3 scripts/smoke_linux_package.py --label 'Installed Debian M28 beta.5' /usr/bin/quireforge
python3 scripts/smoke_linux_package.py --label 'Installed AppImage M28 beta.5' \
  ./target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.5-x86_64.AppImage \
  --appimage-extract-and-run
```

`pnpm validate`, focused desktop/mobile Advisor browser coverage, the Tauri
no-bundle build, and the pinned container package gate passed. The designated
package output directory retains only the complete beta.5 release set; no
installed package, source file, history, remote release, or required evidence
was removed.

## Deferred work

No additional project-context reader, screenshot staging, managed Advisor model
call, prompt editor, approval UI, dispatch bridge, Python sidecar, watcher,
automatic handoff, contradiction resolution, or repository-write capability has
been added. Any follow-on requires a separate Advisor integration and
Approval/Dispatch proposal gate.
