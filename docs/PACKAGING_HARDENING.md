# Packaging hardening checkpoint

This `0.1.0-beta.31` corrective checkpoint follows the completed beta.30
Ubuntu 22.04 release set. It changes packaging evidence and validation only; it
does not alter the Milestone 35 ZIP manifest capability or rewrite milestone
history.

## Output separation

- `pnpm package:linux` is a rapid host-development build. Its raw Tauri output
  is isolated under `target/host-development/` and is non-distributable. It
  never writes a normalized artifact, checksum file, manifest, or release
  evidence.
- `scripts/run_linux_package_container.sh` is the only producer of the
  authoritative `target/ubuntu-22.04/release/packages/` set. It runs the
  digest-pinned Ubuntu 22.04 workflow and is the only path permitted to invoke
  the normalization/finalization script.

The normalizer refuses host execution: it requires the release-builder marker
and verifies `/etc/os-release` is Ubuntu 22.04 before it can touch the
authoritative release directory.

## Release evidence v2

Every future authoritative release set contains a schema-v2 manifest with the
clean source commit, exact pinned container identity, authoritative workflow
command, artifact names and SHA-256 values, plus ABI evidence from the shipped
`usr/bin/quireforge` executable extracted independently from the Debian and
AppImage artifacts. Validation recomputes both GLIBC requirements and fails if
either exceeds Ubuntu 22.04's `GLIBC_2.35` baseline or differs from evidence.

The completed beta.30 set is immutable legacy schema-v1 evidence. Its packaged
Debian and AppImage executables were inspected separately and each requires at
most `GLIBC_2.34`; future release-set validation intentionally requires v2 and
will fail closed for missing provenance or ABI evidence.

CI invokes the same container entrypoint and validates the finalized v2 output;
host builds cannot become uploaded review artifacts. This checkpoint creates no
release, publication, deployment, or package version change.
