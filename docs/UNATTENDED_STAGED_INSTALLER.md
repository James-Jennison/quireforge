# Unattended staged Debian installer

`quireforge-installd` is a root-owned systemd service that accepts installation
requests only for root-owned, non-writable Debian packages directly inside
`/opt/quireforge/packages/`. It rejects every other path after canonical path
resolution. The unprivileged client has no sudo capability and can only send a
bounded JSON request over `/run/quireforge-installd.sock`.

Install and enable the service in one authenticated command:

```bash
sudo scripts/setup_quireforge_installd.sh
```

No logout is required. After a separately trusted root-owned staging step,
invoke the client through `sg`, which supplies the new supplemental group to
that client process immediately:

```bash
sg quireforge-install -c 'quireforge-install /opt/quireforge/packages/quireforge.deb'
sg quireforge-install -c 'quireforge-install /opt/quireforge/packages/quireforge-sandboxd.deb'
```
