# Milestone 38 — Dynamic Sandbox / Malware-Analysis Discovery Gate

## Decision

**No-go for implementation.** QuireForge must not add dynamic binary analysis,
detonation, emulation, debugging, or malware handling to the desktop product at
this time. M39 remains intentionally undefined. Static ELF inspection remains
metadata-only and must not be treated as a path toward execution.

This is a discovery record only: it adds no product code, package, sandbox,
execution capability, dependency, or package-version change.

## Current boundary

QuireForge has three managed execution profiles: read-only, workspace-write,
and danger-full-access. Advisor itself is strictly read-only with no project
root and network disabled. The existing `static-binary-manifest-v1` attachment
never loads, executes, debugs, emulates, or transports executable bytes.

The host has Docker, bubblewrap, QEMU/KVM, user namespaces, and cgroup v2.
Those facts do not make it a safe malware-analysis environment:

- Docker and bubblewrap share the host kernel. They are useful defense-in-depth
  tools, not a sufficient primary boundary for hostile native code.
- A general QEMU invocation is not a product isolation design; it would still
  need a pinned guest image, device and mount policy, resource controls,
  trusted control channel, lifecycle cleanup, and operational ownership.
- No existing QuireForge runtime owns a guest image, VM lifecycle, network
  firewall, forensic workflow, or incident-response process.

The official gVisor security model likewise describes host resource controls
and network policy as host-level responsibilities, and cautions that a sandbox
is not a substitute for a secure architecture. Firecracker documents a narrower
KVM microVM and jailer model, but that is an architecture to build and operate,
not a crate to add to this desktop milestone. See [gVisor's security
model](https://gvisor.dev/docs/architecture_guide/security/) and
[Firecracker's architecture](https://firecracker-microvm.github.io/).

## Threat model

| Threat | Required mitigation if ever approved | Current result |
| --- | --- | --- |
| Kernel, VMM, device-model, or firmware escape | Hardware-virtualized per-run guest, minimal devices, patched host/firmware, independent security review | Blocks implementation |
| Network command-and-control or exfiltration | No virtual NIC, DNS, proxy, metadata service, or host loopback by default; host-enforced egress denial | Blocks implementation |
| Host/project/credential access | No host, project, home, SSH-agent, DBus, Docker, Tauri, clipboard, or credential mounts/sockets | Blocks implementation |
| CPU, memory, disk, fork, or output exhaustion | Per-run cgroup CPU/memory/pids/disk/time/output limits plus host-side kill and accounting | Blocks implementation |
| Persistence or cross-run contamination | Immutable guest image, disposable overlay, no shared folders, guaranteed teardown and verification | Blocks implementation |
| Unsafe analysis output | Typed, bounded, path-free metadata only; no raw sample bytes, terminal stream, executable output, or arbitrary files | Blocks implementation |
| Supply-chain compromise | Pinned and attested guest kernel/rootfs/VMM, vulnerability response, reproducible provenance | Blocks implementation |
| Legal, safety, abuse, and incident handling | Explicit acceptable-use, ownership/authorization, escalation, evidence-retention, and operator policy | Blocks implementation |

No unit or package test can prove resistance to a kernel, hypervisor, firmware,
or hardware side-channel escape. Tests can prove a proposed product policy, not
that hostile code is safe to execute.

## Feasible future architecture, if separately approved

The only candidate worth a separate architecture proposal is a Linux x86_64
KVM microVM worker, such as a narrowly configured Firecracker-class VMM. It
would be a separate service boundary, not an Advisor attachment parser and not
a normal Tauri command.

Minimum non-negotiable design requirements would be:

1. Explicit user-selected sample and per-run confirmation; no automatic
   detonation or handoff from Advisor, Approval/Dispatch, or static inspection.
2. KVM availability verified before a run; fail closed on unsupported hardware
   or host policy. Containers, bubblewrap, and user namespaces may supplement
   the boundary but cannot substitute for it.
3. One disposable guest per run with immutable, pinned kernel/rootfs and a
   minimal device model. No host, project, home, credential, clipboard, socket,
   or shared-folder mount.
4. Network disabled at the virtual-device and host-policy layers. Any future
   network behavior, including an allowlist, is a separate product and security
   decision.
5. Host-enforced CPU, memory, pid, disk, wall-time, and output limits; an
   independent watchdog; kill-on-timeout; verified guest and cgroup cleanup.
6. A one-way typed result protocol with a small bounded metadata schema. It
   must exclude raw sample bytes, paths, arbitrary guest files, terminal logs,
   secrets, and generic uploads.
7. Separate reproducible image/VMM provenance, patching, vulnerability response,
   legal/acceptable-use review, incident handling, and supported-host matrix.

Kata Containers and gVisor are credible isolation technologies in other
deployment models, but would add a container/runtime management stack. Kata
requires nested virtualization or bare metal and packages a kernel and VMM
choices; gVisor retains a process/container model and relies on host resource
and network enforcement. Neither is an approved M39 choice. See [Kata's
runtime architecture](https://katacontainers.io/software/) and [gVisor's
security guidance](https://gvisor.dev/docs/architecture_guide/intro/).

## Policy and lifecycle requirements for any future proposal

- Dynamic analysis is unavailable by default and is not part of Advisor,
  Approval/Dispatch, QuireForge execution, static binary inspection, or the
  normal desktop package.
- No sample, output, state, project context, terminal content, approval record,
  credential, or transcript may cross into the worker except through a new,
  explicitly approved typed contract.
- All guest disks and host-side staging data must be disposed after every
  outcome: success, failure, interruption, timeout, process crash, application
  close, mode change, and restart.
- CI may test lifecycle using benign deterministic probes only. Real malware,
  exploit samples, and live network behavior are outside routine developer and
  package validation.
- Any product proposal must include supported Linux hardware, privilege model,
  kernel/VMM/image update policy, legal and abuse review, operational ownership,
  and incident response before implementation begins.

## Recommendation and next decision

Close M38 as a documented **no-go** for dynamic analysis in the current desktop
architecture. Do not define or begin M39 until a separate, read-only
architecture decision evaluates the microVM-only model above against product
need, operating responsibility, supported hosts, and legal/safety policy.

No package candidate is assigned: M38 is evidence-only. A future implementation
would require a new, strictly greater package candidate after its approved
architecture and dependency/image provenance design are known.
