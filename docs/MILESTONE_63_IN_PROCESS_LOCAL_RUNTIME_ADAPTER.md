# M63 — In-Process Credential-Free Local Runtime Adapter

Status: approved source/build boundary; not runtime-enabled.

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

The model artifact is neither included in this repository nor packaged or
activated. The fixed application resource remains unavailable because this
source/build boundary has no Rust command or public adapter API that invokes
the vendored C API.

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

This is content-free provenance evidence only. It grants no model activation,
runtime API, provider, network, packaging, release, or deployment authority.

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
the closed CMake configuration, and that the build script starts only its two
approved CMake subprocesses. Both subprocesses clear the inherited environment,
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
fixed system tool path, rather than an inherited executable search path. It
also rejects any Rust runtime-source reference to a `llama_*` or
`ggml_*` C API, native FFI declaration, Rust `include!` source injection, or
Rust `#[path = "..."]` module injection,
keeping the vendor integration build-only until a later adapter is approved. It
also rejects CMake command
working-directory overrides or out-of-block command mutations. Full repository
validation remains required; no package or installed-host claim is made until
the separately approved offline model input is available.
The validator also requires its own build script and the scanned Rust runtime
source root to be real, non-symlinked filesystem entries before reading them,
so either guard cannot be redirected outside the reviewed working tree.
