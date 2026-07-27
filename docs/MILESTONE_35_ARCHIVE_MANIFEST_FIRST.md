# Milestone 35 — Advisor Archive Manifest-First Analysis

## Scope

Milestone 35 adds one closed `archive` ingestion entry for a single ZIP source.
The pinned `zip = "=8.6.0"` parser is used with default features disabled to
interpret ZIP container metadata only. QuireForge never extracts, decompresses,
opens, renders, or transports entry contents.

## Bounded contract

The native service validates one regular, non-symlink, absolute source; checks
the ZIP signature and source identity; computes SHA-256; then produces the
path-free `archive-manifest-v1` projection. It permits at most 32 MiB source
bytes, 10,000 inspected entries, 2,000 manifest entries, 512-byte ASCII names,
32 path components, 256 KiB manifest text, 64 MiB declared entry size, 256 MiB
declared aggregate size, and a 100:1 declared compression ratio. It stages one
attachment for 15 minutes and consumes it on one explicitly confirmed send.

Unsafe or ambiguous entry metadata fails closed: absolute, traversal,
backslash, drive-like, control-character, empty, dot, trailing-dot/space, or
duplicate case-insensitive names; links; and unsupported special entry kinds
are rejected. Nested archive-looking entries are listed only as warnings and
are never opened recursively. Encrypted archives fail closed; no passwords are
requested or retained.

## Transport and retention

Advisor receives only bounded, normalized manifest text through the existing
documented text input. It receives no selected path, raw ZIP bytes, base64/data
URL, generic attachment object, extracted content, or new protocol input. The
source and projection are transient, one-use, path-free, and excluded from
SQLite, project metadata, retained references, approvals, dispatch, completion
reports, logs, browser storage, and restart recovery.

## Deliberate exclusions and residual risk

M35 supports ZIP only. TAR-family, GZIP/BZIP2/XZ streams, 7z, and RAR remain
deferred. In-process parser CPU and memory limits are not claimed; malformed
container handling is delegated to the pinned parser and maps to a broad stable
diagnostic. A future isolated-parser project would be required for hard parser
resource isolation.
