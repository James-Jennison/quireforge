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

The latest pinned Ubuntu 22.04 workflow, from clean source commit
`eb7ac2ee5842ec442dfc9201e15594fe4acb8fe3`, passed checksums,
provenance/ABI (`GLIBC_2.34` within `GLIBC_2.35`), a disposable Debian
lifecycle, extracted-package smoke, and a visible X11 launch gate. The desktop
Debian is 5,482,500 bytes with SHA-256
`71a41f5973f6bccd25db9f9bab0e6aba1e089d1fc5f81698486b5cd64e7bf042`; the
worker Debian is 3,234,846 bytes with SHA-256
`4987413c9cc42f2019fb3b7d73abeced6c1b678e34472aa2f460d3f685f79a36`.

The installed `quireforge` Debian package is `0.1.0~beta.42`.
`/usr/bin/quireforge` has SHA-256
`5960d8cb972959d6b68138ff802bbdd7c5cf83ab8b2e77ffbec7343851b091fc`, exactly
matching the desktop binary extracted from the latest package, and passed the
visible installed-package smoke.
