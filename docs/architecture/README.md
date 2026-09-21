# Architecture: crate dependency graph

This diagram is generated from the actual `path = "../..."` dependencies
declared in each crate's `Cargo.toml` (checked by hand across all 40
crates during the 2026-09 documentation pass, not inferred from naming).
It reflects real Cargo-level coupling, not aspirational layering — several
crates that look like they should depend on each other (e.g.
`security_audit` on `security`) don't, in the current code.

For what each crate actually implements (real vs. simulated), see
[README.md](../../README.md)'s "What's real vs. simulated" table.
For the org-wide picture (how this repo relates to `SHER-Graphics`,
`SHER-Display`, `SHER-Input`, `Aurora`), see README's "Cross-repo boundary"
and "Cross-repo compatibility" sections — this doc is intentionally scoped
to *this* repo's internal graph only.

```mermaid
graph LR
    common[common]
    objectmodel[objectmodel] --> common
    security[security] --> common
    security --> objectmodel

    bootstrap[bootstrap] --> common
    core[core] --> common
    core --> objectmodel
    core --> security
    runtime[runtime] --> common
    runtime --> core

    memory[memory] --> common
    memory --> objectmodel
    compute[compute] --> common
    scheduler[scheduler] --> common
    scheduler --> objectmodel
    interrupt[interrupt] --> common
    interrupt --> objectmodel

    device_manager[device_manager] --> common
    device_manager --> objectmodel
    device_manager --> security
    driver_runtime[driver_runtime] --> common
    driver_runtime --> objectmodel
    driver_runtime --> security
    driver_runtime --> memory
    drivers[drivers] --> common
    drivers --> objectmodel
    drivers --> security
    hal[hal] --> common

    compatibility[compatibility] --> common
    lki[lki] --> common
    lki --> objectmodel
    lki --> memory
    lki --> device_manager
    lki --> driver_runtime
    lki --> security

    networking[networking] --> common
    networking --> objectmodel
    storage[storage] --> common
    storage --> objectmodel

    services[services] --> common
    diagnostics[diagnostics] --> common
    aro[aro] --> common

    recovery[recovery] --> common
    snapshot[snapshot] --> common
    updater[updater] --> common
    updater --> recovery
    updater --> snapshot

    ai[ai] --> common
    ai --> objectmodel
    ai --> scheduler
    ai --> memory

    hardening[hardening] --> common
    profiling[profiling] --> common
    digital_twins[digital_twins] --> common
    release_engineering[release_engineering] --> common

    gpu_driver[gpu_driver] --> common
    audio_driver[audio_driver] --> common
    input_driver[input_driver] --> common
    unified_device_manager[unified_device_manager] --> common
    unified_device_manager --> gpu_driver
    unified_device_manager --> audio_driver
    unified_device_manager --> input_driver
    wayland_server[wayland_server] --> common
    wayland_server --> gpu_driver
    wayland_server --> input_driver
    wayland_server --> unified_device_manager

    security_audit[security_audit] --> common
    security_audit --> unified_device_manager

    system_integration[system_integration] --> common
    system_integration --> wayland_server
    system_integration --> gpu_driver
    system_integration --> audio_driver
    system_integration --> input_driver
    system_integration --> unified_device_manager
    system_integration --> hal

    performance_benchmarks[performance_benchmarks] --> common
    performance_benchmarks --> system_integration
    performance_benchmarks --> wayland_server
    performance_benchmarks --> gpu_driver
    performance_benchmarks --> audio_driver
    performance_benchmarks --> input_driver
    performance_benchmarks --> unified_device_manager

    performance_optimization[performance_optimization] --> common
    performance_optimization --> performance_benchmarks

    benchmarks[benchmarks] --> common
    benchmarks --> memory
    benchmarks --> device_manager
    benchmarks --> driver_runtime
    benchmarks --> lki
    benchmarks --> security
    benchmarks --> ai

    kernel[kernel] --> common
    kernel --> objectmodel
    kernel --> security
    kernel --> memory
    kernel --> scheduler
    kernel --> interrupt
    kernel --> networking
    kernel --> storage
    kernel --> device_manager
    kernel --> driver_runtime
    kernel --> lki
    kernel --> ai
```

## Reading this graph

- **Foundation tier** (`common`, `objectmodel`, `security`): depended on
  by nearly everything; no cycles back into higher tiers.
- **`kernel`** is the orchestrator — it has the widest fan-in from the
  "core subsystem" tier (memory/scheduler/interrupt/networking/storage/
  device_manager/driver_runtime/lki/ai) but does **not** depend on the
  driver-shaped subsystems (`hal`, `gpu_driver`, `audio_driver`,
  `input_driver`, `wayland_server`, `unified_device_manager`) — those are
  consumed by sibling repos (`SHER-Graphics`, `SHER-Display`) directly,
  not wired into `SherKernel` itself. `system_integration` is the one
  crate that *does* wire the driver-shaped subsystems together, and it is
  explicitly marked as a deprecated, internal-only Phase-12 test harness
  in its own doc comment — see README's "Cross-repo boundary" section for
  why that's a disclosed exception, not evidence `kernel` secretly depends
  on the driver-shaped tier.
- **`drivers`** is an early, self-contained prototype (discovery →
  matching → sandboxed-load policy) superseded by `device_manager` +
  `driver_runtime` in the real `kernel` wiring; it's kept for its own test
  coverage, not double-wired in.
- Crates with no incoming or outgoing internal edges above
  (`bootstrap`, `compute`, `hal`, `compatibility`, `services`,
  `diagnostics`, `aro`, `hardening`, `profiling`, `digital_twins`,
  `release_engineering`, `runtime`) either only depend on `common`/`core`
  or aren't yet consumed by another in-workspace crate — that's a fan-out
  observation, not a sign they're dead code (several are consumed by
  sibling repos; see README's "Cross-repo boundary").

## Regenerating this diagram

This was built by hand from `grep -A20 '^\[dependencies\]' crates/*/Cargo.toml`
across all 40 crates. If crate dependencies change materially, re-run that
against each `crates/*/Cargo.toml` and update the Mermaid block above —
there is no automated generator for it yet.
