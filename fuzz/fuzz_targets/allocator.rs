#![no_main]

//! Structured fuzzer for the tiered memory allocator's public API.
//!
//! The bytes are decoded into a script of `allocate` / `deallocate` /
//! `refill` / `maintain` operations against a single `MasterAllocator`,
//! with every live pointer tracked so it is eventually returned. Run it
//! under AddressSanitizer (cargo-fuzz's default) or Miri to catch the
//! out-of-bounds writes, aliased free-list slots and double-frees that the
//! slab tiers have historically been prone to.
//!
//! ```
//! cargo +nightly fuzz run allocator
//! ```

use libfuzzer_sys::fuzz_target;
use sher_memory::MasterAllocator;

fn decode_size(data: &[u8], idx: &mut usize) -> usize {
    let hi = data[*idx] as usize;
    *idx = (*idx + 1).min(data.len());
    let lo = data.get(*idx).copied().unwrap_or(0) as usize;
    *idx = (*idx + 1).min(data.len());
    let raw = (hi << 8) | lo;
    // 1..=70000 spans Tier 0 (8-64B), Tier 1 (65B-64KB) and the
    // out-of-range path (>64KB) that must reject cleanly.
    (raw % 70_000) + 1
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let num_cpus = (data[0] as usize % 4) + 1;
    let mut alloc = match MasterAllocator::new(num_cpus, 1) {
        Ok(a) => a,
        Err(_) => return,
    };
    let _ = alloc.refill_tier0_caches(64);

    let mut live: Vec<(*mut u8, usize)> = Vec::new();
    let mut i = 1usize;

    while i < data.len() {
        let op = data[i] % 5;
        i += 1;

        match op {
            // 0/1: allocate a size drawn from the input.
            0 | 1 => {
                if i >= data.len() {
                    break;
                }
                let size = decode_size(data, &mut i);
                if let Ok(ptr) = alloc.allocate(size) {
                    if !ptr.is_null() {
                        live.push((ptr, size));
                    }
                }
            }

            // 2: return a random live pointer.
            2 => {
                if live.is_empty() || i >= data.len() {
                    i = (i + 1).min(data.len());
                    continue;
                }
                let idx = data[i] as usize % live.len();
                i += 1;
                let (ptr, size) = live.swap_remove(idx);
                unsafe {
                    let _ = alloc.deallocate(ptr, size);
                }
            }

            // 3: maintenance (compact / shrink Tier 1). Only safe to call
            // with no outstanding pointers, since compaction may relocate
            // live objects; the API does not promise pointer stability
            // across maintenance.
            3 => {
                if live.is_empty() {
                    let _ = alloc.maintain();
                }
            }

            // 4: top up the per-CPU caches.
            _ => {
                let _ = alloc.refill_tier0_caches(16);
            }
        }

        // Keep the freed/quarantined size bounded so runs stay fast.
        if live.len() > 4096 {
            let (ptr, size) = live.swap_remove(0);
            unsafe {
                let _ = alloc.deallocate(ptr, size);
            }
        }
    }

    for (ptr, size) in live {
        unsafe {
            let _ = alloc.deallocate(ptr, size);
        }
    }
});
