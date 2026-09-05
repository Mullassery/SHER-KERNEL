# SHER Kernel: Roadmap

This file previously described a "16-week roadmap to a v0.1.0 production
release" with weekly engineering schedules, staffing plans, and "10x faster
than Linux" success metrics for a kernel that has no bootloader and has
never been benchmarked against a real Linux kernel. That roadmap did not
describe real work being tracked against real dates; it was aspirational
scaffolding that never got updated as work actually landed. This is the
replacement: what's actually built, grouped the same way CLAUDE.md's
"Implementation Status" section does, plus a realistic near-term plan.

## Status key

Same discipline as the sibling `SHER-Display` repo's ROADMAP.md: `[x]` means
it compiles and has passing tests backing the claim; `[~]` means partially
built or real logic with a documented simulation boundary; `[ ]` means not
started. Verified this pass via `cargo test --workspace` (768 passing, 0
failing), `cargo clippy --workspace -- -D warnings` (clean), `cargo fmt
--check` (clean), and `cargo clippy --workspace --all-targets -- -D
warnings` (not clean — see below).

## What's built (40 crates, grouped by theme)

- [x] **Foundation** — `common`, `objectmodel`, `security`: core types
      (`ObjectId`, `Capability`), kernel-object lifecycle state machine,
      time-bounded capability grants, telemetry/health. Real, tested.
- [x] **Memory management (userspace bookkeeping)** — `memory`: tiered slab
      allocators (per-CPU, per-socket), master allocator routing, DMA
      buffer bookkeeping, page-table map/unmap. Real logic, real tests. No
      actual `kmalloc`/`vmalloc` translation beyond name→subsystem lookup
      tables in `lki`/`compatibility` — there is no real memory to
      translate into, since this isn't running in kernel space.
- [x] **Device manager (simulation)** — `device_manager`, `drivers`: device
      registry, state machine, hot-plug event queue, discovery over a
      caller-populated device list. No real PCI/USB bus enumeration (would
      require kernel/root access this process doesn't have).
- [x] **Driver runtime** — `driver_runtime`, `recovery`: container
      lifecycle, sandbox policy, crash-recovery backoff/quarantine. Real,
      tested object-model isolation — explicitly not OS-level process
      sandboxing (no `unsafe`, no fork, no seccomp/cgroup/namespace; see
      VISION.md).
- [x] **Linux Kernel Interface** — `lki`, `compatibility`: syscall-name and
      driver-API-name → SHER-primitive translation tables with validation
      and audit. Not a binary syscall ABI; real Linux drivers do not load
      against this.
- [x] **Security & capabilities** — `security`, `security_audit`,
      `objectmodel::capabilities`: time-bounded grants, sandbox policy,
      immutable audit log. Real, tested.
- [x] **AI services (on synthetic/caller-supplied metrics)** — `ai`:
      anomaly detection (memory-leak/interrupt-storm/DMA-abuse heuristics),
      predictive allocation, adaptive scheduling, reinforcement learning.
      Real logic and tests; not wired to any live kernel telemetry, because
      there is no live kernel here to instrument.
- [x] **Hardening & supporting infra** — `hardening`, `performance_optimization`,
      `profiling`, `digital_twins`, `system_integration`, `benchmarks`,
      `performance_benchmarks`, `release_engineering`, `diagnostics`,
      `services`, `snapshot`, `updater`, `runtime`, `interrupt`,
      `networking`, `storage`, `compute`, `scheduler`, `aro`: real, tested
      logic per crate (memory-safety checks, syscall-parameter validation,
      object pooling, profiling, event replay, integration harnesses,
      release bookkeeping, priority scheduling, host memory-tier
      detection). `networking`/`storage` are explicitly simulated device
      registries — no real NIC/disk I/O.
- [x] **Driver-shaped subsystems consumed by sibling repos** — `hal`,
      `gpu_driver`, `audio_driver`, `input_driver`, `wayland_server`,
      `unified_device_manager`: real Rust APIs that `SHER-Graphics` and
      `SHER-Display` depend on via Cargo path dependencies (see README's
      "Cross-repo boundary"). None of them talk to physical GPU/audio/input
      hardware.
- [x] **In-process orchestrator** — `kernel`: wires the above into a single
      `SherKernel` with audit logging, AI service wiring, status reporting.
- [~] **Bootstrap/core stage simulation** — `bootstrap`, `core`: Stage-0/1
      primitives are real in-process logic (object manager, IPC, capability
      enforcement, timer wheel, FIFO scheduler); `bootstrap::cpu::get_info()`
      queries real host parallelism via `std::thread::available_parallelism`,
      but everything else simulating CPU/MMU/heap bring-up is a documented
      placeholder, because real hardware bring-up needs ring-0 access this
      workspace doesn't have and isn't attempting to get.
- [x] **In-process IPC is now real, not simulated** — `core::ipc::IpcBus`
      was a `HashMap<String, VecDeque<Message>>` behind `&mut self` (needed
      an external mutex; copied every payload into an owned `Vec<u8>`). It's
      now a bounded `crossbeam_queue::ArrayQueue` per mailbox with
      `Arc<[u8]>` payloads — lock-free, safe to call concurrently via `&self`,
      O(1) refcount clone instead of a byte copy. Verified with a real
      8-producer-thread test and a pointer-identity test. This is a genuine
      fix, not a claim inflation: it's still only an in-process primitive,
      not cross-process transport (see below).

## What's explicitly not started

- [ ] **Cross-process IPC/transport.** The lock-free `IpcBus` above is a
      real in-process building block, but there is still no actual
      mechanism carrying framebuffers or input events across a process
      boundary to `SHER-Display`. Designing that (shared memory segment?
      socket protocol?) is joint work with `SHER-Display`, not a decision
      this repo can make alone.
- [ ] **Fuzzing.** No `fuzz/` directory, no cargo-fuzz/libfuzzer/afl
      anywhere in the repo. `hardening`/`lki`'s syscall-parameter validation
      is unit-tested against known-good/known-bad inputs but never fuzzed.
- [ ] **Bootable kernel.** No bootloader, no ring-0 code, no real
      MMU/interrupt-controller programming. This is a materially different,
      multi-year effort out of scope for this repo as currently structured
      — not on this roadmap at all, to avoid repeating the mistake of
      implying it's a near-term deliverable.
- [ ] **Real OS-level driver sandboxing.** `driver_runtime`'s isolation is
      object-model isolation (crash-restart, capability checks), not actual
      process/namespace isolation. Would require the driver runtime to
      actually spawn isolated OS processes, which is a different
      architecture than "drivers run in-process with the kernel object
      model" that exists today.

## Near-term plan (concrete, grounded in the code above)

1. **Fix the `cargo clippy --workspace --all-targets -- -D warnings`
   failures.** This is real, scoped, already-diagnosed follow-up: warnings
   (mostly `clippy::clone_on_copy` on `ObjectId`, plus a handful of
   `identity_op`/`unnecessary_cast`/`module_inception`/`len_zero`/`unit_arg`)
   in `#[cfg(test)]` code across `ai`, `device_manager`, `driver_runtime`,
   `kernel`, `lki`, `performance_benchmarks`, `security_audit`,
   `system_integration`, `wayland_server`, plus one in
   `memory/benches/allocator_bench.rs`. None require design decisions —
   this is a cleanup pass, not new architecture.
2. **Add fuzz targets for `hardening`/`lki` syscall-parameter validation.**
   The validation logic exists and is unit-tested; cargo-fuzz harnesses
   against malformed/adversarial input would close the gap called out
   above and in README's "Known gaps."
3. **Deepen the `ai` crate's real-telemetry story.** Today's anomaly
   detection and predictive allocation operate on synthetic/caller-supplied
   metrics. The next honest step isn't "connect to live kernel telemetry"
   (there isn't any) — it's wiring `ai` to consume the *real* telemetry
   `diagnostics` and `objectmodel::telemetry` already collect from the
   other real subsystems in this workspace, closing the loop within the
   userspace prototype rather than promising hardware telemetry that
   doesn't exist yet.
4. **Design the cross-process transport question jointly with
   `SHER-Display`**, rather than either repo assuming a shape for it. This
   is the one item on this list that isn't purely internal — see the
   framebuffer/input-event transport gap above.

Explicitly not on this list: a bootable kernel, a Linux performance
comparison, or a release date. Those aren't next steps from where this
codebase actually is; restating them here would repeat the exact mistake
this rewrite exists to correct.
