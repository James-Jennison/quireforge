# Milestone 37 — Advisor/Approval/Dispatch/Execution End-to-End Acceptance Gate

## Scope and result

M37 is an evidence-only, user-authorized human-in-the-loop acceptance gate. It
does not change product code, dependencies, package identity, release artifacts,
or deployment state.

The gate confirms the existing separation between the transient, no-project
Advisor conversation; the digest-bound Approval/Dispatch controller; the
one-time managed-Codex execution handoff; and the bounded completion-report
surface. It adds no authority, automatic retry or redispatch, transcript
retention, or destructive project action.

## Deterministic evidence

- Native Advisor contract tests confirm explicit approval, digest integrity,
  context ownership, opaque dispatch receipts, and rejection of raw paths or
  non-explicit dispatches.
- Native Advisor app-server tests confirm a fixed no-project, read-only,
  no-network, no-tool profile; bounded text-only attachment transport; and
  fail-closed handling of unexpected authority requests.
- Frontend tests confirm the one-time dispatch control is available only after
  an approved draft and dispatches through the supplied execution boundary.
- Mode-reset tests clear transient Advisor composer and attachment state; the
  desktop/mobile browser suite verifies the capability-boundary confirmation
  and no-transfer copy.
- The complete repository gate passed: repository/package contracts, type
  checks, lint, formatting, 255 desktop and 7 website tests, production builds
  and distribution budgets, Clippy, and 247 runnable Rust tests (three existing
  manual probes intentionally ignored).

## Managed-Codex acceptance

The user authorized use of the existing managed ChatGPT sign-in without
collecting or inspecting credentials. Only documented local app-server
interfaces were used, and recorded evidence is limited to terminal state and
profile flags.

- A strict Advisor turn completed with `cwd: null`, `approvalPolicy: never`, a
  read-only sandbox, network disabled, and no authority request.
- A strict Advisor turn was interrupted after an active streaming event; a
  subsequent strict Advisor turn completed, proving recovery without retained
  project context.
- A fresh disposable directory was bound to one execution-profile turn with
  `approvalPolicy: untrusted`, a read-only sandbox, and network disabled. It
  completed without an authority request or project modification. The empty
  temporary directory was removed after the check.

These live checks complement rather than replace the deterministic QuireForge
approval-binding and one-time-dispatch tests. No prompt, reply, thread ID, turn
ID, project path, account data, credential, terminal output, or transcript was
recorded in this evidence.

## Boundaries preserved

- Advisor did not gain project, shell, terminal, Git, write, dispatch,
  credential, provider, or network authority.
- Execution remained a separate read-only/untrusted managed-Codex profile and
  did not return a transcript or terminal stream to Advisor.
- Approval remained explicit, digest-bound, expiring, and one-use; changing a
  bound input requires a new approval.
- No package was rebuilt, released, published, or deployed for M37.
