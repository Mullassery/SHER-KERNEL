# Contributing to SHER Kernel

SHER Kernel is a research-grade kernel project demonstrating a complete architectural reimagining of operating systems for the AI era. We welcome contributions from developers, researchers, and systems engineers interested in kernel architecture and AI-native computing.

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
2. Understand the module you're contributing to
3. Run the full test suite locally: `cargo test --lib`
4. Ensure code compiles without warnings: `cargo check`

## Contribution Process

### For Bug Reports
1. Verify the bug with the latest code: `cargo test --lib`
2. Include the failing test case or reproduction steps
3. Describe the expected vs. actual behavior
4. Note the phase/module where the bug appears

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

1. **Pass all tests**: `cargo test --lib` with 100% pass rate
2. **Compile without warnings**: `cargo check` must be clean
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

All contributions must maintain or improve test coverage:

- Memory Management: 50+ tests
- Device Manager: 65+ tests
- Driver Runtime: 81 tests
- LKI/Translation: 72 tests
- Security: 24+ tests

Total target: 292+ tests with 100% passing rate.

Run tests with options:
```bash
# All tests
cargo test --lib

# Specific subsystem
cargo test --lib --package sher_driver_runtime

# With logging output
RUST_LOG=debug cargo test --lib -- --nocapture

# Single-threaded for debugging
cargo test --lib -- --test-threads=1
```

## Performance Considerations

When optimizing, maintain these targets:

- Boot time: < 2 seconds to interactive shell
- Interrupt latency: < 100 microseconds
- Allocation fast path: < 50 nanoseconds
- Driver isolation overhead: < 5% performance impact

Profile before optimizing:
- Use `perf` for CPU profiling
- Track latency with built-in instrumentation
- Benchmark allocation performance separately

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

Contributors will be recognized in:
- Commit history
- Release notes
- CONTRIBUTORS.md file
- GitHub contributor graph

SHER Kernel is built by the community of developers passionate about systems architecture and AI-native computing. Thank you for contributing to the future of operating systems.
