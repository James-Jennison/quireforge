# Milestone 48 — Generated Artifacts and Explicit Save

Status: complete. Package-source commit
`5d483d0c068c450bbc779ee07b048fe848c7e1f0`.

M48 implements only `advisor-generated-artifact-registry-v1`. The native,
process-local registry accepts explicitly chosen completed Advisor replies and
supported visible fenced blocks as text, markdown, JSON, CSV, or Python text.
It retains only normalized UTF-8 bytes and a short save reservation; it has no
SQLite migration, project/worktree, attachment, Codex, approval, dispatch,
terminal, Git, browser, provider, connector, path, or persistence state.

Save claims an ID/hash pair before opening one native Save dialog. Linux saves
use a private same-directory 0600 temporary file, file synchronization,
`renameat2(RENAME_NOREPLACE)`, and parent-directory synchronization. Successful
saves return only a transient path-free receipt and consume the artifact.

## Beta.43 package evidence

The pinned Ubuntu 22.04 container produced the Debian-only set:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `quireforge_0.1.0.beta.43_amd64.deb` | 5,508,172 | `32d2330c6d53b1a3a8827adcf8ad895772e94647d75d9eaab9f366910a5d1ea9` |
| `quireforge-sandboxd_0.1.0.beta.43_amd64.deb` | 3,233,688 | `3fbe8df3c09f39e851bd967152ac8c02198d71e69adb0501e565c1f948c3b7fa` |

The release manifest and `SHA256SUMS` are in
`target/ubuntu-22.04/release/packages/`. Both shipped binaries require maximum
`GLIBC_2.34`, within the `GLIBC_2.35` Ubuntu 22.04 ceiling. Disposable Debian
lifecycle, pinned-container visible-launch smoke, release validation, the
restricted `sudo -n /usr/local/sbin/quireforge-validate-deb` installation,
and installed-package visible launch passed. The installed `/usr/bin/quireforge`
SHA-256 was `407f99759aed3d970207fefaacec7288884ae1a6c81f6927a009ad1b9809c39a`.

The native unit smoke saved exact bytes, verified the returned SHA-256, and
proved a second publication at the selected target fails without overwrite.
The desktop production measurement is 190.37 KiB startup entry, 307.29 KiB
application shell, 924.79 KiB total JavaScript, and 107.47 KiB CSS: all remain
below the temporary 256 KiB / 448 KiB / 1.5 MiB / 160 KiB ceilings.
