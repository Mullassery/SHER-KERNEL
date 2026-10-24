# Competitive intelligence in roadmap and upgrade work

**Policy date:** 2026-10-19

Competitive findings are an input to product decisions, not a one-off report.
Every pending roadmap item, new feature, and routine dependency upgrade should
check the repository's [`AUDIT_FINDINGS.md`](AUDIT_FINDINGS.md) before work
starts, and record the decision in the PR or changelog.

## Required check before building

1. **Read `AUDIT_FINDINGS.md`.** Find the relevant competitor row and the
   ranked next actions. If the finding is stale, update it with a dated source.
2. **Name the competitor and the user problem.** A feature is not justified by
   "competitor has it". State the workflow it unlocks, who needs it, and how
   often. Mark unverified demand as a hypothesis.
3. **Prefer reuse over new code.** Check whether an existing Mullassery repo
   already provides the capability (see `BUNDLES.md` in the profile repo and
   the bundle role file in this repo, when present). Typical reuse:
   - crawling/fetching/robots/SSRF → **MudFish**
   - architecture maps / PR diffs → **CodeAtlas**
   - data-quality contracts / validation → **StatGuardian**
   - token counting / cost tables → **PyTokenCalc**
   - dependency graph / advisories / SBOM → **PyDeepMesh**
   - segmentation / CLV / churn → **ClusterAudienceKit**
   - activation / lineage / PII → **PyReverseETL**
   - fleet state sync → **SyncFleet** (only after conflict resolution)
4. **Choose one disposition:** build, integrate, defer, or reject. Write it
   down. A rejection with a reason is a successful outcome.
5. **For routine upgrades**, check whether the upgrade closes a known
   competitor gap or breaks a differentiated capability. Run the tests that
   cover the affected path; do not merge a green build that silently changes a
   user-visible contract.

## PR / changelog checklist

- [ ] Competitor and source URL/date recorded (or "no direct competitor").
- [ ] User impact and frequency stated; unverified demand labelled.
- [ ] Reuse check done against sibling repos/bundles.
- [ ] Failure mode and rollback considered (especially auth, PII, cost).
- [ ] Docs/roadmap status updated honestly.
- [ ] Tests or a manual verification command included.

## Review cadence

- Refresh the competitor row when a release changes the capability.
- Re-check the full `AUDIT_FINDINGS.md` at least quarterly and before a major
  release.
- If a finding is wrong or stale, correct it in the same PR that relies on it.

## Non-goals

- Copying features for parity.
- Adding a dependency just because a competitor uses it.
- Treating star counts or download counts as proof of user need.

## OSS gap analysis — 2026-10-24

**What matters most here:** keep it a research artifact. No bootloader or hardware isolation is explicitly out of scope; the useful next steps are in-process IPC, real telemetry wiring, and fuzzing, not chasing seL4 parity.

Scope: assessed SHER-Kernel's userspace kernel-object-model prototype against three actively maintained OSS kernels — seL4 (verified microkernel), Asterinas (Linux-ABI Rust framekernel) and Hubris (embedded Rust microkernel) — on bootability, isolation, ABI coverage, IPC, scheduling and update mechanics.

### Comparable OSS projects
| Project | GitHub | License | Last activity | Why it is the comparison |
| --- | --- | --- | --- | --- |
| seL4 | https://github.com/seL4/seL4 | GPLv2 kernel; BSD-2-Clause libs | 16.0.0, 2026-07-22 | Canonical capability-based microkernel with machine-checked proofs; the bar for isolation and IPC guarantees. |
| Asterinas | https://github.com/asterinas/asterinas | MPL-2.0 | 0.19.0, 2026-10-13 | Rust, Linux-ABI-compatible, bare-metal framekernel; closest "Rust OS" peer to SHER's stated ambition. |
| Hubris | https://github.com/oxidecomputer/hubris | MPL-2.0 | SP 1.83.0, 2026-10-21 | Production embedded Rust microkernel; static task model, memory protection, crash-restart, real hardware. |

### Capability comparison
| Capability | Ours | seL4 | Asterinas | Hubris | Gap |
| --- | --- | --- | --- | --- | --- |
| Boots on bare metal | No | Yes | Yes | Yes | Fundamental |
| Hardware MMU/MPU isolation | No | Yes | Yes | Partial | Fundamental |
| Formal verification | No | Yes | No | No | Large |
| Linux syscall ABI | No | No | Yes | No | Large |
| Cross-process IPC | No | Yes | Yes | Yes | Large |
| Driver isolation | Partial | Yes | Partial | Yes | Large |
| Capability security | Yes | Yes | No | Partial | Narrow |
| Preemptive scheduler | Partial | Yes | Yes | Yes | Large |
| Real device support | No | Partial | Partial | Partial | Large |
| A/B update/rollback | Yes | No | No | Partial | Ours ahead |
| AI/anomaly telemetry | Yes | No | No | No | Ours ahead |

Our "Yes" rows are in-process: capability grants, A/B bookkeeping and AI logic are real and unit-tested but never enforce anything on hardware. Asterinas' Linux ABI is ~240+ syscalls; ours is name-level lookup only. Hubris has no MMU but uses an MPU for region protection, and its A/B support is board/bootloader-specific ("Partial").

### Gaps that matter (ranked)
1. **No bootable or hardware-executed kernel** — README and `docs/architecture/hardware-interop.md` both state there is no bootloader, ring-0 code or MMU programming. Anyone evaluating an OS cannot run it; effort L (multi-year, explicitly out of scope).
2. **Isolation is object-model only, not hardware-enforced** — `driver_runtime` uses in-process capability checks and restart/quarantine, with no fork, seccomp or address-space boundary (ROADMAP.md). This undercuts the core "security by architecture" claim versus seL4/Hubris; effort L.
3. **No cross-process transport for framebuffers/input** — `core::ipc` is a lock-free in-process mailbox; ROADMAP.md lists cross-process transport as not started. It blocks the SHER-Display/Graphics integration story; effort M within the prototype.
4. **We lack a real syscall/ABI surface** — `lki`/`compatibility` translate names, not behaviour; Asterinas demonstrates the real target (240+ syscalls, bootable NixOS). "We have a weaker version" of compatibility, not compatibility; effort L.
5. **No formal verification or independent security review** — seL4 has code-level proofs; our 40 crates and ~768 tests are unfuzzed and unreviewed (README "Known issues"). This is the main credibility gap for safety claims; effort L.
6. **Device discovery and driver loading are simulated** — no PCI/USB enumeration; drivers are in-process structs. Limits any claim of Linux hardware compatibility; effort L (needs privilege).

### Where we are differentiated
- Time-bounded, tiered capability grants with expiry and re-authentication are first-class; seL4 capabilities are not time-bounded by default, and Asterinas/Hubris do not expose this model.
- An explicit AI subsystem (anomaly detection, predictive allocation, adaptive scheduling) with honest synthetic-telemetry boundaries; none of the three peers ships an equivalent.
- An A/B transactional updater with verify/boot-test/rollback is present in-process — unusual for a kernel prototype and absent from seL4/Asterinas.
- Per-crate simulation-boundary documentation and a large in-tree test suite make it unusually easy to study and extend without hardware.

### Disposition
- **Build:** cross-process IPC transport jointly with SHER-Display; wire `ai` to the real telemetry `diagnostics` already collects; fuzz `lki`/`hardening` validation.
- **Integrate/reuse:** adopt seL4's capability/IPC vocabulary and invariant style; borrow Hubris's static task configuration and Humility-style postmortem inspection; use Asterinas's syscall list/test corpus as the compatibility reference rather than reimplementing it.
- **Defer/reject:** bootable kernel, Linux ABI emulation, and any "OS replacement" positioning; keep the repo a research artifact per AUDIT_FINDINGS.md.

### Sources
- https://github.com/seL4/seL4 (accessed 2026-10-24)
- https://github.com/seL4/seL4/releases/tag/16.0.0 (accessed 2026-10-24)
- https://github.com/asterinas/asterinas (accessed 2026-10-24)
- https://asterinas.github.io/2026/10/13/announcing-asterinas-0.19.0.html (accessed 2026-10-24)
- https://github.com/oxidecomputer/hubris (accessed 2026-10-24)
- https://github.com/oxidecomputer/hubris/releases/tag/all-sp-v1.83.0 (accessed 2026-10-24)
- Repo files inspected: `README.md`, `COMPETITIVE_INTELLIGENCE.md`, `AUDIT_FINDINGS.md`, `CLAUDE.md`, `ROADMAP.md`, `ROADMAP_HONEST.md`, `docs/architecture/hardware-interop.md`, `Cargo.toml`
