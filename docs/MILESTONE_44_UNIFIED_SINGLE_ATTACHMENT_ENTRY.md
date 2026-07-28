# Milestone 44 — Unified Single Attachment Entry

Status: complete. Implementation provenance commit
`891abf6d953e3b7c0dd3f0d3bd03baeb29de40fb` is verified by the fresh pinned
Ubuntu 22.04 `0.1.0-beta.41` Debian evidence below.

## Scope

Advisor's composer now presents one compact **Attach a file** entry. It opens a
bounded type chooser and invokes exactly one of the existing native pickers for
text/data, PNG/JPEG, PDF, ZIP, or static ELF inspection. Native code remains
the sole authority for each picker filter, type decision, validation, manifest,
expiry, claim, confirmation, and disposal behavior.

## Boundaries

- Exactly one pending attachment is permitted in the composer.
- The tray has no browser file input, generic uploader, drag-and-drop surface,
  collection, or new supported type.
- The existing typed commands and Advisor conversation transport are unchanged.
- The existing one-use confirmation, claim, expiry, cancellation, and disposal
  rules remain type-specific and unchanged.
- The change grants no project, terminal, Git, worktree, browser, dispatch,
  execution, connector, or Advisor authority.

## Validation and package evidence

The focused UI test proves that the single entry exposes only the five closed
type choices and routes the image choice to its existing typed picker. The full
repository validation and the authoritative pinned Ubuntu 22.04 workflow
passed from the clean implementation provenance commit.

The ignored release records are:

- `target/ubuntu-22.04/release/packages/release-manifest.json`
- `target/ubuntu-22.04/release/packages/SHA256SUMS`

The schema-3 manifest records `treeState: clean`, the digest-pinned Ubuntu
22.04 builder, and maximum shipped `GLIBC_2.34`, within the `GLIBC_2.35`
compatibility ceiling. The workflow passed checksum and provenance validation,
Debian lifecycle, pinned-container smoke, restricted installed-package smoke,
and installed visible X11 launch.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `quireforge_0.1.0.beta.41_amd64.deb` | 5,486,240 | `a6875331fd1e7b022496481e89ad3c7064d1dbcd345f9ec02733115cd6b7c599` |
| `quireforge-sandboxd_0.1.0.beta.41_amd64.deb` | 3,233,620 | `b2129b07cf3bfd26c5a3eabf685a3b5f8ffd2c95230c24d66aa5334565e66e3b` |

The production desktop output remains inside the closed temporary envelope:
194,943-byte startup entry, 309,016-byte application shell, 941,334 bytes total
JavaScript, and 109,082 bytes CSS. No ceiling changed.
