# Desktop Bundle Size Ledger

During the active Advisor and QuireForge UI-construction period, desktop
production bundles remain measured and enforced under temporary ceilings:
256 KiB startup entry, 448 KiB application shell, 1,572,864 bytes (1.5 MiB)
total JavaScript, and 163,840 bytes (160 KiB) CSS. The beta.38 baseline was
194,943 bytes entry, 309,489 bytes application shell, 941,807 bytes total
JavaScript, and 108,515 bytes CSS. The startup-entry limit remains unchanged
because it has 34.5% headroom. The 64-KiB shell, 256-KiB total-JavaScript, and
32-KiB CSS allocation increments supply measured headroom for the approved
M44–M58 UI work, including the three additional implementation packages in
M52, M54, and M56; decision-only M51, M53, M55, M57, and M58 add no package
bytes by themselves. These are not final performance targets and may not
increase without fresh measured evidence and separate approval. Every package
gate records actual totals and largest assets; a post-workspace reconciliation
gate must establish strict permanent ceilings before product readiness.

| Milestone | Version | Commit | JavaScript | CSS | Ceiling | Change | Explanation |
| --- | --- | --- | ---: | ---: | --- | --- | --- |
| M50 | 0.1.0-beta.45 | 1cc7c50ceed6d2b6c2f91274110471d71fe6292a | 958,524 | 112,146 | entry 262,144 / shell 458,752 / JS 1,572,864 / CSS 163,840 | +3,402 JS / +987 CSS from M49 | Bounded layout-preference chunk and accessible review/terminal resize controls; task surface remains dominant and no authority added. |
| M49 | 0.1.0-beta.44 | f1a44324859faa2ed43f24ab60db12b58e6c6836 | 955,122 | 111,159 | entry 262,144 / shell 458,752 / JS 1,572,864 / CSS 163,840 | +30,333 JS / +1,113 CSS from M48 | Closed, separately lazy review shell plus six pane chunks; startup entry unchanged and no authority added. |
| M48 | 0.1.0-beta.43 | 5d483d0c068c450bbc779ee07b048fe848c7e1f0 | 924,789 | 110,046 | entry 262,144 / shell 458,752 / JS 1,572,864 / CSS 163,840 | within all temporary ceilings | Native-owned generated-artifact cards and bounded text preview/save controls; no ceiling increase or authority expansion. |
| M44 | 0.1.0-beta.41 | 891abf6d953e3b7c0dd3f0d3bd03baeb29de40fb | 941,334 | 109,082 | entry 262,144 / shell 458,752 / JS 1,572,864 / CSS 163,840 | -473 JS / +567 CSS from beta.40 | One compact Advisor attachment entry and bounded type chooser reuse the existing closed native pickers; no transport, type, collection, or authority change. |
| Post-M43 preliminary envelope (superseded locally) | 0.1.0-beta.39 | 5d569f643ef6a3ba17ec7cb77bd96dad3cdbe61f | 941,807 baseline | 108,515 baseline | entry 262,144 / shell 393,216 / JS 1,310,720 / CSS 137,216 | +65,536 shell ceiling / all other ceilings unchanged | A clean locally validated package set covered only M44–M50. It was not recorded as authoritative package evidence and is superseded by beta.40 after the full M51–M58 review. |
| Post-M43 temporary bundle construction-envelope checkpoint | 0.1.0-beta.40 | 0fed7983a3f32aa79ea4d1feee9947535d370a9b | 941,807 | 108,515 | entry 262,144 / shell 458,752 / JS 1,572,864 / CSS 163,840 | 0 actual bytes from beta.38; entry unchanged; +131,072 shell / +262,144 JS / +26,624 CSS ceilings from beta.39 | Full M44–M58 envelope. The policy-only checkpoint produced no bundle growth; binary allocation boundaries preserve a small entry while allowing separately loaded task, artifact, review, and template UI. |
| Post-M41 packaging-efficiency checkpoint | 0.1.0-beta.37 | 502e56e46131c64e7821fc98b16152142ac50eff | 937,925 | 108,515 | JS 1,310,720 / CSS 137,216 | 0 JS / 0 CSS from the recorded beta.36 M41 release candidate | Release-workflow source-cache change only; no desktop UI bundle change. |
| M43 | 0.1.0-beta.38 | 6eb526bdb0b1705414f5507081dc37872358198c | 941,807 | 108,515 | JS 1,310,720 / CSS 137,216; temporary application-shell 327,680 | +3,882 JS / 0 CSS from beta.37 | Explicit transient task-handoff UI. The initial Xvfb window probe failed once without a crash, then the unchanged official lifecycle/smoke/visible-launch gate passed; no check was weakened. |
| M41 | 0.1.0-beta.36 | eee6a9ac7e3393fd7dcd73a2c4304894c70839d4 | 937,925 | 108,515 | JS 1,310,720 / CSS 137,216 | +898 JS / +1,194 CSS from the recorded beta.35 M40 release candidate | Bounded Advisor transcript/footer layout, Jump to latest, and independently scrollable details drawer; no ceiling increase. |
| M40 | 0.1.0-beta.35 | 98fa8fa26d740572095c2dcd9d4c1f579156817b | 937,027 | 107,321 | JS 1,310,720 / CSS 137,216 | +3,492 JS / +2,165 CSS from the recorded beta.34 candidate baseline | Opt-in workbench drawer, safe action palette, and existing terminal-dock presentation; no ceiling increase. |
| Post-M39 corrective checkpoint candidate | 0.1.0-beta.34 | pending final corrective commit | 933,535 | 105,156 | JS 1,310,720 / CSS 137,216 | Measured candidate; no preceding integrated-package total recorded | Closed workspace-boundary acknowledgement and its focused browser coverage; no ceiling increase. |
| M34 candidate | 0.1.0-beta.29 | pending local commit | 917,900 | 105,156 | JS 1,310,720 / CSS 137,216 | Measured candidate baseline; previous JS ceiling 917,504 bytes | Accessible workspace selector and current UI expansion period. The preceding integrated package's measured bundle totals must be recorded from its package evidence before any package-to-package delta is claimed. |
