# Milestone 39 — Isolated ELF Analysis

Milestone 39 adds a separately administered analysis component only for
supported Linux x86_64 KVM hosts. It is deliberately separate from Advisor,
Approval/Dispatch, project execution, the integrated terminal, and the static
binary Advisor attachment.

## Closed input and runtime policy

The QuireForge-only panel accepts one explicitly selected, explicitly confirmed
ELF64 little-endian x86_64 `ET_EXEC` or `ET_DYN` source at a time. The native
client rejects every other format, core files, sources above 32 MiB, symlinks,
identity changes, unsafe names, and any `PT_INTERP`. `ET_DYN` is supported only
as static PIE. A dynamically linked source receives the bounded
`unsupported-runtime` diagnostic; no dynamic-loader or guest-library support
exists.

The native client holds source bytes only until the one-use confirmed claim.
It sends a closed typed request and the exact hash-bound bytes over the
administrator-controlled local Unix socket. The worker rechecks the hash and
the static ELF policy before creating a disposable run. Neither selected paths
nor raw bytes enter React, Advisor, project metadata, terminal output, logs,
or retained application state.

## Worker and guest boundary

`quireforge-sandboxd` is a separately installed root-owned Debian component;
the desktop Debian package does not install or start it. The service
uses a pinned Firecracker 1.15.1 and matching jailer, KVM only, a private
network namespace, no host or project mount, one vCPU, 512 MiB guest memory,
and a 30-second wall-clock limit. The immutable guest kernel (Linux 6.1.178),
initramfs, and non-interactive fixed agent are checksum-verified in the pinned
Ubuntu 22.04 packaging workflow.

The guest reads the single sealed block input, executes the static sample as an
unprivileged guest user with stdin/stdout/stderr disconnected, and emits one
fixed outcome only. It cannot provide terminal output, export guest files,
open a network connection, receive a project mount, or invoke Advisor. Output
is a bounded `dynamic-analysis-result-v1` containing only an opaque run ID,
outcome, elapsed time, guest-start indicator, and fixed resource-limit labels.

## Installation and operations

An administrator must install the worker Debian component, validate its release
manifest and asset checksums, ensure `/dev/kvm` is available, and explicitly
manage membership of the `quireforge-sandbox` group. The desktop panel reports
the isolated-analysis feature as unavailable when the worker is absent. The
worker is never started, installed, or enabled by the desktop package.

Only benign probe fixtures are permitted in automated validation. The feature
is not a malware-analysis service, does not provide a hostile-sample guarantee
outside the documented host requirements, and does not authorize releases or
deployment.
