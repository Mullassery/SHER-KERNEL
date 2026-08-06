// SHER LKI: Memory Allocation Translation
// Maps Linux kmalloc/vmalloc/kfree to SHER memory primitives

use sher_common::{ObjectId, Result, Error};
use crate::validation::Validator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// ALLOCATION MODES
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllocationMode {
    Kmalloc,      // Kernel memory, must be < page size
    Vmalloc,      // Virtual memory, can be > page size
    DmaAlloc,     // DMA-safe memory for device I/O
    Kcalloc,      // Kmalloc + zeroed
    Kzalloc,      // Kmalloc + zeroed (same as kcalloc)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GfpFlag {
    GfpKernel,      // Can sleep, normal allocation
    GfpAtomic,      // Cannot sleep, must use existing pool
    GfpNowarn,      // Don't warn on failure
    GfpHighuser,    // Prefer high memory
}

// ============================================================================
// MEMORY ALLOCATION TRACKING
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAllocation {
    pub allocation_id: ObjectId,
    pub driver_id: ObjectId,
    pub address: u64,
    pub size: u64,
    pub mode: AllocationMode,
    pub gfp_flags: u32,
    pub alignment: u32,
    pub is_zeroed: bool,
    pub allocation_time_ms: u64,
    pub freed: bool,
    pub freed_time_ms: Option<u64>,
}

impl MemoryAllocation {
    pub fn new(driver_id: ObjectId, address: u64, size: u64, mode: AllocationMode) -> Self {
        MemoryAllocation {
            allocation_id: ObjectId::new(),
            driver_id,
            address,
            size,
            mode,
            gfp_flags: 0,
            alignment: 0,
            is_zeroed: false,
            allocation_time_ms: 0,
            freed: false,
            freed_time_ms: None,
        }
    }

    pub fn with_flags(mut self, flags: u32) -> Self {
        self.gfp_flags = flags;
        self
    }

    pub fn with_alignment(mut self, alignment: u32) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn with_zeroed(mut self, zeroed: bool) -> Self {
        self.is_zeroed = zeroed;
        self
    }

    pub fn lifetime_ms(&self) -> Option<u64> {
        if let Some(freed_time) = self.freed_time_ms {
            Some(freed_time.saturating_sub(self.allocation_time_ms))
        } else {
            None
        }
    }
}

// ============================================================================
// LINUX MEMORY ALLOCATOR
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct LinuxMemoryAllocator {
    pub validator: Validator,
    pub allocations: HashMap<u64, MemoryAllocation>,  // address -> allocation
    pub total_allocated: u64,
    pub peak_allocated: u64,
    pub allocation_count: u64,
    pub failed_allocations: u64,
}

impl LinuxMemoryAllocator {
    pub fn new() -> Self {
        LinuxMemoryAllocator {
            validator: Validator::new(),
            allocations: HashMap::new(),
            total_allocated: 0,
            peak_allocated: 0,
            allocation_count: 0,
            failed_allocations: 0,
        }
    }

    /// Translate kmalloc(size, flags) to SHER allocation
    pub fn kmalloc(&mut self, driver_id: ObjectId, size: u64, gfp_flags: u32) -> Result<u64> {
        // Validate size
        self.validator.validate_allocation(size, 0)?;

        if size > 128 * 1024 {  // kmalloc limited to ~128KB
            self.failed_allocations += 1;
            return Err(Error::AllocationFailed("kmalloc size too large".to_string()));
        }

        // Simulate address allocation
        let address = 0x1000_0000u64 + (self.allocation_count * 0x1000);

        // Create allocation record
        let allocation = MemoryAllocation::new(driver_id, address, size, AllocationMode::Kmalloc)
            .with_flags(gfp_flags);

        self.allocations.insert(address, allocation);
        self.total_allocated += size;
        self.peak_allocated = self.peak_allocated.max(self.total_allocated);
        self.allocation_count += 1;

        Ok(address)
    }

    /// Translate kzalloc(size, flags) to zeroed SHER allocation
    pub fn kzalloc(&mut self, driver_id: ObjectId, size: u64, gfp_flags: u32) -> Result<u64> {
        // kzalloc = kmalloc + memset to zero
        let address = self.kmalloc(driver_id, size, gfp_flags)?;

        if let Some(alloc) = self.allocations.get_mut(&address) {
            alloc.is_zeroed = true;
        }

        Ok(address)
    }

    /// Translate vmalloc(size) to virtual allocation
    pub fn vmalloc(&mut self, driver_id: ObjectId, size: u64) -> Result<u64> {
        self.validator.validate_allocation(size, 0)?;

        let address = 0x2000_0000u64 + (self.allocation_count * 0x1000);

        let allocation = MemoryAllocation::new(driver_id, address, size, AllocationMode::Vmalloc);
        self.allocations.insert(address, allocation);
        self.total_allocated += size;
        self.peak_allocated = self.peak_allocated.max(self.total_allocated);
        self.allocation_count += 1;

        Ok(address)
    }

    /// Translate dma_alloc_coherent() to DMA-safe allocation
    pub fn dma_alloc(&mut self, driver_id: ObjectId, size: u64, alignment: u32) -> Result<u64> {
        self.validator.validate_allocation(size, alignment)?;

        let address = 0x3000_0000u64 + (self.allocation_count * 0x1000);

        let allocation = MemoryAllocation::new(driver_id, address, size, AllocationMode::DmaAlloc)
            .with_alignment(alignment);

        self.allocations.insert(address, allocation);
        self.total_allocated += size;
        self.peak_allocated = self.peak_allocated.max(self.total_allocated);
        self.allocation_count += 1;

        Ok(address)
    }

    /// Translate kfree(ptr) to deallocation
    pub fn kfree(&mut self, address: u64) -> Result<()> {
        self.validator.validate_deallocation(address)?;

        if let Some(alloc) = self.allocations.get_mut(&address) {
            if alloc.freed {
                return Err(Error::AllocationFailed("Double free detected".to_string()));
            }

            alloc.freed = true;
            alloc.freed_time_ms = Some(0);  // Would be current time
            self.total_allocated = self.total_allocated.saturating_sub(alloc.size);
            Ok(())
        } else {
            self.failed_allocations += 1;
            Err(Error::AllocationFailed("Invalid pointer for kfree".to_string()))
        }
    }

    /// Get allocation information
    pub fn get_allocation(&self, address: u64) -> Option<&MemoryAllocation> {
        self.allocations.get(&address)
    }

    /// Get current memory usage
    pub fn current_usage(&self) -> u64 {
        self.total_allocated
    }

    /// Get peak memory usage
    pub fn peak_usage(&self) -> u64 {
        self.peak_allocated
    }

    /// Get active allocation count
    pub fn active_allocations(&self) -> usize {
        self.allocations.iter().filter(|(_, a)| !a.freed).count()
    }

    /// Detect memory leaks (allocations never freed)
    pub fn find_leaks(&self) -> Vec<&MemoryAllocation> {
        self.allocations
            .values()
            .filter(|a| !a.freed)
            .collect()
    }

    /// Get allocation statistics
    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            total_allocated: self.total_allocated,
            peak_allocated: self.peak_allocated,
            active_allocations: self.active_allocations() as u64,
            total_allocations: self.allocation_count,
            failed_allocations: self.failed_allocations,
            validation_success_rate: self.validator.success_rate(),
        }
    }
}

// ============================================================================
// MEMORY STATISTICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_allocated: u64,
    pub peak_allocated: u64,
    pub active_allocations: u64,
    pub total_allocations: u64,
    pub failed_allocations: u64,
    pub validation_success_rate: f64,
}
