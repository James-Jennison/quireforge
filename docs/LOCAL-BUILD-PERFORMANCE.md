# Local Build Performance

Status: initial generalized baseline captured from required Milestone 3 work on
2026-07-19. These measurements guide local forecasts; they are not release
performance claims or a supported-hardware baseline.

## Generalized development profile

| Resource | Observed profile |
|---|---|
| Operating system | Ubuntu 26.04 LTS, Linux 7.0, x86_64 |
| CPU | AMD Ryzen 5 5600G class, 6 physical cores / 12 logical processors |
| System memory | Approximately 61 GiB total; 47 GiB available at audit |
| Swap | 8 GiB file-backed swap; about 176 MiB used; no zram configured |
| Workspace storage | NVMe-backed ext4; approximately 733 GiB available |
| GPU | NVIDIA GeForce RTX 3050, 8 GiB VRAM; driver available; CUDA toolkit absent |
| Rust | rustc/Cargo 1.97.1 with rustfmt and Clippy |
| JavaScript | Node 22.22.1 and pnpm 11.15.0 |
| Native tools | GCC 15.2.0 and pkg-config 2.5.1 |
| Optional caches/linkers | Cargo and pnpm caches present; no sccache, ccache, mold, or LLVM lld |

The audit recorded no competing QuireForge build and a load average below two.
It did not collect hardware serial numbers, network addresses, private mount
names, credentials, or unrelated process details.

## Milestone 3 measurements

Measurements came from commands already required to implement or verify the
milestone. Caches were preserved; no clean build was run for benchmarking.

| Operation | Observed wall time | Cache state | Result |
|---|---:|---|---|
| Workspace dependency installation after manifest changes | about 5 seconds | Package cache populated; lockfile updated | Passed |
| First successful native `cargo check` | about 37 seconds | Partially populated after dependency download | Passed |
| First Rust test-profile build and tests | about 44 seconds | Cold test profile | Passed |
| First Tauri unbundled release build | about 1 minute 18 seconds | Cold release profile | Passed |
| Warm `cargo check` | about 0.4 seconds | Warm | Passed |
| Warm Clippy with warnings denied | about 0.5 seconds | Warm | Passed |
| Warm Rust tests | about 4 seconds | Warm | Passed |
| Desktop Vite production transform/build | about 0.12–0.15 seconds | Warm frontend cache | Passed |
| Astro static build | about 0.35–0.37 seconds | Warm | Passed, 15 pages |
| Combined desktop and website browser suites | about 8 seconds | Browser installed; two workers per package | Passed, 14 tests |

Peak memory was not instrumented during Milestone 3. The post-build audit showed
substantial available memory, minimal pre-existing swap use, low system load,
and no evidence of memory pressure. GPU computation was not used.

## Current execution guidance

- Default to the Balanced profile and preserve desktop responsiveness.
- Preserve Cargo, target, pnpm, Vite, Astro, and browser caches.
- Use targeted frontend tests and `cargo check` during implementation.
- Run the locked full validation and browser suites at milestone gates.
- Avoid simultaneous cold Rust release builds and other heavy workloads unless
  a current memory/load check shows sufficient headroom.
- Continue using the existing two-worker Playwright configuration.
- Select a Cargo job count for each milestone after checking current load and
  task shape; do not persist a new limit without evidence.
- Do not install sccache, an alternative linker, CUDA, or new build tooling
  without a measured need, compatibility review, and separate approval.

The RTX 3050 is reserved for genuine GPU workloads such as WebGL/WebGPU,
shader, CUDA, ML, or hardware-rendering validation. Rust, Tauri, React,
TypeScript, Vite, and Astro compilation remains CPU/system-memory work.
