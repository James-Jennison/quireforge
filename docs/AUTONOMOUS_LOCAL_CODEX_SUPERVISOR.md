# Autonomous local Codex supervisor

`scripts/quireforge-codex-supervisor.sh` is the repository-owned launcher for
one at-a-time, non-interactive local Codex tasks. The installed user service
uses the same source file and writes its lock, final messages, counter, and
ordinary safe task output only to `~/.local/state/quireforge-codex-supervisor/`.
Its atomically replaced, non-secret status snapshot is
`~/.local/state/quireforge/status.md`.

It reads `AGENTS.md` through Codex, uses `flock` so only one supervisor task
runs, and requires a clean worktree before starting a new task. Supervisor-
launched Codex workers use the installed CLI's `danger-full-access` sandbox
mode, while interactive Codex defaults and other projects remain unchanged.
The outer supervisor alone commits or pushes. Workers run focused tests,
type-checking, linting, and formatting, but request host-listener E2E checks
through an exact
`AUTOPILOT_HOST_VALIDATION: pnpm test:e2e` marker before its
`AUTOPILOT_READY_TO_COMMIT` marker. The trusted outer supervisor accepts only
that explicit allowlisted command, runs it on the local host, then runs the
scoped diff check, stages changed tracked task files and only bounded admissible
untracked text-source files from the clean-start worker, commits, pushes `main`,
and verifies clean post-push alignment. Untracked files outside the approved
source directories/extensions, symlinks, files over 1 MiB, and files containing
private-key or common token markers remain terminal safe failures. A host test
failure is a failed validation state, not a human-only blocker; it becomes a
sentinel only for a real hard-stop. Codex stdout/stderr is written near-raw to `worker.log`
and the tmux pane through the same `tee` pipeline. Only secret-like output is
redacted; source and ordinary task output remain visible for local monitoring.
A real human-only blocker creates
`~/.local/state/quireforge-codex-supervisor/human-only-blocker` and stops the
service; any other two consecutive no-progress runs also stop it. Remove the
sentinel only after the blocker has received explicit owner direction, then
restart the service.

When `tmux` is already installed, the systemd service starts one persistent
session named `quireforge-codex-supervisor`; it never creates a duplicate. If
that session vanishes while a task was running or immediately after a validated
commit, the wrapper starts the next worker. Blocked, failed-validation, and
two-run no-progress states remain terminal. The user service restarts on an
unexpected process failure. When `tmux` is unavailable, the same worker runs
with the status/log fallback. The supervisor never installs packages or uses
`sudo`. Its installed default interval between completed tasks is 60 seconds.

The source unit is `packaging/systemd-user/quireforge-codex-supervisor.service`.
Install it for the current user with:

```bash
install -Dm600 packaging/systemd-user/quireforge-codex-supervisor.service \
  "$HOME/.config/systemd/user/quireforge-codex-supervisor.service"
systemctl --user daemon-reload
systemctl --user enable --now quireforge-codex-supervisor.service
```

The supervisor never deploys, publishes, accesses credentials, or uses browser
sessions. It follows the roadmap's autonomous post-M62 rule while retaining its
hard stops.

## Local inspection and control

```bash
# Attach when tmux is available; detach with Ctrl-b d.
tmux attach -t quireforge-codex-supervisor

# Inspect the bounded status snapshot and follow live safe worker output.
cat "$HOME/.local/state/quireforge/status.md"
tail -F "$HOME/.local/state/quireforge-codex-supervisor/worker.log"
tail -F "$HOME/.local/state/quireforge-codex-supervisor/supervisor.log"

# Stop or restart without sudo.
systemctl --user stop quireforge-codex-supervisor.service
systemctl --user restart quireforge-codex-supervisor.service
```

Do not run the attach command when `tmux` is absent; use the status and log
commands instead. The log excludes secret-like output rather than recording it.
