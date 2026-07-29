# Milestone 53 — Local Artifact and Design Review Proposal

## Status and decision

M53 is complete as a proposal-only milestone. This document is the authoritative, implementation-ready decision for M54. It reconciles the inspection in [M53-A](MILESTONE_53A_ARCHITECTURE_INSPECTION.md), the selected core contract in [M53-B](MILESTONE_53B_CORE_REVIEW_CONTRACT.md), and the selected interaction contract in [M53-C](MILESTONE_53C_REVIEW_INTERACTION_CONTRACT.md).

The one selected design is **`local-review-collection-v1`**. M54 has not started and no runtime, schema, migration, package, or release evidence changed in M53.

The current implemented and validated package remains application `0.1.0-beta.46` and Debian `0.1.0~beta.46`. M54’s approved implementation target is application `0.1.0-beta.47` and Debian `0.1.0~beta.47`; that target does not change any current declaration or artifact.

## Scope, exclusions, and terminology

M54 will add bounded local review of task-contextual briefs, plans, mockups, safe previews, copied evidence, annotations, comparisons, and explicit promotion into M48 generated artifacts. It is not a file manager, browser, provider integration, design connector, execution surface, or approval mechanism.

Terms are exact: a **collection** is a native-owned, task-scoped review record; an **item** is one closed-class copied payload; an **evidence envelope** is copied typed metadata/content, never a live external reference; an **annotation** is a local item-level note; a **comparison binding** identifies two eligible text items while its result is recomputed; a **promotion reservation** is a five-minute one-use native preparation; a **generated artifact** is M48’s separate transient record. M48 Save writes a user-selected file through its existing separate authority. Approval, dispatch, and execution remain separate.

M54 excludes Figma and other third-party connectors, scraping, filesystem indexing, arbitrary files, SVG, PDF, HTML, archives, audio, video, animated images, arbitrary binary, URLs, live remote references, network access, browser automation, cloud sync, accounts, collaboration, rich text, range/image anchors, Git mutation, terminal/shell use, automatic action, automatic approval, automatic promotion, publishing, and deployment.

## Selected architecture

`LocalReviewService` is the sole native authority. It persists bounded records through a migration in the existing private mode-0600 `metadata.sqlite3`; it creates no second database. Strict fixed IPC envelopes and frontend Zod schemas project only validated, path-free records. The frontend owns transient selection, rendering, focus, and presentation only. M48 remains the direct owner of live generated content; copying a currently live digest-matched M48 artifact is an explicit review-item creation source. Review neither references arbitrary paths nor obtains filesystem authority.

Every collection has an immutable UUIDv7 task reference and may have one immutable observed selected-or-alternate-plan reference. Task and plan provide context only: neither owns retention, selects a plan, transfers authority, authorizes promotion, dispatches, approves, or executes. Items, annotations, comparison bindings, evidence references, and promotion reservations use opaque UUIDv7 identifiers; identities never derive from title, timestamp, order, digest, or path. SHA-256 binds copied content, preview source, comparison sides, and promotion source. Duplicate content creates distinct items with distinct provenance; no content-addressed aliasing occurs. Digest mismatch, corruption, or unavailable data withholds content and returns a closed diagnostic.

### Closed item classes and representations

| Class | Representation and validation | Eligible operations | Explicitly excluded |
| --- | --- | --- | --- |
| `text` | Normalized UTF-8 inline payload, one of `plain`, `markdown`, `json`, `csv`, or `python`; stored format, title/label, source kind, provenance, size, timestamps, schema version, and SHA-256. JSON/CSV are syntax-validated; Python is text only. | inert preview, item annotation, same-format pairwise comparison, possible promotion | HTML rendering, execution, arbitrary source paths |
| `image-mockup` | Native-validated static PNG or JPEG bytes with format, dimensions, byte count, digest, source kind, provenance, title/label, and timestamps. | inert preview and item annotation | SVG, animation, comparison, promotion, image-coordinate annotation |
| `evidence` | Copied typed, bounded, path-free evidence envelope with type, source kind, provenance, normalized fields, timestamps, schema version, and digest. | inert preview and item annotation | live activity/Git/diff/approval reference, comparison, promotion, automatic approval |

No original path, path-derived metadata, URL, or remote token is stored or projected. Preview is derived from the validated stored payload and carries its source digest; it is not a separately authoritative content type.

## Exact limits and retention

| Subject | Hard limit | Warning |
| --- | ---: | ---: |
| Non-discarded collections | 24 | 20 |
| Active-task collections | 12 | 10 |
| Items per collection | 12 | 10 |
| Image items per collection | 3 | 2 |
| Evidence items per collection | 6 | 5 |
| Text item | 256 KiB; 32,768 code points | 192 KiB |
| Image item | 1 MiB; 4,096 pixels/dimension; 8,000,000 pixels | 768 KiB |
| Evidence envelope | 16 KiB | 12 KiB |
| Annotations per item | 32 | 24 |
| Annotation | 1 KiB; 1,024 code points | 768 bytes |
| Comparison bindings per collection | 8 | 6 |
| Comparison side | 128 KiB; 2,000 lines | 96 KiB |
| Preview text | 128 KiB; 2,000 lines | truncation notice at limit |
| Collection payload | 4 MiB | 3 MiB |
| Total persisted review payload | 32 MiB | 24 MiB |
| Process-wide promotion reservations | 16; one use; five minutes | capacity diagnostic |

Collections are `active`, `frozen`, `orphaned`, `unavailable`, or `discarded`; items are `ready`, `stale`, `unavailable`, or `discarded`; annotations are `open` or `resolved`; comparisons are `ready`, `stale`, or `unavailable`; reservations are `prepared`, `consumed`, or `expired`. Completion/archive freezes a collection; task restore does not reactivate it, and eligible reactivation is explicit. Task deletion orphans rather than cascades review. Plan edit marks its linked material stale using the observed timestamp; deletion/unavailability never rebinds. Restart revalidates persisted records, while reservations expire across restart. Transactions reject stale writes with no partial mutation. Explicit discard is application-irreversible only for review records. Frozen or orphaned records are eligible for explicit cleanup after 180 days; there is no automatic deletion or silent expiry.

## Preview, annotation, comparison, and evidence

Previews are native-bounded and frontend-inert. Text is escaped; Markdown is displayed as text, not active HTML; JSON, CSV, and Python receive no semantic execution; image data is displayed only after native static PNG/JPEG validation; evidence is a bounded field view. Previews have no script, iframe, object, embed, live link, remote resource, path-opening, browser-navigation, or clipboard auto-write behavior. Unavailable, corrupt, stale, or digest-mismatched data is withheld with a closed diagnostic.

Annotations are item-level, plain normalized UTF-8, and authored only by the local installation. Users may create, edit, resolve, reopen, and delete them while collection/item state permits; open annotations precede resolved annotations and each group is creation-order stable. They have no offsets, ranges, image coordinates, mentions, remote identities, reactions, notification, task/plan mutation, source-content mutation, approval consequence, or promotion payload. Stale/unavailable items preserve content but block mutation as required by their state.

Comparison is pairwise only: two distinct `ready` text items of the same `textFormat`, each still within its comparison limit. The native service recomputes a deterministic line comparison on demand; it stores the binding and side digests, not a result, and never invokes Git. Changed, stale, missing, corrupt, or mismatched sides make it unavailable. Images and evidence cannot be compared, comparison never mutates either side, and it is excluded from promotion.

Evidence is copied as a typed envelope at creation. It may describe existing safe local observations, but cannot retain raw path, raw command output, credentials, approval bodies, remote handles, or live external content. It is never sufficient for approval by itself.

## Promotion and authority matrix

Only a `ready` text item with matching digest, fresh active task, and fresh optional plan can be prepared. Preparation creates one digest-bound five-minute reservation. Explicit confirmation consumes it once and copies the source into the mapped M48 text class; M48 capacity, digest, freshness, duplicate, expiry, or concurrent failure leaves the source valid and reports a closed diagnostic. Source review records remain. Annotations, comparisons, evidence, and image mockups are never transferred. Promotion is neither M48 Save nor a file write.

| Authority | May do | Must not do |
| --- | --- | --- |
| Review | create bounded collections/items, inspect, preview, discard, prepare promotion | paths, traversal, directory search/index, Git, terminal, shell, network, providers, browsing, execution, dispatch, approval, save, publish, deploy |
| Annotation | mutate bounded local annotation records | alter source/task/plan/evidence/approval/artifact/execution state |
| Comparison | recompute bounded same-format text comparison | mutate either side or invoke Git |
| Promotion | explicitly copy eligible text into M48 | Save, user-file write, approval, dispatch, execution, Git, task/plan selection, publish, deploy, automatic later authority |
| Approval and execution | remain separate existing authorities | infer approval from review readiness or promotion eligibility |

## Workbench interaction

M54 adds one seventh lazy `Review` tab (`local-review`) inside the existing `ReviewPanes`, after Preview and before Activity. It reuses the existing top-bar review-shell entry, 360–560 pixel desktop pane, lazy unmount/focus-return behavior, responsive overlay, and horizontally scrollable tabs. It adds no second panel, floating window, generic browser, or persistent custom layout.

The flow is collection list → collection detail → item master/detail. Collections use ordinary tab order, are ordered newest-first, show task/optional plan context, state, count, and quota, and are created only in selected task context. Items are newest-first with stable UUID tie-breaking; item cards use roving navigation and show label, class/format, source kind, state, size, shortened digest with full accessible label, annotation count, and eligible/disabled actions with reasons. No search, filter, reordering, drag/drop, browser file input, automatic import, or task/plan mutation is offered.

Creation is a bounded modal flow for authored text, live digest-matched M48 copy, native single PNG/JPEG selection, and typed evidence snapshot. The image picker is user-initiated, image-only, one file, native-validated before persistence, returns no path to the frontend, permits no directory/batch/drag-drop/URL retry, and cancels without state change. Success selects the new item; validation/quota failures focus their error summary.

The detail view presents a title/metadata strip, inert preview, and item-level annotation list/editor. Image mockups and evidence have item annotations only. Compare opens a same-format eligible-side chooser; results label left/right digest-bound sides and line additions, deletions, and changes. Promote opens preparation, then a modal confirmation with item, format, digest, task, optional plan, M48 destination, five-minute remaining lifetime, and the exact statement that it creates only a transient M48 generated artifact and performs no save, approval, dispatch, code execution, Git change, publishing, or deployment. Approval is read-only presentation: it may explain readiness and reservation state but contains no confirmation control. Activity is a current-session, bounded newest-first projection of 12 redacted local-review events, never a durable audit log; background events may badge/announce but never take focus.

At `<=760px` width or `<=520px` height, the existing overlay applies, resize is hidden, and detail uses a single-level back stack. Long code lines scroll within their own region; images fit their container without hidden remote loading. Opening puts focus on the selected tab/list; Escape closes dialogs then returns to invoking control, or closes the shell and returns to its opener. Arrow keys navigate roving item cards, Home/End go first/last, Enter/Space activate, dialogs trap focus with Cancel initially focused, and deletion/error/stale removal restores focus deterministically to the nearest surviving control. Background change never steals focus.

The pane is a labelled complementary review region with semantic tabs/tabpanels, labelled collection/item lists and selected states, headings, visible worded badges, labelled forms/counters, polite success/status messaging, alert error summaries, semantic dialogs, textual comparison labels, descriptive image alt text, and accessible digest labels. It supports keyboard-only operation, reduced motion, contrast/touch-target requirements, and 200% zoom/reflow.

## Failure, security, and privacy

Capacity/aggregate quota, invalid class/encoding/representation, unsupported format, digest mismatch, stale source/write, missing task/plan, storage/migration failure, corrupt row, preview failure, unavailable side, expired reservation, M48 capacity, and concurrent mutation reject the requested mutation, preserve prior valid data, and return a bounded diagnostic. Corrupt/unavailable payloads are withheld rather than repaired, searched for, or fetched. Explicit user repair/discard is required where permitted.

M54 controls must preserve private SQLite permissions, schema-version rejection, bounded parsing and pixel limits, normalized UTF-8, digest binding, immediate transactions, redacted logging/activity/errors, and safe crash/restart behavior. It must never persist paths, credentials, raw protocol messages, approval bodies, command output, remote URLs, or provider/connector state; expose them to the frontend; perform directory traversal; load remote content; parse SVG/HTML/PDF; execute content; or expose Git, terminal, filesystem, browser, dispatch, approval, publishing, deployment, or privileged authority. SQLite deletion is not a claim of physical erasure from journals, media, or backups.

## Exact M54 implementation scope and acceptance criteria

M54 implements the bounded `LocalReviewService`, private-SQLite migration, strict native types/IPC/Zod bridge, Review tab and all selected interactions, deterministic tests, beta.47 Debian package evidence, and governing-document implementation updates. It does not authorize work before approval.

1. **M54-AC-001 — Baseline and package.** Start from the approved M53 tip; change all authoritative declarations together to `0.1.0-beta.47`/`0.1.0~beta.47`; Debian remains the sole artifact; preserve all prior evidence.
2. **M54-AC-002 — Native storage.** Implement one bounded `LocalReviewService` and migration in existing private mode-0600 `metadata.sqlite3`, with no second database, supported upgrade, closed future-schema failure, immediate transactions, stale-write rejection, and restart recovery.
3. **M54-AC-003 — Identity and integrity.** Use UUIDv7 for every authoritative identity and SHA-256 bindings; never use a path, title, order, or timestamp as identity; preserve separate duplicate copies.
4. **M54-AC-004 — Corruption and deletion.** Withhold corrupt/digest-mismatched rows, preserve valid prior state, return closed diagnostics, and make explicit review discard irreversible only within review records.
5. **M54-AC-005 — Text representation.** Accept only normalized UTF-8 `plain`, `markdown`, `json`, `csv`, and `python`; validate JSON/CSV syntax and retain fixed metadata/provenance/schema/digest fields.
6. **M54-AC-006 — Image representation.** Accept only one native-validated static PNG/JPEG payload meeting byte, dimension, and pixel ceilings; reject SVG, animation, and unsafe/invalid image inputs.
7. **M54-AC-007 — Evidence representation.** Persist only copied typed, bounded, path-free evidence envelopes and never live activity, Git, diff, approval, remote, or provider references.
8. **M54-AC-008 — Intake refusal.** Refuse arbitrary files, URLs, remote references, batch intake, drag/drop, browser file input, and every unsupported content class.
9. **M54-AC-009 — Collection/item ceilings.** Enforce 24/20 non-discarded, 12/10 active-task, 12/10 items, 3/2 images, and 6/5 evidence collection limits.
10. **M54-AC-010 — Payload ceilings.** Enforce text 256 KiB/32,768 code points with 192 KiB warning; image 1 MiB/4,096 dimensions/8,000,000 pixels with 768 KiB warning; evidence 16 KiB with 12 KiB warning; collection 4 MiB with 3 MiB warning; total 32 MiB with 24 MiB warning.
11. **M54-AC-011 — Annotation/comparison ceilings.** Enforce annotations 32/24 per item and 1 KiB/1,024 code points with 768-byte warning; comparisons 8/6 per collection.
12. **M54-AC-012 — Preview/comparison bounds.** Enforce 128 KiB/2,000-line/96 KiB-warning comparison-side limits and 128 KiB/2,000-line preview truncation with explicit notices.
13. **M54-AC-013 — Task binding.** Require one immutable task reference; support no task-independent collection; neither review controls nor records mutate task state or transfer retention/authority.
14. **M54-AC-014 — Plan binding.** Permit one optional immutable observed selected-or-alternate-plan reference without selecting/editing/rebinding plans or transferring authority.
15. **M54-AC-015 — Lifecycle.** Implement exactly collection active/frozen/orphaned/unavailable/discarded, item ready/stale/unavailable/discarded, annotation open/resolved, comparison ready/stale/unavailable, and reservation prepared/consumed/expired transitions.
16. **M54-AC-016 — Task/plan effects.** Freeze on completion/archive, require explicit eligible resume after restore, orphan on task deletion, stale on plan observed-timestamp change, and never cascade/rebind on plan deletion/unavailability.
17. **M54-AC-017 — Retention/recovery.** Revalidate on restart, lose reservations on restart, allow only explicit cleanup after 180 qualifying frozen/orphaned days, and never auto-delete/expire durable review data.
18. **M54-AC-018 — Creation flows.** Provide bounded authored text, digest-matched live M48 copy, native single-image selection, and typed evidence snapshot flows with validation, quota, cancel, success, and failure focus behavior.
19. **M54-AC-019 — Image-picker authority.** Restrict the native picker to user-initiated one-file static PNG/JPEG intake; validate before persistence; never return/persist path, enumerate directory, retry automatically, or accept batch/URL/drop input.
20. **M54-AC-020 — Inert previews.** Escape text; render Markdown/JSON/CSV/Python inertly; display validated image data only; bound evidence view; withhold stale/unavailable/corrupt/mismatched content.
21. **M54-AC-021 — Preview isolation.** Prohibit script, active HTML, live links, remote resources, iframe/object/embed, browser navigation, arbitrary path opening, execution, and clipboard auto-write.
22. **M54-AC-022 — Annotations.** Implement local-author, item-level create/edit/resolve/reopen/delete, deterministic ordering, quota feedback, and state-gated mutation.
23. **M54-AC-023 — Annotation exclusions.** Exclude ranges, offsets, image coordinates, mentions, remote identity, reactions, notifications, source mutation, approval effects, and promotion payloads.
24. **M54-AC-024 — Comparisons.** Permit only two distinct ready same-format text items; bind side digests; recompute native deterministic line results on demand; persist no result and invoke no Git backend.
25. **M54-AC-025 — Comparison refusal.** Refuse image/evidence, semantic JSON/CSV, invalid/oversized/stale/missing/corrupt/mismatched sides; mutate neither side and exclude comparisons from promotion.
26. **M54-AC-026 — Promotion preparation.** Prepare only ready eligible text with matching digest, fresh active task/fresh optional plan, mapped M48 class, and one of 16 native reservations lasting five minutes.
27. **M54-AC-027 — Promotion confirmation.** Require explicit separate confirmation, consume reservation once, copy into M48, preserve source, and expose duplicate/capacity/freshness/expiry/cancel diagnostics without partial mutation.
28. **M54-AC-028 — Promotion isolation.** Exclude image/evidence/annotations/comparisons; make promotion neither M48 Save nor a file write, approval, dispatch, execution, Git, task/plan mutation, publishing, or deployment.
29. **M54-AC-029 — Review pane.** Add lazy `local-review` visible `Review` tab after Preview and before Activity in existing ReviewPanes; add no second panel/window/layout persistence and preserve existing panes.
30. **M54-AC-030 — Navigation/cards.** Implement newest-first collection/detail/master-detail navigation, ordinary collection tab order, roving item cards, required metadata, disabled-action reasons, and no search/filter/reorder/auto-open/task-plan mutation.
31. **M54-AC-031 — Interaction states.** Deterministically present loading, empty, warning, quota, frozen, orphaned, stale, unavailable, corrupt, missing task/plan, digest mismatch, expired, promotion failure, and concurrent-write states with permitted controls and closed diagnostics.
32. **M54-AC-032 — Activity/Approval.** Project at most 12 newest-first redacted session events without durable audit log/raw paths/content/credentials; keep Approval read-only, distinguish readiness/promotion/generated artifact/execution approval, and never confirm promotion there.
33. **M54-AC-033 — Responsive and keyboard.** Preserve 360–560 desktop width/separator, narrow overlay at <=760px width or <=520px height, hidden narrow resize, scrollable tabs, single Back stack, long-line/image behavior, and specified arrows/Home/End/Enter/Space/Escape/dialog behavior.
34. **M54-AC-034 — Focus.** Set deterministic opening, dialog, creation/deletion/error/stale-removal, narrow-back, and closing focus restoration; background updates may badge/announce but never steal focus.
35. **M54-AC-035 — Accessibility.** Implement labelled landmark/tabs/tabpanels/lists/forms/counters, selected/badge wording, polite status and alert errors, modal semantics, image alt text, textual comparison semantics, digest labels, reduced motion, contrast/touch targets, and 200% reflow.
36. **M54-AC-036 — Security/privacy.** Verify no stored/projected paths, traversal, arbitrary filesystem, hidden network, provider/connector/browser automation/raw protocol/credentials/approval-body/command-output exposure; retain private SQLite/redacted logging/closed diagnostics/bounded image parsing.
37. **M54-AC-037 — Native/frontend tests.** Add deterministic migration, identity, quota, corruption, lifecycle, task/plan, preview, annotation, comparison, promotion, strict schema/unknown-field, and path-free/authority-free envelope tests.
38. **M54-AC-038 — UI tests.** Add deterministic component/accessibility/browser checks for desktop/narrow/short-height layouts, keyboard/focus, creation/annotations/comparison stale-side/promotion expiry/freeze-plan/orphan/quota/corruption, no focus theft/network/file input, and no approval/execution confusion; retain evidence.
39. **M54-AC-039 — Package/host validation.** Before installed-host validation, inspect `scripts/validate_installed_deb.sh`, the repository sudoers example, and a prior approved invocation. If that required script is unavailable, stop and diagnose the approved validator rather than substituting a privileged command. Run a `sudo -n` capability check before any password request; use only approved restricted sudo, validate beta.47 Debian identity, ABI, provenance, manifest/checksums, visible launch, sandbox-worker lifecycle, and restricted-host behavior.
40. **M54-AC-040 — Closure.** Run all relevant existing/new tests, independently diagnose/rerun resource contention, record actual counts and ignored probes, preserve Debian-only evidence, write milestone report, make intentional implementation/evidence commits as convention requires, normal push/post-fetch/equality/clean-process closure, and do not release or deploy.

## M54 validation and sequencing

M54 must select exact commands from repository conventions and validate repository/document formatting, frontend lint/type-check/build, native format/lint/build/tests, migration/storage permissions, strict schemas, components/accessibility/browser checks, quota/corruption, no hidden network/file input/path projection/authority expansion, Debian build/ABI/provenance/manifest/checksum/launch/sandbox worker/restricted installed host, and Git/process closure. It must inspect the installed-DEB validator and sudoers example before restricted host validation; M53 does not run packaging or sudo. At M53 closure, `scripts/validate_installed_deb.sh` is not tracked in this checkout, while prior package evidence documents the restricted `/usr/local/sbin/quireforge-validate-deb` wrapper; this is an M54 preflight fact, not permission to invent or broaden privileged access.

Implementation order: (1) verify and plan; (2) migration/types; (3) native service/tests; (4) Zod bridge; (5) Review shell/navigation; (6) creation/preview; (7) annotations; (8) comparisons; (9) promotion; (10) Activity/Approval; (11) responsive/accessibility; (12) complete validation; (13) beta.47 packaging; (14) restricted installed-host validation; (15) evidence and closure. Intermediate checkpoints do not complete M54.

## Rejected alternatives and traceability

M53 rejects persistent M48 expansion (violates M48 transient boundary), task-independent or task-owned review (breaks M52 authority/retention separation), generic file intake (creates filesystem authority), Git comparison (misstates comparison authority), active Markdown/web preview (creates browser/network risk), image comparison/coordinate notes (exceeds bounded v1), automatic/bulk promotion (confuses authority), and a new panel/window (violates M49/M50 constraints). The three phase records above supply the evidence, questions, and detailed traceability for every selected constraint and criterion. No ADR is added: as for M51, this authoritative final milestone proposal is the repository’s decision record.

## Proposal closure and approval gate

M53 has selected one design and M54 has not been implemented. M54 approval authorizes only the bounded implementation target; it does not authorize release or deployment.

Approve M54 — Local Artifact and Design Review implementation targeting 0.1.0-beta.47? Reply exactly: Approve
