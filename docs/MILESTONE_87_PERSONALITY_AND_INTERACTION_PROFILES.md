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
conversation and Advisor composer each show the profile used to begin that
conversation, so a person can choose a different style before sending. The
chosen closed value is sent only as the documented native runtime personality
parameter; it does not add project context, a tool, a browser, a connector, or
an external provider.

## Authority invariants

- Profiles affect conversational prose only.
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

`0.1.0-beta.111` supersedes beta.110 for owner acceptance. It keeps M87's
closed Direct/Conversational and authority contracts unchanged while making the
actual New task workspace a chat-first surface for both profiles: the transcript
is the main workspace, the composer stays at its bottom, and task settings are
collapsed supporting controls. Direct and Conversational change language only;
they never select a different workspace layout. It also ignores streaming
frames made only of whitespace or invisible Unicode formatting markers before
strict native protocol validation, while still rejecting a marker mixed with
visible text. Beta.110's package and installed-host evidence remains historical
evidence only. Beta.111 has passed full validation, desktop and website E2E,
the clean pinned Ubuntu package gate, restricted staged installation,
installed-host integrity validation, and the bounded installed-binary launch
check. Owner acceptance remains required.

## Owner acceptance procedure — beta.111

This is a one-shot owner review, not a new capability or a new application
surface. Use the installed `0.1.0-beta.111` application and its New task chat
workspace:

1. Test both **Direct** and **Conversational** profiles. For each profile, the
   work area must remain the same chat-first transcript with a bottom composer;
   it must not become a split dashboard or operator-console layout. Observe at
   least three representative multi-fragment managed-assistant responses; each
   must remain one bounded response bubble rather than a card for every
   fragment.
2. For each profile, observe a transient polling failure and then a successful
   displayed-task poll. The diagnostic must remain until that success and then
   clear; a persistent failure must remain visible rather than being hidden.
3. Confirm that changing profiles changes conversational presentation only:
   approval prompts, authority scope, and execution capability remain
   unchanged.
4. Record one terminal owner decision in this report: **accepted** or
   **rejected**, with a concise defect note on rejection. A rejection reopens
   M87; it is not a partial pass and does not start M73.

No provider dispatch, browser access, connector call, context transmission,
approval-policy change, sandbox change, or native execution expansion is part
of this review. M73 design and implementation remain deferred until this
terminal decision is recorded.
