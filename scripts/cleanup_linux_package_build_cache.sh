#!/usr/bin/env bash
# Reap only marked, abandoned QuireForge packaging scratch directories.
set -Eeuo pipefail

umask 077

readonly repository_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
readonly package_build_root_file="$repository_root/packaging/linux/package-build-root"
[[ -f "$package_build_root_file" && ! -L "$package_build_root_file" ]] || {
  printf 'Trusted QuireForge package-build root definition is unavailable.\n' >&2
  exit 1
}
readonly build_root="$(<"$package_build_root_file")"
[[ "$build_root" == /* && "$build_root" != / ]] || {
  printf 'Trusted QuireForge package-build root must be an absolute non-root path.\n' >&2
  exit 1
}
readonly build_marker="$build_root/.quireforge-linux-packaging-root"
readonly temporary_root="$build_root/tmp"
readonly run_marker='.quireforge-linux-packaging-run'
readonly maximum_age_seconds=$((7 * 24 * 60 * 60))
readonly maximum_retained_bytes=$((30 * 1024 * 1024 * 1024))

[[ ! -e "$build_root" ]] && exit 0
[[ -d "$build_root" && ! -L "$build_root" ]] || {
  printf 'QuireForge packaging build root is unavailable: %s\n' "$build_root" >&2
  exit 1
}
[[ -f "$build_marker" && "$(<"$build_marker")" == 'quireforge-linux-packaging-build-root-v1' ]] || {
  printf 'Refusing an unrecognized QuireForge packaging build root: %s\n' "$build_root" >&2
  exit 1
}
[[ -d "$temporary_root" && ! -L "$temporary_root" ]] || {
  printf 'QuireForge packaging scratch root is unavailable: %s\n' "$temporary_root" >&2
  exit 1
}

if [[ ${QUIRE_FORGE_BUILD_LOCK_HELD:-} != 1 ]]; then
  exec 9>"$build_root/.release.lock"
  if ! flock -n 9; then
    printf 'QuireForge packaging cleanup deferred: a release build is active.\n'
    exit 0
  fi
fi

is_managed_run() {
  local candidate=$1
  [[ "$candidate" == "$temporary_root"/run.* \
    && -d "$candidate" \
    && ! -L "$candidate" \
    && -f "$candidate/$run_marker" ]]
}

remove_run() {
  local candidate=$1
  is_managed_run "$candidate" || return 1
  rm -rf -- "$candidate"
  printf 'reaped QuireForge packaging scratch directory: %s\n' "$candidate"
}

now=$(date +%s)
while IFS= read -r -d '' candidate; do
  is_managed_run "$candidate" || continue
  modified=$(stat -c %Y -- "$candidate")
  if (( now - modified >= maximum_age_seconds )); then
    remove_run "$candidate"
  fi
done < <(find "$temporary_root" -mindepth 1 -maxdepth 1 -type d -name 'run.*' -print0)

while (( $(du -sb -- "$temporary_root" | awk '{print $1}') > maximum_retained_bytes )); do
  oldest=$(find "$temporary_root" -mindepth 1 -maxdepth 1 -type d -name 'run.*' \
    -printf '%T@ %p\n' | sort -n | head -n 1 | cut -d ' ' -f 2-)
  [[ -n "$oldest" ]] || break
  remove_run "$oldest" || break
done
