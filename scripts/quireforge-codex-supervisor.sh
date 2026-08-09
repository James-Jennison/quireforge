#!/usr/bin/env bash
# Run one sandboxed Codex task at a time. The outer supervisor alone owns Git.
set -Eeuo pipefail
umask 077

readonly supervisor_name="quireforge-codex-supervisor"
readonly tmux_session="quireforge-codex-supervisor"
readonly repository_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
readonly state_root="${QUIRE_FORGE_SUPERVISOR_STATE_DIR:-${XDG_STATE_HOME:-}/$supervisor_name}"
readonly status_root="$(dirname -- "$state_root")/quireforge"
readonly status_path="$status_root/status.md"
readonly sentinel_path="$state_root/human-only-blocker"
readonly progress_path="$state_root/no-progress-runs"
readonly log_path="$state_root/supervisor.log"
readonly worker_log_path="$state_root/worker.log"
readonly lock_path="$state_root/run.lock"
readonly interval_seconds="${QUIRE_FORGE_SUPERVISOR_INTERVAL_SECONDS:-60}"
readonly started_at="$(date --iso-8601=seconds)"
readonly worker_sandbox_mode="danger-full-access"

usage() {
  printf 'Usage: %s [--once] [--dry-run] [--self-test] [--worker] [--finalize-recovery SUBJECT PATH...]\n' "${0##*/}"
}

mode="watch"
if [[ "${1:-}" == "--finalize-recovery" ]]; then
  mode="recovery"
  shift
elif [[ -n "${1:-}" ]]; then
  case "$1" in
    --once) mode="once" ;;
    --dry-run) mode="dry-run" ;;
    --self-test) mode="self-test" ;;
    --worker) mode="worker" ;;
    -h|--help) usage; exit 0 ;;
    *) usage >&2; exit 64 ;;
  esac
  shift
fi
if [[ $# -ne 0 && "$mode" != "recovery" ]]; then
  usage >&2
  exit 64
fi

if [[ -z "$state_root" || "$state_root" == "/$supervisor_name" ]]; then
  printf 'XDG_STATE_HOME or QUIRE_FORGE_SUPERVISOR_STATE_DIR must be set.\n' >&2
  exit 78
fi
if [[ ! -f "$repository_root/AGENTS.md" || ! -d "$repository_root/.git" ]]; then
  printf 'Expected QuireForge repository with AGENTS.md at %s.\n' "$repository_root" >&2
  exit 78
fi

mkdir -p -- "$state_root" "$status_root"
chmod 700 -- "$state_root" "$status_root"

safe_line() {
  local value="${1:-}"
  value="${value//$'\r'/ }"
  value="${value//$'\n'/ }"
  value="${value:0:240}"
  if [[ "$value" =~ (authorization:|cookie:|set-cookie:|x-api-key:|bearer[[:space:]]|api[_-]?key|token=|secret=|password=|signature=|private[[:space:]_]?key|https?://[^[:space:]]*[?&](token|key|secret|signature|sig)=) ]]; then
    printf '[redacted]'
  else
    printf '%s' "$value"
  fi
}

latest_commit() {
  git -C "$repository_root" rev-parse --short HEAD 2>/dev/null || printf 'unavailable'
}

write_status() {
  local task="$1" state="$2" validation="$3" next="$4" blocker="${5:-none}" temporary
  temporary="$(mktemp "$status_root/.status.XXXXXX")"
  {
    printf '# QuireForge Codex supervisor\n\n'
    printf 'Current task: %s\n' "$(safe_line "$task")"
    printf 'State: %s\n' "$(safe_line "$state")"
    printf 'Start time: %s\n' "$started_at"
    printf 'Latest commit: %s\n' "$(latest_commit)"
    printf 'Most recent validation result: %s\n' "$(safe_line "$validation")"
    printf 'Next action: %s\n' "$(safe_line "$next")"
    printf 'Blocker: %s\n' "$(safe_line "$blocker")"
  } > "$temporary"
  chmod 600 -- "$temporary"
  mv -f -- "$temporary" "$status_path"
}

log() {
  printf '%s %s\n' "$(date --iso-8601=seconds)" "$(safe_line "$*")" >> "$log_path"
}

is_sensitive_output() {
  [[ "$1" =~ (authorization:|cookie:|set-cookie:|x-api-key:|bearer[[:space:]]|api[_-]?key|token=|secret=|password=|signature=|private[[:space:]_]?key|https?://[^[:space:]]*[?&](token|key|secret|signature|sig)=) ]]
}

stream_worker_output() {
  local line
  while IFS= read -r line || [[ -n "$line" ]]; do
    if is_sensitive_output "$line"; then
      printf '[redacted sensitive output]\n'
    else
      printf '%s\n' "$line"
    fi
  done
}

subject_is_safe() {
  local pattern='^[[:alnum:]][[:alnum:][:space:].,:;()/_+-]{0,120}$'
  [[ "$1" =~ $pattern ]]
}

human_blocker() {
  local reason="$1"
  printf '%s\n' "$(safe_line "$reason")" > "$sentinel_path"
  chmod 600 -- "$sentinel_path"
  log "human-only blocker: $reason"
  write_status "Autonomous task blocked" "blocked" "failed" "Resolve the blocker, remove the sentinel, then restart" "$reason"
}

host_validation_is_allowed() {
  # Keep this list deliberately exact: worker output must never become a shell
  # command channel. Add an entry only after it is approved for local host use.
  case "$1" in
    "pnpm test:e2e") return 0 ;;
    *) return 1 ;;
  esac
}

run_host_validation() {
  local command="$1"
  if ! host_validation_is_allowed "$command"; then
    human_blocker "Codex requested a host validation command outside the approved allowlist."
    return 2
  fi

  write_status "Trusted supervisor is running host validation" "running" "running: $command" "Await host validation before commit" "none"
  log "running approved host validation: $command"
  case "$command" in
    "pnpm test:e2e")
      if pnpm test:e2e 2>&1 | stream_worker_output | tee -a "$worker_log_path"; then
        write_status "Trusted supervisor completed host validation" "running" "passed: $command" "Commit and push the validated task changes" "none"
        log "approved host validation passed: $command"
        return 0
      fi
      ;;
  esac
  # A host test failure is actionable validation feedback, not a human-only
  # block. Do not create the sentinel unless a real hard-stop is reported.
  write_status "Host validation failed" "failed" "failed: $command" "Fix the validation failure, then restart the supervisor" "none"
  log "approved host validation failed: $command"
  return 1
}

require_clean_worktree() {
  if [[ -n "$(git -C "$repository_root" status --porcelain)" ]]; then
    incomplete_task "Worktree is not clean; preserve existing changes before starting a task."
    return 1
  fi
}

incomplete_task() {
  local reason="$1"
  log "incomplete task: $reason"
  write_status "Task incomplete" "failed" "not committed: $reason" "Preserve and resolve remaining changes, then restart the supervisor" "none"
}

is_admissible_untracked_task_path() {
  local path="$1" absolute_path
  [[ "$path" != /* && "$path" != *".."* ]] || return 1
  case "$path" in
    .github/*|apps/*|docs/*|packaging/*|scripts/*) ;;
    *) return 1 ;;
  esac
  case "$path" in
    *.md|*.rs|*.toml|*.ts|*.tsx|*.js|*.mjs|*.cjs|*.json|*.jsonc|*.css|*.html|*.svg|*.sh|*.py|*.yml|*.yaml)
      ;;
    *) return 1 ;;
  esac
  absolute_path="$repository_root/$path"
  [[ -f "$absolute_path" && ! -L "$absolute_path" ]] || return 1
  [[ "$(stat -c %s -- "$absolute_path")" -le 1048576 ]] || return 1
  ! grep -E -q -- '(-----BEGIN [A-Z ]*PRIVATE KEY-----|(^|[^[:alnum:]_])(sk-|sk-proj-|ghp_|github_pat_|AKIA)[[:alnum:]_-]{16,})' "$absolute_path"
}

collect_admissible_untracked_task_paths() {
  local path
  mapfile -d '' -t untracked_task_paths < <(
    git -C "$repository_root" ls-files --others --exclude-standard -z
  )
  for path in "${untracked_task_paths[@]}"; do
    is_admissible_untracked_task_path "$path" || return 1
  done
}

post_push_completion_state() {
  local head="$1" upstream="$2" worktree_status="$3"
  if [[ "$head" != "$upstream" ]]; then
    printf 'upstream-mismatch\n'
  elif [[ -n "$worktree_status" ]]; then
    printf 'uncommitted-changes\n'
  else
    printf 'aligned-and-clean\n'
  fi
}

status_field() {
  local field="$1"
  sed -n "s/^${field}: //p" "$status_path" 2>/dev/null | head -n 1
}

tmux_session_exit_action() {
  local state="$1" task="$2"
  if [[ "$state" == "running" || "$task" == "Task committed and pushed" ]]; then
    printf 'restart\n'
  else
    printf 'stop\n'
  fi
}

tmux_session_exit_requires_restart() {
  [[ ! -e "$sentinel_path" ]] || return 1
  [[ "$(tmux_session_exit_action "$(status_field State)" "$(status_field 'Current task')")" == "restart" ]]
}

run_completion_state_self_test() {
  [[ "$(post_push_completion_state same same '')" == "aligned-and-clean" ]] || return 1
  [[ "$(post_push_completion_state local upstream '')" == "upstream-mismatch" ]] || return 1
  [[ "$(post_push_completion_state same same '?? task-file')" == "uncommitted-changes" ]] || return 1
  [[ "$worker_sandbox_mode" == "danger-full-access" ]] || return 1
  [[ "$(tmux_session_exit_action running 'Full-access Codex worker is implementing the highest-value safe task')" == "restart" ]] || return 1
  [[ "$(tmux_session_exit_action idle 'Task committed and pushed')" == "restart" ]] || return 1
  [[ "$(tmux_session_exit_action idle 'No task made committed progress')" == "stop" ]] || return 1
  [[ "$(tmux_session_exit_action failed 'Worker test or validation failed')" == "stop" ]] || return 1
  [[ "$(tmux_session_exit_action blocked 'Autonomous task blocked')" == "stop" ]] || return 1
  printf 'Supervisor completion-state, worker-access, and tmux-recovery checks passed.\n'
}

finalize_commit() {
  local subject="$1"
  shift
  local -a paths=("$@")
  if ! subject_is_safe "$subject"; then
    human_blocker "Codex emitted an unsafe or invalid commit subject."
    return 1
  fi
  if [[ ${#paths[@]} -eq 0 ]]; then
    human_blocker "Codex reported ready to commit but changed no tracked files."
    return 1
  fi
  if ! git -C "$repository_root" diff --check -- "${paths[@]}"; then
    human_blocker "git diff --check failed for the task changes."
    return 1
  fi
  if ! git -C "$repository_root" add -- "${paths[@]}" || ! git -C "$repository_root" diff --cached --check; then
    human_blocker "Selective staging or staged diff validation failed."
    return 1
  fi
  if ! git -C "$repository_root" commit -m "$subject"; then
    human_blocker "Trusted outer Git commit failed."
    return 1
  fi
  if ! git -C "$repository_root" push origin main; then
    human_blocker "Trusted outer Git push failed."
    return 1
  fi
  local completion_state
  completion_state="$(post_push_completion_state \
    "$(git -C "$repository_root" rev-parse HEAD)" \
    "$(git -C "$repository_root" rev-parse '@{u}')" \
    "$(git -C "$repository_root" status --porcelain)")"
  case "$completion_state" in
    aligned-and-clean) return 0 ;;
    upstream-mismatch)
      write_status "Post-push upstream mismatch" "failed" "failed: local HEAD does not match its upstream" "Inspect the push result before restarting the supervisor" "none"
      log "post-push upstream mismatch"
      return 1
      ;;
    uncommitted-changes)
      incomplete_task "Task changes remain uncommitted after a successful, aligned push."
      return 1
      ;;
    *)
      write_status "Post-push verification failed" "failed" "failed: unknown completion state" "Inspect the supervisor log before restarting" "none"
      log "unknown post-push completion state: $completion_state"
      return 1
      ;;
  esac
}

recovery_finalize() {
  local subject="${1:-}"
  shift || true
  [[ $# -gt 0 ]] || { usage >&2; exit 64; }
  local path
  for path in "$@"; do
    [[ "$path" != /* && "$path" != *".."* ]] || { printf 'Unsafe recovery path.\n' >&2; exit 64; }
    git -C "$repository_root" ls-files --error-unmatch -- "$path" >/dev/null
  done
  finalize_commit "$subject" "$@"
}

if [[ "$mode" == "dry-run" ]]; then
  printf 'repository=%s\nstate=%s\nstatus=%s\nlog=%s\nworker_log=%s\nsentinel=%s\n' \
    "$repository_root" "$state_root" "$status_path" "$log_path" "$worker_log_path" "$sentinel_path"
  exit 0
fi
if [[ "$mode" == "self-test" ]]; then
  run_completion_state_self_test
  exit $?
fi
if [[ "$mode" == "recovery" ]]; then
  recovery_finalize "$@"
  exit $?
fi

run_in_tmux() {
  if ! command -v tmux >/dev/null 2>&1; then
    return 1
  fi
  trap 'tmux kill-session -t "$tmux_session" 2>/dev/null || true; exit 0' INT TERM
  while :; do
    if tmux has-session -t "$tmux_session" 2>/dev/null; then
      log "existing tmux worker session found; waiting for it"
    else
      log "starting persistent tmux worker session"
      tmux new-session -d -s "$tmux_session" \
        "env QUIRE_FORGE_SUPERVISOR_STATE_DIR=$(printf '%q' "$state_root") QUIRE_FORGE_SUPERVISOR_INTERVAL_SECONDS=$(printf '%q' "$interval_seconds") $(printf '%q' "$repository_root/scripts/quireforge-codex-supervisor.sh") --worker"
    fi

    while tmux has-session -t "$tmux_session" 2>/dev/null; do sleep 2; done
    if ! tmux_session_exit_requires_restart; then
      log "tmux worker session ended in a terminal state; stopping wrapper"
      return 0
    fi
    log "tmux worker session ended unexpectedly; restarting it"
    write_status "Restarting full-access Codex worker" "running" "not reported" "Restart the interrupted worker task" "none"
    sleep 1
  done
}

if [[ "$mode" != "worker" && "$mode" != "once" ]]; then
  if run_in_tmux; then exit 0; fi
fi

exec 9>"$lock_path"
if ! flock -n 9; then
  log "another supervisor task holds the repository lock; exiting"
  write_status "Waiting for active worker" "idle" "not run" "Wait for the existing worker" "none"
  exit 0
fi

stop_requested=0
trap 'stop_requested=1; log "stop signal received"; write_status "Supervisor stopping" "idle" "not running" "Restart when ready" "none"' INT TERM

record_no_progress() {
  local count=0
  [[ -f "$progress_path" ]] && count="$(<"$progress_path")"
  count=$((count + 1))
  printf '%s\n' "$count" > "$progress_path"
  chmod 600 -- "$progress_path"
  log "no progress run $count of 2"
  if [[ $count -ge 2 ]]; then
    write_status "No task made committed progress" "idle" "not reported" "Wait for new work, then restart" "none"
    return 1
  fi
  return 0
}

run_task() {
  local before_head result_path exit_code=0 validation="not reported" subject validation_failure
  local -a host_requests
  require_clean_worktree || return 1
  before_head="$(git -C "$repository_root" rev-parse HEAD)"
  result_path="$state_root/last-message.$(date +%s).txt"
  write_status "Full-access Codex worker is implementing the highest-value safe task" "running" "$validation" "Await tested ready-to-commit marker" "none"
  log "starting full-access Codex task at $before_head"
  if codex exec --ephemeral --sandbox "$worker_sandbox_mode" \
    --config 'shell_environment_policy.inherit="none"' \
    --config 'shell_environment_policy.include_only=["PATH","QUIRE_FORGE_M63_MODEL_PATH"]' \
    --cd "$repository_root" --output-last-message "$result_path" \
    'Read and obey AGENTS.md first. Inspect docs/CURRENT_STATE.md, docs/ROADMAP.md, relevant ADRs, and git status. Advance the approved M63 credential-free local-runtime vertical slice toward a releasable local candidate: prioritize the missing adapter, bounded lifecycle, user-visible local-only flow, package/installed-host acceptance, and release-candidate evidence. Do not make standalone source-boundary hardening, validator expansion, or adversarial guard changes unless a direct validation failure blocks the vertical slice; do not select M64 or any credentialed provider. You are a supervisor-launched full-access worker. The outer supervisor owns Git operations: never run git add, git commit, or git push. The only permitted runtime is the approved in-process local model at the read-only path in QUIRE_FORGE_M63_MODEL_PATH: do not print, modify, copy, hash, download, or persist that path or model. Do not access credentials or accounts, use browser sessions, connect to an external provider, transmit over the network, deploy, publish, take destructive action, make third-party commitments, or make irreversible product-direction decisions. Preserve the Linux Tauri desktop-app scope. Run focused unit tests, type-check, lint, and formatting checks that are relevant to the task. Before reporting Cargo unavailable, run `cargo metadata --locked --no-deps --format-version 1`; report that failure only if the command itself fails. If host validation is required, emit one exact line per required command before the ready marker: AUTOPILOT_HOST_VALIDATION: pnpm test:e2e. During work, emit concise progress summaries prefixed exactly AUTOPILOT_PROGRESS:. Never expose credentials, signing material, secret-bearing URLs, headers, or secrets in progress or final output. Only after relevant checks pass, requested host validation has been declared, and tracked task changes are ready, end your final response with exactly one machine-readable line: AUTOPILOT_READY_TO_COMMIT: <concise commit subject>. If an ordinary test or validation fails, state AUTOPILOT_VALIDATION_FAILED: <concise reason> and do not emit ready. State HUMAN_ONLY_BLOCKER: <concise reason> only for credentials, production access, public release, destructive action, third-party commitment, browser/account access, or a genuinely irreversible product-direction decision.' \
    2>&1 | stream_worker_output | tee -a "$worker_log_path"; then
    :
  else
    exit_code=$?
  fi
  if [[ -f "$result_path" ]] && grep -q '^HUMAN_ONLY_BLOCKER:' "$result_path"; then
    human_blocker "$(grep '^HUMAN_ONLY_BLOCKER:' "$result_path" | tail -n 1 | sed 's/^HUMAN_ONLY_BLOCKER:[[:space:]]*//')"
    rm -f -- "$result_path"
    return 1
  fi
  validation_failure="$( { grep '^AUTOPILOT_VALIDATION_FAILED:' "$result_path" 2>/dev/null || true; } | tail -n 1 | sed 's/^AUTOPILOT_VALIDATION_FAILED:[[:space:]]*//')"
  if [[ -n "$validation_failure" ]]; then
    write_status "Worker test or validation failed" "failed" "failed: $validation_failure" "Fix the test or validation failure, then restart the supervisor" "none"
    log "worker test or validation failed: $validation_failure"
    rm -f -- "$result_path"
    return 1
  fi
  if [[ $exit_code -ne 0 ]]; then
    write_status "Full-access Codex worker failed" "failed" "failed: Codex exited with status $exit_code" "Inspect the worker log, fix the task, then restart the supervisor" "none"
    log "full-access Codex worker exited with status $exit_code"
    rm -f -- "$result_path"
    return 1
  fi
  subject="$( { grep '^AUTOPILOT_READY_TO_COMMIT:' "$result_path" 2>/dev/null || true; } | tail -n 1 | sed 's/^AUTOPILOT_READY_TO_COMMIT:[[:space:]]*//')"
  if [[ -z "$subject" ]]; then
    if ! record_no_progress; then
      rm -f -- "$result_path"
      return 1
    fi
    write_status "Full-access Codex worker made no committable progress" "idle" "ready-to-commit marker absent" "Start the next safe task" "none"
    rm -f -- "$result_path"
    return 0
  fi
  if ! collect_admissible_untracked_task_paths; then
    incomplete_task "Task-created untracked output is outside the approved source-file boundary; nothing was committed."
    rm -f -- "$result_path"
    return 1
  fi
  mapfile -t host_requests < <(sed -n 's/^AUTOPILOT_HOST_VALIDATION:[[:space:]]*//p' "$result_path")
  local request
  for request in "${host_requests[@]}"; do
    [[ -n "$request" ]] || continue
    if ! run_host_validation "$request"; then
      rm -f -- "$result_path"
      return 1
    fi
  done
  write_status "Trusted supervisor is validating, committing, and pushing task changes" "running" "sandbox checks and requested host validation passed" "Verify clean post-push alignment" "none"
  mapfile -t changed_paths < <(git -C "$repository_root" diff --name-only --diff-filter=ACDMRTUXB)
  changed_paths+=("${untracked_task_paths[@]}")
  if ! finalize_commit "$subject" "${changed_paths[@]}"; then
    rm -f -- "$result_path"
    return 1
  fi
  rm -f -- "$progress_path" "$result_path"
  log "validated task committed and pushed"
  write_status "Task committed and pushed" "idle" "passed; outer Git commit and push verified" "Start the next safe task" "none"
}

while [[ $stop_requested -eq 0 ]]; do
  [[ -e "$sentinel_path" ]] && { write_status "Awaiting human-only blocker resolution" "blocked" "not run" "Resolve blocker, remove sentinel, then restart" "$(<"$sentinel_path")"; exit 0; }
  if ! run_task; then
    exit 0
  fi
  [[ "$mode" == "once" ]] && exit 0
  sleep "$interval_seconds" & wait $! || true
done
