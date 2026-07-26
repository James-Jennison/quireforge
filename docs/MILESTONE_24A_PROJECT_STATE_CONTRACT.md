# Milestone 24A — Project State Contract

Status: complete. Final implementation commit
`f62ba5c68fe0002d3d3f6b5faa0bd2d522d81f0d` is verified by the fresh pinned
Ubuntu 22.04 package evidence recorded below.

## Purpose

Version 1 defines a strict, machine-readable representation of project truth
without reading repositories, parsing Markdown, showing a workspace, generating
handoffs, or changing project state. Markdown remains evidence for people; it
is not the sole durable model.

## Canonical contract

Rust Serde types in `apps/desktop/src-tauri/src/project_state.rs` are the
authoritative wire contract. `apps/desktop/src/lib/projectState.ts` mirrors the
closed JSON shape with Zod using existing dependencies. Shared fixtures in
`apps/desktop/fixtures/project-state.json` prove Rust/TypeScript interpretation
of active, pushed, paused, completed, missing-evidence, and contradictory
states.

`schemaVersion: 1` is required. Unknown fields and unknown security-sensitive
enum values are rejected. A future version is rejected; a later migration owner
must explicitly add a compatible reader or migration. No generic migration
framework is introduced. Rust and TypeScript tests both parse the same fixtures,
and Rust round-trips every representative project-state variant.

## Trust, provenance, and ownership

Every material aggregate carries provenance with `verified`, `reported`,
`inferred`, or `unknown` trust and optional source type/reference/commit,
observed/verified timestamps, and note. Verified Git facts cannot be replaced
by an agent report; inferred recommendations remain distinct from approvals.

- User-approved policy and boundaries are canonical policy, but their long-term
  persistence location is intentionally deferred by approved decision.
- Git, documentation, validation reports, and package manifests are derived
  evidence owned by their source systems.
- QuireForge application storage owns local project metadata only.
- Agent-session state is transient reported evidence.

No credentials, raw tokens, secret material, raw paths, or transcripts belong
in the contract.

## Deferred work

24B may read and normalize sources. 24C may display the contract. 24D may
generate handoffs and record contradictions. None is implemented here.

## Final validation and package evidence

The contract is library code compiled into the desktop application. Fresh
candidates were therefore built from the clean final implementation commit
through the pinned Ubuntu 22.04 container
`ubuntu:22.04@sha256:0e0a0fc6d18feda9db1590da249ac93e8d5abfea8f4c3c0c849ce512b5ef8982`.
The ignored evidence is at:

- `target/ubuntu-22.04/release/packages/release-manifest.json`;
- `target/ubuntu-22.04/release/packages/SHA256SUMS`;
- `target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.2_amd64.deb` —
  4,475,272 bytes; SHA-256
  `ce9a854e34964b57f125bdb723266023ef80e625b6d9e2fb56ab87f73dd02fc5`;
- `target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.2-x86_64.AppImage`
  — 83,655,160 bytes; SHA-256
  `933c09194720f12cc8ed991d4261f6acb0254be7f4241792d23548fc0564416e`.

The packaged executable requires maximum GLIBC `2.34` (within the Ubuntu 22.04
`2.35` policy baseline). The pinned workflow passed manifest/checksum checks,
Ubuntu 22.04 compatibility, desktop-entry and icon validation, disposable
Debian install/upgrade/remove, installed-Debian and AppImage X11 launches, and
the installed-app smoke test. Install locally with
`sudo apt install ./target/ubuntu-22.04/release/packages/quireforge_0.1.0.beta.2_amd64.deb`.
Launch with
`./target/ubuntu-22.04/release/packages/QuireForge-0.1.0-beta.2-x86_64.AppImage`.

The direct host rerun of the smoke phase is intentionally not substituted for
the pinned result: this host lacks `xvfb-run`; the authoritative container
provides it and completed the same lifecycle and launch checks.
