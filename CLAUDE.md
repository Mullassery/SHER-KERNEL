# SHER Kernel: Architecture & Implementation Guide

## Project Context

**Project Name**: SHER Kernel (Strength, Resilience, Intelligence, Adaptability)
**Type**: Userspace prototype of OS-kernel object-model/scheduling/memory/driver-lifecycle concepts — **not a bootable kernel**
**Language**: Rust (`std` + `tokio`; no `no_std`, no bootloader, no ring-0 code anywhere in this workspace)
**Status**: 40 crates, 764 tests passing, `clippy -D warnings` and `fmt --check` clean. See [README.md](README.md) for the authoritative real-vs-simulated breakdown per crate.
**Author**: Georgi Mammen Mullassery

Earlier revisions of this file said "Phase 0 Foundation — Architecture Complete, Implementation Started" while other docs in this repo simultaneously claimed "v1.0.0 Production Ready" and "13/13 phases complete" — those two characterizations directly contradicted each other, and neither was accurate. This file has been corrected to describe what the code actually does. It runs as an ordinary process on your existing OS; it does not boot on bare metal.

## Design Philosophy

SHER Kernel is engineered for the AI era with four guiding principles:

1. **AI-Native**: Artificial intelligence is OS infrastructure, not an application
2. **Compatibility Without Dependency**: Linux hardware ecosystem is a compatibility target, not an architectural constraint
3. **Modular by Design**: Every subsystem is independently replaceable and testable
4. **Security by Architecture**: Capability-based permissions, zero-trust model, no component has unrestricted access

These are the *design goals being prototyped*, not claims about a finished, bootable OS.

## Architectural Constraints & Boundaries

### What SHER IS:
- A userspace Rust workspace prototyping a capability-based, driver-isolated kernel object model
- An AI-native *simulation*: inference/anomaly-detection/adaptive-scheduling logic is real and tested, but it runs on synthetic or caller-supplied metrics, not live kernel telemetry (there is no live kernel to instrument)
- A modular design where subsystems operate via well-defined interfaces
- A capability-based security model where permissions are explicit and time-bounded
- Compatible with Linux/POSIX API *names* through lookup-table translation (`lki`, `compatibility`), not through emulation or a real syscall ABI

### What SHER IS NOT:
- A bootable kernel — there is no bootloader, no ring-0/bare-metal code, no real MMU or interrupt-controller programming
- A Linux distribution or fork
- A microkernel or monolithic kernel in the conventional, bootable sense (though the object model borrows vocabulary from both)
- A validated performance comparison against Linux (see README.md's "Performance notes" section)
- Published anywhere (no crates.io/PyPI) — consumed only via Cargo path dependencies by sibling repos (`SHER-Graphics`, `SHER-Display`)

## Crate Structure

40 crates total. This list was previously stale (named only 13); see [README.md](README.md#project-organization) for the current, complete breakdown with a real-vs-simulated classification per crate. Summary:

```
sher-kernel/
├── crates/
│   ├── common/, objectmodel/, security/      # Foundation types, object model, capabilities
│   ├── bootstrap/, core/, runtime/           # Staged boot simulation (Stage 0/1/2)
│   ├── memory/, compute/, scheduler/         # Memory tiers, accelerator queues, priority scheduler
│   ├── device_manager/, driver_runtime/,
│   │   drivers/, hal/                        # Device registry, driver lifecycle, HAL
│   ├── interrupt/, networking/, storage/     # Simulated I/O-adjacent subsystems
│   ├── security_audit/, hardening/           # Memory-safety & syscall hardening
│   ├── recovery/, snapshot/, updater/        # A/B images, rollback, transactional updates
│   ├── diagnostics/, profiling/,
│   │   performance_optimization/             # Telemetry, profiling, object pooling
│   ├── ai/                                   # Anomaly detection, predictive allocation, RL
│   ├── lki/, compatibility/                  # Linux/POSIX API-name translation tables
│   ├── gpu_driver/, audio_driver/,
│   │   input_driver/, wayland_server/,
│   │   unified_device_manager/               # Driver-shaped subsystems (SHER-Display boundary)
│   ├── digital_twins/, system_integration/,
│   │   benchmarks/, performance_benchmarks/,
│   │   release_engineering/                  # Testing/release tooling
│   └── kernel/                                # In-process orchestrator (SherKernel)
```

## Core Concepts

### 1. Kernel Objects

Every entity in SHER is a managed kernel object:

```rust
pub struct KernelObject {
    pub id: ObjectId,
    pub obj_type: ObjectType,          // Process, Thread, Driver, Device, etc.
    pub name: String,
    pub lifecycle: Lifecycle,           // State tracking
    pub capabilities: CapabilitySet,    // Permission grants
    pub telemetry: Telemetry,          // Metrics and monitoring
    pub dependencies: Vec<ObjectId>,    // Dependency graph
    pub metadata: HashMap<String, String>,
}
```

**Types**: Process, Thread, Driver, Device, Socket, StorageVolume, GPU, NPU, Sensor, Robot, Container, VirtualMachine, AiModel

### 2. Capability-Based Security

Permissions are explicit and time-bounded:

```rust
pub struct CapabilityGrant {
    pub capability: Capability,     // Read, Write, Execute, Allocate, Interrupt, Schedule, NetworkAccess, DmaAccess, Admin
    pub tier: PermissionTier,       // Low (1h), Medium (24h), High (2h), Critical (30m)
    pub granted_at: u64,
    pub expires_at: Option<u64>,   // Automatic expiration
}
```

**Principle**: Every permission is time-bound. No silent renewal. Continuous re-authentication required for sensitive operations.

### 3. Lifecycle Management

Objects have explicit state machines:

```rust
pub enum State {
    Initializing,
    Ready,
    Running,
    Paused,
    Stopping,
    Stopped,
    Failed,
    Recovering,
}
```

## Implementation Status

The numbered "Week N" phase roadmap that used to live here was aspirational scaffolding that never got updated as work landed, which is part of how this repo ended up with contradictory status claims across different files. What actually exists today, grouped by the same themes:

- **Foundation** — done: core type system, object model, error handling, 40-crate workspace, compiles cleanly, tests present (`common`, `objectmodel`, `security`).
- **Memory management** — done as userspace bookkeeping: tiered slab allocators, DMA buffer tracking, page-table map/unmap (`memory`). No real Linux API translation exists for `kmalloc`/`vmalloc`/etc. beyond name→subsystem lookup tables in `lki`/`compatibility`.
- **Device manager** — done as simulation: device registry, state machine, hot-plug event queue, discovery over a caller-populated device list (`device_manager`, `drivers`). No real PCI/USB bus enumeration (would need kernel/root access).
- **Driver runtime** — done: container lifecycle, sandbox policy, crash-recovery backoff/quarantine (`driver_runtime`, `recovery`). "Loading" a driver means constructing its in-process representation, not loading a kernel module.
- **Linux Kernel Interface** — done as translation tables + validation, not a real syscall ABI (`lki`).
- **Security & capabilities** — done: time-bounded capability grants, sandbox policy, audit logging (`security`, `security_audit`, `objectmodel::capabilities`).
- **AI services** — done as real logic over synthetic/caller-supplied metrics: anomaly detection, predictive allocation, adaptive scheduling, reinforcement learning (`ai`). Not connected to any live kernel telemetry, because there is no live kernel to instrument.
- **Hardening** — done: memory-safety checks, syscall-parameter/return-value validation, object pooling, profiling (`hardening`, `security_audit`, `performance_optimization`, `profiling`).

Run `cargo test --workspace` for the current, authoritative count (764 as of this revision) rather than trusting any number written in prose here or elsewhere — prose test counts are exactly what went stale before.

## Implementation Patterns

### 1. Error Handling
Always return `Result<T>` from fallible operations. Never panic in kernel code.

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

### 2. Logging
Use tracing macros for observability. Three levels:
- `info!()` — System events (boot, shutdown, driver load)
- `warn!()` — Recoverable errors (driver restart, memory pressure)
- `error!()` — Serious issues (allocation failure, security violation)

### 3. Async/Await
Use tokio for async operations. Kernel subsystems should be async-first to support concurrent operations.

```rust
pub async fn initialize(&mut self) -> Result<()>
```

### 4. Testing
Every subsystem should have:
- Unit tests for core functionality
- Integration tests with other subsystems
- Failure case tests (OOM, timeout, security checks)

## Linux Kernel Interface (LKI) Design

The LKI is the bridge between Linux drivers and SHER primitives:

```
Linux Driver
    ↓
LKI Translation Layer
    ↓
SHER Kernel Primitive
```

### Supported Linux APIs

**Memory**:
- `kmalloc(size)` → SHER MemoryAllocator
- `vmalloc(size)` → SHER MemoryAllocator with page mapping
- `dma_alloc(size)` → SHER DmaManager
- `kfree(ptr)` → SHER deallocate

**Interrupts**:
- `request_irq(irq, handler)` → SHER InterruptController
- `free_irq(irq)` → SHER InterruptController
- `enable_irq(irq)` → SHER interrupt enable

**Devices**:
- `pci_driver_register(driver)` → SHER DeviceRegistry
- `device_register(dev)` → SHER object instantiation
- `bus_register(bus)` → SHER device hierarchy

## Security Model

### Zero Trust Principles
1. **Never trust input** — All driver input is validated
2. **Least privilege** — Drivers get minimum permissions required
3. **Audit everything** — All security-relevant actions logged
4. **Defense in depth** — Multiple security layers (sandbox + capability + audit)
5. **Fail secure** — On error, default to deny access

### Permission Tiers
- **Tier 1 (Low)**: Configurable duration (default 1 hour)
- **Tier 2 (Medium)**: 24 hours maximum
- **Tier 3 (High)**: 2 hours maximum
- **Tier 4 (Critical)**: 30 minutes maximum

No permission lasts forever. No silent renewal. Live countdown. Idle revocation.

## Testing Strategy

### Unit Tests
- Core type tests (ObjectId, Capability, Lifecycle)
- Subsystem initialization tests
- API translation tests

### Integration Tests
- Driver loading pipeline
- Memory allocation + DMA
- Device discovery + driver matching
- Interrupt handling + driver callback

### System Tests
- Multi-driver scenarios
- Resource contention
- Crash recovery
- Performance benchmarks

### Security Tests
- Permission escalation attempts
- Capability expiration enforcement
- Sandbox escape attempts
- Audit log completeness

## Performance Objectives

- Boot time: < 2 seconds to shell, < 10 seconds fully initialized
- Interrupt latency: < 100 microseconds
- Memory overhead: < 50MB kernel + drivers
- Driver isolation overhead: < 5% performance impact
- Heterogeneous scheduling: 80%+ GPU utilization for eligible workloads

## Development Guidelines

### Code Style
- Use explicit types (no type inference on public APIs)
- Prefer composition over inheritance
- Keep functions small (< 50 lines ideal)
- Document the WHY, not the WHAT
- No unsafe code without explicit review

### Naming Conventions
- `ObjectId`, `ObjectType`, `Capability` — Core concepts
- `DriverContainer`, `DriverRuntime` — Driver subsystem
- `LinuxKernelInterface` (LKI) — Compatibility layer
- `CapabilityGrant`, `Sandbox` — Security primitives

### Comments
Only add comments for:
- Non-obvious design decisions
- Workarounds for known kernel bugs
- Subtle invariants that would surprise readers
- Links to relevant architecture documents

### Commit Messages
Format: `[Subsystem] Brief description`

Examples:
- `[Memory] Add DMA buffer lifecycle management`
- `[LKI] Implement kmalloc translation with validation`
- `[Security] Add capability expiration enforcement`

## Debugging Facilities

### Built-in Observability
- `SherKernel::status()` — Real-time kernel state
- Telemetry in every KernelObject
- Full audit log of security events
- Tracing output for subsystems

### Debug Builds
Enable all tracing:
```bash
RUST_LOG=sher=debug cargo run
```

### Log Levels
```bash
RUST_LOG=sher_kernel=info,sher_lki=debug cargo run
```

## Future Extensions

### 1. Robotics Integration
- Real-time scheduling for motor control
- Sensor fusion with AI anomaly detection
- Mission planning and autonomous task execution

### 2. Heterogeneous Computing
- Automatic GPU/NPU offloading for eligible workloads
- Distributed inference across clusters
- Edge computing support

### 3. Machine Learning Optimization
- Model-based scheduling decisions
- Predictive resource allocation
- Automatic performance tuning

### 4. Digital Twins
- Virtual kernel state for testing
- Replay capability for debugging
- What-if analysis for resource planning

## References

- Linux Kernel Architecture: https://github.com/torvalds/linux
- Microkernel Design: https://en.wikipedia.org/wiki/Microkernel
- Capability-Based Security: https://en.wikipedia.org/wiki/Capability-based_security
- Rust Systems Programming: https://github.com/redox-os/redox

---

**Current state**: 40 crates implemented with real, tested logic behind their public APIs, or explicitly labeled as a hardware/privilege simulation where they can't be. **Next steps, if this project continues**: none of them lead to a bootable kernel without a multi-year bare-metal/bootloader effort out of scope for this repo; within the userspace-prototype scope, likely next work is deepening the `ai` crate's connection to real telemetry sources and expanding `lki`'s translation coverage.
