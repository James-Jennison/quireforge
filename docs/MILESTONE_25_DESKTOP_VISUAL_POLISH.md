# Milestone 25 — Desktop Visual Polish

Status: implementation complete; final package evidence pending.

## Objective

Refine the existing QuireForge desktop presentation so its density, hierarchy,
and interaction polish feel familiar to modern native AI-workspace clients
while retaining QuireForge's own name, warm accent, local-first language, and
native behavior.

## Approved boundary

This milestone changes only the React presentation layer, shared CSS, and
visual/accessibility regression coverage. It refines the existing sidebar,
top bar, Home empty state, project entry affordances, and native conversation
composer. The composer continues to expose only existing attachment, model,
reasoning, filesystem, approval, and start/stop controls. No voice capability
exists in the reviewed native contract, so this milestone does not imply or
simulate one.

The presentation remains branded as QuireForge. It does not copy OpenAI or
ChatGPT logos, artwork, or branded content.

## Exclusions

- No Rust, Tauri command, backend, or repository-state contract change.
- No project-state reader, Git, package, validation, or persistence behavior.
- No watcher, background scan, autonomous action, handoff generation, or
  contradiction resolution.
- No QuireForge-Qt work, Qt migration, external asset import, release, or
  merge.

## Implementation

- A denser 244px desktop sidebar with compact 36px navigation targets and a
  clear inset active treatment preserves keyboard links and compact-rail use.
- The 58px top bar groups route context and existing actions on a consistent
  alignment line without introducing native-window controls or commands.
- Neutral layered dark surfaces, borders, and text hierarchy retain the
  QuireForge amber accent rather than copying another product's palette.
- Home prioritizes a centered task entry surface. The conversation route uses
  a calmer header, rounded composer and stream panels, a larger centered empty
  state, and a separated action row.
- Responsive drawer/rail rules, visible focus, forced-colors behavior, and
  reduced-motion handling continue to use the existing shell mechanisms.

## Verification

The focused browser regression checks desktop and mobile composer geometry,
active sidebar state, keyboard focus, axe results, overflow, and screenshots.
Final closure requires the established repository/frontend/Rust/Tauri gates and
fresh pinned Ubuntu 22.04 Debian/AppImage lifecycle and visible-launch smoke
evidence from the clean implementation commit.

## Deferred work

This milestone adds no new workspace, model control, voice control, native
windowing integration, agent behavior, or project-state automation.
