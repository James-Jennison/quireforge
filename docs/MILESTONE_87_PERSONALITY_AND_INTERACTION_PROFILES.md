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
remain truthful. Package, installed-host, and owner acceptance evidence remain
required.
