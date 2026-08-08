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
readonly interval_seconds="${QUIRE_FORGE_SUPERVISOR_INTERVAL_SECONDS:-300}"
readonly started_at="$(date --iso-8601=seconds)"

usage() {
  printf 'Usage: %s [--once] [--dry-run] [--worker] [--finalize-recovery SUBJECT PATH...]\n' "${0##*/}"
}

mode="watch"
if [[ "${1:-}" == "--finalize-recovery" ]]; then
  mode="recovery"
  shift
elif [[ -n "${1:-}" ]]; then
  case "$1" in
    --once) mode="once" ;;
    --dry-run) mode="dry-run" ;;
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

require_clean_worktree() {
  if [[ -n "$(git -C "$repository_root" status --porcelain)" ]]; then
    human_blocker "Worktree is not clean; preserve and resolve existing changes before a new task."
    return 1
  fi
}

finalize_commit() {
  local subject="$1" allow_remaining_changes="$2"
  shift 2
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
  if [[ "$(git -C "$repository_root" rev-parse HEAD)" != "$(git -C "$repository_root" rev-parse '@{u}')" ]] || { [[ "$allow_remaining_changes" != "true" ]] && [[ -n "$(git -C "$repository_root" status --porcelain)" ]]; }; then
    human_blocker "Post-push repository alignment verification failed."
    return 1
  fi
  return 0
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
  finalize_commit "$subject" true "$@"
}

if [[ "$mode" == "dry-run" ]]; then
  printf 'repository=%s\nstate=%s\nstatus=%s\nlog=%s\nworker_log=%s\nsentinel=%s\n' \
    "$repository_root" "$state_root" "$status_path" "$log_path" "$worker_log_path" "$sentinel_path"
  exit 0
fi
if [[ "$mode" == "recovery" ]]; then
  recovery_finalize "$@"
  exit $?
fi

run_in_tmux() {
  if ! command -v tmux >/dev/null 2>&1; then
    return 1
  fi
  if tmux has-session -t "$tmux_session" 2>/dev/null; then
    log "existing tmux worker session found; waiting for it"
  else
    log "starting persistent tmux worker session"
    tmux new-session -d -s "$tmux_session" \
      "env QUIRE_FORGE_SUPERVISOR_STATE_DIR=$(printf '%q' "$state_root") QUIRE_FORGE_SUPERVISOR_INTERVAL_SECONDS=$(printf '%q' "$interval_seconds") $(printf '%q' "$repository_root/scripts/quireforge-codex-supervisor.sh") --worker"
  fi
  trap 'tmux kill-session -t "$tmux_session" 2>/dev/null || true; exit 0' INT TERM
  while tmux has-session -t "$tmux_session" 2>/dev/null; do sleep 2; done
  return 0
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
  local before_head result_path exit_code=0 validation="not reported" subject
  require_clean_worktree || return 1
  before_head="$(git -C "$repository_root" rev-parse HEAD)"
  result_path="$state_root/last-message.$(date +%s).txt"
  write_status "Sandboxed Codex is implementing the highest-value safe task" "running" "$validation" "Await tested ready-to-commit marker" "none"
  log "starting sandboxed Codex task at $before_head"
  if codex exec --ephemeral --sandbox workspace-write --cd "$repository_root" --output-last-message "$result_path" \
    'Read and obey AGENTS.md first. Inspect docs/CURRENT_STATE.md, docs/ROADMAP.md, relevant ADRs, and git status. Implement the highest-value safe, reversible, local QuireForge task. You are sandboxed: never run git add, git commit, or git push. Do not access credentials or accounts, use browser sessions, connect to a real provider/runtime, transmit over the network, deploy, publish, take destructive action, make third-party commitments, or make irreversible product-direction decisions. Preserve the Linux Tauri desktop-app scope. Run the required relevant tests. During work, emit concise progress summaries prefixed exactly AUTOPILOT_PROGRESS:. Never expose credentials, signing material, secret-bearing URLs, headers, or secrets in progress or final output. Only after all required tests pass and tracked task changes are ready, end your final response with exactly one machine-readable line: AUTOPILOT_READY_TO_COMMIT: <concise commit subject>. Otherwise state HUMAN_ONLY_BLOCKER: <concise reason> and do not emit an AUTOPILOT_READY_TO_COMMIT line.' \
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
  if [[ $exit_code -ne 0 ]]; then
    human_blocker "Sandboxed Codex exited with status $exit_code."
    rm -f -- "$result_path"
    return 1
  fi
  subject="$( { grep '^AUTOPILOT_READY_TO_COMMIT:' "$result_path" 2>/dev/null || true; } | tail -n 1 | sed 's/^AUTOPILOT_READY_TO_COMMIT:[[:space:]]*//')"
  if [[ -z "$subject" ]]; then
    if ! record_no_progress; then
      rm -f -- "$result_path"
      return 1
    fi
    write_status "Sandboxed Codex made no committable progress" "idle" "ready-to-commit marker absent" "Start the next safe task" "none"
    rm -f -- "$result_path"
    return 0
  fi
  write_status "Trusted supervisor is validating, committing, and pushing task changes" "running" "tests passed according to Codex ready marker" "Verify clean post-push alignment" "none"
  mapfile -t changed_paths < <(git -C "$repository_root" diff --name-only --diff-filter=ACDMRTUXB)
  if ! finalize_commit "$subject" false "${changed_paths[@]}"; then
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
