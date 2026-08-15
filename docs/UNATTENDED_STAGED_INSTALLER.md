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

`quireforge-stage-deb` is the only narrow sudo entry used by this workflow. It
accepts exactly one non-empty `.deb` directly from a pinned Ubuntu build output
directory named `/tmp/quireforge-beta<N>-package/target/ubuntu-22.04/release/packages`,
then installs a root-owned, non-writable copy directly into the staging root.
It does not install packages and cannot accept other source locations.
The setup binds that entry to the non-root desktop user who invokes it.

Install that helper and validate its sudoers entry with one authenticated
command:

```bash
sudo scripts/setup_quireforge_stage_deb.sh
```

Then stage each package before asking the root-owned daemon to install it:

```bash
sudo quireforge-stage-deb /tmp/quireforge-beta<N>-package/target/ubuntu-22.04/release/packages/quireforge_<version>_amd64.deb
sudo quireforge-stage-deb /tmp/quireforge-beta<N>-package/target/ubuntu-22.04/release/packages/quireforge-sandboxd_<version>_amd64.deb
```
