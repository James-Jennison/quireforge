# Provider Capability Registry Contracts

Status: source-complete implementation milestone within the active
[Provider-Neutral AI Foundation](GOAL_PROVIDER_NEUTRAL_AI_FOUNDATION.md). This
milestone implements static, private, local capability metadata only. It creates
no provider route, network behavior, credential custody, context transmission,
inference, persistence, command, bridge, UI, package, or release.

## Implemented boundary

`apps/desktop/src-tauri/src/provider_capability_registry.rs` is a private Rust
module registered only through `lib.rs`. It supplies closed, Serde-validated
contracts for fictional provider, endpoint/deployment, model, local-runtime,
adapter-identity, capability-definition, and capability-claim descriptors.

Every descriptor uses opaque UUIDv7 identity, schema/descriptor versions, and
deterministic SHA-256 canonical digests. The contracts model provenance/trust,
lifecycle and compatibility states, a closed capability namespace, support
states, qualifiers, limits, and namespaced provider-specific extensions. Model
identity remains endpoint-aware even where display labels match.

Fixtures are compiled-in, fictional, non-executable, and non-operational. The
module contains no endpoint URL, provider session, account binding, credential
reference, context payload, invocation request, generic payload, dispatch
method, transport, process, environment, filesystem, browser, connector, MCP,
or persistence path.

## Validation and safeguards

Focused Rust tests prove deterministic digests, drift rejection, UUID/version/
digest/limit rejection, unknown field and enum rejection, support-state
distinction, extension namespace enforcement, and endpoint-aware same-name model
identity. `scripts/validate_repository.py` now independently guards this module
against prohibited authority imports and exact authority-bearing markers while
requiring the static fixture, strict fixture parser, unknown-field denial, claim
digest, and extension-validation guards.

The existing M57 LocalMock connector foundation remains unchanged. Its separate
guard continues to protect mock connector authority; this registry neither
reclassifies that mock nor creates an adapter or provider route.

## Persistence, bridge, UI, and release policy

There is no SQLite migration or persistence decision. There are no Tauri
commands, frontend bridge types/Zod schemas, or UI. Beta.54 remains the latest
packaged generation. This milestone is source-only; packaging, installed-host
validation, version selection, tag, release, publication, and deployment remain
separate future decisions.

## Deferred boundaries

No provider/vendor is selected. Networking, discovery, model invocation,
inference, credential storage/custody/OAuth/account connection, context
assembly/transmission, retrieval/citations/indexing/M55 admission, provider
sessions, native tools, shell, terminal, filesystem, Git, browser/M58,
connectors, MCP, cloud/deployment, mutation, automation, and multi-agent
behavior remain excluded.

## Next recommendation

The next separately approved implementation milestone is **Canonical
Interaction/Event Contracts and Deterministic Mock Adapter Conformance**. It
must remain local and non-networked, use deterministic fixtures, and implement
no credential, context-transmission, invocation, or native-operation route.
