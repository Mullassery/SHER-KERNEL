//! Memory map discovery (Stage 0).
//!
//! **Simulation notice**: a real Stage 0 reads the firmware-provided memory
//! map (BIOS e820 / UEFI GetMemoryMap), which only exists before an OS has
//! handed control to userspace — there is nothing for a userspace process
//! to read here. `discover()` returns a fixed, illustrative two-region
//! layout (BIOS reserved + RAM) instead of pretending to detect real
//! hardware. For an *actually* real host-memory figure, see
//! `sher_aro::profiler::detect_memory_tier`, which queries `/proc/meminfo`
//! / `sysctl` at the userspace level that is available to this process.

use sher_common::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryRegion {
    pub base: u64,
    pub size: u64,
    pub region_type: String,
}

impl MemoryRegion {
    pub fn end(&self) -> u64 {
        self.base + self.size
    }
}

pub fn discover() -> Result<Vec<MemoryRegion>> {
    Ok(vec![
        MemoryRegion {
            base: 0x0,
            size: 0x100000,
            region_type: "BIOS".to_string(),
        },
        MemoryRegion {
            base: 0x100000,
            size: 8 * 1024 * 1024 * 1024, // 8GB, illustrative fixed layout
            region_type: "RAM".to_string(),
        },
    ])
}

/// Validate that a discovered map has no overlapping regions — the
/// invariant any consumer (e.g. `mmu::setup`) depends on.
pub fn regions_are_disjoint(regions: &[MemoryRegion]) -> bool {
    let mut sorted: Vec<&MemoryRegion> = regions.iter().collect();
    sorted.sort_by_key(|r| r.base);
    sorted.windows(2).all(|pair| pair[0].end() <= pair[1].base)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_returns_disjoint_regions() {
        let regions = discover().unwrap();
        assert!(!regions.is_empty());
        assert!(regions_are_disjoint(&regions));
    }

    #[test]
    fn discover_includes_bios_and_ram() {
        let regions = discover().unwrap();
        assert!(regions.iter().any(|r| r.region_type == "BIOS"));
        assert!(regions.iter().any(|r| r.region_type == "RAM"));
    }

    #[test]
    fn overlapping_regions_detected_as_not_disjoint() {
        let overlapping = vec![
            MemoryRegion {
                base: 0,
                size: 100,
                region_type: "A".into(),
            },
            MemoryRegion {
                base: 50,
                size: 100,
                region_type: "B".into(),
            },
        ];
        assert!(!regions_are_disjoint(&overlapping));
    }
}
