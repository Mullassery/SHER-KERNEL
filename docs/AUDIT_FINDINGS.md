# Audit Findings: SHER-Kernel

**Audit Date:** 2026-10-26  
**Version:** 0.3.0  
**Branch:** `main`  
**Full report:** [`.audit/reports/SHER-Kernel/audit.md`](../.audit/reports/SHER-Kernel/audit.md)

## Scope

Read-only review of the object/capability model, core IPC, memory, scheduler, driver lifecycle, recovery/snapshot/updater, kernel orchestration, tests, CI, and security documentation. No boot, real hardware, multi-process, or privileged execution was performed.

## Findings

| ID | Severity | Finding | Evidence | Recommendation |
|---|---|---|---|---|
| SK-01 | High | Non-expiring capability grants are constructible. | `CapabilityGrant` public `expires_at: Option<u64>`; `None => true`; serde. | Make expiry non-optional/private and gate non-expiring grants behind an admin-only constructor. |
| SK-02 | High | Update “signatures” are a non-cryptographic checksum. | `recovery/src/partition.rs`: `DefaultHasher`; updater docs say signatures. | Use a cryptographic digest plus real signature/trust-root verification; relabel until then. |
| SK-03 | Medium | Capability tier is not enforced. | `CapabilitySet::has_capability` ignores `tier`. | Require a minimum tier for high-impact capabilities and test escalation. |
| SK-04 | Medium | Expiry uses wall-clock time. | `SystemTime::now()`. | Use monotonic/boot-epoch time or document rollback/reissue policy. |
| SK-05 | Medium | IPC sender identity is caller-supplied/spoofable. | `IpcBus::send(to, from, ...)`; `Message.from`. | Bind sender to a capability/handle or unforgeable token. |
| SK-06 | Medium (known) | No real OS isolation. | README: object-model isolation only. | Keep explicit; use process/OS containment for real drivers. |
| SK-07 | Medium (known) | No cross-process IPC transport. | README. | Add socket/shared-memory transport before claiming cross-process security. |
| SK-08 | Medium | README/CI truth drift. | 768 vs 777 tests; “No fuzzing” vs fuzz CI; no CONTRIBUTING vs exists; deny vs audit. | Regenerate claims from CI. |
| SK-09 | Low/Medium | Miri/fuzz scope is narrow. | `sher_memory` only; 30s allocator. | Extend to parsers/IPC/update metadata. |
| SK-10 | Low | Audit log is not durable/tamper-evident. | In-process audit. | Add durable, tamper-evident security events. |

## Positive Controls

- Explicit capability/tier model and fail-closed expiry on unreadable clock.
- Revocation and expired-grant cleanup exist.
- Bounded lock-free IPC mailboxes with zero-copy `Arc<[u8]>` payloads.
- Conservative A/B update/rollback state machine.
- Driver quarantine/backoff policy.
- Miri on the unsafe memory crate and allocator fuzzing in CI.
- Two-architecture CI with strict clippy and rustsec audit.
- Honest documentation of simulation boundaries and non-goals.

## Audit Limitations

- No boot, real hardware, privileged, or multi-process execution.
- Findings are source-inspection based, not exploited.
- Miri/fuzz/audit were not re-run locally.
- Some crates were assessed at interface/policy level.
- Test counts are definitions, not a verified pass count.
