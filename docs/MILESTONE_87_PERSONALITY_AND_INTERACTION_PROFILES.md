# M87 — Personality and Interaction Profiles

## Purpose

M87 gives a person a small, explicit choice over QuireForge's conversational
style. It is not an authority setting, an objective, an approval preference,
or a capability grant.

## Closed first contract

The available profiles are deliberately limited to:

- **Direct** — concise, pragmatic conversational prose.
- **Conversational** — warmer, more exploratory conversational prose.

Appearance settings hold the default for future conversations. The project
conversation exposes the profile selector directly in its compact chat
composer, so a person can choose a different style before sending without
opening operational controls. The chosen closed value is sent only as the
documented native runtime personality parameter; it does not add project
context, a tool, a browser, a connector, or an external provider.

## Authority invariants

- Profiles affect assistant conversational prose only. Action Card content,
  authority and disclosure copy, lock labels, and failure messages are
  byte-identical across profiles.
- They never change objective lifecycle, objective scope, approval policy,
  Action Card requirements, sandbox choice, project access, or external
  capability availability.
- Unknown persisted or request values fall back to Direct on the frontend and
  are rejected by the closed native request contract.
- An active conversation cannot have its profile changed through the composer;
  a later profile selector must be a separately specified, forward-only native
  turn operation.

## Permissions correction carried with M87

Project conversations no longer expose or accept a no-approval policy. Native
start validation rejects `Never` for every project-bearing request, and native
resume parsing rejects legacy records that carry it. The only retained
no-approval paths are fixed no-project Chat and Advisor builders whose runtime
assertions require the full no-capability tuple: null working directory, no
tools, read-only sandbox, and no network. Changing any part of that tuple must
replace the policy with an explicit approval path.

The future permissions milestone may offer only two truthful modes: **Ask each
time** and **Ask once per action type**. It remains deferred until QuireForge
has a human-only approval tier structurally independent of a policy setting and
a real per-action-type owner tap mechanism. It must not present misleading
"Full access" or "Bypass permissions" controls.

## Required acceptance evidence

1. Direct and Conversational are the only selectable and accepted values.
2. The Appearance default persists locally and is used for a new conversation
   unless that composer explicitly selects the other profile.
3. The per-conversation selection is sent through the typed native request and
   maps only to the documented native personality field.
4. Objective, approval, Action Card, project-access, and capability contracts
   are byte-invariant with either profile.
5. Project no-approval starts and legacy no-approval resumes are rejected;
   fixed no-project builders fail immediately if their isolated capability
   tuple changes.
6. Full validation, desktop E2E, clean package, installed-host launch, and
   owner acceptance pass before M87 is marked validated.
7. The normal New task chat surface contains no fixture-workbench launchers.
   The deterministic local-only fixture workflows remain explicitly labeled
   and manually reachable from Task Catalog without gaining a provider,
   browser, connector, or execution route.

## Implementation checkpoint

The closed profile registry, Appearance default, project composer selector,
Advisor selector, typed frontend contracts, and native personality mapping are
implemented. The interaction style is visibly separate from access and
approval controls, and project conversations offer only approval policies that
remain truthful.

`0.1.0-beta.108` passed full validation (440 desktop unit tests, seven website
unit tests, 463 Rust tests, and two sandbox tests), desktop and website E2E
(86 and eight tests respectively), and the clean pinned Ubuntu package gate.
The latter built both Debian packages, completed disposable lifecycle and
visible-launch validation, then promoted the artifacts. Both packages were
staged through the restricted root-owned boundary and installed through the
local installer daemon. The installed-host validator passed package state,
version mapping, protected-file ownership, permissions, and integrity; the
installed `/usr/bin/quireforge` process was then launched. Owner acceptance of
the visible profile controls remains required before M87 is marked validated.

`0.1.0-beta.117` supersedes failed beta.116, beta.115, and beta.114 for owner acceptance. It keeps M87's
closed Direct/Conversational and authority contracts unchanged while making the
actual New task workspace a chat-first surface for both profiles: the transcript
is the main workspace, the composer stays at its bottom as one compact row,
and task settings are collapsed supporting controls. Attachments and boundary
copy appear only when used or explicitly opened. Direct and Conversational change assistant prose
only; they never select a different workspace layout or change Action Card
content, authority/disclosure copy, lock labels, or failure messages. It also
ignores streaming
frames made only of whitespace or invisible Unicode formatting markers before
strict native protocol validation, while still rejecting a marker mixed with
visible text. It also normalizes an empty optional native plan explanation to
`null`, so non-semantic protocol framing cannot invalidate the typed frontend
snapshot. The strict snapshot-recovery path preserves any already rendered
response and can retry only the existing conversation poll; it does not claim
an undetected frontend/native version mismatch. Beta.110 through beta.113
package and installed-host evidence remain historical evidence only. Beta.114
is immutable failed acceptance evidence because the installed app dropped the
opening fragments of a reply and then surfaced a terminal native-response
diagnostic. Beta.115 reconciles each stream with the completed assistant message
and keeps passive item-schema drift from terminating the response, but it is
immutable failed acceptance evidence because one malformed passive event could
still reject an entire already-consumed poll batch. Beta.116 recovers only that
explicitly classified passive member while preserving ordered assistant text;
its exhaustive event classification keeps assistant, lifecycle, approval,
model-selection, error, envelope, and unknown-event failures closed. Full
repository validation, all 88 desktop/mobile E2E cases, and all eight website
E2E cases pass. The clean pinned Ubuntu 22.04 package gate passed for both
Debian packages; both were installed through the restricted local installer.
Installed-host package state, version mapping, protected-file ownership,
permissions, and integrity validation passed, and the installed
`/usr/bin/quireforge` process launched successfully under the bounded
`quireforge-installed-beta116` user service. Owner interaction acceptance
failed because a rejected consumed poll produced no visible assistant response
and was followed by a clean terminal completion. Beta.117 retains each exact
event-bearing native snapshot under an opaque delivery token until the events
have committed to the React conversation state and the renderer acknowledges
that token. Rejected batches replay unchanged, including terminal batches with
co-batched assistant text. Content-free validation classification preserves the
strict consequential-event boundary without recording response values.
Beta.117 source validation passed with 465 desktop tests, 467 native tests, two
sandboxd tests, type-check, lint, format, build, and distribution checks; all
88 desktop/mobile and eight website E2E cases passed. The clean pinned Ubuntu
22.04 package gate at `87076a1511ade8853de6beb99ed9040854a1b57e` passed
lifecycle, visible-launch, and final artifact validation. Both Debian packages
were installed through the restricted local installer. The content-free
installed-host validator passed package state, exact version mapping,
protected-file ownership, permissions, and integrity, and the installed
`/usr/bin/quireforge` process is running under the bounded
`quireforge-installed-beta117` user service. Owner acceptance remains a
separate pending gate.

The chat-first presentation has one functional visual boundary: ordinary
assistant prose, passive progress/evidence, and open clarifying questions stay
in the reading flow. Only a concrete, bounded action that is actually awaiting
an explicit yes/no decision interrupts that flow with the existing semantic
approval surface. Passive activity remains a lightweight, expandable timeline
row; it is not an Action Card and conveys no authority.

`0.1.0-beta.123` supersedes beta.122 as the current owner-acceptance candidate
without replacing beta.122, beta.121, beta.120, beta.119, or beta.118's immutable
acceptance evidence. Beta.121 failed its installed-host validator because the
native consumer read stale checkout artifacts instead of the builder's trusted
persistent output root. Beta.122 gives the builder, cleanup, restricted staging
helper, and native validator one version-controlled build-time root definition
while retaining strict manifest, checksum, direct-file, and no-symlink checks.
It keeps the profile request and all M87 authority invariants intact
while moving the native Direct/Conversational radio group into the compact New
task composer, where it shares the same active-task disable boundary as the
message box and send action. It removes the four deterministic fictional
fixture launchers from the normal chat surface and places them under a clearly
labeled local-only fixture catalogue in Task Catalog. Beta.122 subsequently
failed its native completion receipt at the immutable 32-record package-
validation history ceiling: each summary is atomically identity-bound, so the
existing prune path correctly cannot delete it. Beta.123 raises that single
bounded, fail-closed ceiling to 64 and proves the 65th write leaves no partial
summary or identity record. The fixtures remain
manual test workflows only; they do not contact a provider, browser,
connector, or external service.

`0.1.0-beta.120` supersedes beta.119 as the prior owner-acceptance candidate
without replacing beta.119 or beta.118's immutable acceptance evidence. It
contains the compact passive-status-strip repair from `559515e` and fixes the
native Rust/TypeScript conversation-event wire contract so multiword event
fields serialize as the existing strict frontend camelCase keys, while event
types remain kebab-case and consequential-event validation remains fail closed.
Fresh isolated source validation and the clean pinned Ubuntu 22.04 package
gate passed at `c1d176b`. Both `0.1.0~beta.120` Debian packages are installed;
the content-free installed-host receipt passed package state, exact version
mapping, protected-file ownership, permissions, and integrity, and
`/usr/bin/quireforge` is running under the bounded
`quireforge-installed-beta120` user service. Owner-acceptance evidence remains
required before this candidate is accepted.

## Owner acceptance procedure — beta.123

This is a one-shot owner review, not a new capability or a new application
surface. Use the installed `0.1.0-beta.123` application and its New task chat
workspace:

1. Test both **Direct** and **Conversational** profiles using the selector in
   the compact bottom composer. The selector must disable while a task is
   active, along with the message box and Send action. For each profile, the
   work area must remain the same chat-first transcript with a bottom composer;
   it must not become a split dashboard or operator-console layout. Observe at
   least three representative multi-fragment managed-assistant responses; each
   must remain one bounded response bubble rather than a card for every
   fragment.
2. For each profile, observe a transient polling failure and then a successful
   displayed-task poll. The diagnostic must remain until that success and then
   clear; a persistent failure must remain visible rather than being hidden.
3. Confirm that changing profiles changes assistant conversational prose only:
   the layout, Action Card content, authority/disclosure copy, lock labels,
   failure messages, approval prompts, authority scope, and execution
   capability remain unchanged. Trigger one fixed confirm-tier Action Card in
   each profile and compare its content byte-for-byte.
4. Confirm that ordinary responses, passive activity, and open clarifying
   questions read as part of the transcript; activity may be expanded for
   evidence, but it must not resemble a permission prompt. A concrete pending
   approval must remain semantically and visually distinct, with its explicit
   decisions available.
5. Record one terminal owner decision in this report: **accepted** or
   **rejected**, with a concise defect note on rejection. A rejection reopens
   M87; it is not a partial pass and does not start M73.
6. Confirm that the normal New task chat contains no fictional fixture
   launchers. Open Task Catalog and verify its explicitly labeled local-only
   fixture catalogue is still manually reachable and does not imply a live
   provider, browser, connector, or external-service capability.

No provider dispatch, browser access, connector call, context transmission,
approval-policy change, sandbox change, or native execution expansion is part
of this review. M73 design and implementation remain deferred until this
terminal decision is recorded.
