# M72 — Objective-Scoped Authority Boundaries

## Purpose

M72 turns project-level implementation authority into a native, explicit,
project-bound objective contract. It is the prerequisite for supervised browser
and connector work; it is not itself an executor or a capability grant.

## Closed first contract

An owner creates a bounded objective with a title, objective statement, expiry,
allowed authority lanes, and an optional confirmation-required subset. Lanes are
closed: work with code, browser workspace, browser observation, connector read,
scheduled work, connector mutation, provider inference, and computer use.

The lifecycle is `draft → active → revoked|expired`. Activation and revocation
are separate native owner actions. The identity, project, objective text,
expiry, and lane set are immutable. A new scope requires a new objective.

An objective only records whether a later capability may consider its lane. It
does not launch a browser, attach a project, read a connector, schedule work,
send provider context, execute code, or control the desktop. A later service
must define its own compatible objective-consumption and confirmation rule.

Lane selection is descriptive planning metadata only. It confers no current or
standing authority and must never reduce, replace, or skip a later capability's
own first-use disclosure and approval tier. A confirmation-required subset may
only pre-fill or highlight a later Action Card; it never lowers that card's
required confirmation or permits execution.

## Boundary preservation

- M69C remains a content-free Action Card grammar and is not extended.
- M60 remains the reviewed context boundary until M73.
- A user-owned browser profile never exposes cookies, passwords, or one-time
  codes to an agent.
- A lane requiring confirmation cannot be exercised merely because an objective
  is active.

## Required acceptance evidence

1. Native storage rejects unknown lanes, duplicate lanes, confirmation lanes
   outside the allowed set, empty objectives, invalid expiry, and invalid
   lifecycle transitions.
2. Native/bridge contracts expose only bounded authority metadata and reject
   unexpected fields or executable claims.
3. The owner authority workspace is reachable only from Project State after a
   project is attached; Local Chat cannot open it. It can create, activate,
   inspect, and revoke an objective without starting a capability. Future scope
   is grouped in owner-readable categories, while each individual future lane
   remains visibly locked and requires its own approval when available.
4. An authority workspace cannot be reached without its required attached
   project precondition; a defensive no-project invocation shows only a compact
   "Choose a project first" explanation and no live form. An identity-changed
   attachment is unavailable until it is selected or relinked, and likewise
   cannot expose authority management.
5. Full validation, applicable desktop E2E, clean package, installed-host
   launch, and owner acceptance pass before M72 is validated.

## Implementation checkpoint

The native storage, typed bridge, and owner workspace have been implemented.
Authority management is project-first: it is entered from Project State only
after a project is attached, never from an unattached Local Chat. An owner can
create a draft, select grouped future scope and an optional review flag,
activate it, inspect individual locked lanes, and revoke it. The interface
states explicitly that these records do not start a browser, agent, connector,
or external action. Clean package, installed-host launch, and owner acceptance
remain required before M72 is marked validated.
