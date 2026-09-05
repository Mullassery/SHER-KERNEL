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

- **768 unit/integration tests, 100% passing** (`cargo test --workspace`, re-verified this pass; earlier revisions of this file said 767 — off by one, corrected here rather than left stale)
- **`cargo clippy --workspace -- -D warnings`**: clean
- **`cargo clippy --workspace --all-targets -- -D warnings`**: not clean — this repo had no CI at all until this pass, so nothing had ever checked lints inside `#[cfg(test)]` modules and `benches/`. Real, pre-existing warnings (mostly `clippy::clone_on_copy` on the `Copy`-deriving `ObjectId`, plus a handful of `identity_op`/`unnecessary_cast`/`module_inception`/`len_zero`/etc.) exist in the `#[cfg(test)]` code of `crates/ai`, `crates/device_manager`, `crates/driver_runtime`, `crates/kernel`, `crates/lki`, `crates/performance_benchmarks`, `crates/security_audit`, `crates/system_integration`, and `crates/wayland_server`, plus one `clippy::unit_arg` warning in `crates/memory`'s `benches/allocator_bench.rs`. None of them are in the library/binary code the narrower, currently-enforced command above covers. Tracked as real follow-up work, not fixed in this pass.
- **`cargo fmt --check`**: clean
- **`cargo build --workspace --all-targets`** (including `benches/`): clean as of this pass — 5 benchmark files (`crates/memory/benches/allocator_bench.rs`, `crates/benchmarks/benches/{memory_allocation,lki_translation,device_enumeration,security_checks}.rs`) didn't compile at all before this pass (missing `criterion` dev-dependency, and calls into `sher_lki`/`sher_device_manager`/`sher_security`/`MemoryAllocator` APIs that had since been redesigned). Rewrote each against the current real APIs and verified every benchmark function actually executes (`cargo bench --workspace -- --test`).
- 40 crates surveyed; the ones that were near-empty stubs (`pub fn x() {}`) have been implemented with real, tested logic — see the breakdown below.
- Not published anywhere (no crates.io/PyPI); consumed by sibling repos (`SHER-Graphics`, `SHER-Display`) via Cargo path dependencies.
- **CI**: this repo had no `.github/workflows/` at all before this pass — every claim above had to be re-verified by hand rather than continuously checked. Added a standard `cargo fmt`/`build --all-targets`/`test`/`clippy` workflow, matching the pattern already used by the sibling `SHER-Graphics`/`SHER-Input` repos.

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

## Known gaps (external critique, verified)

- **The in-process IPC primitive is now a real lock-free, zero-copy ring buffer — fixed.** `crates/core/src/ipc.rs` (`IpcBus`) used to be a `HashMap<String, VecDeque<Message>>` mailbox behind `&mut self` (needs an external mutex to share across threads) that copied every payload into an owned `Vec<u8>`. Each mailbox is now a bounded [`crossbeam_queue::ArrayQueue`](https://docs.rs/crossbeam-queue) (a well-established lock-free bounded queue), and `Message::payload` is `Arc<[u8]>` — `send`/`receive` take `&self` and are safe to call concurrently from multiple threads with no mutex, and passing a framebuffer/input-event buffer is an O(1) refcount clone, not a byte copy. Verified with a real multi-threaded test (8 producer threads, no external locking, `IpcBus` shared via `Arc`) and a pointer-identity test proving payloads aren't copied. **Still not real cross-process IPC**: there is still no actual transport for framebuffers/input events to `SHER-Display` — that cross-repo data path isn't implemented here, only the in-process primitive it would build on. This repo is a single-process userspace prototype (see CLAUDE.md); real cross-process transport (shared memory segments, a socket protocol, etc.) is a materially different, OS-process-boundary-crossing feature that would need to be designed jointly with `SHER-Display`, not something this crate can honestly claim on its own.
- **No fuzzing.** No `fuzz/` directory, no cargo-fuzz/libfuzzer/afl anywhere in the repo, and CI (`.github/workflows/ci.yml`) only runs fmt/build/test/clippy. Syscall-parameter validation in `hardening`/`lki` is unit-tested but never fuzzed against malformed/adversarial input.
- **"Isolated driver runtime" is object-model isolation, not OS-level sandboxing** — worth being explicit about this distinction if it ever comes up externally. Crash-restart is real (`crates/recovery/src/crash_recovery.rs`: exponential backoff, quarantine after repeated crashes; `driver_runtime/src/container.rs` tracks `crash_count` and allows `Stopped → Starting`), and `driver_runtime/src/sandbox.rs` enforces real in-process capability/syscall/file-access policy checks. But there's no `unsafe`, no `process::Command`/fork, no seccomp/cgroup/namespace usage anywhere in `driver_runtime` — drivers run in-process with the kernel object model, not as separate unprivileged OS processes. That's consistent with this repo's stated userspace-prototype scope (see CLAUDE.md), not a bug to fix, but the gap between "policy-level isolation" and "real process isolation" matters if this is ever pitched as literal driver crash containment.

## Cross-repo boundary

`SHER-Graphics` and `SHER-Display` depend on this repo via Cargo path dependencies (not published packages). The contracts that matter:

- **`hal::HardwareDriver`** is a real shared trait `SHER-Graphics` depends on — unchanged.
- **`wayland_server`** has functions/types marked `#[deprecated]` with notes that compositor/input/surface/output policy now belongs to `SHER-Display`; `wayland_server::WaylandTransport` is the low-level substrate `SHER-Display` actually consumes. Nothing in that boundary was altered.
- `cargo check --workspace` in `SHER-Graphics` and `SHER-Display` (when present locally) passes against this revision.

### Cross-repo compatibility (verified, whole family)

This repo is the foundation of a family of five sibling repos under the
Mullassery org, all expected to be cloned as sibling directories:
`SHER-Kernel`, `SHER-Graphics`, `SHER-Display`, `SHER-Input`, and `Aurora`
(GitHub: `SHER-Aurora`). Actual Cargo-level coupling, confirmed by reading
every `Cargo.toml` in the family (not assumed from naming):

- **SHER-Kernel** (this repo) — foundation, no dependency on any sibling.
- **SHER-Graphics** — depends on this repo (`sher_common`, `sher_objectmodel`,
  `sher_security`, `hal`, `gpu_driver`) via relative path.
- **SHER-Input** — standalone, no dependency on any sibling.
- **SHER-Display** — depends on all three: this repo (`sher_common`,
  `sher_objectmodel`, `gpu_driver`, `wayland_server`), SHER-Graphics
  (`graphics_api`, `gpu_abstraction`, `graphics_runtime`, `graphics_compat`),
  and SHER-Input (`sher_input_core`, `sher_input_test`) — all via relative
  path, so all four repos must be sibling directories for its build to
  resolve.
- **Aurora** — zero Cargo-level coupling to any of the above four. It's a
  standalone GTK/Qt/Web design-system toolkit; the shared "SHER" naming
  (`SHER-Aurora` on GitHub) is organizational only, not a build dependency.

All four coupled repos use Rust edition 2021 (semver differs by design —
Kernel/Graphics/Input are at 0.2.0, Display at 0.1.0, being the newest).
Verified empirically: a from-scratch `cargo build --workspace` in
SHER-Graphics against this repo's current state compiles clean; a
from-scratch `cargo build --workspace` in SHER-Display against this repo +
SHER-Graphics + SHER-Input compiles clean across all 14 SHER-Display
crates; `cargo test --workspace` in SHER-Display passes 56/56, genuinely
exercising the cross-repo boundary (SHER-Input's simulated backend, this
repo's `gpu_driver` value types, SHER-Graphics's `graphics_api`/
`graphics_runtime`). Only SHER-Graphics depends directly on a low-level
external graphics crate (`ash` 0.38); it isn't leaked across the repo
boundary, so there's no divergent-version risk for shared external types.

Boundary discipline (never instantiate a driver another subsystem already
owns) holds across the live chain: SHER-Display's `outputs` crate does not
instantiate `gpu_driver::GPUDriver`, only consuming its value types; its
`input` crate consumes the real `sher_input_core::InputService` rather than
reimplementing it. One disclosed exception exists *within this repo*: the
`system_integration` crate (a deprecated, internal-only Phase-12
integration-test harness — see its own doc comment) instantiates its own
`GPUDriver`/`WaylandCompositor`/etc. for isolated end-to-end testing. It
explicitly states it is not part of the real cross-repo boundary and warns
against copying the pattern elsewhere — a known, self-documented exception,
not a live violation.

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
cargo test --workspace                      # 768 tests
cargo clippy --workspace -- -D warnings     # lint, clean
cargo fmt --check                           # formatting, clean
cargo build --workspace                     # build everything
cargo doc --workspace --no-deps --open      # browse per-crate simulation-boundary docs
```

## Documentation

- **[CLAUDE.md](CLAUDE.md)** — architecture and implementation guide for this repo
- **[VISION.md](VISION.md)** — why this repo exists and what it's actually
  prototyping, rewritten to match this README's standard (no longer the old
  "operating system for the next decade" pitch)
- **[ROADMAP.md](ROADMAP.md)** — what's built per crate, what's explicitly
  not started, and a concrete near-term plan (also rewritten; no longer the
  old 16-week production-release schedule)
- **[API_REFERENCE.md](API_REFERENCE.md)** — per-crate API reference (see individual crate doc comments for the authoritative simulation-boundary notes)
- Older docs (`FINAL_COMPLETION_STATUS.md`, `PROJECT_COMPLETION_SUMMARY.md`, `RELEASE_NOTES_1_0_0.md`, `BENCHMARK_RESULTS.md`, `PERFORMANCE_METRICS.md`, phase-plan files, etc.) describe an earlier, more marketing-driven characterization of this project ("v1.0.0 Production Ready"). They are kept for history but carry a correction notice at the top pointing back here; this README is the current source of truth.

## License

This project is licensed under the [Apache License 2.0](LICENSE).

## Contact & Attribution

**Project Author**: Georgi Mammen Mullassery
**Email**: mullassery@gmail.com
**GitHub**: [@Mullassery](https://github.com/Mullassery)
