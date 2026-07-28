# Post-beta.38 Temporary Bundle Construction-Envelope Checkpoint

## Purpose

This packaging-only checkpoint replaces repeated reactive application-shell
budget increases during the approved M44–M50 Advisor/QuireForge UI-construction
period. It does not add product functionality, dependencies, authority,
transport, providers, connectors, file handling, or release/deployment scope.

## Measured baseline and enforced limits

The verified beta.38 production desktop output measured:

| Asset class | Measured bytes | Previous ceiling | Previous headroom |
| --- | ---: | ---: | ---: |
| Startup entry | 194,943 | 262,144 | 67,201 (34.5%) |
| Application shell | 309,489 | 327,680 | 18,191 (5.9%) |
| Total JavaScript | 941,807 | 1,310,720 | 368,913 (39.2%) |
| Total CSS | 108,515 | 137,216 | 28,701 (26.5%) |

Only the application shell lacked a stable construction-period margin. The
checkpoint sets that limit to 393,216 bytes (384 KiB): a 64-KiB allocation
increase and 83,727 bytes (27.1%) above the verified shell baseline. The
result is bounded, remains substantially below the total-JavaScript limit, and
matches the already documented temporary CSS policy's roughly 30% construction
margin without changing the other independently adequate limits.

The startup-entry, total-JavaScript, and CSS ceilings remain exactly 256 KiB,
1,280 KiB, and 134 KiB. Their limits were not relaxed.

## Policy contract

- `apps/desktop/scripts/bundle-budget.json` is the closed source for the four
  limits; `validate-dist.mjs` reports totals and largest assets and fails on an
  exceeded limit.
- The package-contract test asserts the exact bounded envelope. No automatic
  ceiling increase exists.
- The envelope covers only the approved M44–M50 UI work: unified attachment
  presentation, a separately approved multi-attachment sequence, artifact
  review/save, typed review panes, and presentation-only workbench refinement.
- A fresh measured case and separate approval are required for any increase.
- The existing post-workbench reconciliation gate remains mandatory. It must
  analyze route/chunk and stylesheet costs, lazy loading, duplicated code/CSS,
  and establish strict evidence-based permanent budgets before product
  readiness.

## Package and evidence boundary

The checkpoint uses the unique `0.1.0-beta.39` package identity. Its Debian
release evidence must be bound to the clean policy commit and include the
measured JavaScript/CSS totals, provenance, checksums, ABI, lifecycle,
installed-smoke, and visible-launch results. M44 and later provisional package
identities shift by one without changing their product scope.
