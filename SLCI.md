# SHER Linux Compatibility Interface (SLCI)

## Core Principle

SHER Kernel is **not** Linux. It only **behaves** like Linux from the perspective of applications and device drivers.

This is the strategic layer that unlocks: **Preserve the ecosystem. Reinvent the architecture.**

---

## Architectural Layers

```
┌─────────────────────────────────────────────────┐
│ Linux Applications                              │
├─────────────────────────────────────────────────┤
│ GNU libc / musl (unchanged)                     │
├─────────────────────────────────────────────────┤
│ Linux System Call Interface                      │
├─────────────────────────────────────────────────┤
│ SHER Linux Compatibility Interface (SLCI)       │
│                                                  │
│ • Linux Syscall Translation                     │
│ • Linux Driver API Translation                  │
│ • Linux Kernel Object Emulation                 │
│ • Linux Memory Manager Mapping                  │
│ • Linux Scheduler Mapping                       │
│ • Linux Filesystem Mapping                      │
│ • Linux Networking Mapping                      │
│ • Linux Security Mapping                        │
├─────────────────────────────────────────────────┤
│ SHER Native Kernel                              │
│                                                  │
│ Native Scheduler (heterogeneous)                │
│ Native Memory Manager (ARO-aware)               │
│ Native Driver Framework (sandboxed)             │
│ Native IPC (capability-based)                   │
│ Native Security (time-bounded capabilities)     │
│ Native Filesystems (immutable by default)       │
├─────────────────────────────────────────────────┤
│ Hardware                                         │
└─────────────────────────────────────────────────┘
```

**Key insight**: The Linux driver has NO IDEA it's on SHER. It calls `kmalloc()`, and the SLCI layer translates it to SHER's memory manager. The driver never sees the difference.

---

## How SLCI Works: Translation by Example

### Memory Allocation

**What Linux driver does**:
```c
void *ptr = kmalloc(256, GFP_KERNEL);
```

**What SLCI does**:
```
kmalloc(256)
    ↓
SHER_ALLOCATE_KERNEL(256)
    ↓
sher_aro_aware_allocator()
    ↓
SHER Memory Manager
    ↓
Hardware
```

**Result**: Driver gets memory, has no idea SHER used ARO-based allocation strategy instead of Linux's buddy allocator.

### Interrupt Handling

**What Linux driver does**:
```c
int irq = request_irq(IRQ_GPIO, gpio_handler, IRQF_SHARED, "gpio", data);
```

**What SLCI does**:
```
request_irq(IRQ_GPIO, ...)
    ↓
SHER_REGISTER_INTERRUPT(IRQ_GPIO)
    ↓
sher_interrupt_manager.register()
    ↓
sher_cpu_scheduler.pin_handler()
    ↓
Hardware IRQ Line
```

**Result**: Driver gets interrupt, has no idea SHER routed it through its native interrupt dispatcher.

### Locking

**What Linux driver does**:
```c
spin_lock(&my_lock);
```

**What SLCI does**:
```
spin_lock(&my_lock)
    ↓
SHER_ACQUIRE_SPINLOCK(&my_lock)
    ↓
sher_lock_primitive()
    ↓
scheduler_aware_spinlock()
```

**Result**: Lock works, but SHER knows about it and can schedule accordingly.

### Device Registration

**What Linux driver does**:
```c
struct pci_driver my_driver = {
    .probe = my_probe,
    .remove = my_remove,
};
pci_driver_register(&my_driver);
```

**What SLCI does**:
```
pci_driver_register(&my_driver)
    ↓
SHER_REGISTER_DEVICE_DRIVER(&my_driver)
    ↓
sher_device_registry.add_driver()
    ↓
sher_driver_runtime.isolate_driver()
    ↓
Driver runs in sandbox
```

**Result**: Driver is registered, isolated in sandbox, can't crash kernel.

---

## Kernel Object Emulation

Linux internals are built around data structures:

- `task_struct` — process/thread
- `inode` — file
- `file` — open file handle
- `super_block` — filesystem
- `vm_area_struct` — memory mapping
- `device` — device object
- `pci_dev` — PCI device
- `net_device` — network interface
- `usb_device` — USB device

SLCI **does not replicate** these. Instead, it provides wrappers:

```
Linux Driver expects: task_struct
                ↓
    SLCI provides: linux_task_wrapper
                ↓
    Which wraps: sher_thread_object
                ↓
    SHER Kernel tracks: native_thread
```

**The driver thinks it's using `task_struct`. SHER uses completely different internal representations.**

### Example: inode Emulation

```rust
// SLCI Layer (what Linux sees)
pub struct linux_inode {
    i_ino: u64,
    i_mode: u16,
    i_uid: u32,
    i_gid: u32,
    // ... 30+ more Linux inode fields
}

// SHER Native (what kernel uses internally)
pub struct SherFileObject {
    id: ObjectId,
    path: String,
    permissions: CapabilitySet,
    telemetry: Telemetry,
    // Much simpler, capability-aware
}

// SLCI Translation
impl From<&SherFileObject> for linux_inode {
    fn from(obj: &SherFileObject) -> Self {
        linux_inode {
            i_ino: obj.id.as_u64(),
            i_mode: 0o644,
            // Convert SHER object to Linux structure on-demand
        }
    }
}
```

---

## Translation Coverage

### Syscalls

| Linux Syscall | SHER Translation | Category |
|---------------|------------------|----------|
| brk() | sher_memory_expand() | Memory |
| mmap() | sher_memory_map() | Memory |
| open() | sher_filesystem_open() | Filesystem |
| read() | sher_filesystem_read() | Filesystem |
| write() | sher_filesystem_write() | Filesystem |
| socket() | sher_networking_socket() | Networking |
| connect() | sher_networking_connect() | Networking |
| fork() | sher_process_fork() | Process |
| execve() | sher_process_exec() | Process |
| clone() | sher_thread_create() | Threading |
| futex() | sher_lock_wait() | Synchronization |
| epoll_wait() | sher_event_wait() | Events |
| mknod() | sher_device_register() | Devices |

### Kernel APIs (for drivers)

| Linux API | SHER Translation |
|-----------|------------------|
| kmalloc() | sher_allocate_kernel() |
| vmalloc() | sher_allocate_virtual() |
| dma_alloc() | sher_allocate_dma() |
| kfree() | sher_deallocate() |
| request_irq() | sher_interrupt_register() |
| free_irq() | sher_interrupt_unregister() |
| spin_lock() | sher_lock_spinlock() |
| mutex_lock() | sher_lock_mutex() |
| schedule() | sher_yield() |
| pci_driver_register() | sher_driver_register() |
| dev_get_drvdata() | sher_object_get_attribute() |
| ioremap() | sher_io_map() |

---

## Memory Translation

### Linux Allocator Model

Linux assumes:
- `kmalloc()` for small objects (< 128 KB)
- `vmalloc()` for large objects
- DMA-safe memory for devices
- Buddy allocator internally

### SHER Allocator Model

SHER is aware of:
- Available RAM (tier-aware)
- NUMA topology (if present)
- CPU cache hierarchy
- Whether device is battery-powered
- AI acceleration available

### Translation

```
Linux: kmalloc(256)
    ↓
SLCI checks: Available memory, ARO tier, workload type
    ↓
SHER: Choose optimal allocator strategy
    ├─ Tier 0 (128 MB): Use slab, compact allocator
    ├─ Tier 2 (8 GB): Use larger slab, add cache
    └─ Tier 4 (128 GB): Use huge pages, NUMA-aware
    ↓
Driver gets pointer (no idea allocation strategy changed)
```

**Result**: Linux driver works unchanged. SHER optimizes memory for actual hardware.

---

## Scheduler Translation

### Linux Assumptions

Linux scheduler expects:
- `wake_up_process(task)`
- `schedule()` for voluntary yield
- `yield()` for cooperative scheduling
- CFS (Completely Fair Scheduler)

### SHER Reality

SHER scheduler is heterogeneous-aware:
```
Task X: Interactive, uses GPU
    ↓
Pin to GPU scheduler, high priority, low latency

Task Y: Background AI inference
    ↓
Route to NPU scheduler, batch mode, power-efficient

Task Z: Robotics real-time
    ↓
Hard real-time scheduler, deterministic timing

Task W: General compute
    ↓
CPU scheduler, interactive mode
```

### Translation

Linux driver calls:
```c
wake_up_process(task);
```

SLCI translates:
```
wake_up_process(task)
    ↓
sher_scheduler.wake(task)
    ↓
Determine task type (AI? Real-time? Interactive?)
    ↓
Route to appropriate compute target (GPU? NPU? CPU?)
    ↓
Scheduler optimizes for that target
```

**Result**: Linux application works unchanged. SHER optimizes scheduling for heterogeneous hardware.

---

## Filesystem Translation

### Linux Model

Linux filesystems are built around:
- `inode` (file metadata)
- `dentry` (directory entry)
- `file` (open file handle)
- `superblock` (filesystem)
- In-place modification

### SHER Model

SHER filesystems are:
- Immutable (by default)
- Versioned
- Snapshot-capable
- Transactional

### Translation

```
Linux: open("file.txt", O_RDWR)
    ↓
SLCI: Create writable snapshot of file
    ↓
SHER: Map Linux fd to snapshot
    ↓
Linux: write(fd, data)
    ↓
SLCI: Commit write to immutable store
    ↓
SHER: Snapshot becomes new version
```

**Result**: Linux sees mutable filesystem. SHER maintains immutability and versions.

---

## Networking Translation

### Linux Model

Linux network stack:
- `socket`
- `sk_buff` (socket buffer)
- `net_device`
- TCP/IP implementation

### SHER Model

SHER could optimize for:
- AI inference communication (low-latency, batch)
- IoT sensors (low-power)
- Real-time robotics (deterministic)
- High-performance computing (throughput)

### Translation

```
Linux: socket(AF_INET, SOCK_STREAM)
    ↓
SLCI: Determine socket type (bulk transfer? interactive? streaming?)
    ↓
SHER: Select networking strategy
    ├─ Bulk: Large buffers, batch ACKs
    ├─ Interactive: Small buffers, low latency
    └─ Streaming: Adaptive buffering
    ↓
Linux socket works as expected
```

**Result**: Linux applications work unchanged. SHER optimizes networking for workload.

---

## Security Translation

### Linux Model

Linux security:
- Capabilities (simple permissions)
- SELinux (role-based)
- AppArmor (path-based)
- Namespaces (isolation)
- cgroups (resource limits)

### SHER Model

SHER security:
- Capability-based (explicit, time-bounded)
- Zero-trust architecture
- Every operation audited
- Automatic enforcement

### Translation

Linux: `cat /etc/shadow`

SLCI checks:
- Does process have `CAP_DAC_READ_SEARCH`?
- Is it time-bounded?
- Audit the access
- Decide: allow or deny

SHER: Denies and logs

**Result**: Linux security model works. SHER adds stronger guarantees underneath.

---

## Driver Translation Architecture

```
┌─────────────────────────────────┐
│ Linux Device Driver             │
│                                 │
│ probe()                         │
│ remove()                        │
│ ioctl()                         │
│ mmap()                          │
│ read()                          │
│ write()                         │
└────────┬────────────────────────┘
         │ Linux Driver ABI
         ↓
┌─────────────────────────────────┐
│ SHER Driver Wrapper             │
│ (Translation Layer)             │
│                                 │
│ • API translation               │
│ • Error mapping                 │
│ • Memory management             │
│ • Interrupt handling            │
└────────┬────────────────────────┘
         │ SHER Native APIs
         ↓
┌─────────────────────────────────┐
│ SHER Driver Runtime             │
│                                 │
│ • Sandbox isolation             │
│ • Resource limits               │
│ • Crash recovery                │
│ • Telemetry                     │
└────────┬────────────────────────┘
         │ SHER Kernel APIs
         ↓
┌─────────────────────────────────┐
│ SHER Native Kernel              │
│                                 │
│ • Memory Manager                │
│ • Interrupt Manager             │
│ • Scheduler                     │
│ • Device Manager                │
└─────────────────────────────────┘
```

---

## Boot Sequence

```
1. Firmware
   ↓
2. SHER Bootloader
   ↓
3. SHER Kernel
   ├─ Stage 0: Bootstrap (CPU, MMU, verification)
   ├─ Stage 1: Core Kernel (Object manager, IPC, CPU scheduler)
   ├─ Stage 2: Native Services (Memory, Interrupt, Drivers)
   ↓
4. Initialize Compatibility Layer
   ├─ Set up Linux syscall handler
   ├─ Set up Linux kernel API translator
   ├─ Set up Linux device model wrapper
   ↓
5. Load Linux Drivers
   ├─ Driver loads and calls probe()
   ├─ All Linux APIs translated through SLCI
   ├─ Driver gets objects that look like Linux objects
   ├─ Driver thinks it's on Linux
   ↓
6. Boot Userspace
   ├─ Libc initialization
   ├─ Init process
   ├─ User applications
   ↓
7. System Running
   └─ Linux apps/drivers work transparently
      SHER kernel evolves independently
```

---

## Strategic Benefits

### 1. **Day One Compatibility**
- All existing Linux drivers work
- All Linux applications run
- No rewriting millions of drivers
- Users have mature ecosystem

### 2. **Freedom to Innovate**
- Scheduler can be completely different
- Memory manager can optimize for ARO
- Security can be stronger than Linux
- IPC can use SHER's capabilities
- Kernel developers freed from Linux legacy constraints

### 3. **Gradual Migration Path**
Over time, rewrite important drivers as SHER-native:

**Phase 1**: 100% Linux driver compatibility
**Phase 2**: High-performance drivers rewritten as SHER-native
**Phase 3**: Vendors publish SHER-native drivers
**Phase 4**: SHER becomes first-class platform with own ecosystem

### 4. **Preserved Compatibility**
- Linux drivers continue to work indefinitely
- Older hardware still supported
- No breaking changes to interface
- Backward compatibility guaranteed

---

## Why This Works

The **Linux Compatibility Interface model** succeeds because:

1. **Clean separation**: Linux behavior at one layer, SHER implementation at another
2. **One-way dependency**: Linux calls translate to SHER, SHER doesn't depend on Linux
3. **Tested interface**: Linux ABI is stable, well-documented, unchanged
4. **Translation is local**: Each Linux call independently translates, no global state coupling
5. **Proven approach**: Wine (Windows API), Proton (gaming), WSL (Windows Linux Subsystem) all use this model successfully

---

## Implementation Strategy

### SLCI Layer (crates/slci/)

The compatibility layer should be organized as:

```
crates/slci/
├── syscall/         - Linux system call translation
├── driver_api/      - Linux kernel driver API translation
├── objects/         - Linux object emulation (task, inode, etc.)
├── memory/          - Memory allocation translation
├── interrupt/       - Interrupt handling translation
├── scheduler/       - Process scheduling translation
├── filesystem/      - Filesystem translation
├── networking/      - Network API translation
└── security/        - Security model translation
```

Each translator is independent:
- `syscall/` doesn't know about drivers
- `driver_api/` doesn't know about networking
- Composition via the SHER kernel below

### Incremental Implementation

1. **Phase 1**: Load Linux drivers, translate basic syscalls
2. **Phase 2**: Full syscall coverage, all driver APIs
3. **Phase 3**: Optimize translations per ARO tier
4. **Phase 4**: Native driver framework (drivers can opt-in to native APIs)
5. **Phase 5**: Vendor native drivers begin appearing

---

## The Vision

SHER doesn't try to **be** Linux. It only tries to **look** like Linux from the outside. Inside, it's a completely different kernel optimized for:

- **AI-native computing** (inference, scheduling, optimization)
- **Adaptive resource use** (scale from 128 MB to 128 GB)
- **Immutable safety** (transactional updates, guaranteed rollback)
- **Heterogeneous computing** (CPU, GPU, NPU, DSP scheduling)
- **Next-decade hardware** (not carrying decades of legacy)

The SLCI makes this possible: **Day one compatibility, infinite innovation potential.**

---

## References

- **Wine** (Windows API compatibility for Linux)
- **Proton** (DirectX translation for Vulkan)
- **WSL2** (Windows Subsystem for Linux)
- **QEMU** (Machine emulation)
- **System call interposition** (Ptrace, seccomp)

SLCI is the OS-level equivalent: a clean interface boundary that preserves compatibility while enabling fundamental architectural innovation.
