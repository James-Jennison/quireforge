# Milestone 53-B — Core Local Review Contract

Status: decision-ready core contract for M53-C and M53-D. This document selects one bounded M54 design; it does not define the final interaction model, final M54 acceptance criteria, or any runtime implementation.

## 1. Scope, inputs, and selected design

This contract resolves every M53-B question in [M53-A](MILESTONE_53A_ARCHITECTURE_INSPECTION.md): item classes and representations; native ownership; IDs and integrity; quotas; lifecycle; task/plan linkage; preview, evidence, annotation, comparison, and promotion semantics; failures; and security/privacy boundaries.

**Selected design — `local-review-collection-v1`:** a native-owned, task-scoped, private-SQLite collection of copied and validated local review payloads. A collection has one immutable M52 task reference and an optional immutable M52 plan reference. It never retains a filesystem path, opens a file, owns a task/plan, or transfers task, plan, approval, dispatch, Git, terminal, shell, network, publishing, deployment, or execution authority. Review data is bounded independently of M48 and M52; a task/plan is context only, never an execution or retention owner.

This is safer than a general local-file or live-artifact review surface: content enters only through a later fixed native command with a closed representation, is copied into a bounded review record after validation, and is projected back through strict schemas. M48 remains the sole owner of live generated content; an M48 artifact may be explicitly copied while live and digest-matched, but no review record keeps an M48 reference or extends its 15-minute lifetime.

## 2. Supported classes and exact representations

The closed `ReviewItemClass` enum is `text`, `image-mockup`, or `evidence`. There are no other classes.

| Class | Purpose and allowed source | Canonical persisted representation | Preview / annotation / comparison / promotion | Explicitly prohibited |
| --- | --- | --- | --- | --- |
| `text` | Briefs, plans, generated-artifact copies, and small design notes. Native accepts only explicitly supplied text or a digest-checked, still-live M48 preview. | Normalized UTF-8 LF bytes plus `textFormat`: `plain`, `markdown`, `json`, `csv`, or `python`; SHA-256 is over these canonical bytes. JSON must parse; CSV must use the existing rectangular CSV validation; all other formats are text, not executable code. | All formats preview and annotate. Same-format pairwise text comparison only. Eligible for explicit M48 promotion to the matching M48 class (`plain` maps to `text`). | HTML, SVG, PDF, archives, arbitrary binaries, links/URLs as content authority, scripts, execution, and arbitrary file import. |
| `image-mockup` | Static local design mockup supplied only through a future fixed native review-input command, not a browser file input. | Validated original PNG or JPEG bytes, MIME `image/png` or `image/jpeg`, width/height, byte count, SHA-256. PNG must be non-animated; JPEG must be complete. | Inert image preview and item-level annotation only. It is not comparable or promotion-eligible in v1. | SVG, APNG/animation, PDF, video, audio, image URLs, image decoding beyond bounded validation, coordinate/range annotation, or M48 promotion. |
| `evidence` | Explicit bounded snapshot of a supported, already-authorized local presentation: M48 manifest/preview metadata, safe-preview metadata, Git/status/diff summary, normalized activity/approval presentation, package-manifest summary, or manually entered validation summary. | A strict UTF-8 `EvidenceEnvelopeV1`: source enum, source schema version, redacted/path-free provenance fields, and normalized summary text; SHA-256 covers the envelope's canonical JSON UTF-8. It copies no raw protocol, path, approval body, command output, Git object ID, file bytes, or remote content. | Preview and item-level annotation. Not comparable and never promotion-eligible. | Live reference, polling/subscription, automatic capture, approval decision, evidence-as-consent, external fetching, or copying source-file content. |

`text` deliberately supports plain text, Markdown, structured JSON, CSV, and Python only as inert UTF-8 text formats. `image-mockup` is the sole binary class. Arbitrary local files are not review items; existing `FilePreviewService` remains a separate attached-project preview boundary and may only supply the narrow metadata input described for `evidence`.

Every item has schema version 1, opaque UUIDv7 `itemId`, a normalized display label (1–120 Unicode code points, 480 UTF-8 bytes; no controls, bidi format characters, slash, or backslash), class-specific MIME/format, byte size, content SHA-256, and `createdAtMs`. Its immutable closed `sourceKind` is exactly `user-authored-text`, `m48-artifact-copy`, `native-image-input`, or `typed-evidence-snapshot`; the class/source combination must match the table above. No path, filename, directory, URL, account, credential, raw task-plan body, or hidden provenance is stored. There is no separate original/derived preview content: text preview is an inert, normalized truncation of the canonical bytes, and image preview is a validated data URL derived on demand from the stored image bytes. Derived previews are never persisted or cached.

## 3. Native ownership, frontend projection, and identity

M54 must add one native `LocalReviewService` beside the existing `ProjectService`, using migration-owned tables in the existing private mode-0600 `metadata.sqlite3`: `review_collections`, `review_items`, `review_annotations`, and `review_comparisons`. It must not modify M48's process-local registry or put review data in layout preferences. Native code owns UUIDv7 generation, SQLite transactions, validation, normalization, SHA-256 calculation/recalculation, quotas, lifecycle derivation, task/plan revalidation, and closed diagnostics.

The frontend receives versioned, strict, path-free projections and sends only fixed Zod-validated request envelopes with `deny_unknown_fields` equivalents. Read-only operations are list, detail, preview, annotation list, and comparison projection. Metadata mutations are create/discard collection, add/discard item, add/edit/resolve/delete annotation, and create/discard comparison; none opens a picker, reads a path, saves a file, runs Git, calls a provider, dispatches, decides approval, starts execution, or contacts a network.

Identity is opaque UUIDv7, never a label, title, timestamp, path, or array position:

| Entity | Identity and integrity |
| --- | --- |
| Collection | UUIDv7 `collectionId`; schema v1; immutable task UUIDv7 and optional immutable plan UUIDv7 plus observed plan `updatedAtMs`. |
| Item | UUIDv7 `itemId`; SHA-256 of canonical text/image/envelope bytes; duplicate bytes are allowed as distinct intentional items and do not deduplicate or share ownership. |
| Annotation | UUIDv7 `annotationId`; item UUIDv7; no content anchor, range, or image coordinate identity. |
| Comparison | UUIDv7 `comparisonId`; left/right item UUIDv7 plus both creation-time SHA-256 values and comparison format. |
| Evidence source | UUIDv7 item identity only; its source is copied into the immutable envelope, not a live external identifier. |
| Promotion candidate | Native in-memory UUIDv7 reservation, bound to `collectionId`, `itemId`, source SHA-256, requested M48 class, and task/plan freshness; one use, five-minute expiry, maximum 16 process-wide. It is never SQLite data. |

An item digest mismatch, invalid stored UTF-8, invalid class representation, invalid reference shape, or impossible quota is an integrity failure. Native projection withholds content, returns an `unavailable` item/collection projection with a closed diagnostic, and permits only explicit discard; it never repairs, rehashes, substitutes, searches for, or refetches content.

## 4. Limits and quotas

All sizes are UTF-8 bytes except dimensions. Native validates before mutation in one immediate transaction and refuses without eviction or partial success.

| Limit | Exact ceiling | Warning |
| --- | ---: | ---: |
| Non-discarded collections | 24 total | 20 |
| Collections with active task review | 12 | 10 |
| Items per collection | 12 | 10 |
| Image-mockup items per collection | 3 | 2 |
| Evidence items per collection | 6 | 5 |
| Text item canonical content | 256 KiB, 32,768 code points | 192 KiB |
| Image-mockup bytes | 1 MiB; PNG/JPEG only, <=4,096 each dimension and <=8,000,000 pixels | 768 KiB |
| Evidence envelope | 16 KiB | 12 KiB |
| Item label / collection title | 120 code points, 480 bytes | none |
| Provenance summary metadata | 256 bytes | none |
| Annotations per item | 32 | 24 |
| Annotation text | 1 KiB, 1,024 code points | 768 bytes |
| Comparison records per collection | 8 | 6 |
| Text comparison side | 128 KiB, 2,000 lines | 96 KiB |
| Collection persisted payload | 4 MiB | 3 MiB |
| All persisted review payload | 32 MiB | 24 MiB |
| In-memory promotion reservations | 16 | none |

Text preview displays at most 128 KiB and 2,000 lines; image preview uses the validated original only. Comparison is unavailable rather than truncated when either side exceeds its comparison limit. These caps preserve room for briefs, plans, a few static mockups, and evidence without becoming a file store.

## 5. Lifecycle, retention, restart, and concurrency

Collection state is `active`, `frozen`, `orphaned`, `unavailable`, or `discarded`. Item state is `ready`, `stale`, `unavailable`, or `discarded`; annotation state is `open` or `resolved` until deleted; comparison state is `ready`, `stale`, or `unavailable`; promotion reservations are `prepared`, `consumed`, or `expired` in memory only.

- A new collection is `active` only when its immutable task reference resolves to an unarchived active or paused M52 task. An optional plan reference may identify any current plan, including a non-selected alternate plan; creation never selects or edits it.
- Task completion or archive freezes the collection; task restore does not silently reactivate it. A later explicit resume may reactivate only if the task is active/paused and the plan reference, if present, is fresh. Task deletion makes it `orphaned`; it does not cascade-delete review data.
- A plan edit after capture makes its reference stale by `updatedAtMs`; plan deletion/unavailability makes the collection/item references stale or unavailable as applicable. Unselected is not stale. A stale plan is never auto-rebound, cloned, selected, or restored.
- M48-sourced text is copied only while its digest-matched preview is live. Once copied, M48 expiry/save/discard does not change the review copy; expiry before copy rejects creation with `source-expired`.
- Durable collections have no silent expiry. Completed, archived, orphaned, and explicitly frozen collections become cleanup-eligible 180 days after their last update, but no automatic cleanup runs. Explicit discard irreversibly deletes the collection and children atomically; it does not delete an M52 task/plan, M48 artifact, source file, package evidence, or external content.
- Promotion never transfers ownership. Source review records remain until explicit discard and retain their normal quota/cleanup status.
- Each mutation uses an immediate transaction and compares the expected item digest, collection `updatedAtMs`, and relevant plan observed timestamp. Concurrent modification returns `stale-write` without a partial change. After restart, durable records are revalidated; in-memory promotion reservations are lost and must be deliberately prepared again. A crash cannot leave an executing promotion or a partially committed collection transaction.

## 6. Task and alternate-plan relationship

Every collection requires one immutable M52 task ID; task-independent review is deliberately unsupported. The optional plan reference is immutable and may point to either selected or non-selected alternate plan at creation. It conveys only declared review context; it does not copy plan content, select a plan, create a plan, alter M52 cleanup, retain a task, grant approval, or change execution state.

Collections may be created for active or paused tasks. Completed/archived tasks freeze their collections; restored tasks require explicit review resume. Missing/corrupt/unavailable tasks yield unavailable projections and reject mutation/promotion. Deleted tasks orphan collections, preserving bounded local review data for explicit disposition. A valid non-selected plan remains a valid context; edited/deleted/unavailable plans make the recorded plan relationship stale/unavailable. Promotion requires a fresh active/paused task and, when present, a fresh existing referenced plan; it never requires plan selection and never changes it.

## 7. Safe preview, evidence, annotation, and comparison

Preview is native-validated and frontend-inert. Text is rendered as escaped plain text in `<pre><code>`; Markdown receives no HTML parser, link navigation, image loading, or script treatment. JSON, CSV, and Python are likewise escaped text; JSON/CSV validation is creation-time only. Image mockups are displayed only from a native-produced `data:image/png` or `data:image/jpeg` URL after byte, dimension, full-file, and animation checks. No iframe, object, embed, SVG, HTML, PDF renderer, remote URL, embedded resource, clipboard auto-write, drag/drop import, or browser file input is allowed.

`evidence` is a copied typed envelope, not a live link. It may describe only a user-selected M48 manifest/preview metadata, safe-preview metadata without display path/content, bounded Git/status/diff summary, normalized activity/approval presentation stripped of decision capability, package-manifest summary, or manual validation summary. Evidence is annotatable, not comparable, not promotion-eligible, and can never satisfy approval automatically.

Annotations are local-user, item-level UTF-8 plain text. They have UUIDv7 IDs, created/updated timestamps, `open`/`resolved` state, and deterministic creation-time/ID ordering. The local installation is the sole author; no account, collaborator, remote identity, mention, notification, activity event, range anchor, source offset, image coordinate, or mutable evidence/approval field exists. Any annotation may be edited, resolved, or explicitly deleted while its item is ready; stale/unavailable items permit viewing and explicit deletion only. Promotion excludes annotations, comparisons, and evidence.

Comparisons are optional pairwise records between two distinct ready `text` items of the same `textFormat`; only plain/Markdown/Python/JSON/CSV canonical text is compared. They bind both item IDs and creation-time digests but persist no result. Native recomputes a deterministic line comparison on demand within the side limits; JSON/CSV receive no semantic parser diff or generated summary. Any changed/missing/corrupt/stale side makes the comparison unavailable; no source is mutated. Annotations stay item-level and cannot target a comparison. Images and evidence cannot be compared in v1.

## 8. Explicit promotion eligibility and authority boundary

Promotion is an explicit, user-initiated preparation and confirmation operation, not M48 Save and not an approval decision. Only a ready `text` item is eligible when its collection is active, its task is fresh active/paused, its optional plan reference is fresh, its canonical bytes remain <=512 KiB, and its SHA-256 matches. Native maps `textFormat` to the corresponding existing M48 class and creates the five-minute in-memory candidate reservation. Confirmation rechecks every binding, copies the exact canonical text into the existing M48 creation boundary only after M54 adds the closed `explicit-review-promotion` provenance variant, and returns the resulting M48 manifest. M48's independent five-item/2-MiB/15-minute limits still apply.

Promotion copies bytes; it does not move ownership, save a file, open a path, transfer annotations/comparisons/evidence, select/modify a task or plan, decide approval, dispatch, execute, use Git, access a terminal, fetch a network resource, publish, or deploy. Duplicate promotion creates a new M48 artifact only through a new explicit candidate and is subject to M48 capacity; no deduplication or automatic retry occurs. Stale, unavailable, corrupt, expired, full, canceled, or failed promotion consumes or invalidates its candidate and preserves prior review/M48 state without partial promotion.

## 9. Failure, recovery, and diagnostics

All diagnostics are closed, path-free enums. Invalid/unsupported class, encoding, representation, digest, label, provenance, task/plan ID, annotation, comparison side, or promotion request rejects before mutation. Capacity/aggregate quota, image safety, malformed envelope, M48 expiry/capacity, stale reference/write, and preview/comparison limit failures preserve the prior valid state. Storage/migration/SQLite failure returns collection unavailable and performs rollback; a future schema is refused.

Corrupt stored review rows or digest mismatches expose only a path-free unavailable projection and closed reason, never content; they are omitted from normal ready lists and require explicit deletion. Missing/deleted task or plan causes the lifecycle outcome in section 6, never broad search or substitution. Restart reopens/revalidates durable SQLite records and drops promotion reservations. No error retries network work, invokes a provider, searches directories, runs shell/Git/terminal work, accepts evidence, or escalates authority.

## 10. Security, privacy, and explicit non-goals

Persisted content is only the bounded collection/item canonical bytes, safe metadata, evidence envelope, annotation text, IDs/digests/timestamps/states, immutable task/plan IDs, and comparison bindings in the existing private mode-0600 QuireForge `metadata.sqlite3`. It never persists paths, directory identity, source files, original M48 state, raw Codex protocol, transcripts, approval bodies/decisions, command output, credentials, provider/connector/browser data, URLs, external account data, terminal/Git state, comparison results, preview data URLs, or promotion reservations. Logs and diagnostics remain path-free and must not include content bytes; crash reports must not attach review payloads.

Native validation rejects traversal/path-like labels, unsafe controls/bidi formatting, oversized input, untrusted MIME, unsupported binary types, animated images, image decompression dimensions, malformed UTF-8/JSON/CSV, digest confusion, forged IDs, stale requests, and unknown fields. The webview receives only escaped text or validated PNG/JPEG data URLs; it cannot execute Markdown/Python, load remote resources, navigate links, use a browser picker, read a filesystem path, drag/drop data, access clipboard automatically, run a shell/terminal, call Git, dispatch, approve, execute, access a provider/connector, or publish/deploy.

Explicit non-goals are Figma and all third-party design connectors; repository scraping/indexing; file-manager behavior; arbitrary filesystem traversal; arbitrary local files; SVG/PDF/video/audio/archive/unrestricted binary support; hidden network/remote fetching; provider integration; cloud synchronization; external accounts; collaborative/multi-user review; browser automation; shell/terminal execution; Git mutation; dispatch; automatic action/approval/promotion/evidence acceptance; publishing/deployment; unbounded retention; content recovery from deleted sources; and any new package artifact.

## 11. Rejected alternatives and M53-C deferrals

Rejected: a live M48 reference model (would couple durable review to transient artifacts); task-independent collections (would add unscoped retention); arbitrary attached-file references (would create path/file authority); generic binary/mockup support (unsafe parser/storage expansion); coordinate/range annotations (requires an image/text anchoring contract beyond the smallest safe core); semantic/generated comparisons (would add parser/model ambiguity); persisted comparison output (unneeded retention); and automatic or approval-coupled promotion (authority confusion).

M53-C may decide only presentation and interaction details within this contract: workbench placement/reused pane, card/list layout, preview selection affordances, annotation/comparison/promotion controls and confirmation wording, focus/tab/keyboard behavior, narrow-layout behavior, live-status wording, empty/stale/corrupt/unavailable visual states, and deterministic component/browser test design. It may not change this core schema, authority, retention, class, quota, lifecycle, or promotion contract.

## 12. M53-A traceability and implementation boundary

| M53-A M53-B question group | Resolution in this contract |
| --- | --- |
| Classes, representations, identity, provenance, digest | Sections 2–3 close the three classes, canonical bytes/envelope, UUIDv7 IDs, SHA-256, and no paths/live references. |
| Limits, lifecycle, retention, task/plan relationship | Sections 4–6 set every quota, persistent bounded lifecycle, explicit discard, cleanup eligibility, immutable task/plan context, and stale/deletion behavior. |
| Preview, evidence, annotations, comparisons | Section 7 closes inert rendering, evidence sources/copying, local item annotations, and pairwise same-format text comparison. |
| Promotion, failures, security/privacy, non-goals | Sections 8–10 close digest-bound explicit M48 promotion, deterministic failures/recovery, and authority/privacy exclusions. |

No M54 implementation, schema, migration, runtime behavior, package identity, packaging, release, publication, deployment, provider, connector, browser, network, or external integration change occurred in M53-B.
