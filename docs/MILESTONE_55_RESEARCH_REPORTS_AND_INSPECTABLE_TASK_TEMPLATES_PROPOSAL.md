# Milestone 55 — Research Reports and Inspectable Task Templates Proposal

Status: approved decision. This is the sole authoritative M55 decision record.
It adds no runtime behavior, migration, command, bridge, UI, test, package,
provider, connector, browser, or network capability. M56 subsequently received
James's separate explicit approval and is now in progress; that later approval
does not change this decision-only record or grant M55 runtime behavior.

## Decision

M55 separates two capabilities which must not share authority:

1. **Research reports are deferred.** QuireForge has no approved durable
   source-manifest authority today. M55 therefore authorizes neither live
   browsing, URL retrieval, provider retrieval, connectors, MCP, OAuth, cookies,
   browser automation, nor a local substitute that silently calls arbitrary text
   “research.”
2. **Inspectable local templates are decision-ready for a future M56.** They
   are user-visible organizational text, explicitly selected and confirmed, and
   grant no execution, provider, project, repository, approval, or dispatch
   authority.

M43 remains a one-use transient handoff; M48 remains a process-local generated
artifact registry and explicit Save boundary; M52 remains the durable task/plan
owner; M54 remains copied local review/evidence. None is a research-source or
template authority by implication.

## Deferred research-report contract

If separately approved, reports may use only explicit, already-approved local
source records with durable canonical bytes, native UUIDv7 identity, visible
label, closed media type, byte length, and SHA-256. URLs may be inert labels
only: they are never fetched, opened, redirected, or used as identity. Current
M48 artifacts, M54 copies, and user text are not automatic research sources;
each would need a named source-admission decision.

A future native source manifest is limited to 12 sources, each label at 120
characters/480 UTF-8 bytes; a report is limited to eight sections, 24
citations, 32 KiB normalized text, and 48 KiB aggregate canonical payload. Each
citation binds one source ID and digest to a bounded section/range. Changed,
missing, malformed, duplicate, cross-task/project, conflicting, or
digest-mismatched sources make a report unavailable; no repair, refresh, or
inference occurs. Reports would require a later explicit storage/lifecycle
decision because M48 and M54 are not source-admission authorities.

Source material is untrusted data. It cannot alter policy, choose sources,
request context, invoke actions, or override confirmation. M57 remains the
connector-governance decision and M58 the browser-verification decision.

## Template-only M56 contract

### Closed records and limits

M56 may add read-only shipped built-ins and user-created local templates.
Built-ins have fixed native ID/version; local templates have native UUIDv7 IDs.
Every visible record has title, purpose, version, normalized plain-UTF-8
instructions, SHA-256 digest, lifecycle state, and origin (`built-in` or
`local`). There are no hidden fields, extension maps, system instructions,
credentials, source URLs, provider state, automatic attachments, or paths.

Instructions normalize whitespace and reject C0/C1/bidi controls. Bounds are:
title 80 characters/320 bytes; purpose 240/960; instructions 32 KiB; canonical
template 64 KiB; 64 templates and 2 MiB aggregate. Warn at 48 templates or
1.5 MiB; never evict or expand limits automatically. A closed optional model,
reasoning, sandbox, or approval-policy recommendation is display-only advice;
it cannot select a model, grant permission, start a sandbox, approve, dispatch,
or execute.

### Lifecycle and application

Create, edit, duplicate, archive, restore, and delete are explicit native
operations. Editing creates a new monotonic local version/digest, invalidating
old previews. Built-ins cannot be edited/deleted; duplication creates a local
draft. Archive is read-only; deletion is confirmed and application-irreversible
(without claiming journal/backup secure erasure). No legacy tasks, plans,
conversations, reports, review records, or frontend state are migrated.

Import/export, file picking, drag/drop, URL import, sharing, sync, marketplace,
and clipboard-write are deferred. Applying a template shows every visible field,
full digest/version/origin, and an explicit checklist limited to an existing M52
task, one of its plans, and user-authored bounded draft title/plan text. It
cannot select paths, repository/worktree, conversation/transcript, attachment,
artifact, review, terminal, Git, approval, dispatch, execution, browser,
connector, credential, or provider context.

Native revalidates digest/version and task/plan ownership before a separate
confirmation. Confirmation creates or updates only permitted M52 task/plan text;
it cannot save, start a conversation, send a provider request, browse, execute,
dispatch, approve, change Git, or create an attachment/artifact. Cancel, stale,
archived/deleted, quota, storage, lifecycle, or concurrent failure preserves the
draft and creates no partial mutation.

### UI and privacy

The later UI is a lazy workbench surface with labelled list/details, visible
version/digest, forms, destructive confirmations, checklist, and Apply dialog.
It must support Arrow/Home/End/Enter/Space/Escape, dialog focus trap with
initial Cancel focus and restoration, 200% reflow, narrow containment, reduced
motion, contrast, touch targets, status/alerts, and no background focus theft.
Bridge schemas deny unknown fields; the frontend never supplies ID, version,
digest, origin, authority, context claim, or capacity facts. Projections and
diagnostics are bounded/path-free; support bundles exclude instructions,
context text, paths, transcripts, credentials, and hidden state.

## Threat model and M56 acceptance criteria

M56 must fail closed for hidden-instruction smuggling, impersonation, unknown
fields, digest/version substitution, post-preview mutation, stale reservations,
cross-task/project context, context expansion, automatic execution, unsafe
import/export, capacity abuse, corruption, and interrupted deletion. Research
work must additionally refuse source/citation substitution, source change, and
prompt injection.

Future M56 acceptance requires: native UUIDv7/canonical digest/immediate-
transaction ownership in the existing database; strict native/Zod schemas;
transactional lifecycle; separate digest-bound preview/confirmation; focused
native, bridge, component, accessibility, responsive, and real-browser tests;
and validation of every exclusion above. No second database, legacy conversion,
retrieval, provider, connector, MCP, OAuth, browser, import/export, or automatic
action is in scope.

The provisional M56 candidate is `0.1.0-beta.54` / `0.1.0~beta.54`, subject to
fresh authority and package validation only if M56 is later approved. This
proposal changes no version declaration.
