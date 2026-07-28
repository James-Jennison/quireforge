# Milestone 45 — Bounded Multi-Attachment Proposal

Status: approved decision record. Its closed contract was exercised only by the
completed Milestone 46 implementation. This proposal itself introduced no
product code, package, transport endpoint, file type, or authority changes.

## Evidence and recommendation

The documented Codex app-server `turn/start.input` contract accepts a list of
input items, including `text` and `localImage`. This establishes that the
already used `localImage` item can participate in a bounded input list; it does
not establish a generic local-file transport. QuireForge's text, PDF, ZIP, and
ELF handlers must therefore continue to contribute only their existing bounded,
path-free text projections. See the [official app-server turns
documentation](https://learn.chatgpt.com/docs/app-server#turns).

Approve a closed `advisor-attachment-collection-v1` for at most **three**
existing attachments per one Advisor send, with at most **one** PNG/JPEG image.
The collection source-byte aggregate is capped at **40 MiB**; each type retains
its existing stricter individual ceiling: text/data 512 KiB, image 4 MiB, PDF
8 MiB, ZIP 32 MiB, and static ELF 32 MiB. No new type, generic picker, browser
file input, drag-and-drop, directory selection, or arbitrary file transport is
permitted.

## Typed collection contract

Each selected entry retains its existing type-specific manifest exactly as it
is validated today. The native-only collection manifest contains only:

- a UUIDv7 collection id and creation/expiry timestamps;
- a maximum-three ordered list of `{ position, type, attachmentId,
  manifestSha256, projectionKind, byteSize, disposal }`; and
- aggregate count, raw-source bytes, and projected-text bytes.

It has no paths, source bytes, image data URLs, archive entries, binary bytes,
credentials, project id, thread id, approval, dispatch, terminal, Git,
worktree, or execution data. Entries are ordered by explicit user insertion;
removing an entry compacts only the visible positions. Reordering is not in
scope for M46.

The UI remains one compact attachment tray. After a first file is staged, it
may offer **Add another supported file** until the three-item or aggregate
limit is reached. It renders a short ordered manifest card for every pending
entry and explicit remove controls. It never silently replaces an entry.

## Confirmation, expiry, claim, and disposal

Every selected item remains independently type-validated, has its current
15-minute transient lifetime, and displays its original manifest/projection
description. Sending opens one collection review dialog that enumerates every
ordered manifest and requires one explicit **Confirm all attachments for this
send** action. That action supplies every immutable item identity/hash and an
explicit confirmation marker; it cannot confirm an item omitted from the
review.

Before starting the turn, native code preflights all entries under a short-lived
collection reservation. A malformed request, duplicate id, stale/expired item,
hash mismatch, failed per-type claim, aggregate breach, or reservation failure
fails the entire request. It starts no turn, claims no item, and reports only a
bounded collection diagnostic. Still-valid unclaimed entries remain visible so
the user can remove or retry; an expired entry remains unavailable and is
disposed under its current rule.

Only after every preflight succeeds does native code atomically commit all
claims, begin the one turn, and retain claimed data only through the existing
terminal-turn lifecycle. A failed start or an interrupted/terminal turn
disposes every committed item using its original type-specific disposal path.
No partial send, silent omission, reuse, retry, or resurrection is allowed.

## Existing transport and authority boundaries

M46 would retain the existing documented app-server input list and existing
`localImage` item. The user prompt remains its current text item. Bounded PDF,
ZIP, ELF, and text/data projections are appended to that text in manifest order;
the optional single `localImage` remains a native-held descriptor item after the
text item, as it is today. This adds no endpoint, provider, connector, browser,
or generic-file transport.

The collection is native-memory-only and one-use. It grants no project,
workspace, terminal, shell, Git, worktree, browser, connector, MCP, approval,
dispatch, execution, or Advisor authority. Advisor remains read-only and does
not receive source paths, raw archive/binary bytes, credentials, or persisted
attachment state.

## Required M46 tests

- Deterministic typed fixtures for each supported entry and every aggregate
  boundary, including three entries, four entries, duplicate ids, and more than
  one image.
- Ordered manifest/card, remove, compact-position, keyboard, focus, narrow
  layout, screen-reader, and no-drag/no-browser-file-input coverage.
- Per-entry validation, expiry, hash mismatch, confirmation omission,
  cancellation, one-use claim, disposal, start failure, interruption, and
  terminal cleanup coverage.
- Atomic preflight/commit tests proving that any invalid item starts no turn and
  consumes no other item; successful sends claim and dispose all items once.
- Transport fixtures proving exactly one text input plus at most one existing
  `localImage`, ordered path-free projections, no source bytes/paths, and no
  new app-server method or settings.
- Package, accessibility, bundle-ceiling, provenance/ABI, lifecycle, smoke,
  installed-package, and visible-launch evidence for `0.1.0-beta.42`.

## Approval required

This decision changes the user-visible attachment cardinality and requires a
multi-item atomic claim lifecycle. M46 must not begin until the recommendation
above is explicitly approved.
