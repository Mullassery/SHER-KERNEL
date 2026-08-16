# SHER Kernel

**A userspace prototype of OS-kernel object-model, scheduling, memory, and driver-lifecycle concepts — not a bootable kernel.**

---

## What this project actually is

SHER Kernel is a Rust **workspace of 40 userspace crates** (`std` + `tokio`, no `no_std`) that prototypes what the internal APIs of a future, from-scratch OS kernel *might* look like: a capability-based object model, a priority scheduler, tiered memory bookkeeping, a driver lifecycle/registry, crash recovery, and an A/B transactional updater.

It is **not**:
- A bootable kernel. There is no bootloader, no ring-0/bare-metal code, no real MMU/interrupt-controller programming anywhere in this repository.
- A Linux replacement you can install or boot.
- A validated performance comparison against Linux (see [Performance notes](#performance-notes) below for what the numbers in this repo's docs actually measure).

It **is**:
- A working, tested simulation of kernel-shaped subsystems, runnable as an ordinary process on macOS/Linux.
- A reasonable place to prototype and unit-test object-model/scheduling/security-policy ideas before any of them would touch real hardware.
- Honest, as of this revision, about which parts are real logic vs. which parts are placeholders for things that fundamentally require kernel privileges (see below).

If you came here expecting a kernel you can boot on bare metal, this is not that project (yet, if ever). If you came here to look at how a capability-based, driver-isolated object model could be designed and tested in Rust, that's exactly what's here.

## Status

- **764 unit/integration tests, 100% passing** (`cargo test --workspace`)
- **`cargo clippy --workspace -- -D warnings`**: clean
- **`cargo fmt --check`**: clean
- 40 crates surveyed; the ones that were near-empty stubs (`pub fn x() {}`) have been implemented with real, tested logic — see the breakdown below.
- Not published anywhere (no crates.io/PyPI); consumed by sibling repos (`SHER-Graphics`, `SHER-Display`) via Cargo path dependencies.

## Quick Start

```bash
git clone https://github.com/Mullassery/SHER-KERNEL.git
cd SHER-KERNEL

# Run the full test suite
cargo test --workspace

# Lint and format checks
cargo clippy --workspace -- -D warnings
cargo fmt --check

# Build everything
cargo build --workspace

# Run the CLI (prints an accurate status summary, not a boot sequence)
cargo run --bin sher-kernel -- --status
```

## What's real vs. simulated

This is the honest part. Every crate falls into one of three buckets.

### Real, tested userspace logic (no hardware claims)

These implement actual algorithms/state machines and have unit tests exercising them — they are genuine, just not privileged:

| Crate | What's real |
|---|---|
| `common` | Core types (`ObjectId`, `Capability`, error types) |
| `objectmodel` | Kernel object identity, lifecycle state machine, time-bounded capability grants, telemetry/health |
| `scheduler` | Priority-queue task scheduler (highest-priority-first, FIFO tie-break), per-target queues |
| `compute` | Shared priority work queue backing simulated CPU/GPU/NPU/DSP dispatch |
| `memory` | Tiered slab allocators (Tier 0 per-CPU, Tier 1 per-socket), master allocator routing, DMA buffer bookkeeping, page-table map/unmap |
| `device_manager` | Device registry, state machine, hot-plug event queue |
| `driver_runtime` | Driver container lifecycle, sandboxing policy |
| `security` | Capability grants, sandbox policy, audit log |
| `interrupt` | Interrupt registration, shared-IRQ priority dispatch, enable/disable — **dispatch is simulated invocation bookkeeping, not real IRQ handling** |
| `diagnostics` | Fixed-capacity ring buffer, counter/gauge/event telemetry collector |
| `services` | Profile-based (Server/Workstation/Headless) lazy service-loading policy |
| `snapshot` | Versioned snapshot store with instant-rollback pointer semantics |
| `updater` | A/B transactional update state machine (download → verify → boot-test → commit/rollback) built on `recovery` |
| `recovery` | Immutable A/B partition bookkeeping, boot pointer with rollback, health-check probes, crash-recovery backoff/quarantine, watchdog heartbeats |
| `compatibility` | Linux/POSIX syscall-name → SHER-subsystem lookup tables (not a binary-compatible syscall ABI) |
| `networking`, `storage` | Device registries with MTU-checked send/receive and bounds-checked in-memory block I/O — **simulated, no real NIC/disk access** |
| `aro` | Host memory-tier detection via `/proc/meminfo` / `sysctl` (real on Linux/macOS, documented fallback elsewhere), battery/thermal adaptation policy |
| `runtime` | Lazy service registry |
| `ai` | Anomaly detection (memory-leak/interrupt-storm/DMA-abuse heuristics), predictive allocation, adaptive scheduling, reinforcement learning — real logic operating on synthetic/caller-supplied metrics, not live kernel telemetry |
| `lki` | Linux Kernel Interface: broader syscall-name → SHER-primitive translation with validation and security/audit layers |
| `hal`, `gpu_driver`, `audio_driver`, `input_driver`, `wayland_server`, `unified_device_manager` | Hardware Abstraction Layer and driver-shaped APIs consumed by sibling repos (see below) — real Rust logic, but they do not talk to physical GPU/audio/input hardware |
| `hardening`, `security_audit`, `performance_optimization`, `recovery`, `profiling`, `digital_twins`, `system_integration`, `performance_benchmarks`, `release_engineering`, `benchmarks` | Real, tested supporting subsystems (memory-safety checks, syscall hardening, object pooling, profiling, event replay, integration test harnesses, release/version bookkeeping) |
| `kernel` | Orchestrates the above into a single in-process `SherKernel` with audit logging, AI service wiring, and status reporting |

### Explicitly simulated — fundamentally requires hardware/kernel privileges

These are kept as clearly-labeled simulations rather than pretending to do real hardware I/O, because doing them for real requires ring-0 access or root/raw-device access this userspace crate does not have:

| Crate | Why it's simulated |
|---|---|
| `bootstrap` | Stage-0 CPU/MMU/heap bring-up — real hardware bring-up needs ring-0. `cpu::get_info()` does query real host parallelism via `std::thread::available_parallelism`; everything else is a documented, illustrative placeholder. |
| `core` | Stage-1 primitives are real in-process logic (object manager, IPC mailboxes, capability enforcement, timer wheel, FIFO CPU scheduler) — listed here only because the crate's job is to sit directly on top of `bootstrap`. |
| `drivers` | An early, self-contained prototype of discovery → driver-matching → sandboxed-load policy, superseded in the actual kernel wiring by `device_manager` + `driver_runtime` (kept for its own test coverage, not double-wired into `kernel`). |

Every module above states its simulation boundary in its own doc comments (`cargo doc --workspace --no-deps --open` to browse them).

## Cross-repo boundary

`SHER-Graphics` and `SHER-Display` depend on this repo via Cargo path dependencies (not published packages). The contracts that matter:

- **`hal::HardwareDriver`** is a real shared trait `SHER-Graphics` depends on — unchanged.
- **`wayland_server`** has functions/types marked `#[deprecated]` with notes that compositor/input/surface/output policy now belongs to `SHER-Display`; `wayland_server::WaylandTransport` is the low-level substrate `SHER-Display` actually consumes. Nothing in that boundary was altered.
- `cargo check --workspace` in `SHER-Graphics` and `SHER-Display` (when present locally) passes against this revision.

## Performance notes

Earlier revisions of this repo's docs (`BENCHMARK_RESULTS.md`, `PERFORMANCE_METRICS.md`) presented tables like "SHER vs Linux ACL: -88% overhead" as if they were a validated systems-level comparison. They were not: those numbers came from microbenchmarking individual in-process Rust operations (e.g. a `HashMap` lookup) against *assumed* Linux syscall costs, not from running an actual Linux kernel side-by-side. Treat any "SHER vs Linux" percentage in this repo's older docs as illustrative of relative micro-op cost, not a kernel performance claim. `crates/benchmarks` and `crates/performance_benchmarks` contain real Criterion/harness benchmarks of this repo's own code, which is what they can honestly measure.

## Project Organization

```
crates/
├── common/, objectmodel/, security/     # Foundation types, object model, capabilities
├── bootstrap/, core/, runtime/          # Staged boot simulation (Stage 0/1/2)
├── memory/, compute/, scheduler/        # Memory tiers, accelerator queues, priority scheduler
├── device_manager/, driver_runtime/,
│   drivers/, hal/                       # Device registry, driver lifecycle, HAL
├── interrupt/, networking/, storage/    # Simulated I/O-adjacent subsystems
├── security_audit/, hardening/          # Memory-safety & syscall hardening
├── recovery/, snapshot/, updater/       # A/B images, rollback, transactional updates
├── diagnostics/, profiling/,
│   performance_optimization/            # Telemetry, profiling, object pooling
├── ai/                                  # Anomaly detection, predictive allocation, RL
├── lki/, compatibility/                 # Linux/POSIX API-name translation tables
├── gpu_driver/, audio_driver/,
│   input_driver/, wayland_server/,
│   unified_device_manager/              # Driver-shaped subsystems (SHER-Display boundary)
├── digital_twins/, system_integration/,
│   benchmarks/, performance_benchmarks/,
│   release_engineering/                 # Testing/release tooling
└── kernel/                              # In-process orchestrator (SherKernel)
```

## Design Principles

- **Capability-based, time-bounded permissions**: every grant carries an expiry; nothing is permanent by default.
- **Driver isolation**: driver lifecycle goes through a sandbox/container abstraction rather than running inline.
- **Immutable A/B updates**: the updater only ever writes to the standby partition; the active boot pointer is a single, reversible pointer flip.
- **Honesty about simulation boundaries**: every module that can't do the real thing (hardware I/O, ring-0) says so in its own doc comment instead of pretending.

## Build & Test

```bash
cargo test --workspace                      # 764 tests
cargo clippy --workspace -- -D warnings     # lint, clean
cargo fmt --check                           # formatting, clean
cargo build --workspace                     # build everything
cargo doc --workspace --no-deps --open      # browse per-crate simulation-boundary docs
```

## Documentation

- **[CLAUDE.md](CLAUDE.md)** — architecture and implementation guide for this repo
- **[API_REFERENCE.md](API_REFERENCE.md)** — per-crate API reference (see individual crate doc comments for the authoritative simulation-boundary notes)
- Older docs (`FINAL_COMPLETION_STATUS.md`, `PROJECT_COMPLETION_SUMMARY.md`, `RELEASE_NOTES_1_0_0.md`, `BENCHMARK_RESULTS.md`, `PERFORMANCE_METRICS.md`, phase-plan files, etc.) describe an earlier, more marketing-driven characterization of this project ("v1.0.0 Production Ready"). They are kept for history but carry a correction notice at the top pointing back here; this README is the current source of truth.

## License

Proprietary License — free to use with explicit attribution to Georgi Mammen Mullassery. See [LICENSE](LICENSE).

## Contact & Attribution

**Project Author**: Georgi Mammen Mullassery
**Email**: mullassery@gmail.com
**GitHub**: [@Mullassery](https://github.com/Mullassery)
