# Autonomous local Codex supervisor

`scripts/quireforge-codex-supervisor.sh` is the repository-owned launcher for
one at-a-time, non-interactive local Codex tasks. The installed user service
uses the same source file and writes its lock, final messages, counter, and
ordinary safe task output only to `~/.local/state/quireforge-codex-supervisor/`.
Its atomically replaced, non-secret status snapshot is
`~/.local/state/quireforge/status.md`.

It reads `AGENTS.md` through Codex, uses `flock` so only one supervisor task
runs, and requires a clean worktree before starting a new task. Codex stays in
the `workspace-write` sandbox and never commits or pushes. After Codex reports
the tested `AUTOPILOT_READY_TO_COMMIT` marker, the trusted outer supervisor runs
the scoped diff check, stages only changed tracked task files, commits, pushes
`main`, and verifies clean post-push alignment. A validation, commit, or push
failure is a blocker. Codex stdout/stderr is reduced to a strict allowlist of
operational metadata and `AUTOPILOT_PROGRESS` summaries before being written to
`worker.log` and the tmux pane through the same `tee` pipeline; all other
Codex output is withheld. A real human-only blocker creates
`~/.local/state/quireforge-codex-supervisor/human-only-blocker` and stops the
service; any other two consecutive no-progress runs also stop it. Remove the
sentinel only after the blocker has received explicit owner direction, then
restart the service.

When `tmux` is already installed, the systemd service starts one persistent
session named `quireforge-codex-supervisor`; it never creates a duplicate. When
`tmux` is unavailable, the same worker runs with the status/log fallback. The
supervisor never installs packages or uses `sudo`.

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
