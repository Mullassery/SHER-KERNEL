# SHER Kernel Quick Start

> **Status correction (see [README.md](README.md)):** This document reflects an early snapshot (13-24 crates, ~1,400-2,000 LOC) and is stale — the workspace has since grown to 40 crates with 764 tests. See README.md for the current, accurate status and per-crate breakdown. Kept for historical reference only.


## Project Initialized ✓

**Status**: Phase 0 Foundation Complete  
**Lines of Code**: 1,419  
**Crates**: 13  
**Build**: ✓ Passing (with cleanup warnings)

## What Was Created

### Core Architecture (13 modular Rust crates)

1. **`common`** — Shared types (ObjectId, Capability, Error types)
2. **`objectmodel`** — Core kernel object model with lifecycle and capabilities
3. **`security`** — Capability-based security with audit logging
4. **`memory`** — Memory allocator, paging, DMA management
5. **`scheduler`** — Heterogeneous compute scheduler (CPU, GPU, NPU, etc.)
6. **`interrupt`** — Interrupt registration and routing
7. **`device_manager`** — Unified device management and discovery
8. **`driver_runtime`** — Isolated driver execution containers
9. **`lki`** — Linux Kernel Interface (compatibility layer)
10. **`networking`** — Network device support
11. **`storage`** — Storage device support
12. **`ai`** — AI-native kernel services
13. **`kernel`** — Main kernel entry point

### Documentation

- **`README.md`** — Project overview and architecture
- **`CLAUDE.md`** — Detailed architecture & implementation guide
- **`QUICK_START.md`** — This file

## Building the Project

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run kernel
cargo run --release

# Run with full debug logging
RUST_LOG=sher=debug cargo run --release

# Build specific crate
cargo build -p sher_kernel
```

## Project Structure

```
SHER-Kernel/
├── Cargo.toml              # Workspace root
├── README.md               # Overview
├── CLAUDE.md              # Architecture & implementation guide
├── QUICK_START.md         # This file
├── .gitignore
└── crates/
    ├── common/             # 150 LoC - Shared utilities
    ├── objectmodel/        # 250 LoC - Core object model
    ├── security/           # 130 LoC - Capability security
    ├── memory/             # 180 LoC - Memory management
    ├── scheduler/          # 140 LoC - Task scheduling
    ├── interrupt/          # 100 LoC - Interrupt handling
    ├── device_manager/     # 120 LoC - Device management
    ├── driver_runtime/     # 140 LoC - Driver isolation
    ├── lki/                # 190 LoC - Linux compatibility
    ├── networking/         # 110 LoC - Network support
    ├── storage/            # 110 LoC - Storage support
    ├── ai/                 # 110 LoC - AI services
    └── kernel/             # 200 LoC - Main kernel
```

## Key Concepts

### Everything is a Kernel Object
```rust
pub struct KernelObject {
    pub id: ObjectId,                           // Unique identifier
    pub obj_type: ObjectType,                   // Process, Device, Driver, etc.
    pub lifecycle: Lifecycle,                   // State machine
    pub capabilities: CapabilitySet,            // Time-bounded permissions
    pub telemetry: Telemetry,                   // Metrics & monitoring
    pub dependencies: Vec<ObjectId>,            // Dependency tracking
}
```

### Capability-Based Security
Every permission is:
- **Explicit**: Must be granted by owner
- **Time-bounded**: Automatic expiration (1h to 30m depending on tier)
- **Audit-logged**: Every grant and use is recorded
- **Isolated**: Drivers run in sandboxes with limited capabilities

### Linux Compatibility (LKI)
Linux drivers don't run on Linux. They run on SHER's translation layer:

```
Linux Driver
    ↓ (LKI Translation)
SHER Memory → kmalloc
SHER Interrupt → request_irq
SHER Device Registry → pci_driver_register
```

## Next Steps (Phase 1: Memory Management)

1. **Implement Memory Allocator**
   - Slab allocator for small objects
   - Buddy allocator for larger blocks
   - Unit tests: 20+ cases

2. **Add Linux API Translation**
   - kmalloc/kfree translation
   - DMA buffer lifecycle
   - Virtual-to-physical mapping
   - Unit tests: 30+ cases

3. **Integration Tests**
   - Memory pressure scenarios
   - Allocation failures
   - DMA correctness

## Testing

Current setup includes:
- Workspace compiles cleanly
- All crates build independently
- Ready for unit tests in Phase 1

Add tests with:
```bash
cargo test
```

## Debugging

### View Kernel State
```rust
let status = kernel.status();
println!("{:#?}", status);
```

### Enable Full Tracing
```bash
RUST_LOG=sher=trace cargo run --release
```

### Filter by Component
```bash
RUST_LOG=sher_lki=debug,sher_memory=debug cargo run
```

## Code Organization Guidelines

### When Adding Features
1. Start in the most specific crate (e.g., `lki` for Linux APIs)
2. Use `sher_common::Result<T>` for errors
3. Add telemetry to KernelObject instances
4. Log with `tracing::info!()`, `warn!()`, or `error!()`
5. Write tests in the same file or `tests/` directory

### When Adding New Crates
1. Add to `Cargo.toml` workspace members
2. Create `crates/<name>/Cargo.toml` with dependencies
3. Create `crates/<name>/src/lib.rs` with doc comments
4. Export public types in `pub use` statements
5. Update `CLAUDE.md` with new crate documentation

## Performance Targets

- **Boot**: < 2 seconds to shell
- **Interrupt latency**: < 100 µs
- **Memory overhead**: < 50 MB kernel
- **Driver isolation**: < 5% overhead
- **GPU utilization**: 80%+ for eligible workloads

## Troubleshooting

### Compilation Errors
```bash
cargo clean
cargo build
```

### Unused import warnings
These are normal during development. Will be cleaned up before Phase 1 release.

### Runtime errors
Check:
1. `RUST_LOG=sher=debug cargo run --release`
2. Look for `error!()` logs
3. Check kernel status: `kernel.status()`

## References

- **Architecture**: See `CLAUDE.md` for complete architecture guide
- **Code**: `README.md` for project overview
- **API Docs**: Run `cargo doc --open` (will be populated)

## Contributing

This is a personal project. Documentation is canonical source of truth. Keep `CLAUDE.md` updated when:
- Adding new subsystems
- Changing architecture decisions
- Implementing new phases

---

**Ready to build the OS of the future!** 🚀

Next step: Begin Phase 1 implementation with memory allocator.
