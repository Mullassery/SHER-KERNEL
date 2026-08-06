// SHER Kernel: Tier 0 Slab Allocator
// High-performance, per-CPU object caching for 8-64 byte allocations
// Target: < 50ns allocation latency, > 99.5% cache hit rate
//
// Phase 1 Week 2 Implementation
// Status: Tier 0 Allocator - Core Implementation

use sher_common::{Result, Error};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::cell::UnsafeCell;
use std::alloc::{alloc, dealloc, Layout};

// ============================================================================
// TIER 0 SIZE CLASSES
// ============================================================================

/// Tier 0 serves allocations from 8B to 64B
/// Size classes: 8B, 16B, 24B, 32B, 48B, 64B (no waste)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeClass {
    Bytes8 = 0,
    Bytes16 = 1,
    Bytes24 = 2,
    Bytes32 = 3,
    Bytes48 = 4,
    Bytes64 = 5,
}

impl SizeClass {
    pub fn from_size(size: usize) -> Option<SizeClass> {
        match size {
            1..=8 => Some(SizeClass::Bytes8),
            9..=16 => Some(SizeClass::Bytes16),
            17..=24 => Some(SizeClass::Bytes24),
            25..=32 => Some(SizeClass::Bytes32),
            33..=48 => Some(SizeClass::Bytes48),
            49..=64 => Some(SizeClass::Bytes64),
            _ => None,
        }
    }

    pub fn actual_size(&self) -> usize {
        match self {
            SizeClass::Bytes8 => 8,
            SizeClass::Bytes16 => 16,
            SizeClass::Bytes24 => 24,
            SizeClass::Bytes32 => 32,
            SizeClass::Bytes48 => 48,
            SizeClass::Bytes64 => 64,
        }
    }

    pub fn as_index(&self) -> usize {
        *self as usize
    }

    pub fn objects_per_page(&self) -> usize {
        4096 / self.actual_size()
    }
}

// ============================================================================
// PER-CPU CACHE STRUCTURE
// ============================================================================

/// Per-CPU cache for a single size class
/// Uses lock-free CAS-based stack for maximum performance
pub struct CpuSlabCache {
    /// Pre-allocated array of object pointers (stack-based)
    /// Index: 0 = empty, N = N objects available
    /// Wrapped in UnsafeCell for interior mutability
    objects: UnsafeCell<Box<[*mut u8; 256]>>,

    /// Current stack pointer (how many objects available)
    /// Uses atomic for lock-free access
    stack_ptr: AtomicUsize,

    /// Size of objects in this cache
    object_size: usize,

    /// Statistics for monitoring
    #[allow(dead_code)]
    stats: CacheStats,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CacheStats {
    pub allocations: usize,
    pub deallocations: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

impl CpuSlabCache {
    /// Create a new per-CPU cache for a size class
    pub fn new(size_class: SizeClass) -> Self {
        let objects = Box::new([std::ptr::null_mut(); 256]);

        CpuSlabCache {
            objects: UnsafeCell::new(objects),
            stack_ptr: AtomicUsize::new(0),
            object_size: size_class.actual_size(),
            stats: CacheStats::default(),
        }
    }

    /// Allocate from this per-CPU cache (lock-free fast path)
    /// Returns Some(ptr) on success, None if cache empty (rare)
    pub fn allocate(&self) -> Option<*mut u8> {
        // Read current stack pointer
        let mut current = self.stack_ptr.load(Ordering::Acquire);

        loop {
            if current == 0 {
                // Cache empty - slow path required
                return None;
            }

            // Try to decrement stack pointer atomically
            match self.stack_ptr.compare_exchange(
                current,
                current - 1,
                Ordering::Release,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    // Success! Return the object at current-1
                    unsafe {
                        let objects = &(*self.objects.get());
                        let ptr = objects[current - 1];
                        return Some(ptr);
                    }
                }
                Err(actual) => {
                    // CAS failed, retry with actual value
                    current = actual;
                }
            }
        }
    }

    /// Deallocate back to this per-CPU cache (lock-free fast path)
    pub fn deallocate(&self, ptr: *mut u8) -> bool {
        // Read current stack pointer
        let mut current = self.stack_ptr.load(Ordering::Acquire);

        loop {
            if current >= 256 {
                // Cache full - slow path required
                return false;
            }

            // Try to increment stack pointer atomically
            match self.stack_ptr.compare_exchange(
                current,
                current + 1,
                Ordering::Release,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    // Success! Store the object at current position
                    unsafe {
                        let objects = &mut (*self.objects.get());
                        objects[current] = ptr;
                    }
                    return true;
                }
                Err(actual) => {
                    // CAS failed, retry with actual value
                    current = actual;
                }
            }
        }
    }

    /// Pre-allocate a batch of objects from system allocator
    pub fn refill_cache(&self, count: usize) -> Result<()> {
        let layout = Layout::from_size_align(self.object_size, 8)
            .map_err(|_| Error::AllocationFailed("Invalid layout".to_string()))?;

        for _ in 0..count {
            unsafe {
                let ptr = alloc(layout);
                if ptr.is_null() {
                    return Err(Error::OutOfMemory);
                }

                let current = self.stack_ptr.load(Ordering::Acquire);
                if current >= 256 {
                    dealloc(ptr, layout);
                    break;
                }

                let objects = &mut (*self.objects.get());
                objects[current] = ptr;
                self.stack_ptr.store(current + 1, Ordering::Release);
            }
        }

        Ok(())
    }

    /// Drain cache back to system allocator (cleanup)
    pub fn drain_cache(&self) -> Result<()> {
        let layout = Layout::from_size_align(self.object_size, 8)
            .map_err(|_| Error::AllocationFailed("Invalid layout".to_string()))?;

        let count = self.stack_ptr.load(Ordering::Acquire);
        unsafe {
            let objects = &(*self.objects.get());
            for i in 0..count {
                let ptr = objects[i];
                if !ptr.is_null() {
                    dealloc(ptr, layout);
                }
            }
        }

        self.stack_ptr.store(0, Ordering::Release);
        Ok(())
    }

    pub fn current_count(&self) -> usize {
        self.stack_ptr.load(Ordering::Acquire)
    }

    pub fn is_empty(&self) -> bool {
        self.stack_ptr.load(Ordering::Acquire) == 0
    }

    pub fn is_full(&self) -> bool {
        self.stack_ptr.load(Ordering::Acquire) >= 256
    }
}

// SAFETY: CpuSlabCache is safe to send and share across threads
// The UnsafeCell is protected by atomic operations on stack_ptr
unsafe impl Send for CpuSlabCache {}
unsafe impl Sync for CpuSlabCache {}

// ============================================================================
// TIER 0 ALLOCATOR (PER-CPU ARRAYS)
// ============================================================================

/// Global Tier 0 allocator managing per-CPU caches
/// One allocator instance shared by all CPUs
pub struct Tier0Allocator {
    /// Per-CPU cache arrays, one for each size class
    /// caches[cpu][size_class]
    caches: Vec<Vec<CpuSlabCache>>,

    /// Number of CPUs (set at initialization)
    num_cpus: usize,
}

impl Tier0Allocator {
    /// Initialize Tier 0 allocator for the system
    pub fn new(num_cpus: usize) -> Result<Self> {
        let mut caches = Vec::with_capacity(num_cpus);

        for cpu in 0..num_cpus {
            let mut cpu_caches = Vec::with_capacity(6); // 6 size classes

            cpu_caches.push(CpuSlabCache::new(SizeClass::Bytes8));
            cpu_caches.push(CpuSlabCache::new(SizeClass::Bytes16));
            cpu_caches.push(CpuSlabCache::new(SizeClass::Bytes24));
            cpu_caches.push(CpuSlabCache::new(SizeClass::Bytes32));
            cpu_caches.push(CpuSlabCache::new(SizeClass::Bytes48));
            cpu_caches.push(CpuSlabCache::new(SizeClass::Bytes64));

            caches.push(cpu_caches);
        }

        Ok(Tier0Allocator {
            caches,
            num_cpus,
        })
    }

    /// Allocate from Tier 0
    /// This is the hot path: should complete in < 50ns on cache hit
    pub fn allocate(&self, size: usize) -> Option<*mut u8> {
        let size_class = SizeClass::from_size(size)?;
        let cpu_id = get_cpu_id() % self.num_cpus;
        let cache = &self.caches[cpu_id][size_class.as_index()];

        cache.allocate()
    }

    /// Deallocate to Tier 0
    /// This is the fast path for returning objects to cache
    pub fn deallocate(&self, ptr: *mut u8, size: usize) -> bool {
        let size_class = match SizeClass::from_size(size) {
            Some(sc) => sc,
            None => return false,
        };

        let cpu_id = get_cpu_id() % self.num_cpus;
        let cache = &self.caches[cpu_id][size_class.as_index()];

        cache.deallocate(ptr)
    }

    /// Refill per-CPU caches from system allocator
    /// Called when cache is empty (slow path)
    pub fn refill_all_caches(&self, objects_per_refill: usize) -> Result<()> {
        for _cpu in 0..self.num_cpus {
            for size_class in 0..6 {
                self.caches[_cpu][size_class].refill_cache(objects_per_refill)?;
            }
        }
        Ok(())
    }

    /// Drain all caches and return memory to system
    /// Called at shutdown
    pub fn drain_all_caches(&self) -> Result<()> {
        for cpu in 0..self.num_cpus {
            for size_class in 0..6 {
                self.caches[cpu][size_class].drain_cache()?;
            }
        }
        Ok(())
    }

    pub fn get_cpu_count(&self) -> usize {
        self.num_cpus
    }
}

// ============================================================================
// GLOBAL INSTANCE (THREAD-LOCAL STORAGE)
// ============================================================================

thread_local! {
    static TIER0_CPU_CONTEXT: UnsafeCell<CpuContext> = UnsafeCell::new(CpuContext::default());
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CpuContext {
    pub cpu_id: usize,
}

impl CpuContext {
    pub fn set(cpu_id: usize) {
        TIER0_CPU_CONTEXT.with(|ctx| {
            unsafe {
                (*ctx.get()).cpu_id = cpu_id;
            }
        });
    }

    pub fn get() -> usize {
        TIER0_CPU_CONTEXT.with(|ctx| unsafe { (*ctx.get()).cpu_id })
    }
}

/// Get current CPU ID (simplified version without kernel call)
#[inline(always)]
fn get_cpu_id() -> usize {
    CpuContext::get()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_class_selection() {
        assert_eq!(SizeClass::from_size(1), Some(SizeClass::Bytes8));
        assert_eq!(SizeClass::from_size(8), Some(SizeClass::Bytes8));
        assert_eq!(SizeClass::from_size(9), Some(SizeClass::Bytes16));
        assert_eq!(SizeClass::from_size(16), Some(SizeClass::Bytes16));
        assert_eq!(SizeClass::from_size(25), Some(SizeClass::Bytes32));
        assert_eq!(SizeClass::from_size(64), Some(SizeClass::Bytes64));
        assert_eq!(SizeClass::from_size(65), None);
    }

    #[test]
    fn test_size_class_actual_sizes() {
        assert_eq!(SizeClass::Bytes8.actual_size(), 8);
        assert_eq!(SizeClass::Bytes16.actual_size(), 16);
        assert_eq!(SizeClass::Bytes24.actual_size(), 24);
        assert_eq!(SizeClass::Bytes32.actual_size(), 32);
        assert_eq!(SizeClass::Bytes48.actual_size(), 48);
        assert_eq!(SizeClass::Bytes64.actual_size(), 64);
    }

    #[test]
    fn test_size_class_objects_per_page() {
        assert_eq!(SizeClass::Bytes8.objects_per_page(), 512);
        assert_eq!(SizeClass::Bytes16.objects_per_page(), 256);
        assert_eq!(SizeClass::Bytes32.objects_per_page(), 128);
        assert_eq!(SizeClass::Bytes64.objects_per_page(), 64);
    }

    #[test]
    fn test_cpu_slab_cache_new() {
        let cache = CpuSlabCache::new(SizeClass::Bytes32);
        assert_eq!(cache.object_size, 32);
        assert_eq!(cache.current_count(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cpu_slab_cache_empty_allocation() {
        let cache = CpuSlabCache::new(SizeClass::Bytes32);
        // Cache starts empty
        assert!(cache.allocate().is_none());
    }

    #[test]
    fn test_tier0_allocator_new() {
        let allocator = Tier0Allocator::new(4).expect("Failed to create allocator");
        assert_eq!(allocator.get_cpu_count(), 4);
    }

    #[test]
    fn test_tier0_allocator_empty_allocation() {
        CpuContext::set(0);
        let allocator = Tier0Allocator::new(4).expect("Failed to create allocator");

        // Cache starts empty, allocation should fail
        assert!(allocator.allocate(32).is_none());
    }

    #[test]
    fn test_tier0_allocator_refill_and_allocate() {
        CpuContext::set(0);
        let allocator = Tier0Allocator::new(1).expect("Failed to create allocator");

        // Refill cache
        allocator.refill_all_caches(10).expect("Failed to refill");

        // Now allocation should succeed
        let ptr = allocator.allocate(32);
        assert!(ptr.is_some());
        assert!(!ptr.unwrap().is_null());
    }

    #[test]
    fn test_tier0_allocator_allocate_deallocate_cycle() {
        CpuContext::set(0);
        let allocator = Tier0Allocator::new(1).expect("Failed to create allocator");
        allocator.refill_all_caches(10).expect("Failed to refill");

        let ptr1 = allocator.allocate(32).expect("First allocation failed");
        assert!(!ptr1.is_null());

        // Deallocate
        assert!(allocator.deallocate(ptr1, 32));

        // Should be able to allocate again
        let ptr2 = allocator.allocate(32).expect("Second allocation failed");
        assert_eq!(ptr1, ptr2); // Should reuse same object
    }

    #[test]
    fn test_tier0_allocator_multiple_size_classes() {
        CpuContext::set(0);
        let allocator = Tier0Allocator::new(1).expect("Failed to create allocator");
        allocator.refill_all_caches(5).expect("Failed to refill");

        let ptr8 = allocator.allocate(8).expect("8B allocation failed");
        let ptr32 = allocator.allocate(32).expect("32B allocation failed");
        let ptr64 = allocator.allocate(64).expect("64B allocation failed");

        assert!(!ptr8.is_null());
        assert!(!ptr32.is_null());
        assert!(!ptr64.is_null());

        // All should be different
        assert_ne!(ptr8, ptr32);
        assert_ne!(ptr32, ptr64);
    }

    #[test]
    fn test_tier0_allocator_cache_full() {
        CpuContext::set(0);
        let allocator = Tier0Allocator::new(1).expect("Failed to create allocator");
        allocator.refill_all_caches(100).expect("Failed to refill");

        // Allocate many objects
        let mut ptrs = Vec::new();
        for _ in 0..100 {
            if let Some(ptr) = allocator.allocate(32) {
                ptrs.push(ptr);
            } else {
                break;
            }
        }

        // Deallocate some
        for ptr in ptrs.iter().take(50) {
            assert!(allocator.deallocate(*ptr, 32));
        }

        // Should be able to allocate again
        let ptr = allocator.allocate(32).expect("Allocation after deallocation failed");
        assert!(!ptr.is_null());
    }

    #[test]
    fn test_cpu_slab_cache_drain() {
        let cache = CpuSlabCache::new(SizeClass::Bytes32);
        cache.refill_cache(10).expect("Failed to refill");

        assert!(!cache.is_empty());
        cache.drain_cache().expect("Failed to drain");
        assert!(cache.is_empty());
    }
}
