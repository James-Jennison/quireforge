# Desktop Permanent Bundle-Budget Reconciliation

## Scope

This local, source-only reconciliation closes the temporary M44–M58 desktop
bundle construction envelope. It changes no runtime behavior, provider
boundary, packaging identity, release state, or deployment authority.

## Measurement

A clean `pnpm --filter @quireforge/desktop build` production build measured:

| Asset class | Bytes | Permanent ceiling | Headroom |
| --- | ---: | ---: | ---: |
| Startup entry | 195,014 | 262,144 (256 KiB) | 67,130 (34.4%) |
| Application shell | 239,280 | 327,680 (320 KiB) | 88,400 (37.0%) |
| Total JavaScript | 1,103,448 | 1,310,720 (1.25 MiB) | 207,272 (18.8%) |
| Total CSS | 118,484 | 147,456 (144 KiB) | 28,972 (24.5%) |

The startup entry remains small. The application shell is separately loaded
from it, the terminal renderer remains lazy, and the review/workbench panes
remain separate chunks. The measured artifact contains no external origin.

## Permanent contract

`apps/desktop/scripts/bundle-budget.json` is the closed source of truth. The
desktop distribution validator rejects an exceeded limit, a missing lazy
terminal chunk, a shell folded into the startup entry, or an external origin.
The package-contract test pins the exact values.

These limits supersede the temporary M44–M58 construction envelope. They may
not increase automatically. A future increase requires fresh measured evidence
and explicit approval; decreasing a limit remains a normal local hardening
change when the current production build proves it safe.
