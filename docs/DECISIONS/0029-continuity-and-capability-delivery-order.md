# ADR 0029: Continuity and Capability Delivery Order

- Status: Superseded by ADR 0030
- Date: 2026-08-27
- Decision owners: Project owner and maintainers

## Context

QuireForge's next product direction has two approved concerns: durable,
agent-neutral project continuity (M70--M75), and real capability parity for
isolated browser research, connector access, and scheduled work (M76+).
Implementing both streams concurrently would make them compete for the same
native bridge, project storage, authority lifecycle, and package acceptance
gate. It would also defer their integration risk until after each branch had
already claimed independent validation.

## Decision

Deliver one serialized, interleaved stream:

1. M70 -- Typed Project Knowledge Foundation;
2. M71 -- Evidence Linkage;
3. M76 -- Isolated Read-Only Browser Research;
4. M72 -- Objective-Scoped Authority Boundaries;
5. M77 -- Connector Read Access;
6. M73 -- Agent-Neutral Context Assembly;
7. M78 -- Scheduled and Background Work;
8. M74 -- Three-Part Completion Model;
9. M79 -- Connector Mutation and Delivery; and
10. M75 -- Cross-Agent Handoff Proof and Recovery.

M76 may not use ambient browser sessions, credentials, downloads, form
submission, or agent-directed context transfer. M77, M78, and M79 each retain
their separate capability, credential, authority, transport, retention, and
external-side-effect gates. No milestone receives authority merely from its
place in this sequence.

## Constraints

- M70 is a new private native Knowledge Ledger service beside M66; M66 remains
  read-only and content-free.
- M60 remains the sole reviewed context-transfer boundary. Durable knowledge is
  not ambient provider context.
- M69C supplies a lifecycle pattern only; its content-free, no-database Action
  Card service is not extended into a ledger or external-capability service.
- QuireForge's own ADR and `CURRENT_STATE.md` process remains authoritative.
  Generated ledger views apply only to managed projects unless a future,
  evidence-triggered ADR changes that decision.
- Work is serialized on the authoritative branch. Isolated investigation may
  use a worktree, but no two implementation milestones are developed in
  parallel or accepted independently before an integrated validation gate.

## Consequences

Capability parity begins after two continuity foundations rather than waiting
for the full M70--M75 arc. Browser research arrives before connector or
scheduled work; connector mutation remains last because it has external side
effects and the strongest finality requirements. Every milestone retains its
own required evidence, full validation, desktop/browser E2E when applicable,
and package gate before it is marked validated.

## Supersession

The initial serialized order completed M70, M71, and M76, and established M72
as the next prerequisite. The project owner subsequently set full feature
parity with leading LLM desktop applications as the authoritative product
objective. [ADR 0030](0030-feature-parity-delivery-roadmap.md) retains the
completed evidence and the single-stream delivery rule, but replaces this
narrow capability sequence with the authoritative end-to-end roadmap.
