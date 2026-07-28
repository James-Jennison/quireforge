# Post-beta.38 Temporary Bundle Construction-Envelope Checkpoint

## Purpose

This packaging-only checkpoint replaces repeated reactive bundle-budget changes
during the approved M44–M58 Advisor/QuireForge construction period. It does not
add product functionality, dependencies, authority, transport, providers,
connectors, file handling, or release/deployment scope.

## Measured baseline and enforced limits

The verified beta.38 production desktop output measured:

| Asset class | Measured bytes | Previous ceiling | Previous headroom |
| --- | ---: | ---: | ---: |
| Startup entry | 194,943 | 262,144 | 67,201 (34.5%) |
| Application shell | 309,489 | 327,680 | 18,191 (5.9%) |
| Total JavaScript | 941,807 | 1,310,720 | 368,913 (39.2%) |
| Total CSS | 108,515 | 137,216 | 28,701 (26.5%) |

The startup entry remains 262,144 bytes (256 KiB): its 67,201-byte (34.5%)
margin is adequate because it must stay a small bootstrap boundary. The three
limits that can grow through the remaining approved UI work are set once to
binary allocation boundaries: application shell 458,752 bytes (448 KiB), total
JavaScript 1,572,864 bytes (1.5 MiB), and CSS 163,840 bytes (160 KiB).

Those limits add, respectively, 149,263 bytes (48.2%), 631,057 bytes (67.0%),
and 55,325 bytes (51.0%) above the verified beta.38 baseline. The 64-KiB shell
increment covers shared drawer, task, and artifact components; the 256-KiB
total-JavaScript increment covers separately loaded attachment, artifact,
review, durable-task, and template UI while preserving route/pane splitting;
and the 32-KiB CSS increment covers responsive workbench, review, and artifact
states. M51, M53, M55, M57, and M58 are decision-only and add no shipped bundle
by themselves; M52, M54, and M56 are the additional implementation packages
that make the total-JavaScript and CSS headroom necessary.

## Policy contract

- `apps/desktop/scripts/bundle-budget.json` is the closed source for the four
  limits; `validate-dist.mjs` reports totals and largest assets and fails on an
  exceeded limit.
- The package-contract test asserts the exact bounded envelope. No automatic
  ceiling increase exists.
- The envelope covers only the approved M44–M58 construction scope: unified
  attachment presentation, a separately approved multi-attachment sequence,
  artifact review/save, typed review panes, presentation-only workbench
  refinement, and—only if their separate proposals are approved—durable task
  records, local artifact/design review, and inspectable local templates.
- A fresh measured case and separate approval are required for any increase.
- The existing post-workbench reconciliation gate remains mandatory. It must
  analyze route/chunk and stylesheet costs, lazy loading, duplicated code/CSS,
  and establish strict evidence-based permanent budgets before product
  readiness.

## Package and evidence boundary

The preliminary beta.39 locally generated release set is superseded before it
is recorded as authoritative package evidence because it covered only M44–M50.
This corrected checkpoint uses
the unique `0.1.0-beta.40` package identity. Its Debian release evidence must
be bound to the clean policy commit and include measured JavaScript/CSS totals,
provenance, checksums, ABI, lifecycle, installed-smoke, and visible-launch
results. M44 and later provisional package identities shift by one without
changing their product scope.

## Beta.40 evidence

The authoritative pinned Ubuntu 22.04 release workflow produced the clean
`0fed7983a3f32aa79ea4d1feee9947535d370a9b` beta.40 set with unchanged measured
desktop output: 194,943-byte entry, 309,489-byte application shell, 941,807
bytes total JavaScript, and 108,515 bytes CSS. Its release manifest records
`treeState: clean`, the digest-pinned Ubuntu 22.04 builder, and highest shipped
`GLIBC_2.34`, within the required `GLIBC_2.35` ceiling.

- `quireforge_0.1.0.beta.40_amd64.deb` —
  `bdf3662530630ee8dad4d6600c77723e2541cc92823d0977babf89e3cba12fd5`
- `quireforge-sandboxd_0.1.0.beta.40_amd64.deb` —
  `5648541e28eeaa44669c8f433ba7cb3bb8b7f250268dab4b5b8e995091cd5458`

Checksum, provenance/ABI, lifecycle, container smoke, restricted installed
Debian validation, and the installed visible X11 launch all passed. No release
or deployment was performed.
