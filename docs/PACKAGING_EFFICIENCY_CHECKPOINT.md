# Post-M41 Packaging-Efficiency Corrective Checkpoint

This checkpoint changes only the authoritative Ubuntu 22.04 Debian release
workflow. It is not a product capability milestone and does not alter M41,
Advisor, QuireForge, attachment, execution, or worker-runtime contracts.

## Verified immutable-input cache

The release container supplies a private `sandbox-sources` cache for the two
immutable upstream archives pinned in `packaging/sandbox/sources.lock`:

- Linux `6.1.178` kernel source;
- Firecracker `1.15.1` distribution containing the matching jailer.

Cache keys include the pinned version and SHA-256. Before every extraction, the
workflow checks the cached archive against that exact lock-file checksum. A
missing or invalid entry is downloaded to a private temporary file, verified,
and atomically promoted into the cache. Cache names and checksums are closed
validated values; traversal-like names fail before any cache path is used.

Only the pinned Ubuntu 22.04 container passes the cache location to the worker
asset build. Host development packaging remains non-authoritative and cannot
write release artifacts, manifests, checksums, or evidence.

## What remains fresh and fail-closed

The cache stores no extracted guest source, guest kernel, initramfs, worker
binary, Debian package, release artifact, manifest, provenance, or ABI
evidence. Every release build still:

- extracts verified immutable inputs into a disposable work directory;
- builds the guest kernel and fixed non-interactive agent anew;
- regenerates Firecracker/jailer worker assets and SHA-256 evidence;
- creates a clean, commit-bound desktop and worker Debian release set;
- validates checksums, metadata, ABI, lifecycle, installed smoke, and visible
  launch before promotion.

The cache is an efficiency mechanism only; it is not release provenance and
never allows an unchecked upstream input to enter the build.

## Package gate

The checkpoint candidate is `0.1.0-beta.37`. It requires focused cache tests,
full source validation, the pinned container release workflow, release-set and
checksum validation, restricted installed-package validation, installed-host
smoke, and visible-launch evidence before it can be recorded as complete.

## Evidence

The clean implementation commit is
`502e56e46131c64e7821fc98b16152142ac50eff`. The focused package/cache suite
passed `14/14`, including cache reuse, tamper eviction, unsafe-name, and
symlinked-cache-directory rejection. Full validation passed with `268` desktop
and `7` website unit tests, `251` Rust tests (with `3` expected ignores), and
the required formatting, lint, type, production-build, distribution, and Tauri
build gates. Existing desktop/narrow Playwright coverage passed `48/48`; website
Playwright coverage passed `8/8`.

The pinned Ubuntu 22.04 Debian-only `0.1.0-beta.37` release set is bound to
that exact clean commit:

- `quireforge_0.1.0.beta.37_amd64.deb` — SHA-256
  `95c6fdcdadc4ea6487efc5fdef958fbc9ecbe2429e9406795fe58ffd3ead7d72`;
- `quireforge-sandboxd_0.1.0.beta.37_amd64.deb` — SHA-256
  `1223cc5bdea3d03bf35c351cc3425585fa67e513547666bf8735d90268946d9e`.

The release manifest records clean pinned-container provenance and a highest
shipped `GLIBC_2.34`, within the Ubuntu 22.04 `GLIBC_2.35` ceiling. Release-set,
checksum, metadata, lifecycle, restricted installed-package, installed-smoke,
and installed-host visible-launch validation passed. No release or deployment
was performed.
