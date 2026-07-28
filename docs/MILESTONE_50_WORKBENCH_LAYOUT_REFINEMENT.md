# Milestone 50 — QuireForge Workbench Layout Refinement

Status: implementation in progress. Package candidate: `0.1.0-beta.45`.

M50 refines existing presentation surfaces only. The task conversation remains dominant; task evidence and the managed terminal are optional and closed by default. No task record, transport, execution, terminal, Git, approval, provider, browser, project, or persistence authority is added.

## Resize and collapse contract

| Surface | Bounds | Keyboard | Collapse and focus rule |
| --- | --- | --- | --- |
| Task evidence review shell | 360–560 px wide | Left/right arrows, 20 px | The labelled close control unmounts the shell and restores focus to its opening control. |
| Managed terminal dock | 220–560 px high | Up/down arrows, 20 px | It is collapsed on every startup. Opening mounts the existing terminal surface; collapse unmounts it and leaves focus on its labelled toggle. |

Both separators are labelled focusable `separator` controls with current, minimum, and maximum values. Pointer listeners are removed on release/cancel and unmount. Narrow (760 px or below) and short (520 px or below) layouts use one full-width review overlay and cap the terminal dock at 42vh; the review resize control is not interactive there. There is no freeform tiling or offscreen interactive content.

## Local preference contract

The sole M50 record is browser-local `quireforge-workbench-layout`:

```json
{"schemaVersion":1,"reviewPaneWidth":480,"terminalDockHeight":320,"selectedReviewPane":"files"}
```

It accepts exactly these four fields, is capped at 512 bytes, requires schema version 1 and integer dimensions within the listed bounds, and permits only the six existing pane identifiers. Missing, corrupt, unknown, oversized, or invalid data restores defaults. It stores no open state, path, task, transcript, artifact, approval, execution, terminal, Git, project, credential, provider, or account data; it has no cloud/account synchronization. Reduced-motion users receive no layout animation or scripted smooth scrolling.
