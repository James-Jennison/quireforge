#!/usr/bin/env bash
# Root-owned staging boundary for locally built QuireForge Debian packages.
set -Eeuo pipefail

readonly staging_root=/opt/quireforge/packages
readonly package_build_root_file=/etc/quireforge/package-build-root

reject() {
  printf '%s\n' "$1" >&2
  exit 64
}

[[ ${EUID:-1} -eq 0 ]] || reject 'quireforge-stage-deb must run as root'
[[ $# -eq 1 ]] || reject 'expected exactly one Debian package path'

readonly requested_path=$1
[[ "$requested_path" == /* && "$requested_path" == *.deb ]] || reject 'expected an absolute .deb path'
[[ -f "$requested_path" && -s "$requested_path" && ! -L "$requested_path" ]] || reject 'package must be a non-empty regular file'

readonly resolved_package=$(/usr/bin/readlink -f -- "$requested_path")
readonly resolved_parent=$(/usr/bin/dirname -- "$resolved_package")
[[ -f "$package_build_root_file" && ! -L "$package_build_root_file" ]] || reject 'trusted package build root definition is unavailable'
readonly persistent_build_root=$(<"$package_build_root_file")
[[ "$persistent_build_root" == /* && "$persistent_build_root" != / ]] || reject 'trusted package build root must be an absolute non-root path'
readonly temporary_build_pattern='/tmp/quireforge-beta[0-9]+-package/target/ubuntu-22.04/release/packages'
readonly workspace_build_pattern='/home/jjennison/.codex/worktrees/[0-9]+/quireforge/target/ubuntu-22.04/release/packages'
readonly persistent_build_output="$persistent_build_root/target/ubuntu-22.04/release/packages"
if ! [[ "$resolved_parent" =~ ^${temporary_build_pattern}$ || "$resolved_parent" =~ ^${workspace_build_pattern}$ || "$resolved_parent" == "$persistent_build_output" ]]; then
  reject 'package is outside the trusted pinned build output directory'
fi

/usr/bin/install -d -o root -g root -m 0755 "$staging_root"
/usr/bin/install -o root -g root -m 0644 -- "$resolved_package" "$staging_root/$(/usr/bin/basename -- "$resolved_package")"
printf 'staged %s\n' "$(/usr/bin/basename -- "$resolved_package")"
