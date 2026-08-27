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

- Strict native and TypeScript contracts permit only the owner-approved exact
  Google target/origin and bounded observation limit.
- The packaged Linux application exposes a separate helper with no ambient
  browser profile and verifies that JavaScript disabling is available before
  loading a target.
- Full source validation, desktop/website E2E, and the clean pinned Linux
  package/visible-launch gate are required before an installed-host claim.
