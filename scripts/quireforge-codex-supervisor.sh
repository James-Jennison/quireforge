#!/usr/bin/env bash
# Run one repository-scoped Codex task at a time. Mutable state stays outside
# the repository; status and logs contain only bounded, non-secret summaries.
set -Eeuo pipefail
umask 077

readonly supervisor_name="quireforge-codex-supervisor"
readonly tmux_session="quireforge-codex-supervisor"
readonly repository_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
readonly state_root="${QUIRE_FORGE_SUPERVISOR_STATE_DIR:-${XDG_STATE_HOME:-}/$supervisor_name}"
readonly status_root="$(dirname -- "$state_root")/quireforge"
readonly status_path="$status_root/status.md"
readonly sentinel_name="human-only-blocker"
readonly interval_seconds="${QUIRE_FORGE_SUPERVISOR_INTERVAL_SECONDS:-300}"

usage() {
  printf 'Usage: %s [--once] [--dry-run] [--worker]\n' "${0##*/}"
}

mode="watch"
case "${1:-}" in
  "") ;;
  --once) mode="once" ;;
  --dry-run) mode="dry-run" ;;
  --worker) mode="worker" ;;
  -h|--help) usage; exit 0 ;;
  *) usage >&2; exit 64 ;;
esac

if [[ -z "${state_root}" || "${state_root}" == "/${supervisor_name}" ]]; then
  printf 'XDG_STATE_HOME or QUIRE_FORGE_SUPERVISOR_STATE_DIR must be set.\n' >&2
  exit 78
fi
if [[ ! -f "$repository_root/AGENTS.md" || ! -d "$repository_root/.git" ]]; then
  printf 'Expected QuireForge repository with AGENTS.md at %s.\n' "$repository_root" >&2
  exit 78
fi

mkdir -p -- "$state_root" "$status_root"
chmod 700 -- "$state_root" "$status_root"
readonly lock_path="$state_root/run.lock"
readonly sentinel_path="$state_root/$sentinel_name"
readonly progress_path="$state_root/no-progress-runs"
readonly log_path="$state_root/supervisor.log"
readonly started_at="$(date --iso-8601=seconds)"

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
  local task state validation next blocker temporary
  task="$(safe_line "$1")"
  state="$(safe_line "$2")"
  validation="$(safe_line "$3")"
  next="$(safe_line "$4")"
  blocker="$(safe_line "${5:-none}")"
  temporary="$(mktemp "$status_root/.status.XXXXXX")"
  {
    printf '# QuireForge Codex supervisor\n\n'
    printf 'Current task: %s\n' "$task"
    printf 'State: %s\n' "$state"
    printf 'Start time: %s\n' "$started_at"
    printf 'Latest commit: %s\n' "$(latest_commit)"
    printf 'Most recent validation result: %s\n' "$validation"
    printf 'Next action: %s\n' "$next"
    printf 'Blocker: %s\n' "$blocker"
  } > "$temporary"
  chmod 600 -- "$temporary"
  mv -f -- "$temporary" "$status_path"
}

log() {
  # Only supervisor-authored, pre-sanitized messages are written to this log.
  printf '%s %s\n' "$(date --iso-8601=seconds)" "$(safe_line "$*")" >> "$log_path"
}

if [[ "$mode" == "dry-run" ]]; then
  printf 'repository=%s\nstate=%s\nstatus=%s\nlog=%s\nsentinel=%s\n' \
    "$repository_root" "$state_root" "$status_path" "$log_path" "$sentinel_path"
  exit 0
fi

run_in_tmux() {
  if ! command -v tmux >/dev/null 2>&1; then
    return 1
  fi
  if tmux has-session -t "$tmux_session" 2>/dev/null; then
    log "existing tmux worker session found; waiting for it"
  else
    log "starting persistent tmux worker session"
    if ! tmux new-session -d -s "$tmux_session" \
      "env QUIRE_FORGE_SUPERVISOR_STATE_DIR=$(printf '%q' "$state_root") QUIRE_FORGE_SUPERVISOR_INTERVAL_SECONDS=$(printf '%q' "$interval_seconds") $(printf '%q' "$repository_root/scripts/quireforge-codex-supervisor.sh") --worker"; then
      if ! tmux has-session -t "$tmux_session" 2>/dev/null; then
        write_status "Supervisor session startup" "failed" "not run" "Inspect the safe local log, then restart if appropriate" "Unable to create the tmux worker session"
        return 1
      fi
    fi
  fi
  trap 'tmux kill-session -t "$tmux_session" 2>/dev/null || true; exit 0' INT TERM
  while tmux has-session -t "$tmux_session" 2>/dev/null; do
    sleep 2
  done
  if [[ ! -f "$status_path" ]]; then
    write_status "Supervisor session startup" "failed" "not run" "Inspect the safe local log, then restart if appropriate" "The tmux worker exited before publishing status"
  fi
  return 0
}

if [[ "$mode" != "worker" ]] && [[ "$mode" != "once" ]]; then
  if run_in_tmux; then
    exit 0
  fi
fi

exec 9>"$lock_path"
if ! flock -n 9; then
  log "another supervisor task holds the repository lock; exiting"
  write_status "Waiting for the existing supervisor task" "idle" "not run by this invocation" "Wait for the active worker" "none"
  exit 0
fi

stop_requested=0
trap 'stop_requested=1; log "stop signal received"; write_status "Supervisor stopping" "idle" "not running" "Restart when ready" "none"' INT TERM

human_blocker() {
  local reason="$1"
  printf '%s\n' "$(safe_line "$reason")" > "$sentinel_path"
  chmod 600 -- "$sentinel_path"
  log "human-only blocker: $reason"
  write_status "Authorized QuireForge work assessment" "blocked" "not run" "Wait for explicit owner direction, remove the sentinel, then restart" "$reason"
}

record_final_message() {
  local result_path="$1"
  [[ -f "$result_path" ]] || return 0
  # Codex receives explicit instructions to return only a safe concise report.
  # Reject suspicious content rather than risk writing it to an ordinary log.
  if grep -Eqi 'authorization:|cookie:|set-cookie:|x-api-key:|bearer[[:space:]]|api[_-]?key|token=|secret=|password=|signature=|private[[:space:]_]?key|https?://[^[:space:]]*[?&](token|key|secret|signature|sig)=' "$result_path"; then
    log "Codex final report omitted because it contained sensitive-looking content"
    return 0
  fi
  sed -E 's/[[:cntrl:]]//g' "$result_path" >> "$log_path"
}

run_task() {
  local before_head after_head result_path exit_code=0 no_progress=0 validation="not reported"
  before_head="$(git -C "$repository_root" rev-parse HEAD)"
  result_path="$state_root/last-message.$(date +%s).txt"
  write_status "Inspecting and implementing the highest-value authorized QuireForge task" "running" "$validation" "Complete the current safe task and validate it" "none"
  log "starting Codex task at $before_head"

  if codex exec --ephemeral --sandbox workspace-write \
    --cd "$repository_root" --output-last-message "$result_path" \
    'Read and obey AGENTS.md first. Inspect docs/CURRENT_STATE.md, docs/ROADMAP.md, relevant ADRs, and git status. Continue only the highest-value safe QuireForge implementation task already authorized by the active milestone. Do not deploy, publish, access credentials, use browser sessions, begin a new milestone, or expand beyond the Linux Tauri desktop scope. Do not pause for ordinary status, tests, documentation, commits, or pushes: for routine validated work, run the required checks, commit only your task files, and push only the authoritative branch. Never put credentials, source payloads, signing material, release artifacts, secret-bearing URLs, headers, or secrets in your final report. If no implementation task is authorized or a true human-only blocker exists, state exactly "HUMAN_ONLY_BLOCKER:" followed by the concise reason and make no product change. End every run with exactly two lines: "SUPERVISOR_VALIDATION: <concise non-secret result>" and "SUPERVISOR_PROGRESS: yes" only when you completed a validated, committed, pushed routine task; otherwise "SUPERVISOR_PROGRESS: no".' \
    >/dev/null 2>&1; then
    :
  else
    exit_code=$?
  fi

  record_final_message "$result_path"
  after_head="$(git -C "$repository_root" rev-parse HEAD)"
  if [[ -f "$result_path" ]] && grep -q '^SUPERVISOR_VALIDATION:' "$result_path"; then
    validation="$(grep '^SUPERVISOR_VALIDATION:' "$result_path" | tail -n 1 | sed 's/^SUPERVISOR_VALIDATION:[[:space:]]*//')"
  fi
  if [[ -f "$result_path" ]] && grep -q '^HUMAN_ONLY_BLOCKER:' "$result_path"; then
    human_blocker "$(grep '^HUMAN_ONLY_BLOCKER:' "$result_path" | tail -n 1 | sed 's/^HUMAN_ONLY_BLOCKER:[[:space:]]*//')"
    rm -f -- "$result_path"
    return 0
  fi
  if [[ $exit_code -ne 0 ]] && [[ -f "$result_path" ]] && grep -Eqi 'credential|authentication|login|required approval|permission denied' "$result_path"; then
    human_blocker "Codex requires human credential or approval intervention."
    rm -f -- "$result_path"
    return 0
  fi
  if [[ $exit_code -ne 0 ]]; then
    log "Codex exited with status $exit_code"
    write_status "Autonomous task execution" "failed" "$validation" "Inspect the safe local log, then restart if appropriate" "Codex exited without a human-only blocker"
    no_progress=1
  elif [[ ! -f "$result_path" ]] || ! grep -qx 'SUPERVISOR_PROGRESS: yes' "$result_path" || [[ "$before_head" == "$after_head" ]]; then
    no_progress=1
  fi

  if [[ $no_progress -eq 1 ]]; then
    local count=0
    [[ -f "$progress_path" ]] && count="$(<"$progress_path")"
    count=$((count + 1))
    printf '%s\n' "$count" > "$progress_path"
    chmod 600 -- "$progress_path"
    log "no progress run $count of 2"
    if [[ $count -ge 2 ]]; then
      log "stopping after two no-progress runs"
      write_status "No authorized task made progress" "idle" "$validation" "Wait for new authorized work, then restart" "none"
      rm -f -- "$result_path"
      return 1
    fi
  else
    rm -f -- "$progress_path"
    log "validated progress committed and pushed"
    write_status "Completed a validated QuireForge task" "idle" "$validation" "Start the next authorized task" "none"
  fi
  rm -f -- "$result_path"
}

while [[ $stop_requested -eq 0 ]]; do
  if [[ -e "$sentinel_path" ]]; then
    log "sentinel exists; stopping cleanly"
    write_status "Awaiting human-only blocker resolution" "blocked" "not run" "Wait for explicit owner direction, remove the sentinel, then restart" "$(<"$sentinel_path")"
    exit 0
  fi
  if ! run_task; then
    exit 0
  fi
  [[ "$mode" == "once" ]] && exit 0
  sleep "$interval_seconds" & wait $! || true
done
