# SHER Kernel: Vision

This file was previously a marketing-style "operating system for the next
decade" pitch, written when the project described itself as "v1.0.0
Production Ready." That characterization was inaccurate and has been
corrected across the repo (see [README.md](README.md) and
[CLAUDE.md](CLAUDE.md) for the authoritative, per-crate real-vs-simulated
breakdown). This file is now the honest version, not a historical artifact
kept for reference — the previous content is recoverable from git history if
needed.

## What this actually is, today

SHER Kernel is a **userspace Rust workspace of 40 crates** (`std` + `tokio`,
no `no_std`) that prototypes the internal APIs a from-scratch OS kernel's
object model, scheduler, memory manager, and driver lifecycle might look
like. It runs as an ordinary process on macOS/Linux. It is **not** a
bootable kernel: no bootloader, no ring-0 code, no real MMU or
interrupt-controller programming exists anywhere in this repository.

Verified as of this pass: `cargo test --workspace` — 768 tests passing, 0
failing. `cargo clippy --workspace -- -D warnings` is clean.
`cargo fmt --check` is clean. `cargo clippy --workspace --all-targets -- -D
warnings` is **not** clean — real, pre-existing lint warnings exist in
`#[cfg(test)]` code in nine crates (see README's Status section for the
full list); they don't affect library/binary code, and are tracked, not
fixed, as of this revision.

None of that is nothing: it's a real, tested simulation of kernel-shaped
subsystems, and a reasonable place to prototype capability-based,
driver-isolated object-model ideas in Rust before any of them would need to
touch real hardware.

## Why this exists

The motivating question behind this project is real, even though the
implementation is early: could an OS kernel's internal object model be
designed capability-first, with driver isolation and time-bounded
permissions as load-bearing primitives rather than bolted-on later? SHER
Kernel is where that design is being prototyped and unit-tested — as Rust
APIs and state machines running in a normal process, not as a claim that any
of this has been proven at the OS-privilege level.

Four ideas keep recurring across the crates, and are worth naming as design
intent (not shipped guarantees):

- **Capability-based security** — every permission grant is explicit and
  time-bounded (`objectmodel::capabilities`, `security`). This is real,
  tested logic today, operating on in-process object state.
- **Driver isolation** — a driver's lifecycle goes through a
  container/sandbox abstraction (`driver_runtime`) with crash-recovery
  backoff and quarantine (`recovery::crash_recovery`), rather than running
  inline. This is real *object-model* isolation with tested crash-restart
  behavior — it is not OS-level process isolation. There is no `unsafe`, no
  `fork`/`process::Command`, no seccomp/cgroup/namespace usage anywhere in
  `driver_runtime`; drivers run in-process with the kernel object model. See
  README's "Known gaps" section for why that distinction matters if this is
  ever described externally.
- **Immutable, transactional updates** — an A/B partition model
  (`recovery`, `snapshot`, `updater`) where updates only ever write to a
  standby image and the boot pointer is a single reversible flip. Real,
  tested bookkeeping logic; there is no actual partition or bootloader
  underneath it, because there is no boot process in this repo at all.
- **AI as part of the object model, not bolted on** — the `ai` crate
  implements real anomaly-detection, predictive-allocation, and
  adaptive-scheduling logic with its own tests. It operates on synthetic or
  caller-supplied metrics, not live kernel telemetry, because there is no
  live kernel to instrument.

Linux/POSIX compatibility is pursued the same honest way: `lki` and
`compatibility` are syscall-*name*-to-SHER-primitive lookup tables with
validation, not a binary-compatible syscall ABI and not real driver loading.

## Non-goals (for this repo, as it exists today)

This repo does not attempt to be, and nothing here should be read as
implying it is close to being:

- A bootable kernel, a Linux fork, or something installable in place of an
  OS.
- A validated performance comparison against Linux — see README's
  "Performance notes" for what the historical benchmark docs actually
  measured (in-process micro-op cost, not kernel-level comparison).
- A real cross-process IPC transport. `crates/core::ipc::IpcBus` is now a
  genuine lock-free, zero-copy (`Arc<[u8]>`), bounded ring buffer — a real
  in-process primitive — but there is still no actual transport carrying
  framebuffers or input events across a process boundary to `SHER-Display`.
  That would be a materially different, OS-process-boundary-crossing
  feature, and one that would need to be designed jointly with
  `SHER-Display`, not decided unilaterally here.
- Fuzzed. There is no `fuzz/` directory and no cargo-fuzz/libfuzzer/afl
  anywhere in the repo; syscall-parameter validation in `hardening`/`lki`
  is unit-tested but has never been fuzzed against adversarial input.

## What success looks like from here

Not "boot on bare metal" — that would be a multi-year bare-metal/bootloader
effort with a fundamentally different scope than anything in this
workspace today. Within the userspace-prototype scope this repo actually
occupies, success looks like: the object model, scheduler, and
driver-lifecycle APIs staying coherent and well-tested enough that
`SHER-Graphics` and `SHER-Display` can keep consuming `hal`, `gpu_driver`,
`wayland_server`, and the shared foundation types (`sher_common`,
`sher_objectmodel`) via Cargo path dependencies without those contracts
shifting under them — see README's "Cross-repo boundary" section for what's
actually consumed today. See [ROADMAP.md](ROADMAP.md) for concrete next
steps toward that.

## Boundary with sibling repos

This repo owns object-model, scheduling, memory, driver-lifecycle, and
security-capability primitives. It does not own desktop compositor policy
(`SHER-Display`'s job), GPU rendering execution (`SHER-Graphics`'s job), or
canonical input-event normalization (`SHER-Input`'s job). `wayland_server`'s
`WaylandCompositor` type is `#[deprecated]` and frozen for exactly this
reason: surface/output/focus policy moved to `SHER-Display`, and this repo
kept only the low-level `WaylandTransport` (client connection lifecycle,
buffer handles). See README's "Cross-repo compatibility" section for the
full, verified dependency graph across all five SHER repos.
