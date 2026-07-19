# Milestone Forecasts

Forecasts use ranges because model latency, downloads, failures, approvals, and
external services vary. Each future milestone receives a preliminary forecast
before manual model selection and a calibrated forecast after the selected
model, current system state, caches, and any approved preflight are verified.

Approval waiting time is reported separately from active Codex and local
command time. Performance measurements come from required project work; caches
are not deleted merely to create benchmarks.

## Milestone 3 — Desktop Scaffold Consolidation

| Field | Record |
|---|---|
| Forecast date | 2026-07-19 |
| Recommended model | GPT-5.6 Sol, High reasoning |
| Preliminary forecast | 18–30 active engineering hours; low confidence |
| Calibrated forecast | Not produced; system-calibrated forecasting was introduced during final validation |
| Observed active execution | Approximately 35–55 minutes across two active work periods; approval delay excluded and exact end-to-end time not instrumented |
| User approval/waiting time | Present for milestone confirmation and sudo prerequisite installation; not measured reliably |
| Sessions | Two active work periods separated by the host-prerequisite checkpoint |
| Usage intensity | High model/tool activity; moderate local CPU use during cold Rust builds |
| Completion status | Complete locally; not pushed, merged, packaged, or released |

The preliminary forecast substantially overestimated implementation time. It
was created before repository-native Rust measurements existed and treated the
large roadmap label as conventional manual engineering time. The actual work
benefited from a narrow scaffold scope, strong existing architecture contracts,
small deterministic fixtures, fast frontend tooling, sufficient system memory,
and effective dependency/compiler caches. A percentage accuracy claim would be
misleading because active time and approval waiting were not instrumented from
the start.

### Required build and test observations

- Workspace dependency installation completed in about five seconds.
- The first successful native check took about 37 seconds.
- The first Rust test-profile build took about 44 seconds.
- The first unbundled release build took about 1 minute 18 seconds.
- Warm native validation fell to about five seconds in aggregate.
- The combined responsive/accessibility suites completed in about eight seconds.

### Unexpected work

- Tauri's Linux development headers required an owner-entered sudo operation.
- The light-theme accessibility check exposed contrast failures that were fixed
  before completion.
- Tauri generated local capability schemas that needed an explicit ignore rule.
- Cross-language contract drift was controlled with one shared JSON fixture and
  runtime TypeScript validation rather than adding a code-generation system.

### Forecasting lessons

- Separate model-driven implementation time from cold compiler/linker time.
- Use the measured warm Cargo gate when forecasting iterative adapter work.
- Reserve cold release builds for acceptance gates rather than ordinary edits.
- Keep approval-bound system dependencies outside active execution ranges.
- Reassess after the first real Codex adapter contract fixture; Milestone 4 has
  materially greater protocol and process-lifecycle uncertainty than the shell.

## Milestone 4 — Codex Process Adapter and Contracts

Status: forecast pending. Before any Milestone 4 implementation, inspect current
models and the post-Milestone 3 repository, produce the required preliminary
forecast and work breakdown, request manual model confirmation, verify the
selection and current system state, then issue the calibrated start report and
wait for separate approval.
