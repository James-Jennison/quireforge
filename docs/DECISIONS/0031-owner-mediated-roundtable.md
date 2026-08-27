# ADR 0031: Owner-Mediated Multi-Provider Roundtable

- Status: Accepted
- Date: 2026-08-27
- Decision owners: Project owner and maintainers
- Extends: ADR 0030

## Decision

M91 is QuireForge's Phase 7B **Shared Multi-Provider Roundtable**. It is a
thread capability, not a top-level destination or a multi-agent executor. Its
purpose is to replace the owner's manual copy/paste/reformat relay across
provider chats with a local, attributed, owner-governed discussion.

The owner starts a roundtable from within a normal thread, chooses two or
three already configured providers, and moderates explicit rounds. The UI
remains one vertical thread: one owner prompt per round followed by distinct,
attributed, collapsible provider response cards. A scope badge shows the
active roundtable providers. Ending the roundtable or removing one provider is
visible and reversible; removing a provider never deletes historical evidence.

The owner, not QuireForge or a provider, is the router. A provider may receive
the owner's selected prompt in parallel with the other selected providers. A
provider response is never automatically sent to another provider. For every
later round the owner chooses both the destination providers and the content
to relay. Relayed material is visibly tagged with its provider and round of
origin in the local thread and in the outgoing projection.

## Authority and evidence

M91 depends on proven M81 single-provider connection, M72 objective authority,
and M73 context assembly. It must not be implemented ahead of those
foundations.

- The first send to each provider uses that provider's full, destination-
  specific first-send disclosure. Adding several providers cannot weaken or
  batch away individual disclosures.
- Each later round requires an explicit owner dispatch, but not a repeated
  full-consent wall. It uses the compact, expandable Action Card variant that
  names selected providers, gives a short preview or length indicator for the
  outgoing payload, and flags any relayed-provider provenance. The owner can
  expand it to inspect full content before sending.
- Admitting a provider to a roundtable never implies project-source access,
  persistent history access, tools, connectors, or any other authority. Those
  remain separately destination-scoped crossings.
- Every provider send produces a separate ledger/evidence record sufficient to
  answer who received what, from whom, and when. Credentials and provider-
  private sessions remain isolated.
- The owner can revoke a provider seat without silently changing other seats
  or erasing the prior local transcript and evidence.
- M91 dispatch is a direct, synchronous owner interaction. It is not invocable
  by M78B scheduled work or M90 delegated objectives, even when an owner
  previously configured those capabilities. A scheduled or delegated request
  cannot convert prior configuration into an unattended roundtable relay.
- M89 single-provider image-generation transport is not an M91 dispatch path.
  M91 owns and enforces its distinct provider list, digest, provenance, and
  per-provider ledger requirements at the crossing itself.

## Explicit non-goals

M91 must not introduce auto-relay, autonomous destination selection,
split-pane or tabbed provider windows, silent/unattributed relay, live
provider-to-provider cross-talk, bulk authority grants, or a general
multi-agent orchestration engine. M90 remains the later, separately bounded
delegated-objectives capability.

## Consequences

M91 follows M89 and precedes M82 in the serialized delivery order. Its
implementation directive must preserve the single-thread conversation grammar,
M69C's Action Card language, per-provider evidence, and the full validation,
package, installed-host, and owner-acceptance gates required by ADR 0030.
