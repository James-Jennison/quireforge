# Milestone 47 — Generated Artifact Workflow Proposal

Status: decision-ready proposal. This document changes no product code, package,
Tauri command, capability, schema, storage, or runtime behavior. M48 must not
begin without explicit approval of this recommendation.

## Recommendation

Approve `advisor-generated-artifact-registry-v1`: a native-owned, process-local
registry for up to five explicitly created, reviewed text/data artifacts. It
reuses the established Rust service plus strict Rust/Zod bridge pattern and the
native dialog ownership used by the existing Advisor text export. It does not
turn Advisor output into a project file, a worktree input, a durable record, or
a dispatchable action.

M48 adds one closed artifact service, typed inline cards in the Advisor
workspace, and two fixed-purpose commands: create an eligible reviewed artifact
and save one claimed artifact through a native dialog. Save receives only an
opaque artifact identity and its manifest hash; the webview never supplies a
path or a destination directory.

## Closed artifact contract

Only these artifact classes are supported:

| Class | MIME type | Required suffix | Representation |
| --- | --- | --- | --- |
| `text` | `text/plain; charset=utf-8` | `.txt` | normalized UTF-8 text |
| `markdown` | `text/markdown; charset=utf-8` | `.md` | normalized UTF-8 text |
| `json` | `application/json` | `.json` | validated UTF-8 JSON text |
| `csv` | `text/csv; charset=utf-8` | `.csv` | validated UTF-8 CSV text |
| `python` | `text/x-python; charset=utf-8` | `.py` | normalized UTF-8 text |

`json` must parse as one JSON value with no trailing non-whitespace content.
`csv` must have UTF-8 text with a consistent field count for every non-empty
record under the selected CSV parser; it is data, never executable. `python` is
only text bearing a `.py` suffix: QuireForge does not parse, import, run,
lint, syntax-check, open, or otherwise execute it. No binary, image, archive,
document, directory, arbitrary extension, rich-text, or generic-file class is
introduced.

Each native registry entry has this conceptual typed shape:

```text
GeneratedArtifactManifestV1 {
  schemaVersion: 1
  artifactId: UUIDv7
  class: text | markdown | json | csv | python
  mimeType: fixed-for-class
  sourceKind: visible-completed-reply | visible-fenced-block
  displayLabel: 1..120 path-free characters
  suggestedFilename: validated basename with required suffix
  byteSize: 1..524288
  sha256: lowercase SHA-256 of normalized UTF-8 bytes
  createdAt: native monotonic instant
  expiresAt: native monotonic instant + 15 minutes
  state: ready | saving | expired | saved
  disposal: transient-memory-one-successful-save
}
```

The native-only entry additionally holds the normalized UTF-8 bytes and a
short-lived save reservation. It holds no source path, destination path,
project id, worktree id, Codex id, account data, credential, approval,
dispatch, terminal, Git, browser, connector, or provider data. The frontend
snapshot receives only the manifest and a bounded preview projection; it never
receives a filesystem path. A preview request uses `{ artifactId, sha256 }` and
returns only the entry's already validated normalized text, capped at the
per-artifact limit.

Suggested names are deterministic and class-bound: a complete reply is
`advisor-response.txt`; a fenced block is `advisor-output.<required-suffix>`.
The user can change the basename only in the native dialog, subject to the
same safe-name and exact-suffix validation. QuireForge never adds counters or
changes an extension to avoid a collision; the user must choose a different
unused name explicitly.

## Eligibility, ownership, limits, and lifecycle

An artifact is created only after a user explicitly chooses **Create artifact**
for either the visible completed Advisor reply as `text`, or one visible fenced
block with an explicitly selected supported class. The selected visible value,
class, label, and suggested basename cross the bridge once for native
validation and creation. There is no automatic extraction, background scan,
model instruction, file picker, project read, or conversion from an attachment.
Empty content, invalid JSON/CSV, unsafe names, unsupported fence labels,
oversized content, stale completed output, and malformed bridge input fail
closed without creating an entry.

The registry is owned by a dedicated `AdvisorGeneratedArtifactService` in the
desktop process, not by React state, SQLite, Advisor conversation persistence,
or the attachment registry. Its fixed bounds are:

- at most five ready or saving artifacts;
- at most 512 KiB normalized UTF-8 bytes per artifact;
- at most 2 MiB aggregate normalized UTF-8 bytes across all entries;
- creation order only, oldest first; no reorder, grouping, pinning, or hidden
  replacement; and
- 15 minutes from native creation. A successful save disposes the entry
  immediately; process exit, reset, and service drop dispose every entry.

When the count or aggregate ceiling would be exceeded, creation is refused and
the existing cards remain unchanged. The user may explicitly discard a ready
entry; disposal zeroes/removes the native byte buffer as far as Rust ownership
allows and removes its card. An expired entry is removed from the native
registry before an `expired` diagnostic is returned; it is never claimable,
recoverable, reused, or silently recreated. Expiry is checked on every
snapshot, preview, discard, reservation, and save operation.

Save claims exactly one `{ artifactId, sha256 }` entry under a short native
reservation. A duplicate, stale, expired, mismatched, or already saving claim
fails without a dialog or write. Cancellation, dialog failure, collision, and
pre-publication write failure release the reservation and retain the still-live
ready entry for an explicit retry or discard. A successful publication consumes
and disposes it exactly once. Saving several artifacts therefore requires
several explicit Save actions and dialogs.

## Preview, cards, and accessibility

M48 places a compact **Generated artifacts** region directly below the completed
Advisor reply that produced the candidate, not in Projects, Files, Worktrees,
Terminal, or an authority/approval pane. A card shows class, label, suggested
filename, byte size, SHA-256 copy affordance, remaining transient lifetime,
and state. It has separate **Preview**, **Save…**, and **Discard** controls.
Preview is inline, text-only, wrapped, selectable, and capped at 512 KiB; it
does not invoke a system opener, browser, syntax executor, renderer, or file
read.

The region uses semantic headings, buttons with explicit artifact names,
programmatic status for expiry/save outcomes, keyboard-operable controls and
dialog return focus, visible focus, adequate contrast, and a narrow-layout
single-column card arrangement. Countdown updates must not create an
unbounded live-region stream; announce only material transitions such as
expired, saved, or save failed. SHA-256 is exposed as text with an accessible
copy label, never as color alone. Cards cannot steal focus when a new reply or
expiry arrives.

## Native Save boundary

**Save…** is the only write operation. Native code opens a platform Save dialog
with the entry's fixed class-specific filter and suggested basename. React does
not provide a path, directory, filter, file handle, overwrite choice, content,
or arbitrary filename at save time. Native code requires a regular selected
absolute destination whose basename is safe and has the entry's exact lowercase
required suffix; it does not append, rewrite, infer, or accept another suffix.
The dialog's suggested basename is informational only and is independently
revalidated after selection.

Cancelling the dialog is a normal `save-cancelled` result: no target, receipt,
or lifecycle change is created and the artifact remains ready. QuireForge never
creates destination directories. Existing paths, including symlinks and
non-regular filesystem objects, cause `file-exists`/`save-failed`; the UI tells
the user to select a different unused name. There is no overwrite control and
no overwrite confirmation because overwrite is never an allowed operation.

The native implementation must publish a save atomically on Linux: write the
validated bytes to a private `0600`, `O_CLOEXEC|O_NOFOLLOW|O_CREAT|O_EXCL`
temporary regular file in the selected parent; write all bytes and `fsync` it;
then atomically publish only if the target does not exist using
`renameat2(RENAME_NOREPLACE)` or an equivalent same-directory no-replace
primitive. It must `fsync` the parent directory after successful publication.
The direct `create_new(target)` write used by the earlier single-file export is
not sufficient for M48 and must not be reused as the artifact implementation.

If preparation, writing, synchronization, publication, or directory sync fails,
M48 must report `save-failed`, never report success, best-effort remove the
temporary file, and release the reservation only if the artifact remains
unconsumed. A cleanup failure is a visible bounded diagnostic, not silent
success. A selected target is never partially published: only a fully written,
synced artifact may become visible at its requested name.

## Receipt and evidence

On success native returns a path-free, transient receipt:

```text
GeneratedArtifactSaveReceiptV1 {
  schemaVersion: 1
  artifactId: UUIDv7
  class: closed class
  filename: selected basename only
  byteSize: manifest byte size
  sha256: manifest SHA-256
  savedAt: native timestamp
}
```

The UI may show that receipt for the current process only. It must not retain a
destination directory or full path in React persistence, SQLite, logs,
diagnostics, support bundles, task records, package evidence, or later
sessions. Test and release evidence may record the class, byte count, hash,
result code, and path-free filename; it must not contain user content or a
user-selected path. No receipt exists after cancellation or failure.

## Security, privacy, and authority boundaries

The registry is a presentation/export boundary, not an Advisor capability. It
adds no shell or terminal execution, Git operation, project/worktree write,
dispatch or approval authority, automatic saving, hidden persistence, cloud
synchronization, provider integration, connector behavior, browser behavior,
generic filesystem access, or Advisor write authority. Advisor remains a
no-project, no-tools, no-approval, read-only conversation; artifact text is not
sent back to Codex and saving does not change the conversation.

The only filesystem authority is the one user-mediated native Save dialog for
one currently claimed artifact. Native code retains the selected path only for
that synchronous save transaction, validates it with no-follow/no-replace
semantics, and discards it before returning. No recent destinations, bookmarks,
directory handles, paths, file metadata, or content are persisted. The service
uses fixed Rust APIs and typed Zod schemas; it never accepts a shell command,
terminal command, URL, project identifier, Git argument, or arbitrary file
operation from the webview.

## Migration and compatibility

M48 introduces `GeneratedArtifactManifestV1`, `GeneratedArtifactSnapshotV1`,
and `GeneratedArtifactSaveReceiptV1` beside—never inside—the existing Advisor
attachment contracts. It adds no SQLite migration and no backward-reading
obligation because entries and receipts are process-local. Reset, app restart,
and upgrade discard them. Existing M30 text-export and M44–M46 attachment
behavior remains compatible and unchanged during M47; M48 may share only
validated-name, normalized-text, SHA-256, dialog, and no-replace primitives.
It must not reinterpret attachments as generated artifacts or merge their
registries, claims, expiry, transport, or authority.

## Explicit non-goals

- M48 implementation, package changes, or production artifact behavior in M47.
- Automatic, background, scheduled, bulk, project-relative, worktree-relative,
  or destination-memory saving.
- Overwrite, append, rename-in-place, directory creation, generic picker,
  open/import/run behavior, and generic filesystem access.
- Artifact execution, shell/terminal actions, Git mutation, dispatch, approval,
  provider, connector, browser, cloud, or sync behavior.
- Images, documents, archives, binaries, arbitrary file formats, templates,
  previews beyond bounded text, or durable artifact/history storage.

## Exact M48 acceptance criteria

1. Implement only `advisor-generated-artifact-registry-v1` as a separate
   process-local native service with the five classes, UUIDv7 IDs, strict
   Rust/Zod parity, five-entry/512-KiB/2-MiB/15-minute bounds, creation order,
   claim, expiry, discard, process-exit disposal, and no SQLite migration.
2. Add only fixed-purpose typed commands for creation, snapshot, bounded text
   preview, discard, and single-artifact save. Save accepts only artifact ID
   plus manifest SHA-256; no command accepts a source or destination path,
   project/worktree ID, arbitrary extension, command, URL, or authority field.
3. Permit creation only from an explicit user choice over visible completed
   Advisor output; validate normalized UTF-8, class-specific JSON/CSV rules,
   safe label/basename, suffix, byte limit, and aggregate limit natively.
4. Provide accessible inline cards and preview/save/discard interactions with
   keyboard, focus-return, status, responsive, screen-reader, and no-focus
   theft coverage. No file opener, browser, executor, or hidden background
   action may be added.
5. Use a native Save dialog per save and enforce fixed suffixes, regular
   absolute selected target, no directory creation, no symlink following, and
   no overwrite. Cancellation must be distinct from failure and preserve a
   ready artifact.
6. Implement the specified same-directory atomic no-replace publication and
   test successful byte/hash equality, existing file, symlink, wrong suffix,
   write/sync/publication failure, cleanup failure, and the guarantee that no
   partial selected target is reported as saved.
7. Return only the path-free transient success receipt; do not persist content,
   path, destination, receipt, or registry state. Verify restart/reset expiry
   and prove logs, fixtures, diagnostics, metadata, and package evidence omit
   source/destination paths and artifact content.
8. Add deterministic Rust, TypeScript, component, bridge, accessibility, and
   narrow-layout tests for every supported class and every stated boundary;
   include malformed bridge values, forged IDs/hashes, duplicate claims,
   capacity limits, aggregate limits, expiry, cancellation, retry after failure,
   one successful save, and no authority expansion.
9. Update architecture, threat model, README/status, roadmap, changelog, and
   package evidence for the M48 implementation. Run repository validation plus
   applicable source, bundle, Tauri, packaging, lifecycle, restricted installed
   package, and visible-launch gates for its unique `0.1.0-beta.43` candidate.

## Approval required

Approve this exact M47 recommendation before starting M48. Approval authorizes
only the bounded M48 implementation described above; it does not authorize any
non-goal or any expanded filesystem, execution, approval, dispatch, provider,
connector, browser, or persistence capability.
