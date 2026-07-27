# Milestone 36 — Advisor Static Binary/Executable Inspection

M36 adds one closed `static-binary` Advisor ingestion entry. It supports only
signature-validated ELF32/ELF64 relocatable objects, executables, and shared
objects. Core files and all non-ELF formats fail closed.

## Boundary

`elf = "=0.8.0"` (MIT OR Apache-2.0) is pinned for low-level ELF table parsing.
QuireForge owns native source selection, regular-file and symlink policy,
descriptor identity, source byte limit, SHA-256, pre-parser table bounds,
path-free manifest, confirmation, one-use claim, expiry, cancellation, reset,
and no-persistence lifecycle. A successful parser load means only that the
pinned parser accepted the bounded source; parser failures map to the broad,
path-free `malformed-or-unsupported-elf` diagnostic.

The source limit is 32 MiB. Program headers are capped at 256, section headers
at 1,024, each header-table range at 1 MiB, dynamic entries at 256, and the
normalized text manifest at 8 KiB. Parser-internal CPU and memory caps are not
claimed in this in-process design.

## Projection

Advisor receives only `static-binary-manifest-v1` through the existing
documented text input: safe basename, byte size, SHA-256, ELF class,
endianness, file type, machine and OS-ABI identifiers, bounded program/section
counts, and bounded dynamic-section presence/count. It receives no source path,
raw bytes, section/symbol/interpreter/RPATH/DT_NEEDED names, notes, debug data,
raw headers, addresses, or executable content.

No loading, execution, debugging, emulation, detonation, generic upload,
network, project access, shell, terminal, Git, dispatch, or execution authority
is added. One attachment is transient, requires manifest/hash confirmation, is
claimed once, and is cleared on completion, failure, cancellation, mode reset,
or restart.
