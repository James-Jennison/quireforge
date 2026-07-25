# Milestone 23 — UI Platform Feasibility Decision

Status: complete decision evidence on
`docs/milestone-23-ui-platform-feasibility`; implementation requires separate
James approval.

## Decision

**Retain Tauri conditionally and reconsider Qt when defined triggers occur.**

Tauri + React + TypeScript best preserves QuireForge's verified Linux product
surface, explicit native safety boundary, accessible browser evidence, and
pinned Ubuntu 22.04 packaging path. Qt 6 has credible Linux-native strengths,
but no repository evidence shows a material product limitation that outweighs
the rewrite, bridge, regression, accessibility, and distribution work. This is
not a rejection of Qt; it is a decision to defer migration until measured
evidence establishes a problem Tauri cannot reasonably solve.

**Confidence: medium-high.** The current architecture and completed validation
are directly inspected. Qt's prospective benefits are supported by primary
platform documentation, but the actual fit of a Rust/Qt bridge, Qt Quick
accessibility, terminal behavior, and packaging for this application remain
unknown without a separately approved prototype.

## Scope and evidence rules

This was a read-only documentation milestone. It did not create a Qt or Tauri
prototype, alter source/dependencies/builds/packages, run an application build,
or change a release, merge, or remote account.

Each material statement below is marked as:

- **Verified repository fact** — inspected in this repository at
  `db1b729b8324b6f12952b3fe627e03a0457902ba`.
- **Primary-source platform fact** — stated in linked primary documentation.
- **Reasoned inference** — a conclusion from the preceding evidence.
- **Unknown** — requires a later approved prototype or platform test.

External sources were retrieved 2026-07-25:

- [Tauri architecture](https://v2.tauri.app/concept/architecture/),
  [commands](https://v2.tauri.app/develop/calling-rust/),
  [permissions](https://v2.tauri.app/security/permissions/), and
  [distribution](https://v2.tauri.app/distribute/).
- [Qt Quick](https://doc.qt.io/qt-6/qtquick-index.html),
  [Qt accessibility](https://doc.qt.io/qt-6/accessible.html),
  [Qt Test](https://doc.qt.io/qt-6/qttest-index.html),
  [Qt for Linux](https://doc.qt.io/qt-6/linux.html),
  [Qt for Windows](https://doc.qt.io/qt-6/windows.html),
  [Qt for macOS](https://doc.qt.io/qt-6/macos.html), and
  [Qt Wayland](https://doc.qt.io/qt-6/wayland-and-qt.html).
- [CXX-Qt bridge documentation](https://kdab.github.io/cxx-qt/book/), the
  primary documentation of a possible Rust/Qt bridge, not Qt Company
  documentation.

## Current boundary map

| Layer                           | Current responsibility                                                                                                                                                                                                    | Evidence class                                                                                                      | Preserve under Qt?                                                                                                     |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| React + TypeScript presentation | Hash routing; persistent shell; workspace and inspector composition; responsive navigation; local UI state; semantic controls; Zod validation of bridge inputs and snapshots; Playwright/axe visual evidence.             | Verified repository fact: `apps/desktop/src/App.tsx`, `workspaceNavigation.ts`, `src/lib/bridge.ts`, desktop tests. | No; rewrite views, routing/state composition, component tests, Playwright assumptions, and web accessibility evidence. |
| Typed application contract      | Closed command names; normalized snapshots; opaque identifiers/tokens; schemas and fixtures shared across Rust and TypeScript.                                                                                            | Verified repository fact: `src/lib/bridge.ts`, `src-tauri/src/contract.rs`, architecture.                           | Yes, as a UI-neutral contract specification; Qt bindings/DTO conversion still need implementation and tests.           |
| Rust services                   | Codex/auth/usage/conversations; SQLite project metadata; directory identity; Git/worktrees; PTY terminal; file preview and attachments; integration controls; policy, redaction, confirmation, and lifecycle enforcement. | Verified repository fact: `src-tauri/src/{codex,project,git,worktree,terminal,preview,attachment}.rs`.              | Largely, subject to extracting Tauri types and UI delivery from service-facing code.                                   |
| Tauri façade and adapters       | Command registration/state ownership; webview IPC; dialog, notification, opener plugins; app-data/window setup; bundle configuration; Linux GTK native-drop capture.                                                      | Verified repository fact: `src-tauri/src/lib.rs`, `desktop.rs`, `tauri.conf.json`, `capabilities/main.json`.        | No; replace with a Qt application shell and platform adapters.                                                         |
| Distribution/test toolchain     | Vite/TypeScript production build, Tauri `.deb`/AppImage configuration, pinned Ubuntu 22.04 package workflow; Vitest and Playwright/axe coverage.                                                                          | Verified repository fact: `package.json`, `Cargo.toml`, `docs/BUILDING.md`, `docs/TESTING.md`, `docs/RELEASING.md`. | Rust tests remain; frontend, UI automation, packaging, budgets, and CI need replacement or major revision.             |

### Smallest safe reusable-core boundary

**Reasoned inference:** retain the existing Rust domain services, normalized
domain snapshots, request validators, confirmation state machines, persistence,
and process ownership behind a UI-neutral Rust API. Put all UI delivery behind
adapters: Tauri commands/events today; Qt-facing QObject/QML or C++/Rust bridge
tomorrow. Keep platform operations (dialog, notification, opener, native drop,
window lifecycle, package metadata) outside the core.

This is a target boundary, not a claim that it already exists as a standalone
crate. The current services are modules inside the Tauri crate and several
commands take Tauri state or plugin handles. Extracting a core would itself be
planned implementation work and must preserve the current closed-input,
opaque-token, and fail-closed behavior.

### Platform-specific façade

**Verified repository fact:** the current façade is explicitly Linux scoped:
the capability is Linux-only and empty, the bundle targets `.deb` and AppImage,
and GTK captures one native drop path while Tauri plugins own dialogs,
notifications, and default-app opening. **Primary-source platform fact:** Tauri
commands provide asynchronous Rust calls and its permissions/capabilities grant
explicit command privileges. Qt Quick is a QML-based rendering and UI system;
Qt supports native platform accessibility APIs and can operate as a Wayland
client. **Reasoned inference:** Qt would replace the façade, not merely the
visual renderer, and therefore requires equivalent controls for lifecycle,
native handoff, permissions, and error boundaries.

## Required decision questions

1. **React/TypeScript responsibility:** **Verified repository fact.** It owns
   all workspace presentation, hash routing, mounted-view state, responsive
   shell/drawer/inspector behavior, local interaction state, semantic UI, and
   strict bridge-side Zod validation.
2. **Tauri responsibility:** **Verified repository fact.** It owns the window,
   command dispatch and managed service state, security configuration,
   plugins, application paths, bundle metadata, and Linux native-drop setup.
3. **Reusable Rust responsibility:** **Verified repository fact.** Rust owns
   the safety-sensitive domain services, persistence, process/PTY lifecycle,
   path identity, redaction, policy, preview/attachment validation, Git and
   worktree controls, and normalized Codex adaptation.
4. **Platform-specific native façade:** **Verified repository fact.** Tauri
   plugin wiring, window and WebView security, dialog/opener/notification
   delivery, GTK drop capture, capabilities, and package configuration are
   platform-adapter work.
5. **Smallest safe boundary:** **Reasoned inference.** A UI-neutral Rust core
   exposing closed application DTOs and operations; thin Tauri or Qt adapters
   own UI transport and native desktop facilities.
6. **Preservation:** **Verified repository fact / reasoned inference.** Keeping
   Tauri preserves the currently verified routed workspace, 172 desktop tests,
   38 browser scenarios, asset budgets, and Ubuntu package workflow. Qt can
   preserve domain services only after intentional extraction; it rewrites
   presentation, transport, test, and package layers.
7. **Qt risks:** **Primary-source platform fact / reasoned inference.** Qt
   Quick requires QML; CXX-Qt uses generated C++/Rust bridge code rather than
   direct idiomatic one-to-one bindings. The bridge, QML lifecycle, async
   ownership, error conversion, and build system introduce new integration
   work. No repository evidence verifies parity for this app.
8. **Tauri limitations:** **Verified repository fact / reasoned inference.**
   The UI remains WebView based and depends on WebKitGTK on Linux; a bespoke
   native-control experience would require continued web presentation work.
   There is no measured current rendering, memory, accessibility, or native
   integration failure that establishes this as a product blocker.
9. **Product alignment:** **Reasoned inference.** Tauri's established closed
   Rust command façade directly supports supervised autonomy, approval
   boundaries, structured evidence, and continuity. Qt could do so, but would
   first need to re-establish those guarantees.
10. **Performance:** **Verified repository fact.** 22B recorded a 99.03 KiB
    CSS bundle and verified packaging, but it did not benchmark startup, idle
    memory, rendering, large projects, or streaming throughput. **Unknown:**
    which platform is faster for QuireForge's representative workloads.

## Option comparison

### Option A — Retain Tauri

- **Benefits:** preserves verified workflows, the command/capability model,
  Rust services, React expertise, Playwright/axe evidence, and the pinned
  `.deb`/AppImage process. **Verified repository fact / primary-source platform
  fact.** Tauri is designed around Rust plus a WebView and platform-specific
  installers.
- **Disadvantages:** remains dependent on a WebView and its Linux runtime;
  application-native control styling and platform behavior need continued
  deliberate work. **Reasoned inference.**
- **Risks:** WebView/GTK regressions and continued cross-language contract
  maintenance. **Verified repository fact / reasoned inference.**
- **Retained/discarded:** retains nearly all current work; discards none.
- **Approximate effort:** 0 migration weeks; normal future milestones only.
- **Roadmap/reversibility:** least disruptive and fully reversible through a
  later, approved feasibility trigger.
- **Safeguards:** retain closed contracts, add measurements before attributing
  UX issues to platform choice, and keep core extraction incremental.

### Option B — Full Qt 6 migration

- **Benefits:** Qt supports QML/Qt Quick, keyboard navigation, platform
  accessibility APIs, Wayland client operation, and documented Linux, Windows,
  and macOS targets. **Primary-source platform fact.**
- **Disadvantages:** replaces the application shell and all React presentation;
  Qt Quick requires QML, while CXX-Qt introduces code generation and a
  C++/Rust bridge. **Primary-source platform fact.**
- **Risks:** feature-parity loss, duplicate live development, bridge lifetime
  or async errors, accessibility/automation regression, package/CI rewrite,
  and loss of Web-based visual evidence. **Reasoned inference.**
- **Retained/discarded:** preserve Rust domain logic only after extraction;
  rewrite Tauri façade, React UI, bridge, component tests, Playwright
  assumptions, distribution budgets, and package flow. **Verified repository
  fact / reasoned inference.**
- **Approximate effort:** 28–52 active engineering weeks for parity and
  stabilization, assuming one experienced engineer, no new product scope, and
  an approved adapter/core-extraction phase. This range is a **reasoned
  inference**, not a benchmark: it includes 5–9 weeks architecture/bridge,
  12–22 weeks UI reconstruction, 5–10 weeks test/accessibility replacement,
  and 6–11 weeks packaging/CI/stabilization, with overlap but no parallel
  feature development.
- **Roadmap/reversibility:** delays meaningful product work and is hard to
  reverse after feature development moves to Qt.
- **Safeguards:** require a separately approved prototype with measurable
  acceptance gates before any migration decision; never run two feature
  frontends in parallel.

### Option C — Deferred conditional migration

- **Benefits:** preserves Option A now while making the Qt decision falsifiable
  and measured rather than ideological. **Reasoned inference.**
- **Disadvantages:** does not immediately pursue a native-Qt UI and requires
  discipline to collect trigger evidence. **Reasoned inference.**
- **Risks:** deferred work can become stale; mitigate with explicit review
  triggers and an ADR re-open rule.
- **Retained/discarded:** same preservation as Option A; no migration work is
  discarded because none begins.
- **Approximate effort:** 1–2 active weeks to define representative benchmarks
  and an extraction proposal only after James approves a follow-on milestone;
  0 weeks are authorized by this decision itself. **Reasoned inference.**
- **Roadmap/reversibility:** maximally reversible and does not delay approved
  product progress.
- **Safeguards:** do not interpret host-installed Qt tooling as feasibility;
  require a narrow, separately authorized prototype only after triggers.

## Weighted decision matrix

Scores are 1 (poor) through 5 (strong). Weights total 100 and emphasize the
approved Linux-first product, continuity, controlled development, and evidence
rather than generic native-UI preference.

| Criterion                                             |  Weight | Why this weight                                         | A: retain Tauri | B: Qt migration | C: defer conditionally | Evidence and confidence                                                                       |
| ----------------------------------------------------- | ------: | ------------------------------------------------------- | --------------: | --------------: | ---------------------: | --------------------------------------------------------------------------------------------- |
| Product alignment and supervised boundaries           |      15 | Core product promise                                    |               5 |               3 |                      5 | Current façade is verified; Qt parity inferred. High / medium.                                |
| Existing capability continuity                        |      15 | Existing user-visible safety work must not be discarded |               5 |               2 |                      5 | Repository fact. High.                                                                        |
| Linux desktop quality                                 |      12 | Linux is primary, but no current defect is measured     |             3.5 |               5 |                    3.5 | Qt docs support; app-specific outcome unknown. Medium.                                        |
| Accessibility and testability                         |      13 | Explicit product and completion gate                    |             4.5 |               3 |                    4.5 | Playwright/axe verified; Qt capabilities documented, parity unknown. High / medium.           |
| Security and approval boundary                        |      10 | Core safety requirement                                 |             4.5 |             3.5 |                    4.5 | Tauri capability/contract verified; Qt architecture inferred. High / medium.                  |
| Packaging reliability                                 |      10 | Fresh Ubuntu package evidence exists                    |             4.5 |             2.5 |                    4.5 | Verified current workflow; Qt packaging unknown here. High / low.                             |
| Maintenance and contributor access                    |      10 | Sustainable progress matters                            |               4 |             2.5 |                      4 | Existing stack/tooling verified; Qt/QML/C++ bridge adds skills. Medium.                       |
| Future Windows/macOS viability and decision readiness |       5 | Important, not immediate Linux requirement              |               4 |             4.5 |                    4.5 | Both platforms document support; Option C adds an explicit evidence gate. Medium.             |
| Performance potential                                 |       5 | Must not outweigh unmeasured evidence                   |             3.5 |               4 |                    3.5 | Platform characteristics only; no app benchmark. Low.                                         |
| Migration risk and time to progress                   |       5 | Prevent disruption from dominating roadmap              |               5 |               1 |                    4.5 | Rewrite scope inferred from inspected ownership. Medium.                                      |
| **Weighted result / 5**                               | **100** |                                                         |        **4.43** |        **3.07** |               **4.48** | Option C wins narrowly because it retains A's evidence while making reconsideration explicit. |

## Platform implications

### Linux

**Verified repository fact:** Tauri currently packages x86_64 `.deb` and
AppImage through a digest-pinned Ubuntu 22.04 workflow; its final 22B candidate
requires GLIBC 2.34 and passed Debian lifecycle and visible launch checks.
**Primary-source platform fact:** Qt documents Ubuntu 22.04 x86_64 support and
Wayland-client support. **Unknown:** whether Qt reduces QuireForge's actual
Wayland/X11, theme, picker, notification, startup, resource, or accessibility
issues; this repository has no current comparative defect measurement.

### Windows and macOS

**Primary-source platform fact:** Tauri documents platform installers, while
Qt documents Windows and macOS configurations; Qt macOS deployment is tied to
an Xcode/SDK deployment target. **Reasoned inference:** either choice needs
new platform adapters and distribution/signing work because QuireForge's
current capability and bundle configuration are Linux-only. Cross-platform
support is not a justification for migration without a funded product need.

### Testing and accessibility

**Verified repository fact:** current evidence uses Rust tests, TypeScript and
component tests, and Playwright/axe desktop/mobile scenarios. **Primary-source
platform fact:** Qt Test and Qt Quick Test exist; Qt provides keyboard focus,
platform accessibility APIs, and system palette/font integration. **Reasoned
inference:** migration replaces, rather than automatically carries, browser
automation and axe coverage; native UI automation and assistive-technology
verification would need a new reproducible gate.

### Packaging, performance, and resource use

**Verified repository fact:** current release tooling produces and validates
`.deb` and AppImage candidates on a pinned Ubuntu 22.04 image. **Primary-source
platform fact:** Tauri bundles platform-specific installers and Qt documents
platform deployment environments. **Unknown:** relative artifact size, startup,
idle memory, rendering, terminal streaming, and large-project responsiveness.
No benchmark numbers are asserted.

## Reconsideration triggers

Re-open ADR 0028 only if one or more conditions are met and documented with
reproducible evidence:

1. A verified Linux accessibility, Wayland/X11, native-dialog, notification,
   terminal, or WebView limitation blocks a required approved workflow and has
   no reasonable Tauri/React remediation.
2. Representative release measurements show a material, repeatable startup,
   memory, rendering, or streaming failure against an agreed acceptance target,
   after normal application optimization.
3. A funded, approved Windows or macOS requirement cannot meet its required
   native integration, packaging, signing, or accessibility target with Tauri.
4. Maintained Tauri/WebKitGTK dependencies cease to support the required Linux
   baseline or create an unmitigable security/support burden.
5. A separately approved, narrow Qt feasibility prototype proves feature- and
   accessibility-relevant parity for the selected core boundary, bridge,
   terminal, dialogs/notifications, package flow, and automation with a
   credible migration estimate at least 25% lower than this assessment's
   midpoint.

Host-installed Qt tooling, aesthetic preference, or an unmeasured claim of
"native performance" are not triggers.

## Follow-on only after approval

If James approves this recommendation, the next implementation milestone may
be **platform-neutral core-boundary hardening and representative measurement**:
define closed Rust DTOs, measure representative Tauri workloads, and preserve
existing command safety guarantees. It must not start a Qt migration or add Qt
dependencies. A Qt prototype, if a trigger is met, requires a separate explicit
approval and acceptance plan.

## Handoff

- **Decision evidence base:** `db1b729b8324b6f12952b3fe627e03a0457902ba`
  (final 22B evidence); implementation baseline
  `6e880672b3faca50170f56fc67e187337d1b11d9`.
- **Decision:** retain Tauri conditionally; do not migrate now.
- **No application or package validation was run:** Milestone 23 changed only
  documentation and relies on the verified 22B baseline plus targeted source
  inspection.
- **Next action:** James decides whether to accept ADR 0028; no implementation
  begins from this document alone.
