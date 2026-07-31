# M54 closed evidence-source contract

This implementation addendum ratifies the `evidence` source set selected by
[M53-B](MILESTONE_53B_CORE_REVIEW_CONTRACT.md). It does not authorize capture
from a path, URL, raw protocol, command output, approval body, Git object, or
live reference.

## Envelope

Every copied `EvidenceEnvelopeV1` is canonical UTF-8 JSON with this field
order: `schemaVersion`, `source`, `sourceSchemaVersion`, `title`, `summary`,
`details`. It has schema versions `1`, normalized LF title/summary, no unknown
fields, and a SHA-256 over the exact bytes. The item is UUIDv7,
`typed-evidence-snapshot`, and is bound only to its target collection.

The envelope ceiling is 16 KiB (warning at 12 KiB); collections permit six
evidence items (warning at five), subject to the existing collection/global
payload limits. Labels remain 120 code points / 480 bytes. There is no separate
evidence line or code-point allowance beyond the canonical envelope ceiling.

Creation is deliberate and validates the active collection/task/observed-plan
binding. It never resumes a collection. Copied evidence remains readable as
read-only recovery in frozen/orphaned collections; malformed, unavailable, or
digest-mismatched stored evidence withholds content with a closed diagnostic.
Preview is inert bounded copied-envelope fields only.

## Closed sources and details

| Source | Details | Association rule |
| --- | --- | --- |
| `manual-validation-summary` | `validationState`: `passed`, `failed`, `mixed`, `not-run` | User supplies only the bounded explanation; target collection binding applies. |
| `m48-generated-artifact-metadata` | `artifactState`, `artifactKind`, `format`, `byteLength`, `truncated`, `manifestSha256` | M48 is source-unbound; capture binds only the target collection and never claims task origin. |
| `safe-preview-metadata` | `previewState`, `kind`, `rendering`, `mediaType`, `byteLength`, `truncated`, nullable `widthPx`/`heightPx` | Source-unbound unless native preview has a task binding; otherwise target collection only. |
| `git-status-diff-summary` | aggregate workspace/change/addition/deletion/diff availability values | Native project/workspace must match the target task project, or capture is unavailable. |
| `activity-presentation` | `scope: current-session`, bounded local-review aggregate counters, `truncated` | Only activity already owned by the target collection/task is eligible. |
| `approval-presentation` | `approvalState`, `requestPresent`, `decisionPresent`, `dispatchPresent`, `executionPresent` | The target task's immutable migration-20 Advisor conversation/dispatch origin must resolve to one approved, started same-project dispatch. A collection plan is checked only for current task-context eligibility; this source does not claim approval of a later plan body or revision. |
| `package-manifest-summary` | application/Debian version, closed validation states, artifact count, completion | Native project identity must match the target task project, or capture is unavailable. |

All numeric detail values are bounded non-negative integers. No details object
may contain a filename, path, URL, source commit/object ID, raw diff, command,
command output, approval body/action ID, receipt, remote handle, file bytes, or
authority. Approval presentation is descriptive only and cannot approve,
dispatch, or execute.

All seven ratified sources have capture implementations: `manual-validation-summary`,
`m48-generated-artifact-metadata`, `safe-preview-metadata`,
`package-manifest-summary`, `git-status-diff-summary`, `activity-presentation`,
and `approval-presentation`. They use fixed native, redacted source-specific
claims. Package-manifest capture
accepts only `{ collectionId, expectedCollectionUpdatedAtMs }`, resolves the
immutable task project binding and deterministic newest complete migration-18
installed-host chain natively, and persists no record identity or host detail.
Git-status capture accepts the same two-field collection request, resolves the
task project and attached repository natively, and stores only closed aggregate
facts from the native Git service. Its preview reads persisted evidence bytes
only. Approval capture accepts only that two-field collection request, resolves
the immutable migration-20 task origin natively, persists only its five closed
redacted presentation fields, and previews persisted bytes only. Frontend
in-memory Activity events are presentation-only and never evidence authority.
Activity capture uses only future native append-only ledger rows owned by the
selected collection and immutable task; no historical frontend activity is
migrated or reconstructed. Snapshots and generic filesystem, Git, or approval
inputs are not evidence APIs.
