GitHub Guide: SHER Kernel

Welcome to the SHER Kernel repository. This guide helps you navigate the
project and understand its structure.

> **Note (2026 documentation-honesty pass):** this file previously
> described SHER Kernel as "a completely new kernel design" that "proves
> you can run existing Linux drivers on a fundamentally different kernel
> architecture," quoted invented performance numbers ("Memory allocation:
> <0.2μs (vs Linux ~0.25μs)"), and pointed at root-level docs
> (`QUICK_START.md`, `PERFORMANCE_METRICS.md`, `ARCHITECTURE.md`) that no
> longer exist at the repo root. None of that was accurate. It's rewritten
> here to match [README.md](../README.md), the current source of truth.

## What is SHER Kernel?

SHER Kernel is a userspace Rust workspace (40 crates) that prototypes what
the internal APIs of a from-scratch OS kernel might look like: a
capability-based object model, a priority scheduler, tiered memory
bookkeeping, a driver lifecycle/registry, crash recovery, and an A/B
transactional updater. It runs as an ordinary process on macOS/Linux — it
is **not** a bootable kernel, has no bootloader or ring-0 code, and has
never been benchmarked against a real Linux kernel. See
[README.md](../README.md) for the full, per-crate real-vs-simulated
breakdown.

## Quick Navigation

- **Start here**: [README.md](../README.md) — what this is, current test
  status, per-crate real-vs-simulated breakdown, known gaps.
- **Architecture**: [CLAUDE.md](../CLAUDE.md) (implementation guide),
  [docs/architecture/README.md](../docs/architecture/README.md) (crate
  dependency graph).
- **Why this exists / non-goals**: [VISION.md](../VISION.md).
- **What's built vs. not, and what's next**: [ROADMAP.md](../ROADMAP.md),
  [ROADMAP_HONEST.md](../ROADMAP_HONEST.md) (technical debt ledger).
- **API surface**: [API_REFERENCE.md](../API_REFERENCE.md) — carries its
  own correction notice; prefer `cargo doc --workspace --no-deps --open`
  for the authoritative current API.
- **Contributing**: [CONTRIBUTING.md](CONTRIBUTING.md).
- **Historical/superseded docs** (kept for history, not current):
  [docs/archive/](../docs/archive/) — 23 older docs from when this project
  described itself as "v1.0.0 Production Ready" and made unvalidated
  Linux-performance-comparison claims; each now carries a correction
  notice.

## Repository Structure

```
SHER-Kernel/
├── README.md                    # Start here: honest project overview
├── VISION.md                    # Why this exists, non-goals
├── ROADMAP.md                   # What's built, what isn't, near-term plan
├── ROADMAP_HONEST.md            # Technical debt ledger / verification snapshot
├── CLAUDE.md                    # Architecture & implementation guide
├── API_REFERENCE.md             # Per-crate API index (see its correction notice)
├── CHANGELOG.md                 # Keep a Changelog-style history
├── SECURITY.md                  # How to report a vulnerability (no formal SLA)
├── LICENSE                      # Apache License 2.0
├── .github/
│   ├── CONTRIBUTING.md          # How to contribute
│   ├── CODE_OF_CONDUCT.md       # Community standards
│   ├── GITHUB_GUIDE.md          # This file
│   ├── ISSUE_TEMPLATE/          # Bug report / feature request forms
│   ├── pull_request_template.md
│   ├── dependabot.yml           # Automated dependency update PRs
│   └── workflows/ci.yml         # fmt / build / test / clippy
├── docs/
│   ├── architecture/README.md   # Crate dependency graph (Mermaid)
│   └── archive/                 # Superseded docs, kept for history
└── crates/                      # 40 crates — see README's "Project Organization"
```

## Test Coverage

Don't trust a number written in prose in any doc, including this one —
they go stale. Run `cargo test --workspace` yourself, or read README.md's
"Status" section, which is re-verified each documentation pass.

## Development

```bash
cargo build --workspace              # build everything
cargo test --workspace               # run the full test suite
cargo fmt --check                    # formatting
cargo clippy --workspace -- -D warnings   # lint (the bar CI enforces)
cargo doc --workspace --no-deps --open    # browse per-crate API + simulation-boundary docs
```

## Project Status

See README.md's "Status" section for the current, re-verified test count,
clippy/fmt state, and per-crate real-vs-simulated table. This file
intentionally does not restate those numbers to avoid going stale again.

## Important Links

- **GitHub**: https://github.com/Mullassery/SHER-KERNEL
- **Author Email**: mullassery@gmail.com
- **License**: [Apache License 2.0](../LICENSE)

## Community

- Code of Conduct: see [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
- Contributing Guidelines: see [CONTRIBUTING.md](CONTRIBUTING.md)
- Security issues: see [SECURITY.md](../SECURITY.md)
- Discussion: use GitHub issues
