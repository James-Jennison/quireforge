# Milestone 46 — Bounded Multi-Attachment

Status: implementation candidate for `0.1.0-beta.42`.

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
