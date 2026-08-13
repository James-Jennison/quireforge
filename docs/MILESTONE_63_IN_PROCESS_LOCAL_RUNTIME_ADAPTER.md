# M63 — In-Process Credential-Free Local Runtime Adapter

Status: initial local-only runtime path implemented; not release-ready.

M63 selects only a native in-process llama.cpp route for the future
credential-free local runtime. It is CPU-only and permits no GPU offload. The
only approved descriptor is `Qwen/Qwen2.5-3B-Instruct-GGUF`, artifact
`qwen2.5-3b-instruct-q4_k_m.gguf`, quantization `Q4_K_M`, with 4,096 input
tokens, 512 output tokens, one concurrent attempt, a 60-second deadline, and a
6 GiB memory ceiling.

The vendored source is the verified `ggml-org/llama.cpp` `b10326` snapshot.
Its provenance is recorded in `third_party/llama.cpp/PROVENANCE.json`. Cargo
configures only static `llama` and CPU `ggml` targets. All command-line,
server, UI, examples, upstream test, benchmark, install, CURL, subprocess,
network, dynamic-backend, and GPU/backend build options are explicitly off.
CMake user and system package registries are disabled, and the build cannot
export a package registry entry.
Each configuration starts with CMake's `--fresh` mode, so generated cache
state cannot add configuration outside the fixed closed option list.

The model artifact is neither included in this repository nor packaged. The
initial adapter consumes one confirmed M60 reviewed bundle once, invokes only
the local CPU-only runtime, bounds input/output/deadline, and keeps its result
in the open local view. It has no provider, network, credential, tool, or
external-action path. End-to-end model execution and installed-host acceptance
remain required before any release-ready claim.

Before an acknowledged review can consume its one local attempt, the governed
view obtains a typed, content-free availability snapshot from the native
adapter, and the native run command enforces the same check before reservation
or durable consumption. A missing supervisor-provided local-model contract
leaves the exact review unconsumed, disables the one-time action, and reports
only `model-unavailable`; it exposes no path or model observation. This
preflight does not load or otherwise inspect the model.

Each attempt applies the fixed 6 GiB process address-space ceiling before model
loading and restores the prior soft limit as the attempt exits. If the ceiling
cannot be applied, the attempt fails locally before model loading with a
bounded diagnostic and no retry.

## Local candidate evidence

The local candidate records the M63 attempt as `dispatching` and clears the
durable canonical bundle bytes before invoking the in-process runtime. It adds
content-free authorization and dispatch audit events before the call, allows
only one terminal transition, and expires an interrupted dispatch on restart.
The CPU-bound model call runs on Tauri's blocking worker pool, leaving the
native command executor available for the exact reviewed-bundle cancellation
request; the shared local-runtime service still admits only one active attempt.
The governed local-only view exposes the nonterminal `running` phase while the
one CPU attempt is pending, including its fixed token, deadline, and 6 GiB
memory limits and no-automatic-retry posture; its close control remains
unavailable until that phase is replaced by the bounded returned result or
failure. While the attempt is pending, the exact
reviewed bundle can request cancellation; the in-process callbacks observe that
request and finish as the bounded `cancelled` terminal outcome, with no retry.
The result remains in the open view only.
The focused storage and workbench tests, repository validation, package
validator, type-check, lint, and formatting gates passed locally on
2026-08-09. The authoritative pinned Ubuntu 22.04 package gate subsequently
produced and validated the clean-tree `0.1.0-beta.64` local candidate at source
commit `707a760a6cd1f3ca08bccaf92677f2f5397d4f88`, with its two required Debian
artifacts and a visible package-launch smoke pass. That gate excluded the model
and did not start the runtime. It includes the storage lifecycle correction
that records a completed local attempt as canonical `closed`, a bounded local
failure as `failed`, and an accepted cancellation as `cancelled`. This remains
local candidate work, not a release-ready claim: installed-host M63
execution/desktop acceptance is still required.

The adapter's context-parameter C ABI now mirrors the pinned vendored llama.cpp
header, and its focused lifecycle test proves a second concurrent local attempt
is rejected immediately before it consumes a second reviewed bundle, while the
completed attempt releases the single slot.
The fixed Qwen route formats its two-message system/reviewed-request exchange
through that model's embedded chat template before tokenization; an unavailable
or oversized template result is a bounded local failure and never falls back to
an ad-hoc prompt format. This remains local candidate evidence only;
installed-host acceptance is still required.

The fresh clean-tree `0.1.0-beta.65` package pair at source commit
`6f77d0c4d73cefe5ed335898830872ca63ad203a` is a release-candidate manifest
with exactly the application and sandbox Debian artifacts. It passed the pinned
Ubuntu 22.04 package, lifecycle, visible-launch, and release-artifact gates;
it excludes the model and did not start the runtime. Focused native, workbench,
and loopback-only browser-fixture tests cover the authorization-lifecycle
correction and prove a completed result is cleared when the governed review is
closed and reopened. Installed-host M63 execution and desktop acceptance remain
required before a release-ready claim.

The current preparation, authorization, concurrent-admission, and availability
corrections require the uniquely versioned `0.1.0-beta.66` candidate before
installed-host acceptance. Beta.65 remains immutable prior evidence and must
not be overwritten.

At the beta.66 source candidate, focused native adapter/lifecycle and governed
workbench tests, repository validation, package-boundary tests, TypeScript
checks, lint, formatting, and the local desktop/mobile browser fixture passed
on 2026-08-12. The browser fixture covers one visible local-only attempt and
its exact-bundle cancellation using deterministic local fixtures; it neither
loads the model nor supplies installed-host acceptance. The clean-tree beta.66
Debian pair from source commit `822b6703968f4cea95ce4828f130739bc56e8a01`
then passed the authoritative pinned Ubuntu 22.04 package, lifecycle,
visible-launch, and release-artifact gates. That gate excluded the model and
did not start the runtime. On 2026-08-13, the focused host-native adapter gate
completed one bounded in-process attempt through the supervisor-owned,
read-only local-model contract. It retained no model location or generated
output and verified only the local-only bounded terminal contract. This is
real-adapter evidence, not installed Debian desktop acceptance. The uniquely
versioned beta.67 source candidate carries that evidence forward for a fresh
authoritative package/lifecycle/visible-launch gate followed by the explicit
installed-host governed-review workflow; both remain pending.

The candidate procedure is recorded in
[Testing](TESTING.md#m63-local-runtime-installed-host-acceptance). It requires
the installed-host operator to retain only content-free package and lifecycle
evidence after one explicit reviewed local attempt. The normal package gate
must still exclude the model and never starts the runtime.

## Approved offline acquisition record

The separately approved offline acquisition completed on
`2026-08-08T17:34:14-07:00` for the fixed artifact only. The local artifact is
outside the repository, excluded from Git, and read-only after verification.

| Field | Recorded value |
| --- | --- |
| Upstream repository | `Qwen/Qwen2.5-3B-Instruct-GGUF` |
| Pinned upstream revision | `7dabda4d13d513e3e842b20f0d435c732f172cbe` |
| Artifact | `qwen2.5-3b-instruct-q4_k_m.gguf` |
| Bytes | `2104932768` |
| SHA-256 | `626b4a6678b86442240e33df819e00132d3ba7dddfe1cdc4fbb18e0a9615c62d` |

This is content-free provenance evidence only. It grants no provider, network,
packaging, release, or deployment authority.

This work grants no credentials, account, OAuth, provider, socket, process,
shell, environment override, arbitrary filesystem, discovery, browser,
connector, MCP, tool, retrieval, automation, external mutation, package,
release, deployment, tag, or push authority. A later adapter implementation
must bind one M60 immutable reviewed bundle to one project/task-bound,
expiring, one-use attempt and preserve M62 cancellation, timeout, interruption,
outcome-unknown, quarantine, and content-free-audit rules.

## Validation

`scripts/validate_llama_cpp_vendor.py` verifies the provenance manifest,
deterministic source-tree digest, license evidence, absence of model artifacts,
bounded archive member counts, unambiguous relative names, regular-file/directory-only TAR entries, nesting, and standalone gzip/bzip2/xz
payload expansion, the closed CMake configuration, and
that the build script starts only its two approved CMake subprocesses. Both
subprocesses clear the inherited environment,
then set only the fixed system build path and pass fixed system C/C++ compiler,
archive-tool, and ranlib paths; this excludes compiler discovery and
CMake-injection, language compiler target/argument/toolchain/archive-tool,
generic archive/linker/symbol-tool, compiler-launcher/initial-flag, GNU Make
flag, compiler search/override, and dynamic-loader injection variables. CMake
user and system package registries
are disabled and no package registry entry can be exported. The build
environment also removes CMake language linker-launcher and static-analysis
tool variables before either configuration or compilation. The build
script compiles only the static `llama` target and its CPU ggml dependencies,
after independently rechecking that the vendored root remains a real directory
and recomputing and matching the pinned vendored source-tree SHA-256 before
CMake configuration, again immediately before compilation, and again after
compilation before Cargo receives static-library link directives.
Configuration starts with `--fresh`, discarding prior generated CMake cache
state before the fixed closed configuration is evaluated.
Both CMake invocations use the fixed system CMake executable and a
fixed system tool path, rather than an inherited executable search path. The
validator rejects `llama_*` or `ggml_*` C API references, native FFI
declarations, Rust `include!` source injection, or Rust `#[path = "..."]`
module injection everywhere except the reviewed `local_runtime.rs` adapter. It
also rejects CMake command working-directory overrides or out-of-block command
mutations. Full repository validation remains required; no package or
installed-host claim is made until the separately approved offline model input
is available.
It also permits only the fixed vendored-source change tracking and static CPU
linkage Cargo directives, rejecting any additional build-script output,
compile-configuration directive, or filesystem reference beyond the reviewed
read-only source-inspection calls.
The validator also pins the build script to its reviewed read-only filesystem
calls, rejecting source or generated-output mutation from that build boundary.
Each approved Cargo directive must be emitted directly by its fixed `println!`
call; indirect or dynamically assembled directive output is rejected.
The validator also requires its own build script and the scanned Rust runtime
source root and immediate parent to be real, non-symlinked filesystem entries
before reading them, so either guard cannot be redirected outside the reviewed
working tree.
