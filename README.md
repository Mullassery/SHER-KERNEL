# SHER Kernel

A userspace Rust prototype of OS-kernel object-model, scheduling, memory, and driver-lifecycle concepts — **not a bootable kernel**.

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![CI](https://github.com/Mullassery/SHER-KERNEL/actions/workflows/ci.yml/badge.svg)](https://github.com/Mullassery/SHER-KERNEL/actions/workflows/ci.yml)

## What it is

SHER Kernel is a Rust workspace of 40 userspace crates (`std` + `tokio`, no `no_std`) that prototypes what the internal APIs of a future, from-scratch OS kernel *might* look like: a capability-based object model, a priority scheduler, tiered memory bookkeeping, a driver lifecycle/registry, crash recovery, and an A/B transactional updater.

It runs as an ordinary process on macOS or Linux — there is no bootloader, no ring-0/bare-metal code, and no real MMU or interrupt-controller programming anywhere in the repository. It is also the foundation of the SHER family: `SHER-Graphics` and `SHER-Display` consume its shared value types and traits (`hal`, `gpu_driver`, `wayland_server`) via Cargo path dependencies.

## Features / Highlights

- **40-crate workspace** with roughly 768 unit/integration tests passing (`cargo test --workspace`; run it for the authoritative count), `clippy -D warnings` and `fmt --check` clean.
- **Capability-based, time-bounded security** — every permission grant carries an expiry, with sandbox policy and an audit log (`objectmodel`, `security`, `security_audit`).
- **Priority scheduler** (highest-priority-first, FIFO tie-break) with per-target queues, plus a shared priority work queue for simulated CPU/GPU/NPU/DSP dispatch (`scheduler`, `compute`).
- **Tiered memory bookkeeping** — per-CPU and per-socket slab allocators, master routing, DMA buffer accounting, and page-table map/unmap (`memory`).
- **Driver lifecycle and crash recovery** — device registry, hot-plug queue, driver container lifecycle, sandbox policy, exponential-backoff crash recovery and quarantine (`device_manager`, `driver_runtime`, `recovery`).
- **A/B transactional updates** — download → verify → boot-test → commit/rollback built on `recovery`, with instant-rollback snapshot pointers (`updater`, `snapshot`).
- **Lock-free, zero-copy in-process IPC** — a bounded `crossbeam_queue::ArrayQueue` mailbox with `Arc<[u8]>` payloads, safe to `send`/`receive` concurrently with no external mutex (`core::ipc`).
- **AI subsystem over caller-supplied metrics** — anomaly detection (leaks, interrupt storms, DMA abuse), predictive allocation, adaptive scheduling, and reinforcement learning (`ai`).
- **Linux/POSIX name-level translation** — syscall-name → SHER-subsystem lookup tables with validation; explicitly not a binary-compatible syscall ABI (`lki`, `compatibility`).
- **Driver-shaped APIs for siblings** — HAL and hardware/GPU/audio/input/Wayland value types consumed by `SHER-Graphics` and `SHER-Display`.

## Who it's for / Use cases

- **Prototyping capability-based, time-bounded permission models** before committing to real kernel/hardware work — `security`/`objectmodel` provide real, unit-tested state machines to design against.
- **Testing driver lifecycle and crash-recovery policy in-process** — `driver_runtime`/`recovery` provide real sandbox/quarantine/backoff logic without privileged driver code.
- **Providing shared value types/traits** that sibling repos `SHER-Graphics` and `SHER-Display` build on — this is the repo's primary real-world role today.
- **Studying a from-scratch design for kernel-shaped subsystems** (object model, scheduling, memory) as an ordinary, testable Rust workspace.

## What it is not

- **Not a bootable kernel.** No bootloader, no ring-0/bare-metal code, no real MMU/interrupt-controller programming. It does not boot on bare metal and is not installable.
- **Not a Linux replacement or distribution.** `lki`/`compatibility` translate API *names*, not a syscall ABI, and provide no emulation.
- **No validated performance comparison against Linux.** Any "SHER vs Linux" percentage in older docs is illustrative micro-op cost, not a kernel claim.
- **No fuzzing.** Syscall-parameter validation is unit-tested but never fuzzed against malformed input. `cargo audit`/`cargo deny` are wired into CI but have not been observed to execute in any sandbox so far (no network route to the advisory database).
- **Driver isolation is object-model isolation, not OS-level sandboxing.** There is no `unsafe`, no fork/`process::Command`, and no seccomp/cgroup/namespace usage in `driver_runtime`; drivers run in-process with the kernel object model.
- **No real cross-process IPC transport yet.** The in-process IPC primitive exists, but there is no shared-memory or socket transport for framebuffers/input events into `SHER-Display`.
- **Subsystems that require ring-0 or raw-device access are explicitly simulated**, not implemented: `bootstrap` (Stage-0 CPU/MMU/heap bring-up), `networking`/`storage` (registries with no real NIC/disk access), and `interrupt` dispatch bookkeeping.
- **Not published** to crates.io or any registry; consumed only via Cargo path dependencies.

## Project status

- **Implemented, with real tested logic:** `common`, `objectmodel`, `security`, `scheduler`, `compute`, `memory`, `device_manager`, `driver_runtime`, `interrupt` (simulated dispatch), `diagnostics`, `services`, `snapshot`, `updater`, `recovery`, `compatibility`, `networking`/`storage` (simulated I/O), `aro`, `runtime`, `ai`, `lki`, the HAL/driver-shaped crates, and supporting subsystems (`hardening`, `security_audit`, `performance_optimization`, `profiling`, `digital_twins`, `system_integration`, `benchmarks`, `release_engineering`).
- **Explicitly simulated (fundamentally requires hardware/privilege):** `bootstrap`, the Stage-1 primitives in `core`, and the early `drivers` prototype. Each module states its simulation boundary in its own doc comments (`cargo doc --workspace --no-deps --open`).
- **Orchestration:** `kernel` wires the above into a single in-process `SherKernel` with audit logging, AI service wiring, and status reporting.
- **CI:** `cargo fmt` / `build --all-targets` / `test` / `clippy`, matching the sibling repos.
- **Not started / no plan to start in-repo:** a bootable bare-metal kernel — that would be a separate, multi-year bootloader/hardware effort out of scope here.

## Installation

Not published to any registry. Build from source with a stable Rust toolchain (edition 2021):

```bash
git clone https://github.com/Mullassery/SHER-KERNEL.git
cd SHER-KERNEL
cargo build --workspace
```

To consume it from a sibling project, add a path dependency (as `SHER-Graphics` and `SHER-Display` do):

```toml
[dependencies]
sher_common = { path = "../SHER-KERNEL/crates/common" }
sher_objectmodel = { path = "../SHER-KERNEL/crates/objectmodel" }
gpu_driver = { path = "../SHER-KERNEL/crates/gpu_driver" }
wayland_server = { path = "../SHER-KERNEL/crates/wayland_server" }
```

## Quick start

```bash
cargo test --workspace

# Prints an accurate status summary, not a boot sequence
cargo run --bin sher-kernel -- --status
```

## Usage

Build, test, lint, and browse per-crate simulation-boundary docs:

```bash
cargo test --workspace                      # run the full test suite
cargo clippy --workspace -- -D warnings     # lint
cargo fmt --check                           # formatting
cargo build --workspace --all-targets       # build binaries, tests, benches
cargo doc --workspace --no-deps --open      # per-crate docs
```

## Documentation

- [`CLAUDE.md`](CLAUDE.md) — architecture and implementation guide for this repo
- [`VISION.md`](VISION.md) — why this repo exists and what it is prototyping
- [`ROADMAP.md`](ROADMAP.md) — what is built per crate, what is not started, and near-term plans
- [`ROADMAP_HONEST.md`](ROADMAP_HONEST.md) — technical-debt ledger and verification snapshot
- [`API_REFERENCE.md`](API_REFERENCE.md) — per-crate API reference
- [`SECURITY.md`](SECURITY.md) — how to report issues
- [`CHANGELOG.md`](CHANGELOG.md) — Keep a Changelog-style history
- [`docs/architecture/README.md`](docs/architecture/README.md) — crate dependency graph (Mermaid)
- [`docs/architecture/hardware-interop.md`](docs/architecture/hardware-interop.md) — hardware/platform interoperability matrix (Linux x86_64/aarch64, macOS arm64; what is and isn't supported, incl. the KVM question)
- [`docs/archive/`](docs/archive/) — older docs kept for history, each carrying a correction notice

## Known issues and deferred work

**Blocked / failed:**
- Miri/fuzz leak job; no bootable kernel.

**Rejected / deferred:**
- Bootable-kernel/Linux parity.

Full shipped/blocked/rejected detail: [`QUEUE_STATUS.md`](QUEUE_STATUS.md).

## Queue status

Shipped, blocked/failed, and rejected work for this repository is documented in
[`QUEUE_STATUS.md`](QUEUE_STATUS.md).

## Contributing

Contributions are welcome. There is no `CONTRIBUTING.md` in this repo yet, so the practical workflow is: open an issue describing the change, keep new subsystems honest about their simulation boundaries (every module that cannot do the real thing should say so in its own doc comments), and run `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, and `cargo fmt --check` before opening a PR. This is a single-maintainer project, so review happens on a best-effort basis. Report security issues per [`SECURITY.md`](SECURITY.md).

## Part of the SHER OS family

SHER-KERNEL is part of the SHER OS family — an AI-native operating system built from scratch in Rust. The subsystems are designed to work together as a coherent OS stack.

| Tool | Role | What it does |
|---|---|---|
| [SHER-KERNEL](https://github.com/Mullassery/SHER-KERNEL) | Kernel core | **This project** — core kernel scaffolding: object model, scheduling, memory, driver-lifecycle concepts, and a Linux Kernel Interface (LKI) compatibility layer. |
| [SHER-Graphics](https://github.com/Mullassery/SHER-Graphics) | Graphics/GPU | GPU abstraction and rendering — software simulation plus a real, working Vulkan/MoltenVK backend. |
| [SHER-Display](https://github.com/Mullassery/SHER-Display) | Display/compositor | Display server, compositor, and window management. |
| [SHER-INPUT](https://github.com/Mullassery/SHER-INPUT) | Input | Canonical, backend-independent input event stream — real evdev on Linux, simulated backend for hardware-free testing. |
| [SHER-Aurora](https://github.com/Mullassery/SHER-Aurora) | Design system | GNOME-style design system — tokens, typography, motion, and an automated WCAG contrast audit. |
| [SHER-Process-Explorer](https://github.com/Mullassery/SHER-Process-Explorer) | Observability | Evidence-based Linux process investigation tool — /proc telemetry, bpftrace integration, daemon + CLI + desktop UI. |

Each works standalone; they're complementary by design. [Browse all projects →](https://github.com/Mullassery?tab=repositories)

## License

Apache License 2.0 — see [`LICENSE`](LICENSE).
