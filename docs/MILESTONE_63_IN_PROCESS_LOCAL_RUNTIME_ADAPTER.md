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

No model artifact is included, acquired, hashed, packaged, or activated. The
fixed application resource therefore remains unavailable until a separately
approved offline acquisition records the model's upstream revision and
SHA-256. No Rust command or public adapter API invokes the vendored C API in
this source boundary.

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
approved CMake subprocesses. Both subprocesses strip inherited compiler,
CMake-injection, and GNU Make flag variables. The build script compiles only
the static `llama` target and its CPU ggml dependencies, after independently
recomputing and matching the pinned vendored source-tree SHA-256 before CMake
is invoked. It also rejects any Rust runtime-source reference to a `llama_*` or
`ggml_*` C API, keeping the vendor integration build-only until a later adapter
is approved. Full repository
validation remains required; no package or installed-host claim is made until
the separately approved offline model input is available.
