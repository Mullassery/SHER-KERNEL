# Contributing to SHER Kernel

SHER Kernel is a userspace Rust workspace prototyping OS-kernel object-model, scheduling, memory, and driver-lifecycle concepts — see [README.md](../README.md) for what it actually is (not a bootable kernel). We welcome contributions from developers interested in that kind of systems-architecture prototyping in Rust.

## Code of Conduct

- Be respectful and professional in all interactions
- Focus on the quality and correctness of the work
- Provide constructive feedback
- Maintain the high architectural standards of the project

## Development Philosophy

SHER Kernel is built on core principles that guide all contributions:

### 1. Security by Conviction
- Capability-based permissions from first principles
- Zero-trust architecture (verify everything)
- Time-bounded grants with automatic expiration
- Complete audit trail of all security-relevant operations
- No component has unrestricted access

### 2. Modular Design
- Every subsystem must be independently testable
- Clear interfaces between components
- No tight coupling or hidden dependencies
- Easy to replace or evolve components
- Pluggable implementations where appropriate

### 3. Safe Systems Programming
- No unsafe code without explicit review and documentation
- Prefer safe Rust abstractions over unsafe optimizations
- Performance comes second to correctness
- All unsafe code must have clear reasoning in comments

### 4. Comprehensive Testing
- Every feature must have accompanying tests
- All tests must pass before submission
- Target 100% test passing rate (never decrease test count)
- Tests serve as executable documentation

### 5. Clear Documentation
- Document the WHY, not the WHAT (code speaks for itself)
- Add comments only when behavior would surprise readers
- Keep functions small and focused (ideal: < 50 lines)
- Link related architecture documents

## Before Contributing

1. Read CLAUDE.md for architectural guidelines
2. Understand the module (crate) you're contributing to
3. Run the full test suite locally: `cargo test --workspace`
4. Ensure code compiles without warnings: `cargo clippy --workspace -- -D warnings`

## Contribution Process

### For Bug Reports
1. Verify the bug with the latest code: `cargo test --workspace`
2. Include the failing test case or reproduction steps
3. Describe the expected vs. actual behavior
4. Note the crate where the bug appears

### For New Features
1. Discuss the feature in an issue first
2. Ensure it aligns with architectural constraints (see CLAUDE.md)
3. Implement with full test coverage
4. Submit with clear description of what/why/how

### For Documentation
1. Fix documentation errors promptly
2. Add architecture docs for major features
3. Keep inline code comments minimal and purposeful
4. Update CLAUDE.md if architectural changes are made

## Pull Request Guidelines

Every pull request must:

1. **Pass all tests**: `cargo test --workspace` with 100% pass rate
2. **Compile without warnings**: `cargo clippy --workspace -- -D warnings` must be clean (this is what CI enforces; `cargo clippy --workspace --all-targets` currently has pre-existing warnings in test/bench code — see ROADMAP_HONEST.md — don't let a PR add new ones)
3. **Follow naming conventions**:
   - PascalCase for types and modules
   - snake_case for functions and variables
   - SCREAMING_SNAKE_CASE for constants
   - Descriptive names over abbreviations

4. **Include test coverage**:
   - New functionality requires new tests
   - Test both success and failure paths
   - Use meaningful assertion messages

5. **Document complex logic**:
   - Only comment the WHY, not the WHAT
   - Reference related architectural docs
   - Explain non-obvious design decisions
   - Point out workarounds for known kernel bugs

6. **Maintain architecture**:
   - Don't introduce circular dependencies
   - Keep subsystems modular
   - Preserve capability-based security model
   - No hidden side effects or state

### Example PR Structure

```
Title: [Subsystem] Brief description of change

## What
- One-sentence summary of changes
- List key modifications

## Why
- Architectural motivation
- Problem being solved
- Reference to issue or design doc

## How
- High-level approach
- Key implementation details
- Performance implications if any

## Testing
- New tests added: X
- Test coverage: Y%
- All tests passing: Yes

## Checklist
- [ ] cargo test --lib passes
- [ ] cargo check is clean
- [ ] No unsafe code added
- [ ] Documentation updated
- [ ] Architecture constraints maintained
```

## Code Review Expectations

Your PR will be reviewed for:

1. **Correctness**: Does it solve the stated problem?
2. **Architecture**: Does it follow SHER principles?
3. **Safety**: Is it safe and auditable?
4. **Performance**: Is it efficient without sacrificing correctness?
5. **Testing**: Are all cases covered?
6. **Documentation**: Is it understandable to future readers?

Reviewers may request changes to:
- Align with architectural principles
- Improve test coverage
- Clarify documentation
- Simplify complex logic
- Enhance safety or security

## Development Workflow

```
1. Fork repository
2. Create feature branch: git checkout -b feature/description
3. Make changes following guidelines
4. Run full test suite: cargo test --lib
5. Commit with clear message: [Module] Description of change
6. Push branch and create PR
7. Address review feedback
8. Merge when approved
```

## Testing Standards

All contributions must maintain or improve test coverage. Don't trust a
fixed per-crate test count in this doc — they go stale (this section
previously listed specific counts, e.g. "Driver Runtime: 81 tests," that
were no longer accurate). Run `cargo test --workspace` and check
README.md's "Status" section for the current, re-verified total.

Run tests with options:
```bash
# All tests
cargo test --workspace

# Specific crate
cargo test --workspace --package sher_driver_runtime

# With logging output
RUST_LOG=debug cargo test --workspace -- --nocapture

# Single-threaded for debugging
cargo test --workspace -- --test-threads=1
```

## Performance Considerations

This repo does not boot, has no interrupt controller, and has never been
benchmarked against a real Linux kernel — so there is no honest "boot
time" or "interrupt latency" target to hold code to (an earlier version of
this section listed such targets; they described a bootable kernel this
repo isn't). If you're optimizing `crates/memory`'s allocators or
`crates/benchmarks`/`crates/performance_benchmarks`, the actual bar is:
don't regress the existing Criterion benchmarks (`cargo bench --workspace`)
without calling that out in the PR description, and profile with `perf`/
`cargo flamegraph` before micro-optimizing rather than guessing.

## Commit Message Guidelines

Follow this format:

```
[Subsystem] Brief description of change

Longer explanation of why this change was necessary.
- Include key technical details
- Reference relevant architectural constraints
- Note any performance implications

Affects: X tests (all passing)
```

Examples:
- `[Memory] Add NUMA-aware allocation for socket locality`
- `[LKI] Implement kmalloc translation with validation`
- `[Security] Add capability expiration enforcement`

## Questions?

- Architecture questions: Review CLAUDE.md and related design docs
- Implementation questions: Check existing test code for examples
- Design decisions: Look at the git history and commit messages
- General guidance: Open an issue for discussion

## Recognition

Contributors are recognized via commit history, CHANGELOG.md entries, and the GitHub contributor graph. (There is no separate `CONTRIBUTORS.md` file at this time.)

Thank you for contributing.
