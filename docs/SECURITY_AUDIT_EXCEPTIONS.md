# Node audit exceptions

`pnpm security:audit:node` remains the raw, truthful Node audit. The normal security gate runs `pnpm security:audit:node:exception`: it runs that audit as JSON and accepts only the reviewed record in `security/node-audit-exceptions.json`.

There are currently no approved Node audit exceptions. The exception record intentionally contains an empty list, and the validator fails if the raw audit returns any advisory. This preserves an explicit, reviewable path for a future temporary exception without treating a clean audit as an error.

ESLint 9 remains intentionally retained. `eslint-plugin-jsx-a11y` supplies the static accessibility rules used by React and Astro's `jsx-a11y-recommended` configuration. No supported ESLint 10 peer release exists under the repository's strict peer-dependency policy. This is not an audit exception; retain the existing checks when that upgrade becomes compatible.
