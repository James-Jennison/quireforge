# QuireForge sandbox worker packaging

`quireforge-sandboxd` is a separately installed Debian system component. It is
not bundled into the desktop Debian package and is not a normal desktop
dependency.

The authoritative Ubuntu 22.04 workflow builds the fixed guest kernel,
initramfs agent, Firecracker `v1.15.1`, and matching jailer from the checksum
pinned inputs in `sources.lock`. The worker package must record the desktop
version, source commit, container provenance, and SHA-256 values for every
worker asset.

The service is root-owned solely to create jailer/cgroup state. It starts each
Firecracker process as the dedicated unprivileged sandbox identity, accepts no
TCP connections, creates no network device, and exposes a Unix socket only to
the locally authorized desktop client group. Deleting a package is separate
from any sample lifecycle: every run deletes its own input, serial capture,
jail root, and cgroup state before responding.
