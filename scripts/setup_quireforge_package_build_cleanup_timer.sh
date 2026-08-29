#!/usr/bin/env bash
# Install the user-scoped janitor for QuireForge packaging scratch directories.
set -Eeuo pipefail

readonly repository_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
readonly user_home="$(getent passwd "$(id -un)" | cut -d: -f6)"
readonly config_home="${XDG_CONFIG_HOME:-$user_home/.config}"
readonly unit_root="$config_home/systemd/user"

[[ -n "$user_home" && -d "$user_home" ]] || {
  printf 'Could not determine the invoking user home directory.\n' >&2
  exit 1
}

install -Dm600 \
  "$repository_root/packaging/systemd-user/quireforge-package-build-cleanup.service" \
  "$unit_root/quireforge-package-build-cleanup.service"
install -Dm600 \
  "$repository_root/packaging/systemd-user/quireforge-package-build-cleanup.timer" \
  "$unit_root/quireforge-package-build-cleanup.timer"

systemctl --user daemon-reload
systemctl --user enable --now quireforge-package-build-cleanup.timer
printf 'Enabled QuireForge packaging scratch cleanup timer.\n'
