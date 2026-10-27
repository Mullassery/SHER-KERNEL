# Audit Report: SHER-Kernel

**Audit Date:** 2026-10-26  
**Repository Path:** `/Users/georgimullassery/Desktop/Projects/SHER-Kernel`  
**Audit Type:** Read-only systems design audit  
**Version:** 0.3.0 (workspace)  
**Branch:** `main`  
**Source Size:** ~30,407 Rust LOC; 40 workspace crates; 777 tests

## Executive Summary

SHER-Kernel is an in-process userspace prototype of OS-kernel concepts: object model, capabilities, scheduling, memory, driver lifecycle, IPC, snapshots, recovery, updates, AI/runtime services, and a large set of simulated hardware subsystems. It is explicitly not a bootable kernel, and most of its value is in the coherence of the object/capability model and the breadth of testable, in-process policy.

The most serious issues are security-model truth, not missing features. `CapabilityGrant` exposes public fields and treats `expires_at: None` as valid forever, so any in-process caller can construct a non-expiring grant and insert it into a `CapabilitySet`. `has_capability` ignores tier, so a low-tier grant satisfies a high-impact capability. The updater advertises “verify signatures + hashes,” but the implementation is a non-cryptographic `DefaultHasher` checksum with no signature or trust root. Capability expiry uses wall-clock time, and IPC `from` is caller-supplied with no sender authentication. The documented “zero trust” and “every permission expires” invariants are therefore not enforced by construction.

## Purpose and Positioning

- **What it is:** a userspace, in-process prototype of kernel object-model, capability, scheduling, memory, driver-lifecycle, IPC, recovery/update, and service concepts.
- **Who it is for:** the SHER OS stack, systems-design research, and testing policy/lifecycle logic without privileged hardware code.
- **What it is not:** a bootable kernel, a real hypervisor, an OS sandbox, a production update system, or a real cross-process IPC transport.
- **Differentiator:** a broad, tested object/capability model with real allocator work, driver lifecycle policy, A/B update state machine, and an in-process kernel orchestrator.

## Architecture

```text
crates/kernel (SherKernel)
  audit logging, AI service wiring, status reporting
        |
        +--> common / objectmodel / security
        |      object IDs, capabilities, tiers, audit
        |
        +--> bootstrap / core / runtime / scheduler / compute / memory
        |      Stage-1 simulation, object lifecycle, scheduling, allocator
        |
        +--> drivers / device_manager / driver_runtime / unified_device_manager
        |      registry, hotplug, lifecycle, sandbox/quarantine/backoff
        |
        +--> interrupt / hal / gpu_driver / input_driver / audio_driver
        |      hardware-shaped interfaces and simulations
        |
        +--> networking / storage / wayland_server / compatibility
        |      simulated or edge adapters
        |
        +--> recovery / snapshot / updater
        |      A/B partitions, checksums, boot pointer, health checks
        |
        +--> services / ai / aro / diagnostics / profiling / benchmarks
               optional services and orchestration
```

### Capability path

```text
CapabilityGrant::new(capability, tier)
  -> SystemTime wall clock
  -> expires_at = Some(now + tier duration)
CapabilitySet::grant
CapabilitySet::has_capability
  -> any matching grant with is_valid()
  -> tier is not considered
```

### Update path

```text
download to standby partition
  -> write_image computes DefaultHasher checksum
  -> verify compares checksum
  -> health/boot test
  -> switch boot pointer
  -> rollback keeps previous slot intact
```

### IPC path

```text
IpcBus::send(to, from, Arc<[u8]>)
  -> named bounded ArrayQueue mailbox
  -> Message { from, payload }
IpcBus::receive(mailbox)
  -> FIFO, lock-free, payload refcount clone
```

## Repository Layout

```text
crates/common/               Capability, PermissionTier, ObjectId, errors
crates/objectmodel/          object model + CapabilityGrant/Set
crates/security/             security policy primitives
crates/core/                 kernel core + IPC
crates/kernel/               SherKernel orchestration and binary
crates/bootstrap/            simulated Stage-0/Stage-1 bring-up
crates/memory/               allocator/slab/free-list (Miri target)
crates/scheduler/            scheduling policy
crates/compute/              accelerator/queue modeling
crates/drivers/              driver-shaped prototypes
crates/device_manager/       device registry/hotplug
crates/driver_runtime/       lifecycle, sandbox policy, quarantine/backoff
crates/recovery/             A/B partitions, boot pointer, health checks
crates/snapshot/             versioned snapshots
crates/updater/              transactional update state machine
crates/networking/, storage/ simulated I/O
crates/ai/, services/, aro/  optional service layers
crates/hardening/, security_audit/, profiling/, benchmarks/
fuzz/                        allocator fuzz target + corpus
.github/workflows/ci.yml     test/audit/Miri/fuzz-smoke
```

## Technology Stack and Dependencies

- **Rust:** 2021 edition; Tokio, crossbeam-queue, serde, uuid, tracing, thiserror/anyhow.
- **Testing:** Rust unit/integration tests, Miri on `sher_memory`, cargo-fuzz allocator target.
- **CI:** x86_64 + arm64 Linux; fmt, build all targets, test all features, clippy `-D warnings`, rustsec audit, Miri, 30s allocator fuzz.
- **No network, no OS sandbox, no unsafe in driver runtime**; the allocator contains the main unsafe/memory work.
- **No published package**; consumed via path dependencies.

## Public API

- `sher_common::{ObjectId, Capability, PermissionTier, Error, Result}`.
- `sher_objectmodel::{Object, ObjectModel, CapabilityGrant, CapabilitySet, ...}`.
- `sher_security` policy primitives.
- `sher_scheduler`, `sher_compute`, `sher_memory`.
- `sher_device_manager`, `sher_driver_runtime`, `sher_unified_device_manager`.
- `sher_recovery::{PartitionSlot, ImmutablePartition, BootPointer, HealthCheckReport}`.
- `sher_snapshot::SnapshotStore`.
- `sher_updater::{Transaction, TransactionState}`.
- `sher_core::ipc::{IpcBus, Message}`.
- `sher_kernel::SherKernel` and the `sher-kernel` status binary.
- AI/service/orchestration crates (`ai`, `services`, `aro`, `diagnostics`, etc.).

## Design Patterns

- **Object-capability model:** typed capabilities attached to object IDs, with tiers and expiry.
- **Simulation boundaries:** every hardware-dependent module documents what is simulated.
- **In-process orchestration:** `SherKernel` wires subsystems and exposes status/audit.
- **In-process IPC primitive:** named lock-free bounded mailboxes with `Arc<[u8]>` zero-copy payloads.
- **A/B update state machine:** stage, verify, test, commit, rollback.
- **Driver lifecycle policy:** registry, quarantine, backoff, crash recovery.
- **Fail-closed clock handling:** unreadable clock denies rather than grants.
- **Explicit non-goals:** no bootable kernel, no OS sandbox, no real transport.

## Data and Storage

- In-memory object model, capability sets, device registry, scheduler state, snapshots, and mailboxes.
- `ImmutablePartition` stores an in-memory image, version, and a `u64` `DefaultHasher` checksum.
- `SnapshotStore` is in-memory versioned history.
- IPC mailboxes are bounded `ArrayQueue`s keyed by string name; payloads are `Arc<[u8]>`.
- No persistent disk format, no database, no credential store.
- No encryption at rest; this is a simulation.

## Concurrency and Performance

- `IpcBus` mailboxes are lock-free bounded queues; send/receive take `&self`.
- `ArrayQueue` defaults to 64 messages per mailbox; full mailboxes return an error rather than growing.
- `Arc<[u8]>` payloads avoid byte copies on send/receive.
- `IpcBus` registration mutates a `HashMap` and is not a concurrent registration path.
- Scheduler/compute/models are in-process; no real parallelism across cores was validated.
- Miri covers `sher_memory`; fuzzing is a 30-second allocator smoke.
- No throughput/latency benchmark for IPC, scheduler, or driver lifecycle.
- No real cross-process transport, so IPC performance claims are model-level only.

## Security and Privacy

### Strengths

- Capability-gated object operations with explicit tiers.
- Expiry checks fail closed when the clock is unreadable.
- `cleanup_expired` exists; revocation removes matching grants.
- No `unsafe` in driver runtime; object-model isolation is explicit.
- Driver crash recovery/quarantine/backoff is modeled.
- Update state machine leaves the active partition untouched until commit.
- Miri and allocator fuzzing target the main unsafe memory code.
- CI runs fmt/build/test/clippy/audit.
- Kernel orchestration includes audit logging.
- Documentation is candid about simulations and missing transports.

### Gaps

- **Non-expiring grants:** `expires_at` is `Option<u64>`, public, serializable, and `None => true`. Any caller can construct `CapabilityGrant { expires_at: None, .. }` or mutate an existing grant. The “every permission expires” invariant is not enforced by the type.
- **Tier is not enforced:** `has_capability` checks capability and validity only. A `Low` grant for a high-impact capability is accepted; tier is metadata, not a minimum-privilege constraint.
- **Wall-clock expiry:** `SystemTime` is not monotonic; clock rollback can extend a grant. There is no boot-epoch or secure-time source.
- **No signature/trust root in updates:** `ImmutablePartition::verify` compares a non-cryptographic `DefaultHasher` checksum. An attacker who can alter the image can recompute the checksum. The updater comment says “verify signatures + hashes,” which overstates the implementation.
- **IPC sender spoofing:** `Message.from` is caller-supplied; `send(to, from, payload)` does not authenticate the sender or check a capability. Any in-process holder can enqueue to any registered mailbox.
- **No real OS sandbox:** driver isolation is object-model isolation only; in-process panics/crashes are not contained by hardware/OS boundaries.
- **No cross-process transport:** capabilities and IPC do not cross a real process boundary, so the security model is not exercised adversarially.
- **Audit log scope:** kernel audit logging exists, but it is not a tamper-evident, durable security log, and capability/update/IPC events may not all be covered.
- **No secret handling:** no key management for update signing or capability issuance.

## Reliability and Observability

- A/B update/rollback is structurally conservative: standby is written before boot pointer switch.
- Health checks gate commit.
- Driver quarantine/backoff is modeled.
- Bounded mailboxes prevent queue growth and report full errors.
- `CapabilityGrant::is_valid` fails closed on clock errors.
- Kernel status/audit reporting exists.
- No structured metrics/tracing for scheduler, IPC, driver recovery, or update outcomes.
- No durable audit log or tamper evidence.
- No real crash/reboot test because this is in-process.
- Miri/fuzz scope is narrow (memory allocator only).

## Testing, CI, and Quality

- 777 static tests; README says roughly 768.
- 40 crates; broad unit/integration coverage across object model, capabilities, memory, scheduler, drivers, recovery, updater, IPC, services.
- CI: x86_64 + arm64, fmt, build all targets/features, test all features, clippy `-D warnings`, rustsec audit, Miri on `sher_memory`, 30s allocator fuzz.
- Miri is scoped to one crate; fuzzing to one target.
- No tests for non-expiring grants, tier enforcement, clock rollback, IPC sender spoofing, or checksum tampering.
- No adversarial update tests (image+checksum replacement).
- No coverage gate, stress test, or real multi-process test.
- README claims “No fuzzing” while a fuzz target and CI job exist; README says no `CONTRIBUTING.md` while one exists; README claims cargo-deny in CI while the workflow has cargo-audit only.
- README’s “Miri/fuzz leak job” known issue is stale relative to the CI workflow.

## Documentation, Packaging, and Release

- Workspace version is 0.3.0.
- Documentation is unusually explicit about simulation boundaries, no bootable kernel, no OS-level driver isolation, and no cross-process IPC.
- README test count (768) is stale vs 777 static tests.
- README says “No fuzzing” and “no CONTRIBUTING.md,” both contradicted by the current tree.
- README says cargo-deny is wired into CI; the observed CI uses rustsec `audit-check`, not cargo-deny.
- Not published; path dependencies only.
- No release provenance, signing, or update-key management.

## Strengths

- Broad, coherent object/capability model with real tests.
- Bounded, lock-free IPC primitive with zero-copy payloads.
- Conservative A/B update state machine and rollback model.
- Driver lifecycle/quarantine/backoff policy is modeled and tested.
- Fail-closed clock handling in `is_valid`.
- Miri on the unsafe memory crate and allocator fuzzing in CI.
- Explicit simulation boundaries and non-goals.
- Kernel orchestration/audit/status surface.
- CI runs on two architectures with strict clippy.

## Risks and Gaps

| Risk | Severity | Evidence | Impact |
|---|---|---|---|
| Non-expiring capability grants are constructible | High | `CapabilityGrant` public `expires_at: Option<u64>`; `None => true`; serde | “Every permission expires” is bypassable by any in-process caller. |
| Update “signatures” are actually a non-cryptographic checksum | High | `recovery/src/partition.rs`: `DefaultHasher`; updater docs say signatures | Tampered images can pass verification; update trust claim is false. |
| Capability tier is not enforced | Medium | `CapabilitySet::has_capability` ignores `tier` | Low-tier grants satisfy high-impact capabilities; least privilege is not enforced. |
| Expiry uses wall-clock time | Medium | `SystemTime::now()` | Clock rollback extends grants; no monotonic/boot-epoch source. |
| IPC sender identity is spoofable | Medium | `IpcBus::send(to, from, ...)`; caller-supplied `Message.from` | Any in-process caller can impersonate another sender. |
| No real OS isolation | Medium (known) | README: object-model isolation only | Driver/capability failures are not contained by OS boundaries. |
| No cross-process transport | Medium (known) | README | Security/IPC not exercised across a real trust boundary. |
| README/CI truth drift | Medium | 768 vs 777; “No fuzzing” vs fuzz CI; no CONTRIBUTING vs exists; deny vs audit | Misleads reviewers and release decisions. |
| Miri/fuzz scope is narrow | Low/Medium | `sher_memory` only; 30s allocator | Other unsafe/policy code is not memory-checked/fuzzed. |
| Audit log is not durable/tamper-evident | Low | In-process audit | Security decisions are not independently verifiable. |

## Prioritized Recommendations

### P0 — capability and update truth

1. Make expiry non-optional in the type: remove `expires_at: Option`, add explicit `NonExpiring` only behind a privileged constructor, and make fields private. Reject deserialized grants with `None` unless an admin-only path is used.
2. Enforce minimum tiers in capability checks (for example, `require(capability, minimum_tier)`) rather than `has_capability` alone. Add tests proving a Low grant cannot satisfy a High/Critical operation.
3. Replace the `DefaultHasher` checksum with a cryptographic digest and add a real signature/trust-root verification path for updates. Until then, rename/document `verify` as corruption detection only, not authenticity.
4. Use a monotonic/boot-epoch clock for expiry, or explicitly document wall-clock rollback and add an upper bound/reissue policy.

### P1 — IPC and isolation

5. Authenticate IPC senders: bind `from` to a capability/handle or an unforgeable sender token; do not accept an arbitrary string as identity.
6. Add a real out-of-process transport (socket/shared memory) before claiming cross-process IPC security, and test capability transfer at that boundary.
7. Extend Miri/fuzz coverage beyond `sher_memory` to object/capability parsing, IPC decoders, and update metadata.
8. Make audit records durable/tamper-evident for capability and update events.

### P2 — documentation and verification

9. Regenerate README counts and claims from CI: 777 tests, fuzzing present, CONTRIBUTING exists, cargo-audit not cargo-deny.
10. Add adversarial tests: non-expiring grant, tier escalation, clock rollback, sender spoofing, image+checksum tampering.
11. Add stress/benchmark coverage for IPC, scheduler, and driver lifecycle.
12. Document the exact simulation boundary for every security claim (capability, update, isolation, IPC).

## Audit Limitations

- No boot, real hardware, multi-process, or privileged execution was performed.
- Security findings are based on source inspection, not exploitation.
- Miri/fuzz/cargo-audit were not re-run locally; CI configuration was reviewed.
- Some crates were assessed at the interface/policy level rather than line-by-line.
- Test counts are static definitions, not a verified pass count.
