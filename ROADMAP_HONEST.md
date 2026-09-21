# ROADMAP_HONEST: technical debt ledger & verification snapshot

This file is a **supplement** to [ROADMAP.md](ROADMAP.md), not a
replacement — ROADMAP.md already documents, in the same honesty register
this file uses, what's built vs. not-started per crate. This file exists
specifically to hold the **technical debt** inventory (outdated tooling,
lint suppressions, unwrap/unsafe density, missing CI gates) that
ROADMAP.md's crate-by-crate framing doesn't have room for, plus a
timestamped record of exactly what was verified during the 2026-09
documentation-standardization pass that added this file.

## Verification snapshot (this pass, real commands, real output)

Run from a clean `cargo build --workspace --all-targets` (5.5s, warm
cache) on macOS/Darwin, `rustc`/`cargo` from the active toolchain:

| Command | Result |
|---|---|
| `cargo build --workspace --all-targets` | Clean. ~5.5s (warm target dir). |
| `cargo test --workspace` | **768 passed, 0 failed** across all crates (summed from every `test result: ok` line). ~27s. |
| `cargo clippy --workspace -- -D warnings` | Clean. ~2.7s. This is the bar CI (`.github/workflows/ci.yml`) actually enforces. |
| `cargo clippy --workspace --all-targets` (no `-D warnings`) | **Not clean: 95 warnings**, spanning 9 distinct clippy lint rules across 17 crate test/bench targets. Full breakdown below. |
| `cargo fmt --check` | Clean. |

## Technical debt found this pass (concrete, file:line)

### 1. `Cargo.lock` was gitignored and untracked — fixed this pass

`.gitignore` had a bare `Cargo.lock` entry, and `git ls-files Cargo.lock`
returned nothing — the lockfile was never committed, for a workspace that
produces a real binary (`sher-kernel`, `src/bin/sher-kernel.rs`) and
several benches. That means `cargo build`/`cargo test` on a fresh clone
could silently resolve different dependency versions than whatever the
last person to run it locally had, with no CI catching the drift (CI's
cache key is `hashFiles('Cargo.lock')`, which was hashing a file that
didn't exist in the checkout). Fixed: removed the `Cargo.lock` line from
`.gitignore` and committed the (freshly regenerated) lockfile.

### 2. `cargo clippy --workspace --all-targets` — 95 warnings, real breakdown

Previously, README/ROADMAP described this qualitatively ("a handful of
X/Y/Z warnings in ~9 named crates"). Re-counted exactly this pass:

| Lint | Count | Example |
|---|---|---|
| `clippy::clone_on_copy` (on `Copy`-deriving `ObjectId`/`u64`) | 72 | `crates/system_integration/src/lib.rs:145`, `crates/performance_benchmarks/src/lib.rs:288` |
| `clippy::len_zero` | 5 | `crates/ai/src/tests.rs:157` |
| `clippy::module_inception` | 4 | `crates/lki/src/tests.rs:4` |
| `clippy::unnecessary_cast` | 4 | `crates/lki/src/tests.rs:547` |
| `clippy::identity_op` | 3 | `crates/lki/src/tests.rs:736` |
| `clippy::field_reassign_with_default` | 3 | `crates/kernel/src/kernel.rs:155` |
| `clippy::redundant_pattern_matching` | 1 | `crates/performance_benchmarks/src/lib.rs:464` |
| `clippy::unit_arg` | 1 | `crates/memory/benches/allocator_bench.rs:99` |
| `clippy::bool_comparison` | 1 | `crates/system_integration/src/lib.rs:386` |

Per-target warning counts (from `cargo clippy --workspace --all-targets`,
"generated N warnings" summaries): `system_integration` 27,
`sher_recovery` 12, `audio_driver` 10, `unified_device_manager` 8,
`sher_lki` 7, `performance_benchmarks` 5, `sher_ai` 5, `security_audit` 4,
`hardening` 4, `sher_driver_runtime` 3, `sher_device_manager` 3, `hal` 2,
`wayland_server` 1, `input_driver` 1, `sher_aro` 1, `sher_kernel` 1,
`sher_memory` (bench `allocator_bench`) 1. All of it is confined to
`#[cfg(test)]`/`tests.rs`/`benches/` code — none of it is in library or
binary code, which is why the narrower, CI-enforced
`cargo clippy --workspace -- -D warnings` stays clean. This is real,
scoped, low-risk cleanup (mostly mechanical `clone()` → nothing-needed on
`Copy` types); it was deliberately **not** fixed in this pass per this
project's "disclosure-first" standardization convention — it's exactly
the kind of self-contained follow-up a dedicated session should do so the
fix is reviewable on its own, not buried in a docs commit.

### 3. `unsafe` code: confined to `crates/memory`, no `# Safety` doc comments

`grep -rn "unsafe " crates/ src/` finds `unsafe` in exactly one crate:
`crates/memory` (`master_allocator.rs`, `tier0_slab.rs`, `tier1_slab.rs`,
`stress_tests.rs`, `benches/allocator_bench.rs` — 41 occurrences total).
This confirms README's claim that `driver_runtime` has zero `unsafe` is
accurate (verified: no matches there). But none of the `unsafe fn`
declarations in `crates/memory` (e.g.
`master_allocator.rs:99 pub unsafe fn deallocate(&mut self, ptr: *mut u8, size: usize) -> bool`,
`tier1_slab.rs:176`, `tier1_slab.rs:314`) carry a `# Safety` doc-comment
section explaining their caller invariants (e.g. "ptr must have been
returned by a prior `allocate` call of the same size on this allocator").
For allocator code doing raw pointer arithmetic, that's a real gap —
`clippy::missing_safety_doc` would catch this but isn't enabled. Worth a
follow-up pass specifically over `crates/memory`'s unsafe API surface.

### 4. `.unwrap()` density in non-test library code paths

A repo-wide `grep -c '\.unwrap()'` across `crates/*/src/*.rs` (including
both library code and in-tree `tests.rs` test modules, which is why some
of these numbers are inflated by tests) shows concentration in:
`crates/system_integration/src/lib.rs` (23), `crates/driver_runtime/src/tests.rs` (17, test-only),
`crates/wayland_server/src/lib.rs` (15), `crates/device_manager/src/tests.rs` (15, test-only),
`crates/lki/src/tests.rs` (14, test-only), `crates/ai/src/tests.rs` (12, test-only),
`crates/gpu_driver/src/lib.rs` (11), `crates/kernel/src/kernel.rs` (10),
`crates/core/src/ipc.rs` (10), `crates/services/src/manager.rs` (8),
`crates/scheduler/src/scheduler.rs` (8),
`crates/performance_optimization/src/lib.rs` (8), `crates/memory/src/dma.rs` (8),
`crates/aro/src/lib.rs` (8), `crates/interrupt/src/controller.rs` (7).
The `.rs` files named `lib.rs`/`kernel.rs`/`ipc.rs`/`manager.rs`/
`scheduler.rs`/`dma.rs`/`controller.rs` in that list are **real
library-code unwraps**, not test code — each is a potential panic on
malformed/unexpected internal state rather than a propagated `Result`.
Not individually audited for exploitability in this pass (that's the
scope of a dedicated follow-up, not a docs pass); flagged here as a
concrete starting point rather than a vague "improve error handling"
line.

### 5. No fuzzing, no `cargo-audit`/`cargo-deny` CI job

Confirmed: no `fuzz/` directory, no cargo-fuzz/libfuzzer/afl dependency
anywhere in the repo. `.github/workflows/ci.yml` runs `fmt`/`build`/
`test`/`clippy` only — no dependency-vulnerability scan
(`cargo audit`/`cargo deny`) exists, so a CVE landing in `tokio`/`serde`/
etc. would only surface via the new `.github/dependabot.yml` (added this
pass) opening a PR, not via a CI gate failing on the *current* lockfile.
Adding a `cargo audit` (or `cargo deny check advisories`) CI job is real,
scoped follow-up work — not done in this pass because it requires
network access to the advisory database, which this sandbox couldn't
verify actually works in GitHub's runners without a live test run.

### 6. Stale/self-contradicting top-level docs found and fixed this pass

- `API_REFERENCE.md` still said "Version 1.0.0", "Production Use," and
  labeled sections "Phase 11/12/13" — the same category of claim that
  README/CLAUDE/VISION/ROADMAP were corrected for in an earlier pass, but
  this file was missed. Fixed: added a correction notice banner (same
  pattern already used on the 23 files in `docs/archive/`), left the
  underlying per-crate code examples in place since they're still a
  reasonable index of the real API.
- `.github/GITHUB_GUIDE.md` claimed SHER "proves you can run existing
  Linux drivers on a fundamentally different kernel architecture,"
  quoted invented performance numbers ("Memory allocation: <0.2μs (vs
  Linux ~0.25μs)"), described "Phase 6 (Week 3) Complete" as current
  status, and linked to root-level docs (`QUICK_START.md`,
  `PERFORMANCE_METRICS.md`, `ARCHITECTURE.md`) that no longer exist at
  the repo root (moved to `docs/archive/` in the earlier pass). Fixed:
  rewritten to match README's current, honest framing and correct links.
- `.github/CONTRIBUTING.md` referenced `cargo test --lib`/`cargo check`
  instead of the actual CI-enforced `cargo test --workspace`/
  `cargo clippy --workspace -- -D warnings`; listed fixed per-crate test
  counts ("Driver Runtime: 81 tests," "Total target: 292+ tests") that no
  longer matched reality; and listed fabricated performance targets
  ("Boot time: < 2 seconds to interactive shell," "Interrupt latency: <
  100 microseconds") for a kernel that doesn't boot and has no interrupt
  controller. Fixed: corrected commands, removed the stale counts and
  invented targets, replaced with an honest "don't trust a fixed number
  here" pointer to `cargo test --workspace`.
- `CLAUDE.md` said "767 tests passing" in its status line while
  README.md said "768" (README had already corrected an off-by-one but
  CLAUDE.md wasn't updated). Fixed: rephrased both mentions in CLAUDE.md
  to point at `cargo test --workspace` as the source of truth rather than
  hardcoding a number that will go stale again.

## What was deliberately not done this pass, and why

- **Not fixing the 95 clippy `--all-targets` warnings.** Real, scoped,
  mechanical work (see table above) — but it touches 17 crates' test/
  bench code, and bundling it into a documentation-standardization commit
  would make the diff harder to review for either purpose. Left as the
  single, focused item ROADMAP.md's "Near-term plan" already names first.
- **Not adding a `cargo audit`/`cargo deny` CI job.** Would need a real
  CI run against GitHub's network-enabled runners to confirm it actually
  works (advisory-database fetch, config syntax); this sandbox has no
  network access to crates.io/GitHub to validate that locally. Flagged as
  a real gap (item 5 above), not silently added and hoped-for.
- **Not auditing dependency versions for known CVEs by hand.** No network
  access in this environment to cross-reference `Cargo.lock` against
  RustSec's advisory database. `.github/dependabot.yml` (added this pass)
  will surface outdated versions going forward; it does not retroactively
  audit what's pinned today.
- **Not adding `# Safety` doc comments to `crates/memory`'s unsafe
  functions.** Flagged (item 3) rather than fixed — writing accurate
  safety-invariant documentation requires understanding each call site's
  actual guarantees, which is implementation work, not a docs-pass fix.
- **Not deeply auditing the other 39 crates line-by-line.** Given this
  repo's size (40 crates), this pass prioritized workspace-wide signals
  (build/test/clippy/fmt, doc consistency, CI, dependency-update
  automation) over exhaustive per-crate review. Crate-level real-vs-
  simulated claims were spot-checked against README's existing table
  (which was itself produced by a prior, more granular pass) rather than
  re-derived from scratch.
