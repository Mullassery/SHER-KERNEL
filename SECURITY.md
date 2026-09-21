# Security Policy

## Scope and expectations, honestly stated

SHER Kernel is a userspace research/prototype project (see
[README.md](README.md)): a Rust workspace of 40 crates simulating
kernel-shaped subsystems in an ordinary process. It is **not** a bootable
kernel, does not run with elevated privileges, and does not talk to real
hardware. There is:

- **No dedicated security team.** This is a single-maintainer project
  (Georgi Mammen Mullassery). There is no security response SLA, no CVE
  program, and no compliance certification (SOC2/ISO/etc.) — any doc or
  third party claiming otherwise about this repo is wrong.
- **No fuzzing today.** `hardening`/`lki`'s syscall-parameter validation is
  unit-tested against known-good/known-bad inputs but has never been
  fuzzed against adversarial input. See [ROADMAP_HONEST.md](ROADMAP_HONEST.md).
- **No formal threat model document.** The capability/sandbox model in
  `crates/security`, `crates/security_audit`, and `crates/driver_runtime`
  is real, tested, in-process logic (time-bounded capability grants,
  audit logging, sandbox policy checks) — but it is *object-model*
  isolation, not OS-level process/namespace isolation. See README's
  "Known gaps" section for the precise distinction.

Given that, "security" issues in this repo fall into two categories:

## 1. Bugs in the simulated capability/sandbox/audit logic

If you find a case where `crates/security`, `crates/security_audit`,
`crates/driver_runtime`, or `crates/hardening` fail to enforce what they
claim to enforce (e.g., a capability check that can be bypassed, an audit
log entry that can be lost or forged, a sandbox policy that doesn't
actually block what it says it blocks) — that's a real bug worth reporting
even though nothing here is privileged, because these crates exist
specifically to prototype correct security-relevant state machines.

## 2. Dependency vulnerabilities

This workspace depends on crates from crates.io (tokio, serde, uuid,
crossbeam-queue, etc. — see root `Cargo.toml`). Dependabot
(`.github/dependabot.yml`) is configured to open PRs for outdated/
vulnerable dependencies, but no `cargo audit`/`cargo deny` CI job exists
yet to catch this automatically — that's a tracked gap, not a solved
problem (see [ROADMAP_HONEST.md](ROADMAP_HONEST.md)).

## How to report

- **Preferred**: open a public GitHub issue at
  https://github.com/Mullassery/SHER-KERNEL/issues — given this is a
  non-privileged userspace prototype with no live deployment, there is
  little reason for most findings to need private disclosure.
- **If you believe private disclosure is warranted** (e.g., you're
  demonstrating a technique against the capability model that you don't
  want copy-pasted before a fix lands): email mullassery@gmail.com. There
  is no guaranteed response time — this is maintained by one person,
  unpaid, alongside other projects.

## What to expect

No bug bounty, no guaranteed patch timeline, no coordinated-disclosure
program. You will get an honest acknowledgment and, if the report is
valid, a fix or an honest explanation of why it won't be fixed (e.g.,
because it's already a documented, disclosed simulation boundary rather
than a bug).
