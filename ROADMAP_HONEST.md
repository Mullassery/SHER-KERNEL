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

**Updated snapshot (quick-fix pass, 2026-09)** — see "Quick-fix pass
(2026-09), summary" at the end of this file for what changed:
`cargo test --workspace` now passes **769/769**;
`cargo clippy --workspace --all-targets` is now **clean (0 warnings)**,
same as the `-D warnings` variant; `cargo build --workspace --all-targets`
and `cargo fmt --check` remain clean. The rows above are left as originally
written (a snapshot of that pass, not retroactively edited) — items 2 and 4
below are annotated inline with what was fixed since.

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

### 2. `cargo clippy --workspace --all-targets` — 95 warnings — ~~fixed this pass~~ FIXED (quick-fix pass, 2026-09)

Previously, README/ROADMAP described this qualitatively ("a handful of
X/Y/Z warnings in ~9 named crates"). Re-counted exactly in the pass that
added this file:

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
`sher_memory` (bench `allocator_bench`) 1. All of it was confined to
`#[cfg(test)]`/`tests.rs`/`benches/` code — none of it was in library or
binary code, which is why the narrower, CI-enforced
`cargo clippy --workspace -- -D warnings` was already clean.

**Fixed in a follow-up "quick-fix" pass** (2026-09), as anticipated below:
`cargo clippy --fix --workspace --all-targets --allow-dirty` mechanically
resolved 87 of the 95 (all `clone_on_copy`, `len_zero`, `unnecessary_cast`,
`identity_op`, `redundant_pattern_matching`, `unit_arg`, `bool_comparison`
instances, plus 1 `field_reassign_with_default`); the remaining 8 needed a
one-line hand fix each: 3 more `field_reassign_with_default` sites
(`crates/kernel/src/kernel.rs`, `crates/recovery/src/crash_recovery.rs`,
`crates/aro/src/lib.rs` — rewritten as struct-update-syntax literals) and 1
`identity_op`-adjacent unnecessary-parens warning
(`crates/lki/src/tests.rs:736`). The 4 `module_inception` warnings
(`crates/{device_manager,driver_runtime,ai,lki}/src/tests.rs`) come from a
structural quirk, not a real naming collision: `lib.rs` in each of those
crates declares `#[cfg(test)] mod tests;` pointing at `tests.rs`, and
`tests.rs` *itself* wraps its entire contents in another `mod tests { ... }`,
nesting the module as `tests::tests`. Fully resolving that would mean
re-indenting ~900-1200 lines per file for a purely cosmetic nesting issue
that doesn't affect `cargo test` (it finds `#[test]` fns regardless of
module depth) — left as `#[allow(clippy::module_inception)]` with a comment
rather than risking a large mechanical re-indent in the same commit as
everything else. `cargo clippy --workspace --all-targets` is now clean (0
warnings); `cargo test --workspace` still passes in full (769/769, up by 1
new regression test added in the same pass — see item 4).

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

### 4. `.unwrap()` density in non-test library code paths — mostly a false alarm; 5 real ones found and fixed (quick-fix pass, 2026-09)

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

**Correction (quick-fix pass, 2026-09):** the claim above — that every file
in that list *not* literally named `tests.rs` was "real library-code
unwraps, not test code" — was checked line-by-line and was **wrong** for
9 of the 11 `lib.rs`/`kernel.rs`-named files. `system_integration/src/lib.rs`,
`wayland_server/src/lib.rs`, `gpu_driver/src/lib.rs`, `core/src/ipc.rs`,
`services/src/manager.rs`, `scheduler/src/scheduler.rs`,
`performance_optimization/src/lib.rs`, `memory/src/dma.rs`, and
`aro/src/lib.rs` each have exactly one `#[cfg(test)] mod tests { ... }`
block, and *every single* `.unwrap()` in each of those files falls inside
it (verified by counting occurrences before vs. after each file's
`#[cfg(test)]` line) — they're test-only, same as the files already
labeled that way. The "filename heuristic" used to produce this section
originally (`lib.rs`/`kernel.rs`/etc. ⇒ production code) doesn't hold when
a crate keeps its tests in a `mod tests { ... }` block at the bottom of
`lib.rs` instead of a separate `tests.rs` file, which is the majority
pattern in this repo.

Only `crates/kernel/src/kernel.rs` had real production-code unwraps: 2,
both `SystemTime::now().duration_since(UNIX_EPOCH).unwrap()` (in `new()`
and `uptime()`). **Fixed**: `new()` now propagates a clock error through
its existing `Result` return type instead of panicking;
`uptime()` (which returns a bare `u64`, part of this crate's public API, so
its signature was left unchanged per this project's cross-repo API-stability
rule) now uses `.unwrap_or_default()` plus `saturating_sub` instead of plain
`-`, since the original `current_time - self.boot_time` would also panic
on `u64` underflow if the clock ever read behind `boot_time`. Added a
regression test, `uptime_does_not_panic_when_boot_time_is_in_the_future`,
that reaches into the (test-visible) private `boot_time` field to set it to
`u64::MAX` and asserts `uptime()` returns `0` instead of panicking; verified
this test fails (`attempt to subtract with overflow`) against the
pre-fix `current_time - self.boot_time` and passes against the fix.

While fixing this, the same `duration_since(UNIX_EPOCH).unwrap()` pattern
was found (via a separate `grep -rn "duration_since(UNIX_EPOCH)"`, not part
of the original `.unwrap()` count above) in 4 more production-code
call sites, all foundational object-model types used across the whole
workspace: `crates/objectmodel/src/capabilities.rs:16` (`CapabilityGrant::new`)
and `:39` (`CapabilityGrant::is_valid`), and
`crates/objectmodel/src/lifecycle.rs:29,47,56` (`Lifecycle::default`/`start`/`stop`).
Also fixed, with different fallbacks depending on how the value is used:
`lifecycle.rs`'s three call sites are plain bookkeeping timestamps (not
compared against anything), so they use `.unwrap_or_default()`, same as
`kernel.rs`. `capabilities.rs`'s two call sites needed more care because
they gate a security decision: `new()` uses `.unwrap_or_default()` (a
clock failure there produces a grant stamped at the epoch, which reads as
already-expired against any real subsequent clock reading — fails secure).
`is_valid()` deliberately does **not** use the same `unwrap_or_default()`
pattern — `now = 0` there would make `0 < expiry` true for any real
`expiry`, i.e. it would fail *open* (every grant reads as valid during a
clock glitch) — so it was written as
`.map(|d| d.as_secs() < expiry).unwrap_or(false)` instead, which fails
secure (an unreadable clock reads as "expired," not "valid forever" and
not a panic). This asymmetry was caught by re-deriving the failure
semantics for each call site individually rather than applying one
mechanical find-replace across all 5 — flagging it here since it's the
kind of subtlety a future pass touching this code should be aware of.
Existing test suites for both files (`cargo test -p sher_objectmodel`, 18
tests) pass unchanged; no new regression test was added for the clock-error
branch specifically, because forcing `SystemTime::now()` to read before
`UNIX_EPOCH` isn't mockable without introducing a clock-injection
abstraction into these types, which would be a larger refactor than this
quick-fix pass's scope (and would touch `CapabilityGrant`/`Lifecycle`,
which other crates construct directly).

Net result of re-auditing the 11 `lib.rs`/`kernel.rs`-named files in the
original count: 9 were entirely test-only (this file's claim was wrong),
and the 2 real production unwraps that did exist (both in `kernel.rs`) are
now fixed, along with the 4 related sites found in `objectmodel` above.
That leaves no known real `.unwrap()`-panic-on-clock-read sites in this
repo's production code as of this pass.

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

- ~~**Not fixing the 95 clippy `--all-targets` warnings.**~~ **Fixed in a
  follow-up quick-fix pass (2026-09)** — see item 2 above and the summary
  section at the end of this file. Originally deferred because it touched
  17 crates' test/bench code and bundling it into a documentation-
  standardization commit would have made the diff harder to review for
  either purpose; done later as its own focused commit, as anticipated
  here.
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

## Quick-fix pass (2026-09), summary

A follow-up pass scoped specifically to low-risk, well-understood fixes
(explicitly *not* architectural rewrites, *not* new kernel features, and
conservative about anything touching memory/scheduler/interrupt code).
What changed, in full:

- **Item 2 (95 clippy `--all-targets` warnings): fixed.** 0 warnings now.
  See item 2 above for the breakdown of mechanical `cargo clippy --fix`
  auto-fixes vs. the 8 sites that needed a one-line hand fix vs. the 4
  `module_inception` sites resolved with a documented `#[allow]` instead
  of a large re-indent.
- **Item 4 (`.unwrap()` density): corrected and partially fixed.** The
  original claim that 9 named `lib.rs`/`kernel.rs` files had "real
  library-code unwraps" was checked and found wrong for 8 of them (all
  test-only); the 2 real unwraps that did exist (`crates/kernel/src/kernel.rs`,
  both `SystemTime::now().duration_since(UNIX_EPOCH).unwrap()`) are fixed,
  along with 4 more of the same pattern found in
  `crates/objectmodel/src/capabilities.rs` and `lifecycle.rs` while
  investigating. See item 4 above for full detail, including the fail-open
  bug that a naive `unwrap_or_default()` swap would have introduced in
  `CapabilityGrant::is_valid()` (caught and avoided — fixed with
  `.unwrap_or(false)` instead, which fails secure).
- **Items 3 and 5 (unsafe `# Safety` docs, no `cargo audit`/fuzzing):
  left untouched**, per this pass's own conservatism rule — item 3 is
  `crates/memory`'s raw-pointer allocator code (explicitly in the
  kernel-safety-conservative category this pass was told to avoid unless
  a fix is truly trivial; writing correct `# Safety` invariants for
  pointer arithmetic is not), and item 5 needs network access this
  environment doesn't have to validate.
- **Tests**: `cargo test --workspace` — 769 passed, 0 failed (up from 768;
  1 new regression test, `kernel::tests::uptime_does_not_panic_when_boot_time_is_in_the_future`,
  added and confirmed to fail against the pre-fix code and pass against
  the fix). `cargo clippy --workspace -- -D warnings` and
  `cargo clippy --workspace --all-targets` both clean. `cargo fmt --check`
  clean.
- **Not touched**: no public API signatures changed (`SherKernel::uptime()`
  keeps returning bare `u64`, not `Result<u64>`, specifically so sibling
  repos consuming this crate as a path dependency don't need to change);
  no changes to `crates/memory`, `crates/scheduler`, or `crates/interrupt`
  (the explicitly flagged safety-critical crates) beyond what clippy's
  mechanical `--fix` touched in `crates/memory/benches/allocator_bench.rs`
  (a single `black_box`/`unit_arg` reshuffle in bench-only code, verified
  to still compile and run correctly).
