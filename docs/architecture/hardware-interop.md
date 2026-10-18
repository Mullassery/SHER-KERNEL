# Hardware & platform interoperability matrix

This document states, per host platform and CPU architecture, what SHER-Kernel
actually does — and, just as importantly, what it does **not** do. It exists to
settle the recurring "does it run under KVM / on bare metal / on ARM?" question
with a table rather than an inference.

## Framing: SHER-Kernel is a userspace process

SHER-Kernel runs as an ordinary process on a normal host OS. There is no
bootloader, no ring-0/bare-metal code, and no MMU or interrupt-controller
programming anywhere in the repository. Any capability below that sounds like it
needs a real supervisor (KVM, hypervisor, bare metal) is therefore marked
*Not applicable* rather than claimed — see `README.md` ("Not a Linux
replacement") and `docs/archive/` for the historical claims that this table now
supersedes.

## Matrix

Legend: **Yes** = supported and exercised; **Name-level** = API/syscall *name*
translation only, no ABI or behavioural emulation; **No / N/A** = not supported.

| Capability | Linux x86_64 | Linux aarch64 | macOS arm64 (Apple Silicon) |
| --- | --- | --- | --- |
| Builds + tests (`--all-targets --all-features`) | Yes (CI) | Yes (CI) | Yes (local dev; not in CI) |
| Examples / benches compile | Yes (CI) | Yes (CI) | Yes (local dev) |
| Object model, scheduler, memory, driver-lifecycle concepts | Yes | Yes | Yes |
| Linux Kernel Interface (LKI) syscall-name translation | Name-level | Name-level | Name-level (mapping tables only) |
| POSIX compatibility layer | Name-level | Name-level | Name-level |
| Real KVM / hypervisor vCPU execution | No / N/A | No / N/A | No / N/A |
| Bare-metal / ring-0 execution | No / N/A | No / N/A | No / N/A |
| Real MMU / interrupt-controller programming | No / N/A | No / N/A | No / N/A |
| Binary-compatible Linux syscall ABI | No | No | No |

## ARM64 vs x86_64 (the KVM question)

There is **no KVM integration on either architecture**. The ARM64-vs-x86_64
distinction that matters to SHER-Kernel is only the Rust target triple it is
compiled for (`aarch64` vs `x86_64`); the workspace currently contains no
`#[cfg(target_arch = ...)]` behaviour that diverges between them. Consequently:

- Neither architecture gets real hardware virtualization through SHER-Kernel.
- The CI matrix builds and tests both `x86_64` (GitHub `ubuntu-latest`) and
  `aarch64` (GitHub `ubuntu-24.04-arm`) to keep the pure-Rust code
  architecture-neutral; macOS arm64 is the local-development host.
- "Run SHER under KVM" is not a supported configuration; if/when a hypervisor
  integration is designed it should arrive with its own ADR rather than being
  implied by this scaffold.

## Verifying on your own machine

```bash
uname -m                     # aarch64 / arm64 vs x86_64
rustc -vV | grep host        # the actual target triple
cargo build --workspace --all-targets --all-features
cargo test --workspace --all-features
```
