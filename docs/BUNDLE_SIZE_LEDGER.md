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
| Post-M39 corrective checkpoint candidate | 0.1.0-beta.34 | pending final corrective commit | 933,535 | 105,156 | JS 1,310,720 / CSS 137,216 | Measured candidate; no preceding integrated-package total recorded | Closed workspace-boundary acknowledgement and its focused browser coverage; no ceiling increase. |
| M34 candidate | 0.1.0-beta.29 | pending local commit | 917,900 | 105,156 | JS 1,310,720 / CSS 137,216 | Measured candidate baseline; previous JS ceiling 917,504 bytes | Accessible workspace selector and current UI expansion period. The preceding integrated package's measured bundle totals must be recorded from its package evidence before any package-to-package delta is claimed. |
