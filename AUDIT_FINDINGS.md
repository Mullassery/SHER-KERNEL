# Audit findings — 2026-10-19

This file records the repository-specific findings from the portfolio audit
conducted on 2026-10-19. It is documentation only; no code or release artifacts
were changed in this pass unless explicitly noted under "Implementation status".

Sources: the repository's own README/source/workflows, GitHub metadata and
Actions runs, PyPI metadata, and official competitor documentation. Claims are
marked as verified, inferred, or open questions. "Absent" means not found in the
inspected repository/docs, not proof that it cannot exist.

## What it is

Userspace Rust prototype of OS-kernel object-model, scheduling, memory, driver lifecycle, updates, IPC, AI, and Linux-name compatibility.

## Verified strengths

- 40 crates, ~768 tests; capability-based time-bounded security; scheduler; tiered memory; driver lifecycle/recovery; A/B updates; lock-free IPC; AI subsystem; LKI name translation; honest simulation boundaries.
- Substantial, well-tested systems design work.

## Verified gaps and user-facing risks

- Not bootable; no ring-0/bare-metal/MMU/interrupt-controller code; no real cross-process IPC; driver isolation is object-model only.
- CI red (miri/clippy/fuzz); no hardware path.

## Direct competitors

- Linux, Redox, Fuchsia, seL4.

## Ranked next actions

- P3: freeze scope; publish one status/design site; stop adding crates.
- P3: fix CI (miri/clippy/fuzz) if the repo is kept public.

## Do not build

- Bootable kernel/Linux parity; not achievable or useful for the target audience.

## Evidence gaps / validation needed

- No hardware validation; no independent security review.

## Recommended disposition

Freeze/maintain as a research artifact; do not present it as an OS.

## Implementation status

No code changes in this pass.
