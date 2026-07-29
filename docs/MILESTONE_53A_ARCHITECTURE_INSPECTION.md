# Milestone 53-A — Local Artifact and Design Review Architecture Inspection

Status: complete architecture-inspection phase. This is the authoritative input to M53-B. It records existing repository facts and unresolved proposal questions only; it selects no M53 design and implements no M54 behavior.

## Phase scope and exclusions

M53-A inspected the established M48 generated-artifact, M49 review-pane, M50 layout, and M52 durable-task foundations for the roadmap's proposed Local Artifact and Design Review decision gate. Its scope is limited to repository verification, factual architecture mapping, reuse classification, and a closed question register for later M53 phases.

This phase does not select review-item classes, a storage model, a pane, an interaction model, annotation semantics, comparison semantics, promotion semantics, or M54 acceptance criteria. It makes no runtime, schema, package, package-evidence, release-manifest, provider, connector, integration, network, filesystem, shell, Git, dispatch, approval, or execution change. It neither packages nor installs QuireForge.

## Verified repository baseline

The required pre-change `git fetch origin --prune` found local `main` and `origin/main` equal at `8fce12607eeec0a41a46c4bb50620b8540e34c6d`:

| Field | Verified value |
| --- | --- |
| Current branch | `main` |
| Local `main` SHA and subject | `8fce12607eeec0a41a46c4bb50620b8540e34c6d` — `docs: record M52 package evidence` |
| `origin/main` SHA and subject | `8fce12607eeec0a41a46c4bb50620b8540e34c6d` — `docs: record M52 package evidence` |
| Ahead / behind | `0 / 0` |
| Starting staged, unstaged, and untracked files | none |

The start process inspection used `ps -eo pid,ppid,stat,etime,args` filtered for QuireForge, Codex, development-server, build, test, package, Git, and watcher terms. It found no QuireForge process from this checkout and no repository-mutating/locking process, development server, watcher, test, build, package, or Git operation. Unrelated Codex/Desktop processes were outside this checkout and were not disturbed.

## Verified package identity

The authoritative source declarations agree on application version `0.1.0-beta.46`: root [package.json](../package.json), [apps/desktop/package.json](../apps/desktop/package.json), [apps/desktop/src-tauri/Cargo.toml](../apps/desktop/src-tauri/Cargo.toml), [apps/sandboxd/Cargo.toml](../apps/sandboxd/Cargo.toml), and the desktop metainfo release entry. The package-contract test also names that source version in [scripts/tests/test_package_contract.py](../scripts/tests/test_package_contract.py).

The directly inspected existing local release manifest at `target/ubuntu-22.04/release/packages/release-manifest.json` reports version `0.1.0-beta.46`, Debian package version `0.1.0~beta.46` for both Debian artifacts, and clean source commit `6df055999d2ad01d2385096a14bc71f8aada2a8c`. The tracked [M52 report](MILESTONE_52_DURABLE_TASK_RECORDS.md) independently records the same beta.46 application and Debian identity, artifact filenames, checksums, and evidence. No version declaration, manifest, package, or package evidence was modified or regenerated during M53-A.

## Governing roadmap and architecture

[docs/ROADMAP.md](ROADMAP.md) defines M53 as a decision-only gate over approved task briefs, plans, mockups, safe previews, validation evidence, and generated artifacts. It requires specification of comparison, selection, annotations, review state, and user-approved promotion into a QuireForge task, while excluding repository scraping, automatic execution, publishing, deployment, and direct third-party connectors including Figma. M54 may begin only after M53 approval, must reuse M48 typed artifacts and M49 preview/review services, and has provisional package candidate `0.1.0-beta.47`.

[docs/CURRENT_STATE.md](CURRENT_STATE.md) marks M48, M49, M50, and M52 complete and identifies M52 as beta.46. [docs/ARCHITECTURE.md](ARCHITECTURE.md) records the M48 process-local service, M49 presentation-only lazy shell, M50 strict browser-local preference record, and M52 migration-11 task catalogue. [docs/THREAT-MODEL.md](THREAT-MODEL.md) separately records their bounded artifact, read-only pane, layout-preference, and task-record boundaries.

Proposal-milestone closure is therefore documentation and explicit later approval, not a package change. M53-B and M53-C must preserve existing terms such as generated artifact, safe file preview, task evidence, pending approval, task record, task plan, selected plan, typed command, unavailable, and transient rather than inventing parallel authority-bearing concepts.

## M48 generated-artifact architecture map

### Existing contract

Native module [apps/desktop/src-tauri/src/advisor_generated_artifact.rs](../apps/desktop/src-tauri/src/advisor_generated_artifact.rs) owns `AdvisorGeneratedArtifactService`, `GeneratedArtifactManifestV1`, `GeneratedArtifactSnapshotV1`, `GeneratedArtifactPreviewV1`, `GeneratedArtifactSaveReceiptV1`, and the closed classes `text`, `markdown`, `json`, `csv`, and `python`. It accepts only `visible-completed-reply` text or `visible-fenced-block` content, normalizes and validates UTF-8 data, validates JSON/CSV where applicable, uses UUID artifact IDs and SHA-256 manifest/content claims, and restricts labels and suggested filenames.

The service is process-local memory, not SQLite. It holds at most five items, 512 KiB per item, 2 MiB aggregate, and removes expired entries after a 15-minute native monotonic lifetime. Manifest state is `ready`, `saving`, `expired`, or `saved`; disposal is `transient-memory-one-successful-save`. It exposes neither activity records nor approval/task/plan associations. It retains no path, project/worktree, attachment, Codex, provider, connector, browser, dispatch, terminal, Git, or persistent content state.

The fixed commands in [apps/desktop/src-tauri/src/lib.rs](../apps/desktop/src-tauri/src/lib.rs) are `advisor_generated_artifact_create`, `advisor_generated_artifact_snapshot`, `advisor_generated_artifact_preview`, `advisor_generated_artifact_discard`, and `advisor_generated_artifact_save`. The frontend Zod mirror in [apps/desktop/src/lib/advisorGeneratedArtifact.ts](../apps/desktop/src/lib/advisorGeneratedArtifact.ts) strictly validates all request/response fields; `bridge.ts` names only those commands. [apps/desktop/src/AdvisorWorkspace.tsx](../apps/desktop/src/AdvisorWorkspace.tsx) creates cards from visible completed reply/fenced-block candidates and consumes the same typed service for preview, discard, and explicit save.

Save is a separate native authority: it reserves one `(artifactId, sha256)` claim, opens one native Save dialog, writes a private same-directory `0600` temporary file, synchronizes, atomically publishes with `renameat2(RENAME_NOREPLACE)`, synchronizes the parent directory, returns a path-free receipt, and consumes only the successful artifact. It cannot overwrite, open, run, import, or project-write automatically.

### Reuse and constraints

M53/M54 can directly reuse M48 manifest identity, SHA-256 integrity claims, bounded text preview, visible provenance (`sourceKind`), strict bridge schemas, failure diagnostics, lifecycle expiry, and explicit-save non-overwrite rules. It must not redefine the registry as persistent, path-bearing, arbitrary-file, binary/image, task-owned, approval-owned, or automatic-promotion storage. Whether a later review record may reference a still-live M48 manifest without extending its retention is an M53-B question.

Native tests in that module cover closed classes, invalid content, capacity, aggregate limits, manifest mismatch, expiry, concurrent saving, no-overwrite publication, and cleanup. Frontend contract tests are in [apps/desktop/src/lib/advisorGeneratedArtifact.test.ts](../apps/desktop/src/lib/advisorGeneratedArtifact.test.ts), and UI behavior is in [apps/desktop/src/AdvisorWorkspace.test.tsx](../apps/desktop/src/AdvisorWorkspace.test.tsx).

## M49 pane reuse and constraint map

`ReviewPanes` in [apps/desktop/src/ReviewPanes.tsx](../apps/desktop/src/ReviewPanes.tsx) is a closed-by-default, lazy `<aside>` named **Task evidence**. It owns only open-shell focus restoration, selected pane forwarding, and resize interaction; the parent owns open state and supplies all snapshots/loaders. Its six lazy modules and `ReviewPaneId` union are fixed in [apps/desktop/src/review-panes/types.ts](../apps/desktop/src/review-panes/types.ts). Closing unmounts the active pane and restores focus to the opening control.

| Pane | Current source and rendering | State / accessibility / authority | M53 reuse finding |
| --- | --- | --- | --- |
| Files | `FilePreviewSnapshot`; [FilesPane.tsx](../apps/desktop/src/review-panes/FilesPane.tsx) shows selected safe-file metadata only. | Parent-owned snapshot; explicit empty/unavailable/truncated statuses; no picker, opener, read, or write. | Direct for a factual metadata card only; unsuitable as arbitrary local-file intake. |
| Diff | `git_status` then one `git_diff`; [DiffPane.tsx](../apps/desktop/src/review-panes/DiffPane.tsx). | Local loading/ready/failure state; fetches only after tab selection; first bounded changed file; unavailable/truncated state. | Informative bounded comparison presentation; unsuitable as generic review-item comparison or Git mutation path. |
| Git | `git_status`; [GitPane.tsx](../apps/desktop/src/review-panes/GitPane.tsx). | Local loading/ready/failure state; explicit selection; clean/unavailable/truncated outcomes; no mutation. | Informative repository evidence only. |
| Preview | M48 snapshot then explicit M48 preview; [PreviewPane.tsx](../apps/desktop/src/review-panes/PreviewPane.tsx). | Local list/preview selection; loading, unavailable, and empty states; `<pre><code>` inert text. | Direct for M48 text artifact selection and bounded preview; needs bounded extension for any new supported review representation. |
| Activity | normalized `ConversationEvent[]`; [ActivityPane.tsx](../apps/desktop/src/review-panes/ActivityPane.tsx). | Parent-owned bounded current conversation activity; empty status; no durable event store. | Informative presentation only; task/review history retention is not established. |
| Approval | `ConversationSnapshot.pendingApproval`; [ApprovalPane.tsx](../apps/desktop/src/review-panes/ApprovalPane.tsx). | Parent-owned ephemeral pending proposal; no decision control; explicit status says it cannot decide/dispatch. | Direct read-only presentation pattern; unsuitable for review approval, promotion, or approval bypass. |

The shell uses semantic tablist/tabs/tabpanel, labelled close and resize controls, ordinary keyboard flow, short `role=status` feedback, and scrollable tab overflow. `ReviewPanes.test.tsx` verifies deferred calls until pane selection, all six labelled tabs, resize bounds/listener cleanup, and focus behavior. The M49 report records that background events may badge but never steal focus; there is no polling, subscription, timer, durable record, or new native command.

## M50 layout constraints

[apps/desktop/src/layoutPreferences.ts](../apps/desktop/src/layoutPreferences.ts) owns the only M50 preference: browser-local `quireforge-workbench-layout`. It accepts exactly schema version 1, `reviewPaneWidth` (360–560), `terminalDockHeight` (220–560), and one existing `selectedReviewPane`; it is capped at 512 bytes and defaults on malformed, unknown, oversized, or invalid data. It cannot store a task, path, content, artifact, open state, approval, terminal content, Git, credential, provider, or account value.

`ReviewPanes.tsx` presents the desktop review shell as a fixed right overlay with a 20-pixel left/right keyboard separator. The managed terminal dock in [apps/desktop/src/App.tsx](../apps/desktop/src/App.tsx) is separately optional, collapsed at startup, unmounted when closed, and has a 20-pixel up/down keyboard separator. [apps/desktop/src/styles.css](../apps/desktop/src/styles.css) uses a full-width bounded-inset review overlay at width `<=760px` or height `<=520px`, hides its resize control there, keeps tab overflow scrollable, and caps the terminal dock at `42vh`. Reduced-motion styling removes layout smooth scrolling. Layout preference parsing is tested in [apps/desktop/src/layoutPreferences.test.ts](../apps/desktop/src/layoutPreferences.test.ts).

M53-C is constrained to existing surface placement and must not assume freeform tiling, a persistent open review surface, offscreen interactive content, or a layout preference that encodes review data/authority. It must preserve focus restore, separator semantics, keyboard traversal, narrow behavior, and unmounting of closed secondary surfaces.

## M52 durable-task and alternate-plan constraints

M52 adds migration 11 in [apps/desktop/src-tauri/src/project/storage.rs](../apps/desktop/src-tauri/src/project/storage.rs): `task_records` and `task_plans` in the existing private mode-0600 `metadata.sqlite3`; it does not create a second database. Types in [project/types.rs](../apps/desktop/src-tauri/src/project/types.rs) define schema version 1, opaque UUIDv7 IDs, `TaskStatus` (`active`, `paused`, `completed`), `TaskRecordSummary`, `TaskPlanSummary`, `TaskCatalogSnapshot`, and closed task diagnostics. `ProjectService` in [project/mod.rs](../apps/desktop/src-tauri/src/project/mod.rs) owns validation, mutex/transaction access, snapshot construction, capacity outcomes, and unavailable/corrupt handling.

The task catalogue stores only normalized title, status/timestamps, archive/last-opened timestamp, selected-plan identity, and bounded plan label, position, and visible body. Limits are 200 tasks, four plans per task, 48 KiB per task, 8 MiB aggregate, with warnings at 160 tasks/6 MiB; cleanup is only an explicit deterministic 180-day eligibility indication. Task lifecycle has explicit archive/restore and irreversible row deletion; no eviction, automatic cleanup, partial mutation, or retained external content occurs. Invalid stored rows are omitted with a closed diagnostic, and errors resolve to unavailable or specific non-authoritative diagnostics.

The fixed IPC commands in `lib.rs` are `task_catalog_status`, `task_catalog_create`, `task_catalog_rename`, `task_catalog_status_set`, `task_catalog_archive`, `task_catalog_restore`, `task_catalog_delete`, `task_plan_create`, `task_plan_select`, `task_plan_edit`, and `task_plan_delete`. Their strict frontend schemas and closed snapshot projection are in [apps/desktop/src/lib/taskRecords.ts](../apps/desktop/src/lib/taskRecords.ts), and fixed bridge command envelopes are tested in [apps/desktop/src/lib/taskBridge.test.ts](../apps/desktop/src/lib/taskBridge.test.ts).

Plan selection clears the transient conversation-attachment selection first; if clearing fails, selection does not occur. It does not start/restore/clone a conversation, send a model request, dispatch, approve, execute, retrieve, save, access Git/worktrees, alter terminal state, or transfer artifacts. M52 stores no review item, attachment/artifact, approval, activity, evidence, project, conversation, session, transcript, path, credential, provider, connector, browser, prompt, log, or Advisor state. Its relationships are therefore currently **none**, not implied references.

M53-B must determine whether and how a later review item can reference a task or selected alternate plan without importing authority or silently making a task/plan a retention owner. It must state behavior if that task/plan is archived, completed, replaced, unavailable, stale, or deleted. Existing M52 facts do not support a task-independent durable review item, task/plan activity, approval linkage, or task/plan-controlled promotion.

## Adjacent reusable-service inventory

| Classification | Existing service / files | Reuse limit |
| --- | --- | --- |
| Directly reusable | M48 manifests, SHA-256 claim checks, bounded inert text previews, expiry and save receipts. | Only M48's transient closed text classes and explicit save rules. |
| Directly reusable | M49 `ReviewPanes`, `PreviewPane`, `FilesPane`, `ActivityPane`, `ApprovalPane`, semantic statuses and lazy loading. | Presentation only; no new authority follows from reuse. |
| Directly reusable | M50 layout parser/bounds and responsive CSS. | Presentation values only; do not add review data to localStorage. |
| Directly reusable | M52 opaque UUIDv7 validation, strict Zod/Rust command envelopes, private SQLite migration/transaction/capacity patterns. | A future bounded extension requires an approved schema/lifecycle contract; task tables themselves cannot absorb arbitrary review content. |
| Reusable with bounded extension | `FilePreviewService` and [preview/types.rs](../apps/desktop/src-tauri/src/preview/types.rs), plus [apps/desktop/src/lib/filePreview.ts](../apps/desktop/src/lib/filePreview.ts). | Its existing safe, attached-project-only text/PNG/JPEG/PDF metadata representations and refusal states inform safe review previews; M53 must not turn it into arbitrary traversal, active document rendering, or generic picker access. |
| Reusable with bounded extension | Normalized `ConversationEvent`/`ConversationApproval` source models in `codex/conversation/types.rs` and `lib/conversation.ts`. | Current activity/approval is ephemeral conversation state. A review audit/annotation history needs separate retention and authority decisions. |
| Informative but unsuitable | Read-only Git status/diff (`git/repository_state.rs`, `lib/git.ts`, Diff/Git panes). | Bounded attached-project evidence is not a general comparison engine and cannot authorize Git mutation. |
| Informative but unsuitable | M48 native save and M10B reviewed Git mutation mechanisms. | Neither supplies automatic review promotion or a generic filesystem/Git write capability. |
| Prohibited | Generic path/file APIs, browser file inputs, arbitrary shell/terminal/Git commands, raw Codex protocol, provider/connector/browser automation, network fetching, Figma or other third-party connectors. | These would expand authority beyond M53/M54 roadmap and ADR boundaries. |

No existing annotation/comment type, review-item schema, image/mockup record, comparison model, review-state lifecycle, or promotion record was found. This is evidence for an explicit M53-B decision, not authorization to add one in M53-A.

## Native/frontend ownership, storage, and lifecycle map

| Domain | Native ownership | Frontend ownership | Storage / lifecycle |
| --- | --- | --- | --- |
| M48 generated artifacts | `AdvisorGeneratedArtifactService`, UUID/digest/validation/expiry/reservation/save. | Strict schemas, visible cards, explicit preview/save/discard interactions. | Process memory only; five items, 15 minutes, discarded/consumed after save; no persistence. |
| M49 task evidence panes | Existing typed status/diff/preview calls and normalized conversation state. | Lazy shell/panes, selection, loading/failure/empty rendering, focus. | View state transient; no pane persistence, polling, or subscription. |
| M50 layout | none. | Bounded parsing/writing of presentation preference. | Browser-local four-field JSON only; no open state/data/authority. |
| M52 task plans | `ProjectService`/`ProjectRepository`, SQLite migration, IDs, validation, transactions, lifecycle. | Strict request/response schemas and accessible controls. | Private `metadata.sqlite3`, bounded durable organizational data only. |
| Safe project preview | `FilePreviewService`, attached-root revalidation, containment and format validation. | Strict snapshot display; no frontend path. | Transient UI/process state; one-use opaque handoff. |

This separation means a future review feature cannot let the frontend select arbitrary representations, paths, persistence locations, approval decisions, or execution targets. Any new durable review record would need native-owned identity, schema, normalization, capacity, migration, failure, lifecycle, and deletion behavior; any new presentation remains a strict frontend projection.

## Approval and authority boundaries

ADR 0011 confines approval to one native-owned active conversation request, normalized pending presentation, and the closed `approve`/`decline`/`cancel` decision command. Its activity and pending approval are ephemeral; they are not task or artifact records. M49's Approval pane intentionally displays the existing proposal but cannot decide or dispatch it. No review observation, selection, annotation, comparison, or stale-state acknowledgement can be treated as a conversation approval.

ADR 0012 confines Git review to fixed attached-project status/diff/editor handoff commands, native revalidation, bounded text, and no persistence. ADR 0013 uses a separate preview/confirmation/recovery gate for selected Git mutations. M53 review must not route around either boundary or treat artifact promotion as Git staging/commit/push authority.

ADR 0021 confines safe preview to a native-selected file under a revalidated attached root. It permits normalized text, bounded PNG/JPEG data URLs, and PDF metadata only; HTML/SVG are inert text and active/unknown/binary content is refused. Its picker/open handoff is not a review-item import API.

## Prohibited authority expansion

The following boundary map is mandatory for M53 and M54:

| Authority | What may be displayed/reviewed | What may not be introduced |
| --- | --- | --- |
| Display/review | Existing bounded typed snapshots, M48 text, existing safe previews, current normalized activity/pending approval. | Arbitrary filesystem reads, unrestricted traversal, remote content fetches, unsafe HTML/SVG/PDF/active rendering, browser automation, scraping. |
| Annotation | No current authority; later contract must be explicit and bounded. | Hidden persistence, unbounded retention, automatic evidence acceptance, capability-bearing annotation payloads. |
| Comparison | Existing one-file Git diff and inert bounded text are read-only evidence. | Generic diff engine over paths/repositories, Git mutation, shell/Git command dispatch, execution. |
| Approval | Existing native conversation approval only. | Review-as-approval, approval bypass, promotion as consent, automatic approval/dispatch. |
| Artifact promotion | Existing M48 explicit single-artifact Save only. | Automatic promotion, project write, overwrite, task-authority grant, automatic evidence acceptance. |
| Execution | None from review. | Shell/terminal execution, task execution, dispatch, provider integration, publishing, deployment, privileged operations. |

M53/M54 must additionally exclude Figma and all third-party design connectors, broad repository scraping, hidden network access, publishing, deployment, provider integration, automatic actions, arbitrary filesystem/shell/terminal access, Git mutation, dispatch, approval bypass, remote content fetching, and unbounded retention. Review itself grants none of approval, dispatch, Git, shell, filesystem, network, publishing, deployment, or execution authority.

## Security and privacy observations

- QuireForge metadata must remain separate from Codex authentication, configuration, sessions, connector credentials, transcripts, raw protocol, hidden context, and provider/browser state.
- Digest/provenance fields are present for M48 generated text but not a general content-addressed review store. An M53 reference must avoid claiming integrity guarantees it does not actually validate.
- M48 content and M49 pane selections are transient; M52 durable content is narrowly user-controlled organizational text. A review proposal must make persistence/expiry/deletion/recovery choices explicit instead of inheriting them implicitly.
- Safe preview formats are intentionally closed. Image/mockup/design review cannot presume SVG, PDF rendering, remote URL loading, animation, arbitrary binary decoding, or document execution.
- Existing strict schemas reject unknown and capability-bearing fields, and native services fail closed to unavailable/diagnostic states. New review contracts need the same malformed, stale, corrupt, unavailable, and capacity behavior.

## Open architecture ambiguities

1. The roadmap names task briefs, plans, mockups, safe previews, validation evidence, and generated artifacts but does not close their representations, retention, or whether each is stored, referenced, or transient.
2. The only current durable task-plan body is organizational text; M52 expressly excludes artifact/evidence relationships. A reference model and deletion semantics are not decided.
3. M48's expiry and consumed-save lifecycle conflicts with any expectation that a later durable review record can always reproduce a generated artifact.
4. M49 Preview is text-only for M48, while FilePreview supports restricted attached-file text/images/PDF metadata. A unified surface, not a unified authority, remains an unresolved interaction and contract question.
5. No existing annotation, comparison, review state, promotion eligibility, or review audit model establishes who owns data, how it expires, or what a failure means.
6. Existing activity and approval are conversation-scoped and ephemeral; neither defines durable task/review history or approval linkage.

## M53-B core-contract question register

| Question | Why it must be resolved; existing constraint and safety risk | Existing reusable service | Resolving phase |
| --- | --- | --- | --- |
| Which closed review-item classes are supported? | Roadmap examples are not a schema. Classes determine parser/rendering, content, and privacy exposure. | M48 classes; FilePreview kinds are bounded examples only. | M53-B |
| What exact representation, native identity, provenance, and digest is retained or referenced? | Avoid ambiguous task ownership and false integrity claims. | M48 UUID/SHA-256 manifests; M52 UUIDv7 validation. | M53-B |
| What per-item/aggregate count and byte limits apply? | Existing services refuse at capacity rather than evict. | M48/M52 capacity accounting. | M53-B |
| Is an item transient, durable, expiring, archivable, deletable, or recoverable? | M48 and M52 have incompatible lifecycle models; hidden retention is prohibited. | M48 expiry; M52 lifecycle/SQLite patterns. | M53-B |
| Can an item reference a task, alternate plan, both, or neither? | M52 currently has no artifact/evidence relation and plan switching has no authority. | M52 task/plan IDs, but no existing relationship. | M53-B |
| What happens when a linked task/plan is stale, completed, archived, replaced, unavailable, or deleted? | Prevent dangling authority, silent deletion, or data resurrection. | M52 explicit lifecycle diagnostics. | M53-B |
| Which safe previews are allowed, and what validation/sanitization applies? | Active rendering and arbitrary path/file input are prohibited. | M48 text preview; FilePreview closed formats. | M53-B |
| Is validation evidence stored, referenced, or only displayed? | Evidence acceptance/promotion must not become automatic or unbounded. | M49 read-only evidence patterns. | M53-B |
| Are annotations durable, what fields/limits/expiry apply, and can they be edited/deleted? | No current annotation type exists; content may carry privacy/capability risk. | M52 normalization/limits pattern only. | M53-B |
| What comparisons exist and what inputs are legal? | Prevent a generic path/repository/Git comparison capability. | M49 Diff presentation is informative only. | M53-B |
| What promotion is eligible and what exact explicit user confirmation is required? | Existing M48 Save is not task promotion; review cannot grant authority. | M48 explicit save/claim pattern, not semantics. | M53-B |
| How do failure, corrupt, stale, unavailable, capacity, and partial-operation states behave? | Existing contracts fail closed and avoid partial mutation. | M48/M52 diagnostics and snapshots. | M53-B |
| What security/privacy boundaries and non-goals are contractually closed? | M53 cannot broaden local/remote/privileged authority by implication. | Threat model and ADRs. | M53-B |

## M53-C interaction-contract question register

| Question | Why it must be resolved; existing constraint and safety risk | Existing reusable service | Resolving phase |
| --- | --- | --- | --- |
| Where does local review live in the workbench, and which M49 pane(s) are reused? | M50 constrains placement; a new surface must not imply new authority. | `ReviewPanes`, especially Preview/Files/Activity/Approval. | M53-C |
| What cards/lists expose item class, provenance, selection, state, and limits? | Needs an honest bounded projection without paths or raw content. | M48 card/manifest and M49 card/list patterns. | M53-C |
| How are text/image/PDF-metadata previews selected and rendered? | Preserve explicit selection and safe-format restrictions. | M48 PreviewPane and FilePreview patterns. | M53-C |
| What is the annotation workflow and its visible state? | No current UI or ownership semantics; must not hide persistence or approval. | Existing labelled controls/status patterns only. | M53-C |
| What comparison interaction is available? | It must not look like or trigger Git/path operations. | DiffPane presentation pattern only. | M53-C |
| What is the promotion flow and how is explicit confirmation separated from review? | Prevent review observation from granting task/approval/dispatch authority. | M48 save flow and existing confirmation patterns, with bounded contract from M53-B. | M53-C |
| How do focus, keyboard tabs, resize/collapse, responsive layout, and accessibility work? | Preserve M49/M50 tabpanel, focus restore, separator, narrow and reduced-motion guarantees. | ReviewPanes/layout preferences/styles. | M53-C |
| What exact empty, stale, corrupt, unavailable, loading, and failure states appear? | Existing panes expose honest states; new UI must not fabricate data/retry silently. | M49 status patterns; M52 unavailable diagnostics. | M53-C |
| What activity is shown, and can it be durable? | Existing activity is conversation-local and not a review audit log. | ActivityPane presentation only. | M53-C |
| How is approval wording kept distinct from review/promotion? | Approval pane is display-only and ADR 0011 is the sole decision authority. | ApprovalPane and normalized pending approval. | M53-C |
| What deterministic unit/component/browser tests establish selection, safety, focus, narrow layout, and failures? | Routine tests cannot require providers, connectors, local user files, or billable calls. | Existing M48/M49/M50/M52 fixture/test conventions. | M53-C |

## Risks requiring later acceptance criteria

Later approved acceptance criteria must cover: closed representations and parsers; native/frontend validation; identity/provenance/integrity claims; per-item and aggregate capacity refusal; explicit lifecycle/expiry/deletion; task/plan stale/deleted behavior; safe-preview refusal of unsupported active or binary content; annotation and comparison bounds; explicit promotion without authority transfer; no automatic evidence acceptance; accessible keyboard, focus, narrow, reduced-motion, empty/unavailable/corrupt/failure behavior; and deterministic tests with sanitized fixtures and no provider/connector/network dependency. These are risk categories, not final M54 acceptance criteria.

## Evidence and test references

Primary evidence inspected for this record:

- [ROADMAP.md](ROADMAP.md), [CURRENT_STATE.md](CURRENT_STATE.md), [ARCHITECTURE.md](ARCHITECTURE.md), [THREAT-MODEL.md](THREAT-MODEL.md), and [M52 report](MILESTONE_52_DURABLE_TASK_RECORDS.md).
- [M48 report](MILESTONE_48_GENERATED_ARTIFACTS_AND_EXPLICIT_SAVE.md), [M49 report](MILESTONE_49_REVIEW_PANES.md), and [M50 report](MILESTONE_50_WORKBENCH_LAYOUT_REFINEMENT.md).
- ADRs [0011](DECISIONS/0011-native-approvals-and-activity-contract.md), [0012](DECISIONS/0012-read-only-git-review-boundary.md), [0013](DECISIONS/0013-reviewed-git-mutation-boundary.md), [0021](DECISIONS/0021-safe-project-file-previews.md), and [0007](DECISIONS/0007-quireforge-metadata-sqlite.md).
- Existing M48/M49/M50/M52 Rust, TypeScript, React, CSS, fixture, and test files named throughout this document, including `advisor_generated_artifact.rs`, `project/{types,mod,storage}.rs`, `ReviewPanes.tsx`, all six `review-panes/*Pane.tsx`, `layoutPreferences.ts`, `taskRecords.ts`, `filePreview.ts`, and their tests.

No final M53 recommendation was selected in M53-A. No M54 implementation occurred.
