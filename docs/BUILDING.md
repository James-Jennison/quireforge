# Building QuireForge

Status: the Milestone 2 website can be developed and built. The desktop
application does not exist yet; its prerequisites and commands will be added in
Milestone 3.

## Supported development baseline

- Linux development host
- Node.js `22.12.0` or newer in the Node 22 line
- pnpm `11.15.0`, as pinned by the root `packageManager` field
- Python 3 for the dependency-free repository validator
- Git

Do not install dependencies with npm or commit an additional lockfile. The
workspace uses the root `pnpm-lock.yaml` and rejects unreviewed dependency build
scripts. Only `esbuild` and `sharp` are allowed to build during installation.

## Install dependencies

From the repository root:

```bash
pnpm install --frozen-lockfile
```

If the distribution-provided Corepack cannot launch the pinned pnpm version,
use the non-persistent fallback used during Milestone 2:

```bash
npx --yes pnpm@11.15.0 install --frozen-lockfile
```

Do not use `--ignore-scripts` as a substitute for the committed pnpm build
allowlist; Astro's approved native dependencies need their normal install
steps.

## Develop and build the website

```bash
pnpm dev
pnpm build
pnpm preview
```

The generated static artifact is `apps/website/dist/`. It is ignored by Git and
must not contain credentials, local account data, Codex state, or locally
installed integration information.

The production origin is `https://quireforge.jamesjennison.net` with base path
`/`. Local development continues to use Astro's local origin. No server runtime,
database, Pages Function, or Cloudflare adapter is required.

## Full non-browser validation

```bash
pnpm validate
```

Browser and accessibility checks are documented separately in
[Testing](TESTING.md). Deployment remains a separate approval-gated operation;
building the artifact does not authorize Cloudflare project, custom-domain, DNS,
or production changes.
