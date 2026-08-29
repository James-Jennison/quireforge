#!/usr/bin/env bash
# One-time root setup for the narrowly scoped QuireForge package staging helper.
set -Eeuo pipefail

readonly repository_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
readonly stage_helper=/usr/local/sbin/quireforge-stage-deb
readonly sudoers_file=/etc/sudoers.d/quireforge-stage-deb
readonly package_build_root_source="$repository_root/packaging/linux/package-build-root"
readonly package_build_root_file=/etc/quireforge/package-build-root
readonly install_user="${SUDO_USER:-}"

[[ ${EUID:-1} -eq 0 ]] || { printf 'run through sudo\n' >&2; exit 77; }
[[ -n "$install_user" && "$install_user" != root ]] || { printf 'run through sudo as the intended desktop user\n' >&2; exit 64; }
readonly sudoers_line="$install_user ALL=(root) NOPASSWD: /usr/local/sbin/quireforge-stage-deb"

/usr/bin/install -o root -g root -m 0755 "$repository_root/scripts/quireforge_stage_deb.sh" "$stage_helper"
/usr/bin/install -d -o root -g root -m 0755 /etc/quireforge
/usr/bin/install -o root -g root -m 0644 "$package_build_root_source" "$package_build_root_file"
/usr/bin/install -d -o root -g root -m 0755 /opt/quireforge/packages

temporary_sudoers=$(/usr/bin/mktemp /etc/sudoers.d/quireforge-stage-deb.tmp.XXXXXX)
trap '/usr/bin/rm -f -- "$temporary_sudoers"' EXIT
printf '%s\n' "$sudoers_line" > "$temporary_sudoers"
/usr/bin/chown root:root "$temporary_sudoers"
/usr/bin/chmod 0440 "$temporary_sudoers"
/usr/sbin/visudo -cf "$temporary_sudoers"
/usr/bin/install -o root -g root -m 0440 "$temporary_sudoers" "$sudoers_file"
/usr/sbin/visudo -c

printf 'Staging helper setup complete.\n'
