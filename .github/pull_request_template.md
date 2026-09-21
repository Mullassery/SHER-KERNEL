## What

<!-- One-sentence summary. Which crate(s) does this touch? -->

## Why

<!-- What problem does this solve? Link an issue if there is one. -->

## How

<!-- High-level approach and key implementation details. -->

## Testing

- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` is clean
- [ ] `cargo fmt --check` is clean
- [ ] New/changed behavior has test coverage (or an explanation of why not)

## Honesty check

- [ ] If this PR touches what a crate claims to do vs. what it actually
      does (real logic vs. simulated/placeholder), README.md's "What's
      real vs. simulated" table is updated to match.
- [ ] No new `TODO`/placeholder code was left claiming to work when it
      doesn't (see repo's no-fake-stubs standard).
