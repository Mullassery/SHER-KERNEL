//! MMU setup (Stage 0).
//!
//! **Simulation notice**: programming real page tables and enabling
//! PAE/SMEP/SMAP/KPTI requires ring-0 access this userspace crate does not
//! have. What *is* real: `setup()` validates its input (the discovered
//! memory map must be internally consistent — no overlapping regions)
//! before "proceeding", which is the kind of precondition check a real MMU
//! bring-up would also need.

use crate::memory_map::{regions_are_disjoint, MemoryRegion};
use sher_common::{Error, Result};

pub fn setup(regions: &[MemoryRegion]) -> Result<()> {
    if regions.is_empty() {
        return Err(Error::Memory(
            "cannot set up MMU with an empty memory map".to_string(),
        ));
    }
    if !regions_are_disjoint(regions) {
        return Err(Error::Memory(
            "memory map contains overlapping regions".to_string(),
        ));
    }
    // Real page table construction (simulated — see module docs):
    // - Set up 4-level page tables
    // - Enable PAE/SMEP/SMAP/KPTI
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn region(base: u64, size: u64) -> MemoryRegion {
        MemoryRegion {
            base,
            size,
            region_type: "RAM".to_string(),
        }
    }

    #[test]
    fn setup_accepts_disjoint_regions() {
        let regions = vec![region(0, 100), region(100, 100)];
        assert!(setup(&regions).is_ok());
    }

    #[test]
    fn setup_rejects_empty_map() {
        assert!(setup(&[]).is_err());
    }

    #[test]
    fn setup_rejects_overlapping_regions() {
        let regions = vec![region(0, 100), region(50, 100)];
        assert!(setup(&regions).is_err());
    }
}
