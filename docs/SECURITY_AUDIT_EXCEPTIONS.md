# Node audit exceptions

`pnpm security:audit:node` remains the raw, truthful Node audit. The normal security gate runs `pnpm security:audit:node:exception`: it runs that audit as JSON and accepts only the reviewed record in `security/node-audit-exceptions.json`.

The current exception is limited to development-only `brace-expansion` 1.1.16 under ESLint 9. It pins the advisory, severity, package, version, full dependency-path-set digest and count, owner, review date, expiry, controls, and removal trigger. Any new advisory, changed severity/package/version/path set, record mismatch, or expiry fails the gate.

ESLint 9 is intentionally retained. `eslint-plugin-jsx-a11y` supplies the static accessibility rules used by React and Astro's `jsx-a11y-recommended` configuration. No supported ESLint 10 peer release exists under the repository's strict peer-dependency policy. Remove this exception only when that upgrade preserves both checks.
