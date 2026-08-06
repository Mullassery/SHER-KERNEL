# SHER Kernel: Complete Architecture Index

## 📊 Project Status

| Component | Status | Files | LOC |
|-----------|--------|-------|-----|
| Core Architecture | ✅ Complete | 24 crates | 2,000+ |
| Documentation | ✅ Complete | 6 documents | 5,000+ |
| Boot Phases | ✅ Designed | 3 crates | 500+ |
| ARO System | ✅ Designed | 1 crate | 400+ |
| SLCI Layer | ✅ Designed | - | - |
| Build System | ✅ Working | Cargo workspace | - |
| **TOTAL** | | **24 crates** | **2,000+** |

---

## 📚 Documentation Structure

### 1. Project Overview
- **[README.md](README.md)** — Vision, mission, core architecture overview
- **[QUICK_START.md](QUICK_START.md)** — Developer quick reference and setup

### 2. Architectural Design
- **[ARCHITECTURE.md](ARCHITECTURE.md)** — Four-pillar architecture (boot phases, immutable-first, ARO, lazy loading)
- **[SLCI.md](SLCI.md)** — SHER Linux Compatibility Interface (translation layer)
- **[CLAUDE.md](CLAUDE.md)** — Original architecture guide and implementation patterns
- **[PHASE0_SUMMARY.md](PHASE0_SUMMARY.md)** — Phase 0 completion summary

### 3. This File
- **[INDEX.md](INDEX.md)** — Complete architecture index (you are here)

---

## 🏗️ The Four Architectural Pillars

### Pillar 1: Staged Boot (< 500ms)

**Stage 0 (< 50ms)**: Bootstrap
- Location: `crates/bootstrap/`
- Responsibility: CPU, memory map, MMU, kernel heap, immutable verification
- Nothing else loads

**Stage 1 (< 200ms)**: Core Kernel
- Location: `crates/core/`
- Responsibility: Object manager, IPC, capabilities, timer, CPU scheduler
- Applications can execute

**Stage 2 (Dynamic)**: Runtime Services
- Location: `crates/runtime/`
- Responsibility: Service registry, on-demand service loading
- Everything else loads when requested

**See**: [ARCHITECTURE.md](ARCHITECTURE.md#part-1-staged-boot-architecture)

### Pillar 2: Immutable-First (Always Safe)

- **Dual Partitions**: System A (active) + System B (standby)
- **Transactional Updates**: Build → Verify → Commit → Switchable
- **Instant Rollback**: If boot fails, switch back to previous version
- **Cryptographic Verification**: Every state verified before boot

**Implementation**:
- `crates/recovery/` — Partition management
- `crates/snapshot/` — Versioning and rollback
- `crates/updater/` — Transactional update system

**See**: [ARCHITECTURE.md](ARCHITECTURE.md#part-2-immutable-first-architecture)

### Pillar 3: Adaptive Resource Orchestrator (ARO)

**Auto-Detection**: Hardware profiling at boot
- 128-512 MB → Tier 0: Embedded only
- 512 MB - 2 GB → Tier 1: Minimal IoT
- 2-8 GB → Tier 2: Light desktop
- 8-32 GB → Tier 3: Desktop workstation
- 32+ GB → Tier 4: AI workstation

**Runtime Adaptation**:
- Battery low → Shrink caches, reduce workers
- Plugged in → Expand caches, enable features
- Thermal limits → Throttle compute
- Memory pressure → Enable compression

**Implementation**: `crates/aro/` with 5 memory tiers and dynamic feature matrix

**See**: [ARCHITECTURE.md](ARCHITECTURE.md#part-3-adaptive-resource-orchestrator-aro)

### Pillar 4: Lazy-Loading Subsystems

**Compute**: CPU at Stage 1, GPU/NPU/DSP on-demand
**Drivers**: Discovery finds, loading on first access
**Services**: Display, audio, networking, storage all optional
**Compatibility**: Linux/POSIX layers load only if needed

**Implementation**:
- `crates/compute/` — Heterogeneous schedulers
- `crates/drivers/` — Driver runtime with sandbox
- `crates/services/` — Optional services
- `crates/compatibility/` — Linux/POSIX layers

**See**: [ARCHITECTURE.md](ARCHITECTURE.md#part-4-lazy-loading-subsystems)

---

## 🔗 Linux Compatibility Interface (SLCI)

**Strategic Layer**: Preserves Linux ecosystem while enabling SHER innovation

**How It Works**:
```
Linux App/Driver
    ↓ (doesn't know it's on SHER)
Linux Syscall / Driver API
    ↓
SHER Linux Compatibility Interface
    ↓ (translates to SHER primitives)
SHER Native Kernel
    ↓
Hardware
```

**Translation Scope**:
- **Syscalls**: 50+ Linux syscalls translated
- **Driver APIs**: kmalloc, request_irq, pci_driver_register, etc.
- **Kernel Objects**: task_struct, inode, file, device emulated
- **Memory**: Linux allocator strategy → SHER ARO-aware allocator
- **Scheduling**: Linux CFS → SHER heterogeneous scheduler
- **Filesystem**: Linux inode → SHER immutable object
- **Networking**: Linux socket → SHER native networking
- **Security**: Linux capabilities → SHER time-bounded capabilities

**Benefit**: Day one Linux compatibility + unlimited SHER innovation

**See**: [SLCI.md](SLCI.md)

---

## 📦 Crate Organization (24 Total)

### Core Primitives (Always Present)
```
common/              - Shared types: ObjectId, Result<T>, Capability
objectmodel/         - Kernel object model with lifecycle management
security/            - Capability-based security, audit logging
```

### Boot Stages
```
bootstrap/           - Stage 0 (< 50ms): CPU, MMU, verification
core/                - Stage 1 (< 200ms): Object manager, IPC, scheduler
runtime/             - Stage 2 (Dynamic): Service loader registry
```

### Adaptive Resource Orchestration
```
aro/                 - Hardware profiling, tier detection, feature selection, adaptation
```

### Compute Subsystems (Lazy)
```
compute/             - CPU (Stage 1), GPU/NPU/DSP (on-demand)
                       ├── cpu.rs
                       ├── gpu.rs
                       ├── npu.rs
                       └── dsp.rs
```

### Hardware & Drivers
```
drivers/             - Driver runtime with sandbox isolation
                       ├── discovery.rs
                       ├── registry.rs
                       └── sandbox.rs
memory/              - Allocator, paging, DMA management
                       ├── allocator.rs
                       ├── paging.rs
                       └── dma.rs
interrupt/           - Interrupt dispatcher and routing
                       ├── handler.rs
                       └── controller.rs
```

### Optional Services (Load on-demand)
```
services/            - Filesystem, networking, storage, display, audio
                       ├── filesystem.rs
                       ├── networking.rs
                       ├── storage.rs
                       ├── display.rs
                       └── audio.rs
compatibility/       - Linux/POSIX compatibility layers
                       ├── linux.rs
                       └── posix.rs
ai/                  - AI inference, optimization, prediction
                       ├── inference.rs
                       ├── monitoring.rs
                       └── optimization.rs
```

### System Reliability (Immutable-First)
```
recovery/            - Immutable partition management
                       ├── partition.rs
                       ├── bootptr.rs
                       └── healthcheck.rs
snapshot/            - Version management, rollback engine
                       ├── version.rs
                       ├── store.rs
                       └── restore.rs
updater/             - Transactional atomic updates
                       ├── transaction.rs
                       ├── verify.rs
                       └── commit.rs
```

### Observability & Diagnostics (Deferred)
```
diagnostics/         - Ring buffer (initial), telemetry (later)
                       ├── ringbuffer.rs
                       └── telemetry.rs
```

### Linux Compatibility (Kept from original design)
```
lki/                 - Linux Kernel Interface (now subsumed by SLCI)
networking/          - Network device support
storage/             - Storage device support
device_manager/      - Device lifecycle (now subsumed by drivers/)
driver_runtime/      - Driver containers (now subsumed by drivers/)
scheduler/           - Merged into compute/
```

### Main Entry Point
```
kernel/              - Main entry point and orchestration
                       ├── config.rs
                       ├── kernel.rs (orchestrator)
                       └── main.rs (startup)
```

---

## 🔄 Boot Sequence

```
Power On
    ↓ (Stage 0: 50ms)
CPU Initialization
Memory Map Discovery
MMU Setup
Kernel Heap Allocation
Immutable Root Verification
    ↓ (Stage 1: 150ms)
Object Manager
IPC Subsystem
Capability Manager
Timer
CPU Scheduler Only
    ↓ (System Ready)
ARO Hardware Profiling (100ms)
    ├─ Detect memory tier (Tier 0-4)
    ├─ Calculate resource budget
    └─ Build feature matrix
    ↓ (Stage 2: Dynamic)
Runtime Service Registry
    ├─ Application opens /dev/gpu
    ├─ GPU scheduler loads
    ├─ Application continues
    ↓ (~500ms to interactive shell)
Shell/Application Startup
    └─ User sees prompt
```

---

## 🎯 Performance Targets

| Metric | Target | Architecture | Phase |
|--------|--------|--------------|-------|
| Boot to shell | < 2 seconds | Staged boot | Phase 1 |
| Interrupt latency | < 100 µs | Direct dispatch | Phase 2 |
| Kernel overhead | < 50 MB | Lazy loading | Phase 1 |
| Driver isolation | < 5% overhead | Sandbox design | Phase 3 |
| Update atomicity | 100% guaranteed | Dual partition | Phase 0 |
| Rollback time | < 30 seconds | Boot snapshot | Phase 2 |

---

## 🔐 Security Architecture

### Capability-Based Model
- Every permission is **explicit** (must be granted)
- Every permission is **time-bounded** (automatic expiration)
- Every permission is **audited** (every use logged)
- Every component is **isolated** (sandboxed)

### Zero-Trust Principles
- No component receives unrestricted access
- Every service must prove identity
- Permissions verified at access time
- Audit log is immutable

### Driver Isolation
- Each driver runs in sandbox
- Can't access other drivers' memory
- Can't crash kernel
- Can be restarted independently
- Telemetry collected

**See**: [ARCHITECTURE.md](ARCHITECTURE.md#security-model)

---

## 🚀 Roadmap (7 Phases)

### Phase 0: Foundation ✅ COMPLETE
- [x] Core type system and object model
- [x] Cargo workspace with 24 crates
- [x] Boot phase architecture
- [x] ARO design
- [x] SLCI design
- [x] Comprehensive documentation

### Phase 1: Memory Management (Weeks 1-3)
- [ ] SHER memory allocator
- [ ] Linux API translation (kmalloc, vmalloc)
- [ ] DMA buffer management
- [ ] 50+ unit tests
- [ ] ARO integration

### Phase 2: Device Manager (Weeks 3-5)
- [ ] Hardware discovery
- [ ] PCI/USB enumeration
- [ ] Driver registry
- [ ] Driver matching
- [ ] 40+ unit tests

### Phase 3: Driver Runtime (Weeks 5-7)
- [ ] Driver sandbox isolation
- [ ] Linux driver loading
- [ ] Resource limits
- [ ] Live restart capability
- [ ] 60+ unit tests

### Phase 4: Linux Kernel Interface (Weeks 7-10)
- [ ] Syscall translation
- [ ] Driver API translation
- [ ] Kernel object emulation
- [ ] 100+ unit tests

### Phase 5: Security & AI (Weeks 10-12)
- [ ] Capability system enforcement
- [ ] Audit logging
- [ ] AI services integration
- [ ] Anomaly detection

### Phase 6: Production Hardening (Weeks 12-16)
- [ ] Performance optimization
- [ ] Boot optimization
- [ ] Security audit
- [ ] Production testing

### Phase 7: Release (Week 16+)
- [ ] Documentation
- [ ] Performance benchmarks
- [ ] Compatibility testing
- [ ] First release

---

## 📖 How to Use This Index

1. **Start with**: [README.md](README.md) for project vision
2. **Deep dive**: [ARCHITECTURE.md](ARCHITECTURE.md) for four pillars
3. **Understand compatibility**: [SLCI.md](SLCI.md) for Linux compatibility strategy
4. **Begin coding**: [QUICK_START.md](QUICK_START.md) for development setup
5. **Reference**: [CLAUDE.md](CLAUDE.md) for implementation patterns

---

## 🏔️ The Vision

> SHER Kernel is to Linux what modern electric vehicles are to combustion engines.
> Not an incremental improvement. A complete rethinking built for the next several decades of computing.

**Four Pillars**:
1. ⚡ **Staged Boot** — Ultra-fast startup through lazy loading
2. 🔒 **Immutable-First** — Always-safe, always-recoverable system
3. 🎯 **ARO** — Scales from 128 MB IoT to 128 GB AI workstations
4. 🔗 **SLCI** — 100% Linux compatibility with complete architectural freedom

**Outcome**: One kernel. Infinite scale. Always fast. Always safe. Always recoverable.

---

## 🔗 Quick Links

- **Build**: `cargo build --release`
- **Run**: `cargo run --release`
- **Docs**: `cargo doc --open`
- **Test**: `cargo test` (coming Phase 1)
- **Git**: Already set up with `.gitignore`

---

**Built with**: Rust, async/await, tokio  
**License**: Proprietary — Free to use with explicit attribution  
**Author**: Georgi Mammen Mullassery  
**Date**: August 6, 2026  

🚀 **The future of kernel design starts here.**
