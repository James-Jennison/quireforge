# Milestone 46 — Bounded Multi-Attachment

Status: complete. Clean provenance commit `1bc2e787ab785016041d70845c97ca9c2c4f84db`.

Advisor's compact tray now stages up to three distinct existing typed
attachments. Existing text/data, PNG/JPEG, PDF, ZIP, and static-ELF handlers
remain the sole validators and manifests; an image remains limited to one.
Native preflight requires identities and hashes to still match the current
typed snapshots and rejects collections exceeding 40 MiB before claiming any
entry. The existing typed app-server input list is retained: one text item plus
at most one existing `localImage` descriptor.

The collection confirmation enumerates every staged manifest and requires an
explicit confirmation for all entries. No generic upload, drag-and-drop, new
type, new endpoint, hidden reuse, persistence, or authority is introduced.

The pinned Ubuntu 22.04 workflow passed checksum, provenance/ABI (`GLIBC_2.34`
within `GLIBC_2.35`), a disposable Debian lifecycle, extracted-package smoke,
and a visible X11 launch gate inside the pinned container. It did not install
or replace the live host package. `quireforge_0.1.0.beta.42_amd64.deb` is 5,483,760 bytes
with SHA-256 `033536248fd5cf3d26410376f93b4db083c3e125f4466653a525d6d1998013bd`;
the worker Debian is 3,233,210 bytes with SHA-256
`79e55a20948410798249d74c15a285ad82fa09aa5ec57696871850b34eb21c22`.
