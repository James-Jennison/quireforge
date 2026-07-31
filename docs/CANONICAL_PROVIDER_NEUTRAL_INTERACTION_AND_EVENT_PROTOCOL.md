# Canonical Provider-Neutral Interaction and Event Protocol

Status: complete decision-only architecture gate within the active
[Provider-Neutral AI Foundation](GOAL_PROVIDER_NEUTRAL_AI_FOUNDATION.md). This
record defines no protocol implementation, provider adapter, network transport,
credential route, context-transmission route, model invocation, persistence,
bridge, or UI. M58 remains separate and unstarted; beta.54 remains the latest
packaged generation.

## Purpose and authority

The canonical protocol is a future native-owned representation of an attempted
intelligence interaction and its bounded lifecycle. It lets QuireForge preserve
project/task authority while translating different provider and local-runtime
event models without reducing them to an OpenAI-compatible chat abstraction.

The protocol is a communication contract, not permission. A canonical attempt,
input, event, tool proposal, citation, provider-session reference, usage record,
or terminal state does not authorize provider access, credentials, context
transmission, retrieval, native tools, browser access, connected services, or
external mutation.

## Core ownership model

QuireForge owns the canonical interaction-attempt identity, lifecycle,
project/task binding, context-authorization reference, policy decision, and
user-visible normalized projection. A provider or local runtime may own an
opaque subordinate session, request, stream, job, or continuation reference;
it never owns or replaces the QuireForge project or durable task.

Every future attempt must bind a selected descriptor identity and digest, an
adapter compatibility identity, one QuireForge project/task context where the
future route requires it, canonical protocol version, and native-issued opaque
attempt identity. Display labels, provider thread IDs, model strings, endpoint
names, and frontend selectors are never sufficient authority.

## Interaction attempt and envelopes

A future `InteractionAttempt` represents one requested provider-neutral
exchange. It is distinct from an invocation: its lifecycle can be unavailable,
blocked, prepared, awaiting a later authority, active only after a separately
approved dispatch route, cancelling, terminal, interrupted, or quarantined.
This decision authorizes no active dispatch route.

The canonical envelope family is closed and versioned:

- **Attempt envelope** — identity/version/digest bindings, selected descriptor
  references, native lifecycle state, and bounded policy classification.
- **Input envelope** — declared input role, modality, bounded reference or
  digest, and future context-authorization reference. It does not imply that
  bytes, text, files, images, or project data may be transmitted.
- **Event envelope** — one normalized lifecycle, output, usage, error, or
  extension event correlated to exactly one attempt and monotonically ordered
  within that attempt.
- **Terminal envelope** — final closed outcome, cancellation/interruption
  classification, and safe aggregate evidence.

Unknown envelope versions, unknown safety-relevant fields, invalid ordering,
cross-attempt references, descriptor drift, incompatible adapters, or missing
required bindings fail closed. This decision chooses no serialization,
database, IPC, or wire format.

## Inputs and outputs

Canonical inputs classify declared roles rather than provider-specific message
shapes. Future approved routes may represent user-authored text, structured
values, and explicitly authorized references to image, audio, video, document,
or file material. Inputs retain modality, declared role, provenance class,
content/digest reference where later policy permits, and bounds. They must not
silently include project trees, transcripts, browser state, credentials,
retrieved content, provider-managed files, or hidden instructions.

Canonical outputs may represent text, structured data, image, audio, video,
embedding/vector results, provider-grounding metadata, citations, tool
proposals/results, usage, progress, and terminal classification. A later
implementation decides which bounded display or artifact projections are safe;
this gate does not authorize retention, artifact admission, or rendering of
provider content.

## Streaming, ordering, and lifecycle

An adapter may later translate provider progress into ordered canonical events.
Events identify whether they are a lifecycle transition, append-only output
delta, replacement snapshot, structured result, usage observation, safe error,
or governed extension. Event sequence values are native-correlated, scoped to
one attempt, and cannot be supplied by the frontend as authority.

The minimum conceptual lifecycle is prepared or unavailable; dispatched only
through a later approved route; receiving; cancellation requested; terminal
succeeded, failed, cancelled, interrupted, timed out, or outcome-unknown; and
quarantined for integrity/compatibility failure. A provider may expose finer
states only through governed extensions. Duplicate, out-of-order, malformed, or
post-terminal events cannot revive an attempt or create a second dispatch.

Streaming is presentation and correlation behavior only. It does not create a
background job, imply delivery, grant retry, or permit an unbounded event log.

## Cancellation, interruption, and continuation

Cancellation is an explicit native lifecycle request against one current
attempt. A request to cancel, a provider acknowledgement, and a terminal
cancelled result are distinct facts. If interruption or transport failure makes
the external outcome ambiguous, the canonical result is outcome-unknown; it is
never inferred as success and never automatically retried.

Continuation is a new native-issued attempt bound to its predecessor through an
opaque reference and explicit future policy. It cannot reuse a provider session,
prior context, credential, descriptor, capability, or authority silently. A
provider-managed continuation reference is opaque, scoped to the selected
descriptor/adapter, and invalidated on descriptor or compatibility drift.

## Provider-session and job references

Provider sessions, threads, request IDs, stream IDs, asynchronous jobs, and
continuation handles remain opaque subordinate references. They are not project
or task IDs, cannot select arbitrary provider state, and confer no context,
account, credential, retrieval, tool, or mutation authority. A future approved
persistence decision must define retention and recovery; none is approved here.

## Structured and multimodal events

Structured output claims require a declared future schema reference, validation
outcome, and closed failure classification; no generic JSON passthrough is
approved. Multimodal events retain declared modality and bounded metadata, with
content handling governed separately. Image, audio, video, document, and file
references must not disclose paths, raw provider handles, or data outside a
later approved context/transmission policy.

Protected reasoning representations are provider-specific and unavailable by
default. A provider may expose a bounded reasoning summary only through a
governed extension and later display/retention policy; hidden reasoning is not
a canonical output requirement.

## Tool proposals and results

Tool-related events are model communication only. A canonical tool proposal may
identify a declared capability, bounded structured argument digest/reference,
and requested-result shape. A tool result may describe an already completed
native operation only when a separately approved native operation route has
validated, authorized, and executed it.

Neither event grants tool execution, terminal, filesystem, Git, browser, MCP,
connector, provider, cloud, approval, dispatch, or mutation authority. An
attempt cannot approve its own tool request, and an adapter cannot turn a model
proposal into an operation.

## Citations, grounding, and source boundaries

Citations, grounding metadata, provider-managed file references, and
provider-managed retrieval observations are provider claims. Canonical events
may classify them without treating them as verified facts, durable evidence, or
research sources. M55 remains the controlling authority for durable source
identity, admission, provenance, retention, and citation mapping. This protocol
does not authorize retrieval, source admission, report generation, or cited
research.

## Usage, errors, and terminal states

Usage events may later preserve raw provider-specific units, measurement scope,
observation time, and confidence; they do not imply billability, pricing,
account access, or a cost policy. Pricing, budget, and aggregate accounting
remain later decisions.

Errors use a closed native classification that distinguishes local validation,
policy denial, descriptor/adapter incompatibility, cancellation, timeout,
provider-declared failure, malformed event, and ambiguous external outcome.
Raw provider diagnostics, credentials, paths, URLs, request payloads, and
private content do not become canonical error fields. Terminal states are
idempotent, cannot be overwritten by late events, and preserve ambiguity rather
than repairing it by automatic retry.

## Governed provider-specific extensions

The protocol is a canonical superset, not a forced common denominator.
Provider-specific controls, events, lifecycle detail, reasoning summaries,
grounding fields, and modality behavior use namespaced, versioned extension
envelopes bound to descriptor and adapter compatibility claims. Unknown
extensions cannot be interpreted as canonical behavior, cannot weaken policy,
and cannot grant authority. An extension affecting credentials, context,
retention, tool behavior, retrieval, browser behavior, or mutation requires its
own later approved policy support.

## Privacy, retention, and recovery boundaries

This gate approves no raw content retention, transcript storage, provider event
storage, session persistence, context manifest, credential reference, or
provider-result artifact. A later implementation may retain only what a
separately approved persistence, context, privacy, and provenance policy
permits. Restart recovery must never replay an attempt, cancellation, context,
or authority merely because a provider reference exists.

## Relationship to existing architecture

Existing Codex conversation and Advisor contracts are reference evidence for
native-owned IDs, normalized bounded projections, strict bridge exclusion, and
explicit interruption. They remain Codex-specific: their app-server methods,
thread lifecycle, managed authentication, approval behavior, model/reasoning
labels, and transport assumptions are not adopted as this protocol. Existing
project/task, M56 template, M57 binding/confirmation, M55 source-admission,
artifact, review, and native-operation boundaries remain unchanged.

## Later implementation acceptance

A future implementation requires closed native and bridge schemas;
deterministic fixtures; adversarial tests for unknown fields, cross-project/task/
session substitution, ordering, duplicate/late events, cancellation,
continuation, descriptor drift, extension handling, structural redaction, and
outcome-unknown behavior; and proof that protocol events cannot invoke a native
operation. It requires separate approval and does not follow automatically.

## Recommended next gate

The next recommended decision is **Provider Adapter Lifecycle and Conformance
Governance**. It must define adapter identity/trust, isolation, compatibility,
upgrade, rollback, quarantine, revocation, conformance evidence, and failure
handling. It must not implement adapters, select providers, invoke models,
connect to a network, use credentials, transmit context, or begin inference.

## Explicit exclusions

This decision grants no provider/model/adapter selection or implementation,
network access, credentials or account connection, context transmission, model
invocation or inference, retrieval, source admission, citations authority,
native tool execution, browser behavior, connector or MCP execution, external
mutation, persistence, migration, Tauri command, bridge, UI, background work,
automation, package, release, tag, host change, or deployment.
