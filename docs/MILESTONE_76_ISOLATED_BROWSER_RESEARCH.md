# M76 — Isolated Read-Only Browser Research

## Outcome

M76 introduces a narrow, native-owned browser-research controller. This first
accepted scope is exactly `https://google.com/` at the exact origin
`https://google.com`, approved by the owner on 2026-08-27.

## Boundary

- A launch requires an explicit, expiring, one-use owner confirmation bound to
  the project, optional task, target, origin, and observation limit.
- The adapter creates an ephemeral WebKitGTK context with JavaScript disabled.
  It receives no profile path, ambient cookie/session, credential, extension,
  upload, download, form-submission, native-tool, connector, or agent
  capability.
- HTTPS only is accepted. Redirects or any final-origin difference stop
  observation. Prompt-injection indicators, timeout, incompatibility,
  cancellation, and revocation are terminal and never retry automatically.
- Observation is capped at 2,048 bytes and reports only timestamp, byte count,
  and SHA-256 digest. No page text is persisted, transferred to M60, linked
  into M71 automatically, or supplied to an agent.
- The controller retains attempts only in process memory. Process exit removes
  all state; there is no crash recovery, retained content, external delivery,
  or scheduled work.

## Acceptance

- The `0.1.0-beta.102` follow-up candidate adds the previously missing visible
  user control: Local Chat can open, but cannot invoke, a separate Google-only
  research review. Prepare and Confirm once remain explicit user actions;
  beta.102 validation, package, and installed-host UI acceptance are pending.
- Strict native and TypeScript contracts permit only the owner-approved exact
  Google target/origin and bounded observation limit.
- The packaged Linux application exposes a separate helper with no ambient
  browser profile and verifies that JavaScript disabling is available before
  loading a target.
- Full source validation, desktop/website E2E, and the clean pinned Linux
  package/visible-launch gate are required before an installed-host claim.
- `pnpm validate` passed with 431 desktop, 7 website, and 462 Rust tests;
  `pnpm test:e2e` passed with 80 desktop and 8 website tests; the clean Ubuntu
  package/visible-launch gate passed for `0.1.0-beta.101`.
- The one owner-authorized packaged launch on 2026-08-27 used the exact
  `https://google.com/` target and `https://google.com` origin. Its result was
  terminal `origin_drift`, with no content digest, observed byte count, or
  timestamp, after Google redirected outside that approved origin. A later
  `www.google.com` observation is outside this approval and requires a new
  exact-origin authorization.
- The staged Debian package was installed through the root-owned local
  installer daemon. The installed `0.1.0~beta.101` binary repeated the exact
  Google probe and returned the same terminal `origin_drift` state without
  content digest, observed byte count, or timestamp.
