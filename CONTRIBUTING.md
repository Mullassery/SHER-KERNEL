# Contributing to SHER Kernel

Thanks for contributing. SHER Kernel is an experimental userspace prototype of
kernel concepts, not a bootable kernel. Review times vary and there is no merge
SLA.

## Before you start

- Read [`README.md`](README.md) for what is implemented and what is explicitly
  not.
- For memory, scheduling, or driver-runtime changes, open an issue first; these
  areas have invariants documented in code and tests.
- Security issues go through [`SECURITY.md`](SECURITY.md), not public issues.

## Development setup

Requires a current stable Rust toolchain.

```bash
cargo build --workspace
cargo test --workspace
```

Some checks are nightly-only (Miri, fuzzing). See the CI workflow for the exact
invocations.

## Checks before a pull request

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-features
```

CI runs the same checks, plus Miri and fuzz smoke jobs where applicable.

## Pull requests

- Keep changes focused and paste the command and result you ran.
- New invariants need tests.
- Do not describe the prototype as production-ready or as a Linux replacement.

## License

Contributions are accepted under the Apache License 2.0.
