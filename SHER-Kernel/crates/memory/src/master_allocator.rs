// SHER Kernel: Master Memory Allocator
// Routes allocations to appropriate tier based on size and architecture
// Supports Tier 0 (8-64B per-CPU) and Tier 1 (65B-64KB per-socket)
//
// Phase 1 Week 2 Implementation (Day 5)
// Status: Master Allocator Integration

use crate::tier0_slab::Tier0Allocator;
use crate::tier1_slab::Tier1Allocator;
use sher_common::{Result, Error};

// ============================================================================
// MASTER ALLOCATOR
// ============================================================================

/// Central memory allocator combining Tier 0 and Tier 1
pub struct MasterAllocator {
    /// Tier 0: Per-CPU fast path (8-64B)
    tier0: Tier0Allocator,

    /// Tier 1: Per-socket medium path (65B-64KB)
    tier1: Tier1Allocator,

    /// Number of CPUs
    num_cpus: usize,

    /// Number of NUMA sockets
    num_sockets: usize,

    /// Statistics for monitoring
    stats: AllocatorStats,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AllocatorStats {
    pub tier0_allocations: usize,
    pub tier1_allocations: usize,
    pub tier0_deallocations: usize,
    pub tier1_deallocations: usize,
    pub allocation_failures: usize,
}

impl MasterAllocator {
    /// Initialize master allocator with given CPU and socket configuration
    pub fn new(num_cpus: usize, num_sockets: usize) -> Result<Self> {
        Ok(MasterAllocator {
            tier0: Tier0Allocator::new(num_cpus)?,
            tier1: Tier1Allocator::new(num_sockets)?,
            num_cpus,
            num_sockets,
            stats: AllocatorStats::default(),
        })
    }

    /// Allocate memory with optimal tier selection
    /// Routing logic:
    /// - 8-64B → Tier 0 (per-CPU, <50ns)
    /// - 65B-64KB → Tier 1 (per-socket, <100ns)
    /// - >64KB → Return error (would need Tier 2 buddy allocator)
    pub fn allocate(&mut self, size: usize) -> Result<*mut u8> {
        match size {
            // Tier 0: Per-CPU fast path
            8..=64 => {
                match self.tier0.allocate(size) {
                    Some(ptr) => {
                        self.stats.tier0_allocations += 1;
                        Ok(ptr)
                    }
                    None => {
                        self.stats.allocation_failures += 1;
                        Err(Error::OutOfMemory)
                    }
                }
            }

            // Tier 1: Per-socket medium path
            65..=65536 => {
                let socket_id = get_current_socket();
                self.tier1.allocate(size, socket_id).map(|ptr| {
                    self.stats.tier1_allocations += 1;
                    ptr
                })
            }

            // Out of range for Tier 0/1 (would need Tier 2)
            _ => {
                self.stats.allocation_failures += 1;
                Err(Error::AllocationFailed(
                    format!("Allocation size {} out of range for Tier 0/1", size),
                ))
            }
        }
    }

    /// Deallocate memory (route to appropriate tier)
    pub fn deallocate(&mut self, ptr: *mut u8, size: usize) -> bool {
        match size {
            // Tier 0
            8..=64 => {
                if self.tier0.deallocate(ptr, size) {
                    self.stats.tier0_deallocations += 1;
                    true
                } else {
                    false
                }
            }

            // Tier 1
            65..=65536 => {
                let socket_id = get_current_socket();
                if self.tier1.deallocate(ptr, size, socket_id) {
                    self.stats.tier1_deallocations += 1;
                    true
                } else {
                    false
                }
            }

            _ => false,
        }
    }

    /// Perform periodic maintenance (compact, shrink)
    pub fn maintain(&mut self) -> Result<()> {
        // Compact Tier 1 slabs (move full slabs)
        self.tier1.compact_all()?;

        // Shrink Tier 1 (destroy empty slabs)
        self.tier1.shrink_all()?;

        Ok(())
    }

    /// Get current statistics
    pub fn stats(&self) -> AllocatorStats {
        self.stats
    }

    /// Get number of CPUs
    pub fn cpu_count(&self) -> usize {
        self.num_cpus
    }

    /// Get number of sockets
    pub fn socket_count(&self) -> usize {
        self.num_sockets
    }

    /// Calculate allocation efficiency
    pub fn efficiency_percent(&self) -> f64 {
        let total = self.stats.tier0_allocations
            + self.stats.tier1_allocations
            + self.stats.allocation_failures;

        if total == 0 {
            return 100.0;
        }

        let successful = self.stats.tier0_allocations + self.stats.tier1_allocations;
        (successful as f64 / total as f64) * 100.0
    }
}

// ============================================================================
// CPU/SOCKET CONTEXT
// ============================================================================

/// Get current socket ID (simplified stub)
/// In production, this would query the kernel's CPU topology
fn get_current_socket() -> usize {
    // Stub: always return socket 0
    // In production: would integrate with kernel scheduler
    0
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_allocator_new() {
        let allocator = MasterAllocator::new(4, 2).expect("Failed to create allocator");
        assert_eq!(allocator.cpu_count(), 4);
        assert_eq!(allocator.socket_count(), 2);
    }

    #[test]
    fn test_master_allocator_tier0_allocation() {
        let mut allocator = MasterAllocator::new(1, 1).expect("Failed to create allocator");

        // Need to refill Tier 0 first
        allocator.tier0.refill_all_caches(10).expect("Failed to refill");

        // Allocate from Tier 0
        let ptr = allocator.allocate(32).expect("Allocation failed");
        assert!(!ptr.is_null());
        assert_eq!(allocator.stats.tier0_allocations, 1);
        assert_eq!(allocator.stats.tier1_allocations, 0);
    }

    #[test]
    fn test_master_allocator_tier1_allocation() {
        let mut allocator = MasterAllocator::new(1, 1).expect("Failed to create allocator");

        // Allocate from Tier 1
        let ptr = allocator.allocate(256).expect("Allocation failed");
        assert!(!ptr.is_null());
        assert_eq!(allocator.stats.tier0_allocations, 0);
        assert_eq!(allocator.stats.tier1_allocations, 1);
    }

    #[test]
    fn test_master_allocator_mixed_allocations() {
        let mut allocator = MasterAllocator::new(1, 1).expect("Failed to create allocator");
        allocator.tier0.refill_all_caches(10).expect("Failed to refill");

        // Tier 0 allocation
        let ptr0 = allocator.allocate(32).expect("32B allocation failed");
        assert_eq!(allocator.stats.tier0_allocations, 1);

        // Tier 1 allocation
        let ptr1 = allocator.allocate(256).expect("256B allocation failed");
        assert_eq!(allocator.stats.tier1_allocations, 1);

        // Both should succeed
        assert!(!ptr0.is_null());
        assert!(!ptr1.is_null());
        assert_ne!(ptr0, ptr1);
    }

    #[test]
    fn test_master_allocator_deallocate_tier0() {
        let mut allocator = MasterAllocator::new(1, 1).expect("Failed to create allocator");
        allocator.tier0.refill_all_caches(10).expect("Failed to refill");

        let ptr = allocator.allocate(32).expect("Allocation failed");
        assert!(allocator.deallocate(ptr, 32));
        assert_eq!(allocator.stats.tier0_deallocations, 1);
    }

    #[test]
    fn test_master_allocator_deallocate_tier1() {
        let mut allocator = MasterAllocator::new(1, 1).expect("Failed to create allocator");

        let ptr = allocator.allocate(256).expect("Allocation failed");
        assert!(allocator.deallocate(ptr, 256));
        assert_eq!(allocator.stats.tier1_deallocations, 1);
    }

    #[test]
    fn test_master_allocator_out_of_range() {
        let mut allocator = MasterAllocator::new(1, 1).expect("Failed to create allocator");

        // Too small
        assert!(allocator.allocate(4).is_err());

        // Too large for Tier 1 (would need Tier 2)
        assert!(allocator.allocate(65537).is_err());
    }

    #[test]
    fn test_master_allocator_efficiency() {
        let mut allocator = MasterAllocator::new(1, 1).expect("Failed to create allocator");
        allocator.tier0.refill_all_caches(10).expect("Failed to refill");

        // No allocations yet
        assert_eq!(allocator.efficiency_percent(), 100.0);

        // Successful allocation
        let ptr = allocator.allocate(32).expect("Allocation failed");
        assert_eq!(allocator.efficiency_percent(), 100.0);

        // Test with failed allocation (out of range: 4 is too small)
        // This will increment failure count
        allocator.allocate(4).ok();
        let efficiency = allocator.efficiency_percent();

        // Should now be < 100% due to one failed allocation
        assert!(efficiency < 100.0);
        // With 1 success and 1 failure, should be 50%
        assert!((efficiency - 50.0).abs() < 0.01);

        allocator.deallocate(ptr, 32);
    }

    #[test]
    fn test_master_allocator_maintain() {
        let mut allocator = MasterAllocator::new(1, 1).expect("Failed to create allocator");

        // Perform maintenance
        allocator.maintain().expect("Maintenance failed");
    }

    #[test]
    fn test_master_allocator_cycle() {
        let mut allocator = MasterAllocator::new(1, 1).expect("Failed to create allocator");
        allocator.tier0.refill_all_caches(20).expect("Failed to refill");

        let mut ptrs = Vec::new();

        // Allocate from both tiers
        for i in 0..20 {
            let size = if i % 2 == 0 { 32 } else { 256 };
            if let Ok(ptr) = allocator.allocate(size) {
                ptrs.push((ptr, size));
            }
        }

        // Deallocate all
        for (ptr, size) in ptrs {
            assert!(allocator.deallocate(ptr, size));
        }

        let stats = allocator.stats();
        assert_eq!(stats.tier0_deallocations, stats.tier0_allocations);
        assert_eq!(stats.tier1_deallocations, stats.tier1_allocations);
    }
}
