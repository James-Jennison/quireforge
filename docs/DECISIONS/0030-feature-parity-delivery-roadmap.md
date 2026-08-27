# ADR 0030: Feature-Parity Delivery Roadmap

- Status: Accepted
- Date: 2026-08-27
- Decision owners: Project owner and maintainers
- Supersedes: ADR 0029's future delivery order

## Decision

QuireForge's authoritative product objective is outcome-level feature parity
with leading LLM desktop applications, while remaining a local-first,
provider-neutral engineering workspace. Parity means equivalent, visible
workflows for durable project continuity, browser collaboration, supervised
agent work, coding review, connected work, automation, remote access, and
computer use. It also includes a human-led shared multi-provider roundtable:
the owner can select two or three configured providers for one attributed
discussion without manually relaying messages between provider chats. Its
owner-mediated interaction and authority model are specified by
[ADR 0031](0031-owner-mediated-roundtable.md). It does not mean reusing
private provider interfaces, scraping provider sessions, or exposing
credentials to an agent.

Work remains serialized on the authoritative branch. M70, M71, and M76 remain
completed historical foundations; M72 is the active next milestone. The
authoritative delivery order after M72 is defined in `docs/ROADMAP.md`.

The serialized order follows authority dependencies rather than surface
similarity: context assembly precedes completion and acceptance; structured
connector mutation precedes open-ended interactive browser mutation; and
scheduled work is split into M78A local/read-only triggers and M78B
mutation-capable work only after manual mutation and non-bypassable handoff
authority have matured. Profile, skill, and image-generation capabilities are
placed at their earliest compatible authority gates rather than bundled with
the later autonomy cluster.

## Constraints

- A user-owned authenticated browser profile is never equivalent to agent
  possession of credentials, cookies, passwords, or one-time codes.
- Agent inspection, browser interaction, uploads, downloads, form submission,
  connector mutation, remote control, and desktop-wide computer use each need
  their own explicit authority lane, expiry, revocation, and evidence.
- M69C remains a non-executing Action Card grammar. M72 owns objective-scoped
  authority; later services consume only the compatible authority they define.
- M60 remains the reviewed context-transfer boundary until M73 delivers its
  agent-neutral successor contract.
- A roundtable transcript is local and attributed. Each provider receives only
  an owner-visible, destination-specific projection through the approved live
  provider boundary; QuireForge never silently shares one provider's private
  session or credentials with another.
- Every implementation milestone requires scoped tests, full validation,
  applicable desktop/browser E2E, a clean package gate, installed-host launch,
  and owner acceptance where behavior is visual, interactive, or consequential.

## Consequences

The narrow M76 controller is a safe research baseline, not the endpoint for
in-app-browser parity. A user-visible browser workspace follows M72; supervised
agent browsing follows M73; and interactive browser authority follows the
three-part completion model in M74. General computer use remains separately
gated because it extends browser authority to the entire desktop.
