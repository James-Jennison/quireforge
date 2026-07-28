# Milestone 50 — QuireForge Workbench Layout Refinement

Status: complete. Package-source commit `1cc7c50ceed6d2b6c2f91274110471d71fe6292a`.

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

## Beta.45 package evidence

The pinned Ubuntu 22.04 container (`scripts/run_linux_package_container.sh`)
validated lifecycle and visible package launch, then produced:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `quireforge_0.1.0.beta.45_amd64.deb` | 5,511,816 | `75bb98b7e7196662da79aa3c868f7ad18ab04e291119763d65e42ae83dc6d210` |
| `quireforge-sandboxd_0.1.0.beta.45_amd64.deb` | 3,233,564 | `c8ab1bc5f4a4817e6f3fc19f79a0ac5cdd8194ebc0c94b9d29ffbd981d594ed2` |

Both binaries require at most `GLIBC_2.34`, within the `GLIBC_2.35` baseline.
The restricted `sudo -n /usr/local/sbin/quireforge-validate-deb` wrapper
installed and verified `quireforge 0.1.0~beta.45`. Production bundle totals
were 190.41 KiB startup entry, 309.73 KiB application shell, 936.06 KiB total
JavaScript, and 109.52 KiB CSS: all within the active 256 KiB / 448 KiB / 1.5
MiB / 160 KiB ceilings.
