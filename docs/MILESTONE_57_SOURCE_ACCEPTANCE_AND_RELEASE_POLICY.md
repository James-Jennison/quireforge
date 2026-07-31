# Milestone 57 — Source Acceptance and Release Policy

Status: complete source-only closure. This is the authoritative M57 source
acceptance and release-policy decision. It assigns no package version and
creates no package, tag, release, or runtime capability.

## Authority

The accepted M57 source commit is
`a1d407469626e34cd5d4921abdb6c8d305895d7e` (`feat: add local mock connector
foundation`). At the start of this decision, `main` and `origin/main` were
aligned at that commit with a clean worktree. The latest packaged product
generation remains beta.54: annotated tag `v0.1.0-beta.54` is bound to M56
package/source commit `e2b084ed0bdf17fb6f4b0b47663cdf6952ec8e73`, and its
GitHub prerelease remains a draft.

## Source acceptance

The M57 local mock-only connector foundation is accepted as source-complete.
Its accepted scope is private native in-memory behavior only:

- one static fictional `LocalMock` descriptor with canonical version and
  SHA-256 digest identity;
- opaque project, account, binding, and credential-reference identifiers, with
  inert credential references only;
- closed lifecycle, binding, proposal, confirmation, dispatch/result, and audit
  models with exact project/account/scope matching;
- explicit, one-use, expiring confirmations with cancellation, invalidation,
  descriptor-drift detection, revocation, quarantine, replay prevention, and
  mismatch rejection; and
- deterministic local mock outcomes with content-free, visibly mock-only audit
  records.

## Validation evidence

Fresh source validation passed without package or host work:

- `pnpm validate` and the repository validator passed;
- 34 package-contract tests passed;
- TypeScript and Astro checks, ESLint, and formatting passed;
- 378 frontend tests passed (371 desktop and seven website);
- desktop and website production builds, distribution validation, and bundle
  budgets passed;
- Rust workspace check and Clippy with warnings denied passed;
- 382 Rust tests passed, with three explicitly ignored; sandbox-worker tests
  passed;
- the focused connector-foundation suite passed six tests; and
- `git diff --check` passed.

This evidence does not claim package, installed-host, release, deployment, or
runtime connector validation.

## Source-only closure decision

M57 is closed as source-only. The private local mock foundation introduces no
user-visible, bridge-exposed, persisted, networked, provider-backed, or
operational connector behavior and therefore does not warrant a package or a
new application version. No Debian package, worker package, AppImage, tag,
release, or beta.54 draft-release update is required. The beta.54 draft must
not be relabelled or amended to include M57.

No missing package is a validation failure for this milestone. Packaging may be
reconsidered only when separately approved user-visible or operational behavior
depends on the foundation. That later decision must select a new version and
require fresh evidence bound to its exact future source commit.

## Continued exclusions

M57 grants no network/provider/inference/retrieval capability, real credential
handling or custody, OAuth, external-service connection, browser authority,
external mutation, synchronization, background work, scheduling, automation,
generic MCP execution, persistence, SQLite migration, Tauri command, frontend
bridge, UI, durable source-manifest authority, research-report implementation,
new Codex/shell/terminal/Git/repository authority, deployment authority, or
multi-agent/parallel-agent behavior.

## All-in-one north-star guidance

The long-term all-in-one workspace direction is non-authorizing architectural
guidance. Projects and tasks remain authoritative over provider-owned threads
or sessions. Provider adapters may translate intelligence traffic but receive
no implicit native tool authority. QuireForge retains ownership of context
assembly, credential references and custody policy, operation validation,
approval, artifacts, audit, evidence, and recovery.

Future conceptual categories may include inference providers, retrieval
providers, connected services, local runtimes, execution targets, credential
authorities, and browser-verification services. None is implemented or approved
by M57. Real provider work requires separately approved decisions for capability
taxonomy/registry, canonical interaction and event protocol, provider-adapter
lifecycle, credential broker, context-transmission manifests, retention/privacy,
limited inference, usage/cost accounting, and source admission/citations where
applicable. Provider adapters must never bypass QuireForge's closed operation
and confirmation system. A generic OpenAI-compatible chat abstraction must not
be presumed to express every provider's capabilities; future design should use
a canonical superset with separately governed provider-specific extensions.

## M58 and next decision paths

M58 remains unstarted: it is a decision-only Controlled Browser Verification
proposal limited to verification-only browser questions. It must not become a
general provider, OAuth, connector, generic browser automation, research
browser, web-agent, automation, or substitute credential/context/inference
milestone.

The existing roadmap path remains M58 as the next named gate. Separately, any
provider-neutral foundation requires a future decision family chosen and
approved independently; it must not begin automatically or be folded into M58.
