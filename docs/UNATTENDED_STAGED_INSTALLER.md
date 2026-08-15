# Unattended staged Debian installer

`quireforge-installd` is a root-owned systemd service that accepts installation
requests only for root-owned, non-writable Debian packages directly inside
`/opt/quireforge/packages/`. It rejects every other path after canonical path
resolution. The unprivileged client has no sudo capability and can only send a
bounded JSON request over `/run/quireforge-installd.sock`.

Install these source files manually; this repository does not install or enable
the service:

```bash
sudo groupadd --system quireforge-install
sudo usermod -aG quireforge-install james
sudo install -o root -g root -m 0755 scripts/quireforge_installd.py /usr/local/sbin/quireforge-installd
sudo install -o root -g root -m 0755 scripts/quireforge_install.py /usr/local/bin/quireforge-install
sudo install -o root -g root -m 0644 packaging/systemd/quireforge-installd.service /etc/systemd/system/quireforge-installd.service
sudo install -d -o root -g root -m 0755 /opt/quireforge/packages
sudo systemctl daemon-reload
sudo systemctl enable --now quireforge-installd.service
```

Start a new login session after the group change. After a separately trusted
root-owned staging step, invoke the client from the build workflow with:

```bash
quireforge-install /opt/quireforge/packages/quireforge.deb
quireforge-install /opt/quireforge/packages/quireforge-sandboxd.deb
```
