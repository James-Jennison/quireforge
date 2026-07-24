# Milestone 22 — Routed Desktop Workspace

Status: complete and verified locally.

## Objective

Replace the combined scrolling desktop document with dedicated application
workspaces while keeping the QuireForge shell mounted, preserve completed tool
behavior and temporary presentation state, give the account summary a genuine
supported action, and establish a professional responsive desktop pane
hierarchy without adding a backend capability or competing router.

## Implemented boundary

- Home, New task, Projects, Threads, Scheduled, Integrations, Files, Changes,
  Worktrees, and Terminal are closed typed routes with one visible primary
  workspace at a time.
- URL hashes provide deep links and browser history. The last valid location is
  also retained as a local QuireForge presentation preference. Invalid hashes
  fail closed to Home.
- The shell, navigation, and established workspace components remain mounted.
  Inactive destinations use `hidden` and `aria-hidden`, preserving ordinary
  component state while removing inactive content from layout and the
  accessibility tree.
- Route changes move focus to the destination heading. Navigation exposes
  `aria-current`, descriptive titles, visible focus, a skip target, and a
  scrollable mobile drawer.
- The frame has a route-aware toolbar, independent sidebar/workspace/inspector
  scrolling, an optional meaningful inspector, keyboard and pointer resizing,
  a persistent status bar, a remembered inspector width, and a remembered
  compact-sidebar choice.
- The account summary is a button into Settings → Accounts & connections.
  Settings uses the existing normalized Codex status/refresh, remaining-usage,
  and two-step logout actions. It separately presents local Appearance and
  About sections.

No direct ChatGPT account-management API was found in the supported
integration. QuireForge therefore does not fabricate one. The Settings surface
does not expose billing, identifiers, email, credentials, tokens, raw protocol
data, or private account pages, and this milestone adds no external-browser
handoff.

## Architecture and compatibility

The implementation uses local React state, the existing component/state
ownership, and a small typed hash parser. No routing, pane-management, or other
dependency was added. No Rust command, Tauri permission, SQLite migration,
Codex route, credential boundary, hosting path, or release workflow changed.

Desktop pane behavior is responsive:

- above 1180 px, meaningful context can occupy a bounded resizable third pane;
- from 761–960 px, navigation becomes an icon rail without losing labels to
  assistive technology or title hints;
- at 760 px and below, navigation becomes an opaque scrollable drawer and the
  inspector becomes an overlay;
- when no route-relevant context exists, the inspector is absent.

Obsolete continuous-dashboard and decorative scaffold CSS was removed so the
new interface remains within the existing 100 KiB production CSS budget.

## Acceptance evidence

- Repository policy validation: passed.
- Package-script tests: 7 passed.
- Desktop unit/component/contract tests: 162 passed across 32 files.
- Website tests: 7 passed.
- Rust tests: 178 passed, 3 deliberate live probes ignored.
- Desktop Playwright: 38 passed across desktop and mobile Chromium.
- Formatting, TypeScript, Astro checks, ESLint, Cargo formatting, Cargo check,
  warning-denying Clippy, production builds, and generated-artifact validators:
  passed.
- Desktop production assets: 189.01 KiB entry, 286.79 KiB application chunk,
  817.66 KiB total JavaScript, and 99.96 KiB total CSS; all enforced budgets
  passed.
- Local Linux packaging: Debian and AppImage Tauri bundles built, then
  normalized into local release candidates; nothing was published.

The Playwright evidence pass captured:

- Home with the desktop three-pane shell at 1920×1080;
- every non-Home sidebar destination at 1440×900;
- account Settings at 1366×768;
- the collapsed Changes layout at 720×900; and
- mobile Home and open navigation-drawer states.

The supplied pre-milestone sidebar/account images remain the before reference.
Generated after images are local ignored test artifacts and can be recreated
with the command in [Testing](TESTING.md).

## Time and authorization

- Start: `2026-07-23T16:09:10-07:00`.
- Completion: `2026-07-23T17:06:00-07:00`.
- Model and reasoning: GPT-5.6 Sol, XHigh.
- Forecast: 6–10 active hours plus 1–2.5 hours of checks; 7–12.5 hours total
  elapsed.
- Actual: approximately 0.68 active hour and 0.27 automated hour, with no
  post-start user-blocked time; counted project time and total elapsed were
  approximately 0.95 hour.

Implementation remained local. No push, merge, publication, release,
deployment, DNS/hosting mutation, external authentication, personal account
mutation, or billable model call occurred.

## Intentionally deferred

- Direct remote ChatGPT/Codex account administration remains unavailable
  because no supported interface was identified.
- Inspector open/closed state is session-local; bounded inspector width and
  compact navigation are persisted.
- Generic docking, detachable OS windows, multi-window workspaces, and
  arbitrary pane composition are outside this focused milestone.
- Package publication and owner-hosted download activation retain their
  separate authorization gates.
