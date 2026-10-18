# Fuzzing the memory allocator

`allocator` is a structured `cargo-fuzz` target that drives
`MasterAllocator`'s public `allocate` / `deallocate` / `refill` /
`maintain` surface from arbitrary bytes, tracking every live pointer so it
is eventually returned. It exists to catch the class of bugs the tiered
slab allocators have historically produced: out-of-bounds writes from the
slab color offset, free-list slots published before they are written, and
in-range-but-never-allocated pointers handed back out on deallocate.

## Running

```bash
cargo install cargo-fuzz

# long run with AddressSanitizer (default)
cargo +nightly fuzz run allocator

# short bounded smoke run
cargo +nightly fuzz run allocator -- -max_total_time=30

# reproduce a specific crash
cargo +nightly fuzz run allocator artifacts/allocator/<crash-file>
```

Miri is not used here (libFuzzer's runtime is unsupported under Miri);
the repository's Miri job exercises the same crate's unit and stress
tests instead — see `.github/workflows/ci.yml`.

`corpus/`, `artifacts/` and the fuzz target's build output are gitignored.
