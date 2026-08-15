#!/usr/bin/env bash
# One-time root setup for the staged QuireForge installer daemon.
set -Eeuo pipefail

readonly repository_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
readonly install_user="${SUDO_USER:-}"

[[ ${EUID:-1} -eq 0 ]] || { printf 'run through sudo\n' >&2; exit 77; }
[[ -n "$install_user" && "$install_user" != root ]] || { printf 'run through sudo as the intended desktop user\n' >&2; exit 64; }

if ! /usr/bin/getent group quireforge-install >/dev/null; then
  /usr/sbin/groupadd --system quireforge-install
fi
/usr/sbin/usermod -aG quireforge-install "$install_user"

/usr/bin/install -o root -g root -m 0755 "$repository_root/scripts/quireforge_installd.py" /usr/local/sbin/quireforge-installd
/usr/bin/install -o root -g root -m 0755 "$repository_root/scripts/quireforge_install.py" /usr/local/bin/quireforge-install
/usr/bin/install -o root -g root -m 0644 "$repository_root/packaging/systemd/quireforge-installd.service" /etc/systemd/system/quireforge-installd.service
/usr/bin/install -d -o root -g root -m 0755 /opt/quireforge/packages

/usr/bin/systemctl daemon-reload
/usr/bin/systemctl enable --now quireforge-installd.service

printf 'Setup complete. Without logging out, invoke the client through:\n'
printf "sg quireforge-install -c 'quireforge-install /opt/quireforge/packages/quireforge.deb'\n"
