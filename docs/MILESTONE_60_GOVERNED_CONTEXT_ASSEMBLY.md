# M60 — Governed Context Assembly Vertical Slice

Status: released as `v0.1.0-beta.63`. This milestone implements
only the fictional, local-only sink ratified by
[M59](MILESTONE_59_CONTEXT_ASSEMBLY_AND_TRANSMISSION_CONTRACT.md). It does not
select or contact a provider, use a credential, perform inference or network
transmission, dispatch a connector/browser/MCP operation, automate work, or
grant filesystem, shell, Git, deployment, or mutation authority.

## Delivered boundary

M60 adds a native-owned, project/task-bound context-bundle lifecycle. Nothing
is selected by default. A user may explicitly select a bounded user instruction,
active M55 durable text source, selected current task plan, approved local
review evidence, or bounded project/task metadata. Native code validates
ownership and task compatibility, canonicalizes UTF-8 text to NFC with LF line
endings, structurally redacts prohibited sensitive material, applies fixed
bounds, length-frames every field, and records SHA-256 digests.

The prepared bundle is immutable, private, and retained for no more than 30
minutes while awaiting review. The UI shows only bounded attribution, size,
redaction/truncation, expiry, audit state, and fictional-local-only scope; it
does not display canonical bytes, raw source references, secrets, paths, or
ordinary internal identifiers. Review acknowledgement and a digest-bound,
expiring one-use confirmation are required before the deterministic fictional
sink can reach one of accepted, rejected, timed-out, or ambiguous terminal
outcomes. There is no automatic retry. Cancellation, revocation, expiry,
replay, storage failure, and restart fail closed and clear retained bytes.

Migration 26 stores immutable bundle metadata, item attribution, private
prepared bytes, and content-free audit linkage. Startup recovery expires every
unconsumed prepared, open-review, or awaiting-confirmation bundle rather than
reconstructing authority from a prior process.

## Verification scope

The M60 source suite covers strict bridge parsing, deterministic canonical
assembly, redaction, hostile length-framed evidence, bounds, project/task
ownership, review/confirmation transitions, replay, cancellation/revocation,
timeout/ambiguity without retry, migration and restart recovery. Regression
and release acceptance additionally require the repository, desktop browser,
pinned Ubuntu 22.04 packaging, artifact, installed-host, and public-release
gates recorded for beta.63.

## Final release evidence

`v0.1.0-beta.63` is an annotated tag for source commit
`8ee92d58052c209d76233c40a3be12a58e501e0c`. Two independent clean pinned
Ubuntu 22.04 container builds produced byte-identical canonical assets:

- `quireforge_0.1.0.beta.63_amd64.deb` — SHA-256
  `84f059ac08912f36f8d5c38d0c1145f3b11a248317cc08875e73fab8bbe4316c`.
- `quireforge-sandboxd_0.1.0.beta.63_amd64.deb` — SHA-256
  `1e94a2ab466edcd030f05bee167f039f90ae88c2069396fa4bba07fced519291`.

The canonical four-file set (the two Debian packages, `SHA256SUMS`, and
`release-manifest.json`) passed provenance, checksum, lifecycle, smoke,
visible-launch, AppStream, and GLIBC validation. Installed-host acceptance
passed with a digest-bound receipt, package integrity verification, and the
sandbox worker disabled and inactive. The GitHub prerelease was published only
after independent download and byte comparison of all four assets.

## Explicit exclusions

M60 does not add M55 source admission, M57 connector authority, M58 browser
authority, real provider selection or transmission, provider sessions,
credentials, retrieval, inference, model output handling, tools, MCP,
automation, external mutation, website activation, or deployment. Those remain
separate milestones and approvals.
