# Milestone 53-C — Local Review Interaction Contract

Status: implementation-ready interaction contract for M54. This document selects one presentation and interaction design for the M53-B local-review-collection-v1 core contract. It does not alter that core contract, assemble the final M53 proposal, define final numbered M54 acceptance criteria, or implement M54.

## 1. Phase scope

M53-C defines the local-only workbench interaction for collection/item navigation, inert preview, item-level annotations, pairwise comparison, explicit two-step M48 promotion, Activity and Approval presentation, state messaging, keyboard/focus behavior, responsive behavior, accessibility, and deterministic test design.

It does not change M53-B storage, IDs, representations, quotas, lifecycle, retention, authority, or native ownership. It does not add runtime code, schema, migration, command, style, test, package change, release, deployment, provider, connector, browser automation, or network capability.

## 2. Inputs and traceability

### Required inputs

- [M53-A architecture inspection](MILESTONE_53A_ARCHITECTURE_INSPECTION.md) supplies the existing surface and reusable-service facts.
- [M53-B core review contract](MILESTONE_53B_CORE_REVIEW_CONTRACT.md) controls data, storage, lifecycle, quota, integrity, and authority.
- Existing implementation facts were inspected in **ReviewPanes.tsx**, all six **review-panes** modules, **App.tsx**, **AdvisorWorkspace.tsx**, **TaskCatalog.tsx**, **layoutPreferences.ts**, **styles.css**, and their existing tests.

### M53-A questions assigned to M53-C

| M53-A question | M53-C resolution |
| --- | --- |
| Where review lives and which M49 pane is reused | Sections 3–4 select one new Review tab in the existing ReviewPanes shell. |
| Cards/lists exposing class, provenance, selection, state, and limits | Sections 5–6 select bounded collection and item lists. |
| Text/image/evidence preview selection and rendering | Section 8 selects native-validated inert master/detail preview. |
| Annotation workflow and visible state | Section 9 selects selected-item local plain-text annotations. |
| Comparison interaction | Section 10 selects a non-Git pairwise comparison subview. |
| Promotion and explicit confirmation | Sections 11–12 select prepare then confirm with a five-minute reservation. |
| Focus, keyboard tabs, resize, responsive layout, accessibility | Sections 16–19 preserve M49/M50 behavior and extend it deterministically. |
| Empty, stale, corrupt, unavailable, loading, failure states | Section 15 defines closed presentation for every relevant state. |
| Activity and whether it is durable | Section 13 selects a bounded current-session projection only. |
| Approval wording | Section 14 selects read-only non-authority wording and no promotion controls. |
| Deterministic unit/component/browser tests | Sections 20–24 define native, schema, component, accessibility, and browser requirements. |

### M53-B constraints the UI exposes, not redesigns

The UI must preserve these closed core rules:

- Collections are native-owned, task-scoped, durable only in existing private mode-0600 metadata.sqlite3, and have immutable task plus optional plan context.
- The only classes are text (plain, Markdown, JSON, CSV, Python), static PNG/JPEG image-mockup, and copied typed evidence.
- Text may preview, annotate, compare only with ready same-format text, and may promote. Images/evidence may preview and annotate only.
- Annotations are local, item-level plain UTF-8, and have open/resolved state only.
- Comparisons bind two distinct ready same-format text items and both digests, recompute natively, persist no result, and never use Git diff.
- Promotion is explicit, digest/task/plan-freshness-bound, two-step, five-minute ephemeral native reservation, and a copy into M48.
- Closed native diagnostics remain path-free; content is withheld whenever M53-B requires it.

If a presentation preference conflicts with those constraints, M54 must preserve M53-B rather than broaden capability.

### M53-B interaction-deferment and constraint checklist

| M53-B closed decision or M53-C deferral | Interaction-contract location |
| --- | --- |
| Workbench placement, pane reuse, cards/lists, preview affordance | Sections 3–8 |
| Annotation, comparison, promotion controls and confirmation wording | Sections 9–12 |
| Focus, tab, keyboard, narrow layout, live-status wording | Sections 16–18 |
| Empty, stale, corrupt, unavailable visual state | Section 15 |
| Native ownership, strict path-free projection, opaque IDs, copied content | Sections 3–4 and 19–21 |
| Closed classes, class eligibility, source kinds, representations, inert preview | Sections 6–8 and 19 |
| Collection/item/annotation/comparison/reservation lifecycle and mutation restrictions | Sections 5–6, 9–12, and 15 |
| Exact quota warning/refusal and no eviction/partial mutation | Sections 5, 7, and 15 |
| Task/plan context-only relationship, stale/deleted/restore behavior | Sections 3 and 5, plus 15 |
| Pairwise same-format digest-bound native comparison without Git/semantic result | Section 10 and Sections 20–24 |
| Explicit M48-only digest/freshness-bound promotion with no authority transfer | Sections 11–12, 14, and 19 |
| Closed diagnostics, content withholding, storage/migration/concurrent-write failure | Sections 8, 15, and 20–24 |
| Security/privacy exclusions and no provider/network/file/browser authority | Sections 7–8, 19, and 25 |
| Deterministic component/accessibility/browser test design | Sections 20–24 |

## 3. Selected interaction design

**Selected design: Review is a seventh lazy tab in the existing right-side ReviewPanes shell.** It is not an extension of Preview, a second side panel, a floating window, or a file browser.

The shell retains the **Task evidence** heading. M54 adds tab ID **local-review** and label **Review** after Preview and before Activity. Files, Diff, Git, Preview, Activity, and Approval retain their labels, behavior, and authority boundaries. The tab module is lazy; selecting it is the first point that requests a local-review snapshot.

The existing top-bar **Review panes** control is the sole workbench entry point.
M54 retains its current App condition: it appears only in the Codex conversation
workbench route. It opens the same shell; the user selects Review. No new
global toolbar, freeform window, command-palette command, task mutation
control, or automatic opening exists. M54 may extend the existing
presentation-only selectedReviewPane enum with local-review; it must never
store open state, collection ID, item ID, task, plan, content, reservation, or
authority in browser preferences.

Review is available without a currently selected task: it can list existing collections, while **New review collection** opens a read-only eligible-task picker. The picker never changes Task Catalog selection. Opening a collection never selects, edits, restores, or switches an M52 task or plan. If a collection’s task differs from the current Task Catalog task, its immutable captured context is shown and the Review tab remains open.

Background native projection changes may badge Review and send one concise polite status only while the shell is open. They never open the shell, switch tabs, open a dialog, scroll, or move focus.

## 4. Pane reuse and unchanged surfaces

| Existing surface | M54 reuse | Must remain excluded |
| --- | --- | --- |
| **ReviewPanes.tsx** | Lazy aside, semantic tabs/tabpanel, close/focus restoration, 360–560px separator, loading status, tab overflow. | No second shell; no content/authority persistence. |
| **PreviewPane.tsx** | Explicit selection, native snapshot/preview request pattern, inert pre/code rendering, empty/loading/unavailable behavior. | Existing M48 Preview remains M48-only; Review does not make M48 durable. |
| **FilesPane.tsx** | Factual metadata-card and unavailable/truncated presentation pattern. | No path display, generic chooser, arbitrary inspection, or traversal. |
| **DiffPane.tsx** | Bounded loading/failure and line-presentation pattern only. | No git_diff, Git identity, repository path, or Git control. |
| **ActivityPane.tsx** | Bounded ordered-list and empty-state pattern. | No durable audit log, polling, or conversation ownership transfer. |
| **ApprovalPane.tsx** | Read-only non-authority wording and pending-proposal separation. | No decision, dispatch, promotion confirmation, or cancellation control. |
| **TaskCatalog.tsx** | Read-only task/plan label/status, dialogs, list semantics, explicit-delete focus-return pattern. | No task/plan create/edit/select/archive/restore/delete from Review. |
| **layoutPreferences.ts/styles.css** | Width bounds, separator, overlay threshold, scrollable tabs, reduced-motion rules. | No review data or persistent custom layout. |

## 5. Collection navigation

### Collection list

The default Review view is **Local review collections** with **New review collection**. Collections are a labelled nav/ul of buttons, not a tabset. With a maximum of 24 collections, there is no search, filter, paging, grouping control, drag-and-drop, or user reordering.

Native ordering is deterministic: active, frozen, orphaned, unavailable; then descending updatedAtMs; then ascending collectionId. Each row states collection title, state, task title/status or **Task unavailable**, optional plan label plus **selected plan** or **alternate plan** context, plan fresh/stale/unavailable state, ready/total item count, open annotation count, comparison-binding count, payload/count quota summary, and warning badge.

The accessible row name contains those facts. Full UUIDs are not ordinary content. If a technical detail is available, it is explicitly labelled **Opaque local collection identifier**, does not copy automatically, and is not a selection mechanism.

Selecting a row opens detail. First open has no selection. Selection may remain in memory only while the Review tab remains mounted; it never survives close/restart.

### Create, resume, discard, and reference changes

**New review collection** opens a labelled modal using the TaskCatalog dialog pattern: modal semantics, trapped Tab, Escape/Cancel, error summary, Cancel initial focus, and trigger focus restoration. It contains:

1. Collection title with native-authoritative 120-code-point/480-byte feedback.
2. Required **Task context** single-select from native-projected active/paused M52 tasks.
3. Optional **Plan context** single-select with **No plan context**, showing selected versus alternate.
4. Visible statement: **Task and plan context do not approve, run, save, dispatch, or retain review content.**
5. **Create review collection** and **Cancel**.

Creation uses one fixed native operation. It creates no item and does not mutate task/plan state. Success selects the collection, focuses its heading, and says **Review collection created.** Invalid title, missing/ineligible task, plan change, quota, storage, or concurrent-write responses leave the dialog open and focus its error summary. No retry is automatic.

A frozen collection shows **Resume review** only when native projects that the task is active/paused and optional plan is fresh. It is an explicit review action, not Task Catalog restore; success focuses the collection heading. A restored task never reactivates review automatically. Orphaned collections have no resume control and say **The referenced task was deleted. Review data remains local until you discard it.**

**Discard collection…** is destructive confirmation. It says local items, annotations, and comparison bindings will be permanently removed while task, plan, M48 artifacts, project files, Git history, packages, and external material stay unchanged. Confirm is **Discard review collection**. Success clears selection, focuses deterministic next collection or New review collection, and announces discard. Failure focuses the error summary.

### Collection detail

Detail starts with **Back to collections**, title/state, and a read-only context card: task title/status, plan label/status, counts, payload use, quota warning, cleanup eligibility. No task/plan action appears. A stale plan says **Plan context changed after this review collection was created; it was not replaced.**

Frozen means inspection remains available; add-item, comparison creation, and promotion are disabled with **Resume review for a fresh active or paused task before changing review content.** Existing ready annotations follow native ready-item projection. Unavailable withholds content and permits only projection retry where supported and explicit discard. Discarded is never a live detail.

## 6. Item list and cards

**Items** is one creation-time ordered ul inside collection detail, not a canvas, table editor, tree, or file manager. Items are not grouped, filterable, searchable, or reorderable. Native ordering is descending createdAtMs, then ascending itemId.

Each item card is one selectable button followed by an action bar. The selected card has aria-current=true. Visible and accessible metadata is:

- label;
- class badge: **Text**, **Image mockup**, or **Evidence**;
- text format, image MIME/dimensions, or evidence source type;
- human label for closed sourceKind;
- ready, stale, or unavailable state;
- byte size, creation time, shortened digest, and labelled full **SHA-256 digest** technical detail;
- annotation and open-annotation count;
- comparison/promotion eligibility or the precise unavailable reason; and
- a closed warning/diagnostic summary.

Image-mockup and evidence cards always say **Not comparable** and **Not promotion eligible**. Digest mismatch/corruption makes an item unavailable; content, dimensions, and evidence body are withheld.

Action bar controls are **Preview**, **Annotations**, and conditionally **Compare**, **Prepare promotion**, and **Discard item…**. Ineligible actions are disabled with visible reason text rather than hidden. Discard confirmation states it removes only the copied item, annotations, and comparison bindings; success focuses next item or Add review item.

Items use roving tabIndex: Up/Down focuses/selects adjacent item, Home/End focuses/selects first/last, Enter/Space opens preview, and Tab leaves the card for action controls. No undocumented global shortcut is added.

## 7. Item creation entry points

**Add review item** opens one labelled modal with a radio group of four fixed sources. Switching source clears only in-memory draft data; no service is called until a deliberate action. There is no browser file input, drag/drop target, path field, URL field, generic chooser, batch intake, or automatic retry.

| Source | Fields and eligibility | Fixed user action |
| --- | --- | --- |
| **Write text** | Label; plain/Markdown/JSON/CSV/Python format; UTF-8 textarea; code-point/byte counters. | **Add text item** sends one fixed text envelope. JSON/CSV and limits are native-authoritative. No pre-persistence preview; success selects item and focuses Preview heading. |
| **Copy generated artifact** | Path-free native list of still-live M48 manifests with label, class, size, digest, and eligibility. | **Copy into review** sends selected opaque claim. Native copies immediately or returns source-expired/capacity/etc.; no live reference remains. |
| **Add image mockup** | Label and PNG/JPEG-only, static-one-file, no-path-retention explanation. | **Choose PNG or JPEG…** invokes one fixed native user-initiated image-intake operation: one selected file, native validation before persistence, no returned/persisted frontend path, no directory/batch intake, no drag/drop/URL, no automatic retry. |
| **Snapshot evidence** | Label, closed evidence-source selector, path-free source metadata or manual bounded validation summary. | **Add evidence snapshot** sends strict envelope only; no raw approval body, command output, Git object ID, file content, protocol, or remote data. |

Quota warning/reached facts are shown before submit when projected. Local incomplete-field validation may disable submit; native decides all real validation. Failure leaves dialog open, focuses error summary, preserves safe authored draft only in memory, and offers Cancel. Successful creation selects the new item and does not alter task/plan selection.

## 8. Preview presentation

Desktop Review uses vertical master/detail inside the existing 360–560px shell: list above selected detail. Selecting does not scroll the workbench; only the Review tabpanel may bring detail heading into view. At narrow width/short height, detail replaces the list as one level with **Back to items**.

Every preview begins with title, class/format, source kind, byte size, creation time, state, and digest. There is no Copy control or clipboard operation.

| Type | Exact rendering |
| --- | --- |
| Plain, Markdown, JSON, CSV, Python | Native-projected bounded bytes in escaped pre/code, labelled **Plain text**, **Markdown source**, **JSON source**, **CSV source**, or **Python source**. Markdown is not HTML; JSON/CSV are not semantic renderers; Python never executes. Long lines horizontally scroll inside code. |
| PNG/JPEG mockup | Native-produced validated data:image/png or data:image/jpeg only; fit-to-width with intrinsic dimensions. No zoom, pan, canvas, coordinate pin, remote source, SVG, animation, iframe, object, or embed. Alt: **Static mockup: {label}; {MIME}; {width} by {height} pixels.** |
| Evidence | Escaped labelled envelope fields and normalized summary in a definition-list/preformatted region, headed **Copied evidence snapshot — not a live source or approval.** |

Ready and stale copied content may be viewed when native projects it; stale adds warning and disables comparison/promotion. Unavailable/corrupt/digest-mismatched content is not rendered: show closed reason and **Content withheld for safety**, with only allowed retry/discard. Loading is role=status; empty detail says **Select a review item to inspect its safe local preview.**

## 9. Annotation interaction

Annotations appear only beneath selected-item preview under **Annotations (n)**; they are never overlays or comparison comments. Open annotations appear first; resolved annotations use the same creation-time/annotation-ID order and are collapsed by default behind **Show resolved annotations**.

For active collection plus ready item, **Add annotation** reveals labelled plain textarea with 1,024-code-point/1KiB counters, **Save annotation**, and **Cancel**. Counter threshold/reached change is polite; it does not announce every keystroke. Success focuses new annotation heading and announces **Annotation added.** Cancel restores Add annotation.

Each entry says **Local annotation** and open/resolved state with no author identity. Its action menu offers Edit, Resolve or Reopen, and **Delete annotation…**. Reopen is permitted only for active collection plus ready item. Edit uses the same labelled editor. Resolve/reopen focuses entry heading and announces. Delete uses inline Cancel/Remove confirmation, Cancel initial focus, and focuses next annotation, group heading, or Add annotation after success.

In frozen collection, M54 follows native mutation projection and never adds a bypass. For stale/unavailable item, annotations remain visible; create/edit/resolve/reopen are unavailable, while explicit deletion remains available exactly as M53-B permits. Promotion changes no annotation. No annotation creates a durable Activity event, notification, mention, range, offset, coordinate, reaction, collaborator, or remote identity.

## 10. Comparison interaction

**Compare** is enabled only on selected ready text in active fresh collection when native projects eligibility. It opens **Compare text items** modal. Current item is fixed **Left side**. **Right side** is required radio list of distinct ready items with same textFormat. Current item is excluded; images/evidence, stale/unavailable, differing format, and over-limit text are omitted. Both sides show label, format, shortened digest, byte/line status.

**Create comparison** creates only digest-bound binding. Success opens **Comparison** subview in Review; it never opens Git Diff. Up to eight bindings appear under **Comparisons**, newest first then ID. Opening binding asks native to recompute; no result is stored. **Remove comparison…** is inline Cancel/Remove confirmation and removes only binding.

Comparison has Back to item, accessible left/right title/format/digests, and one interleaved line list:

- **Unchanged** collapsed runs;
- **Removed from left** with left line number/text;
- **Added on right** with right line number/text;
- **Changed** with both line numbers and labelled left/right code blocks.

Color never conveys change alone. Long lines scroll inside code. JSON/CSV are ordinary lines, never semantic diff. If side changed/missing/corrupt/stale/unavailable or exceeds 128KiB/2,000 lines, show **Comparison unavailable** closed reason; no stale partial result. **Recompute comparison** is enabled only when native can recompute. It never mutates either item. Cancel/failure restores Compare; Back restores selected item Compare.

## 11. Promotion preparation

Only selected ready text in active collection with fresh task/optional-plan context exposes **Prepare promotion**. Other cards state precise reason: unsupported class, not ready, frozen, task/plan stale/unavailable, digest mismatch, source too large, or M48 capacity/reservation limit.

Prepare invokes one fixed native preparation request. Control becomes **Preparing…** until response. It does not save, dispatch, approve, execute, or create M48 artifact. Success opens Section 12 modal. Refusal restores focus to Prepare promotion, presents path-free alert, and allows only deliberate retry after user changes condition.

## 12. Promotion confirmation

The prepared-reservation modal title is **Create a transient generated artifact?** It is modal, Escape/Cancel, trapped Tab, Cancel initial focus, no backdrop confirmation. It shows item label/format, full labelled digest, immutable task/plan label/status, M48 destination class, **Prepared**, and **Expires at {localized time} (within five minutes)**.

Required wording:

> Confirming copies this digest-bound reviewed text into M48 as one transient generated artifact. It does not save a file, approve or dispatch anything, run code, modify Git, access files, publish, or deploy.

The affirmative control is exactly **Create transient generated artifact**. It is never Save, Approve, Run, Apply, Dispatch, Publish, or Deploy. **Cancel promotion** invalidates reservation and restores Prepare focus. Expiry, restart loss, stale digest, freshness failure, duplicate, M48 capacity, or native refusal invalidates/consumes candidate as M53-B requires, focuses source item heading, reports closed error, and creates no partial M48 artifact.

Success says **Transient generated artifact created; no file was saved and no approval or execution occurred.** It offers explicit **View generated artifact in Preview**; only user activation selects existing Preview tab. Default focus remains success heading. Source item, annotations, comparisons, task, and plan do not change. Duplicate promotion requires fresh prepare.

## 13. Activity behavior

M54 adds bounded **current-session local-review activity projection** to existing Activity pane. It is not SQLite data, review audit log, task/activity retention relationship, notification system, or collaboration history. It is in-memory reducer over successful M54 command receipts and lifecycle transitions observed in this session, maximum newest 12, discarded at app end.

It may show collection created/frozen/resumed/orphaned/discarded; item added/discarded; annotation added/resolved/reopened/deleted; comparison created/discarded; promotion prepared/canceled/expired/succeeded/failed. Lifecycle transitions are derived only on new native projection, not invented history. Failure uses closed reason only.

Rows show local timestamp, action label, collection/item label, and short digest only when needed; full digest has accessible technical label. No path, content, command output, approval body, UUID, credential, provider, or remote identity. Empty: **No local review activity in this session.** Unavailable: **Local review activity is unavailable.** New entry may badge Activity and politely announce only if shell open; never steals focus or switches pane.

## 14. Approval-pane behavior

Approval remains presentation of current execution-approval proposal and retains **This review pane cannot decide or dispatch the proposal.** M54 adds static subsection **Local review is not an approval**:

> Review readiness and a prepared promotion reservation do not approve, dispatch, save, execute, modify Git, publish, or deploy anything. Confirm a review promotion only in the Review tab.

When selected review item has prepared reservation, subsection may show **Prepared local review promotion — expires at {time}** and item label/digest, but no Confirm, Cancel, Save, or approval button. Pending execution approval and prepared review promotion can coexist in separate headings; neither changes the other. Expiry is polite status only.

## 15. Complete state presentation

| State | Title/explanation | Permitted controls, focus, live region |
| --- | --- | --- |
| Loading | **Loading local review** — requesting bounded native projection. | No content action until result; role=status; no focus move. |
| Empty | **No review collections/items** — no bounded local record exists. | New collection/Add item only when context permits; focus remains invoking control. |
| Active/ready | State badge; native validation passed. | Normal class-specific controls; no automatic announcement. |
| Warning/quota warning | **Near local review capacity** plus exact count/bytes/threshold. | Actions remain until refusal; no auto-cleanup; polite once per projection change. |
| Quota reached | **Local review capacity reached** plus closed limit. | Affected creation disabled; discard remains; refusal focuses error summary. |
| Frozen | **Review frozen** — task completed/archived. | Inspect; Resume only when native permits; no add/compare/promote. |
| Orphaned | **Task reference deleted** — copied data retained for disposition. | Inspect/discard only; no substitute, resume, add, compare, promote. |
| Stale | **Review context changed** — binding not fresh. | View projected content; comparison/promotion/stale mutation disabled except M53-B-permitted annotation deletion. |
| Unavailable | **Local review unavailable** plus closed reason. | Withhold content; supported retry and discard only; no automatic retry. |
| Corrupt/digest mismatch | **Integrity check failed** — content withheld for safety. | Discard and optional projection retry only; role=alert on transition; no repair/replacement. |
| Discarded | Not live content. | Deterministic next row/heading and polite discard status. |
| Missing task | **Referenced task is unavailable.** | Native chooses unavailable/orphaned; no search/restore/substitute control. |
| Missing/changed plan | **Plan context is unavailable/changed; it was not replaced.** | Inspect; resume/promotion blocked until fresh; no auto-bind. |
| Invalid source/unsupported format | **This source cannot become a local review item.** | Creation dialog error summary; no fallback conversion/picker. |
| Preview unavailable | **Safe preview unavailable.** | Metadata/discard only; no browser-renderer fallback. |
| Comparison unavailable/limit | **Comparison unavailable** plus side/limit reason. | Back/remove; recompute only when both sides ready; no partial result. |
| Promotion ineligible | **Promotion unavailable** plus eligibility reason. | Explain disabled Prepare; no reservation. |
| Promotion prepared | **Promotion prepared** and expiry time. | Confirm/Cancel only in modal; no Activity/Approval decision. |
| Promotion consumed | **Promotion completed or no longer available.** | The reservation is not shown as active; success/failure destination messaging replaces it. A new promotion requires fresh preparation. |
| Promotion expired/failed | **Promotion not created** and closed reason. | Source unchanged; focus source heading; deliberate re-prepare only. |
| Storage unavailable/migration unsupported | **Local review storage unavailable** / **Local review version unsupported**. | Closed diagnostic; retry/discard only as native permits; no frontend migration. |
| Concurrent stale write | **Review changed before this update completed.** | Preserve safe draft in memory; focus error summary; Reload/Cancel, no merge/overwrite/retry. |

Warnings are role=status/aria-live=polite. Rejected user operation and integrity content withholding use labelled role=alert summary. Never announce raw content, path, source file, approval body, or provider detail.

## 16. Keyboard and focus contract

- **Open/close:** Review panes button opens; existing tab buttons use ordinary order. Closing unmounts shell and restores opening control through ReviewPanes.
- **Resize:** desktop separator retains Left/Right 20px changes and 360–560px bound; hidden in overlay.
- **Collections/items:** collection rows ordinary Tab buttons; item cards use Section 6 roving arrows/Home/End; Enter/Space selects/opens without mutation.
- **Detail:** Back to collections/items restores exact selected row/card. No global P/A/C shortcut.
- **Annotation:** Add focuses textarea; Escape cancels unsaved editor and restores Add/Edit; Ctrl/Command+Enter is not Save.
- **Comparison:** Compare focuses right-side list; Escape restores Compare; Back returns Compare.
- **Promotion:** modal initial focus Cancel; Escape cancels; Enter only acts on focused labelled button.
- **Destructive actions:** traps Tab, Escape cancels, Cancel initial focus, restores trigger/next deterministic list target.
- **Background updates:** retain current focus. If focused row disappears, focus nearest next sibling, previous sibling, then list heading; announce concise status. Never auto-focus tab/dialog/alert/control.

## 17. Responsive and narrow-layout behavior

Desktop stays inside existing fixed 360–560px right overlay. Collection/item/detail is vertical and scrolls only in tabpanel. Long labels wrap; technical digest/code/comparison lines may horizontal-scroll locally; images fit width.

At width <=760px or height <=520px, existing overlay applies: bounded inset/full width, no resize separator, horizontally scrollable tabs. Review becomes single-level stack:

**Collections → Collection → Items → Preview/Annotations/Comparison**

Each deeper level has visible Back and remembers only mounted transient parent selection. Keyboard/short height keep dialog heading/actions reachable using existing scrollable dialog panel. Promotion modal stacks actions if constrained. No narrow state persists. Closing returns top-bar trigger; Back returns prior selected row/card.

## 18. Accessibility contract

- Existing aside/heading/tablist/tab/tabpanel remain; Review tab name is **Review**, panel name **Review review pane**.
- Collections/items use labelled nav/ul; selected state and badges have words, never color-only meaning.
- Dialogs have aria-modal, labelled title, error summary, Cancel initial focus, Escape, contained Tab order.
- Annotation labels/state/counters are explicit; thresholds announce politely, not every keypress.
- Code regions name format/truncation. Image alt is exact Section 8 text; unavailable image renders no empty img.
- Comparison names both sides/digests and expresses addition/deletion/change in text, not color alone.
- Promotion confirmation names destination/non-authority result. Destructive dialogs name scope/irreversibility.
- Statuses polite; rejected/integrity states alerts. Test focus after close/cancel/delete/failure/expiry/background removal.
- Review adds no essential animation; existing reduced-motion behavior applies. Existing contrast/touch targets apply. At 200% zoom/reflow, only tabs/digests/code/comparison-line regions may horizontal-scroll.

## 19. Security and authority language

Review is copied local content, not a live file. Task/plan context is not authority. Evidence is not approval. Comparison is not Git diff. Promotion is transient M48 copy, not Save or approval.

No Review control opens path/directory, changes task/plan, calls provider, loads URL, uses browser file input, drag/drop, clipboard automation, shell/terminal, Git, dispatch, approval, execution, filesystem Save, publish, or deploy. Native owns validation, freshness, capacity, integrity, reservation, persistence, and diagnostics; frontend renders strict path-free projections and fixed envelopes.

## 20. Native-contract test requirements

Native M54 tests must cover:

- collection list/detail active/frozen/orphaned/unavailable/discarded, immutable task/plan context, restore/resume;
- every text format, image MIME/validation bound, evidence envelope, source kind, label, digest, quota warning/reached, stale/corrupt/content-withholding result;
- inert preview response plus limit/unavailable/digest mismatch;
- annotation create/edit/resolve/reopen/delete, limits, ready/stale/unavailable restrictions, and no anchors/coordinates/authors;
- comparison create/discard/projection, distinct same-format sides, digest binding, deterministic line result, limit/stale/missing/corrupt side, no persisted result/Git;
- promotion prepare/confirm/expiry/restart loss/duplicate/M48 capacity/task-plan freshness/digest/cancel/consumed and no partial promotion;
- unknown-field/malformed ID/storage unavailable/migration unsupported/stale-write rollback and no path/authority fields.

## 21. Frontend-schema tests

Frontend schemas must reject unknown fields and invalid enum/ID/digest shape for every request/response. Fixtures cover every lifecycle/class/format/source, quota, missing/stale task/plan, unavailable/corrupt diagnostic, prepared/consumed/expired reservation. They assert no path, URL, file chooser value, approval decision, command, provider/account field, raw diagnostic content, or browser-layout authority is accepted.

## 22. Component tests

Component tests use deterministic fixtures and cover:

- lazy Review loading, seventh-tab order, no request before Review tab selection, open/close focus;
- collection list/order/create/resume/discard and no Task Catalog selection mutation;
- item card fields/order/selection/ineligible reason/focus after creation/discard;
- inert all-class preview, truncation, stale/unavailable/corrupt withholding, no HTML/live link/remote image;
- annotation workflow/counter/frozen-stale behavior/delete confirmation/focus;
- comparison filtering/identical-side prevention/line semantics/stale-side/removal/no Git-pane call;
- two-step promotion/exact warning/cancel/expiry/failure/success/Preview handoff/no Approval control/no auto retry;
- bounded session Activity, Approval non-authority, every Section 15 state, background badge/no focus theft;
- desktop/narrow transition and no layout-preference collection/item/content/authority write.

## 23. Accessibility tests

Tests assert semantic seventh tab/panel, list labels, selected state, worded badges, dialog modal/name/trap, keyboard-only operation, focus after close/delete/failure/expiry, status/alert behavior, image alt, comparison text semantics, reduced motion, color-independent state, 200% zoom, and narrow touch/reflow reachability.

## 24. Browser checks

Browser checks use sanitized deterministic fixtures and capture DOM/accessibility output plus screenshots free of user content, paths, credentials, provider IDs, approval bodies, and command output. They cover:

- desktop 360px/480px/560px, overlay <=760px, short height <=520px, scroll/focus/Back;
- keyboard traversal, dialogs, desktop resize, no overlay resize;
- annotation create/resolve/delete and stale delete-only behavior;
- comparison creation/filtering/stale-side/limit/removal;
- promotion prepare/confirm/cancel/expiry/task freeze/plan stale/M48 capacity with visible no-save/no-approval/no-execution wording;
- orphaning, unavailable/corrupt withholding, quota, stale write, discard;
- no browser input/drag-drop/external request/URL/polling/provider/Git/approval decision/focus theft;
- Activity/Approval coexistence with no review-promotion decision control.

## 25. Explicit UI non-goals

Review excludes Figma-like canvas, freeform editor, rich text, image-coordinate annotation, active Markdown, live links/web browsing/external previews, arbitrary file manager, drag/drop/browser file input, multi-file/bulk intake, bulk/automatic promotion, automatic approval/dispatch, execution/Git/shell/terminal control, publishing/deployment, collaboration/remote users/comments/mentions/notifications/cloud sync, persistent custom layout, detachable windows, plugin panes, PDF/SVG/audio/video/animation/archive support, semantic JSON/CSV comparison, persisted comparison results, clipboard automation, task/plan mutation, and network/provider authority.

## 26. Rejected interaction alternatives

- Extending M48 Preview with durable review would blur transient M48 versus durable review ownership.
- Reusing Diff would imply Git/path comparison and violate M53-B.
- Second dock/modal workbench/detachable canvas would bypass M49/M50 focus/overlay limits.
- Auto-selected Task Catalog collection would make task selection look like authority.
- Always side-by-side list/preview fails bounded 360px shell; vertical master/detail is reachable.
- Pins/ranges/line comments/media markup/collaborators require unsupported anchor/identity/retention.
- One-click Promote, Save wording, or Approval-pane confirmation confuses M48 copy with Save/approval.
- Durable review Activity creates unauthorized audit retention.

## 27. Questions intentionally deferred to M53-D

M53-D may assemble M53-A/B/C into final proposal, integrate documentation/navigation, formulate final numbered M54 acceptance criteria, record closure evidence, and present the approval request. It must not reopen selected M53-B core or this interaction design without a new bounded decision. M54 cannot begin from this document alone.

## 28. Implementation boundary

This phase created documentation only. No M54 runtime implementation, schema/migration, frontend schema, style, component, test, package/version, package evidence, packaging, installation, release, publication, deployment, provider, connector, browser automation, hidden network, filesystem, Git, approval, dispatch, or execution change occurred.
