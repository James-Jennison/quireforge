# Desktop Bundle Size Ledger

During the active Advisor and QuireForge UI-construction period, desktop
production bundles remain measured and enforced under temporary ceilings:
JavaScript 1,310,720 bytes (1,280 KiB) and CSS 137,216 bytes (134 KiB). The
CSS ceiling is 130% of the M34 baseline 105,156 bytes, rounded up from
136,703 bytes. These are not final performance targets and may not increase
without explicit approval. Every package gate records actual totals and largest
assets; a post-workspace reconciliation gate must establish strict permanent
ceilings before product readiness.

| Milestone | Version | Commit | JavaScript | CSS | Ceiling | Change | Explanation |
| --- | --- | --- | ---: | ---: | --- | --- | --- |
| Post-M41 packaging-efficiency checkpoint | 0.1.0-beta.37 | 502e56e46131c64e7821fc98b16152142ac50eff | 937,925 | 108,515 | JS 1,310,720 / CSS 137,216 | 0 JS / 0 CSS from the recorded beta.36 M41 release candidate | Release-workflow source-cache change only; no desktop UI bundle change. |
| M43 | 0.1.0-beta.38 | 6eb526bdb0b1705414f5507081dc37872358198c | 941,807 | 108,515 | JS 1,310,720 / CSS 137,216; temporary application-shell 327,680 | +3,882 JS / 0 CSS from beta.37 | Explicit transient task-handoff UI. The initial Xvfb window probe failed once without a crash, then the unchanged official lifecycle/smoke/visible-launch gate passed; no check was weakened. |
| M41 | 0.1.0-beta.36 | eee6a9ac7e3393fd7dcd73a2c4304894c70839d4 | 937,925 | 108,515 | JS 1,310,720 / CSS 137,216 | +898 JS / +1,194 CSS from the recorded beta.35 M40 release candidate | Bounded Advisor transcript/footer layout, Jump to latest, and independently scrollable details drawer; no ceiling increase. |
| M40 | 0.1.0-beta.35 | 98fa8fa26d740572095c2dcd9d4c1f579156817b | 937,027 | 107,321 | JS 1,310,720 / CSS 137,216 | +3,492 JS / +2,165 CSS from the recorded beta.34 candidate baseline | Opt-in workbench drawer, safe action palette, and existing terminal-dock presentation; no ceiling increase. |
| Post-M39 corrective checkpoint candidate | 0.1.0-beta.34 | pending final corrective commit | 933,535 | 105,156 | JS 1,310,720 / CSS 137,216 | Measured candidate; no preceding integrated-package total recorded | Closed workspace-boundary acknowledgement and its focused browser coverage; no ceiling increase. |
| M34 candidate | 0.1.0-beta.29 | pending local commit | 917,900 | 105,156 | JS 1,310,720 / CSS 137,216 | Measured candidate baseline; previous JS ceiling 917,504 bytes | Accessible workspace selector and current UI expansion period. The preceding integrated package's measured bundle totals must be recorded from its package evidence before any package-to-package delta is claimed. |
