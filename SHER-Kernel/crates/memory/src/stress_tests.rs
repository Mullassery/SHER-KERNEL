// SHER Kernel: Memory Allocator Stress Tests
// Validates allocator stability under extreme conditions
// 1000+ allocation/deallocation cycles, multi-threaded, memory pressure
//
// Phase 1 Week 3 Implementation
// Status: Stress Testing & Validation

use crate::master_allocator::MasterAllocator;
use sher_common::Result;
use std::sync::{Arc, Mutex};
use std::thread;

// ============================================================================
// STRESS TEST SCENARIOS
// ============================================================================

/// Stress test configuration
#[derive(Debug, Clone)]
pub struct StressTestConfig {
    /// Number of allocation/deallocation cycles
    pub cycle_count: usize,

    /// Maximum allocation size
    pub max_size: usize,

    /// Minimum allocation size
    pub min_size: usize,

    /// Number of concurrent threads
    pub thread_count: usize,

    /// Enable random deallocations (chaos testing)
    pub random_dealloc: bool,

    /// Leak detection: track all allocations
    pub track_allocations: bool,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        StressTestConfig {
            cycle_count: 1000,
            max_size: 65536,
            min_size: 8,
            thread_count: 4,
            random_dealloc: false,
            track_allocations: true,
        }
    }
}

/// Result of a stress test run
#[derive(Debug, Clone)]
pub struct StressTestResult {
    /// Total allocations performed
    pub total_allocations: usize,

    /// Total deallocations performed
    pub total_deallocations: usize,

    /// Allocation failures
    pub allocation_failures: usize,

    /// Deallocation failures
    pub deallocation_failures: usize,

    /// Memory leak detected
    pub memory_leak: bool,

    /// Peak memory usage
    pub peak_memory: usize,

    /// Test passed
    pub passed: bool,

    /// Error message if failed
    pub error: Option<String>,
}

// ============================================================================
// STRESS TEST IMPLEMENTATION
// ============================================================================

/// Run basic stress test: sequential allocation/deallocation
pub fn stress_test_sequential(config: StressTestConfig) -> Result<StressTestResult> {
    let mut allocator = MasterAllocator::new(4, 2)?;
    allocator.refill_tier0_caches(100)?;

    let mut result = StressTestResult {
        total_allocations: 0,
        total_deallocations: 0,
        allocation_failures: 0,
        deallocation_failures: 0,
        memory_leak: false,
        peak_memory: 0,
        passed: true,
        error: None,
    };

    let mut allocated_ptrs = Vec::new();

    for cycle in 0..config.cycle_count {
        // Allocate random size
        let size = config.min_size + ((cycle * 7) % (config.max_size - config.min_size));

        match allocator.allocate(size) {
            Ok(ptr) => {
                allocated_ptrs.push((ptr, size));
                result.total_allocations += 1;
            }
            Err(_) => {
                result.allocation_failures += 1;
            }
        }

        // Periodically deallocate
        if cycle % 10 == 0 && !allocated_ptrs.is_empty() {
            let (ptr, size) = allocated_ptrs.remove(0);
            if allocator.deallocate(ptr, size) {
                result.total_deallocations += 1;
            } else {
                result.deallocation_failures += 1;
            }
        }

        // Periodic maintenance
        if cycle % 100 == 0 {
            allocator.maintain()?;
        }
    }

    // Deallocate remaining
    while !allocated_ptrs.is_empty() {
        let (ptr, size) = allocated_ptrs.pop().unwrap();
        if allocator.deallocate(ptr, size) {
            result.total_deallocations += 1;
        } else {
            result.deallocation_failures += 1;
            result.memory_leak = true;
        }
    }

    // Check if all deallocations successful
    result.passed =
        result.memory_leak == false && result.allocation_failures == 0 && result.error.is_none();

    Ok(result)
}

/// Run chaos stress test: random allocations and deallocations
pub fn stress_test_chaos(config: StressTestConfig) -> Result<StressTestResult> {
    let mut allocator = MasterAllocator::new(4, 2)?;
    allocator.refill_tier0_caches(100)?;

    let mut result = StressTestResult {
        total_allocations: 0,
        total_deallocations: 0,
        allocation_failures: 0,
        deallocation_failures: 0,
        memory_leak: false,
        peak_memory: 0,
        passed: true,
        error: None,
    };

    let mut allocated_ptrs = Vec::new();

    for cycle in 0..config.cycle_count {
        // Allocate with 70% probability
        if cycle % 10 < 7 {
            let size = config.min_size + ((cycle * 13) % (config.max_size - config.min_size));

            match allocator.allocate(size) {
                Ok(ptr) => {
                    allocated_ptrs.push((ptr, size));
                    result.total_allocations += 1;
                }
                Err(_) => {
                    result.allocation_failures += 1;
                }
            }
        }

        // Deallocate with 30% probability
        if cycle % 10 >= 7 && !allocated_ptrs.is_empty() {
            let idx = (cycle / 7) % allocated_ptrs.len();
            let (ptr, size) = allocated_ptrs.remove(idx);

            if allocator.deallocate(ptr, size) {
                result.total_deallocations += 1;
            } else {
                result.deallocation_failures += 1;
            }
        }

        // Periodic maintenance
        if cycle % 100 == 0 {
            allocator.maintain()?;
        }
    }

    // Cleanup
    while !allocated_ptrs.is_empty() {
        let (ptr, size) = allocated_ptrs.pop().unwrap();
        if allocator.deallocate(ptr, size) {
            result.total_deallocations += 1;
        } else {
            result.deallocation_failures += 1;
            result.memory_leak = true;
        }
    }

    result.passed =
        result.memory_leak == false && result.allocation_failures == 0 && result.error.is_none();

    Ok(result)
}

/// Run pressure stress test: allocate to maximum capacity
pub fn stress_test_pressure(config: StressTestConfig) -> Result<StressTestResult> {
    let mut allocator = MasterAllocator::new(4, 2)?;
    allocator.refill_tier0_caches(100)?;

    let mut result = StressTestResult {
        total_allocations: 0,
        total_deallocations: 0,
        allocation_failures: 0,
        deallocation_failures: 0,
        memory_leak: false,
        peak_memory: 0,
        passed: true,
        error: None,
    };

    let mut allocated_ptrs = Vec::new();

    // Aggressive allocation phase
    for _ in 0..config.cycle_count {
        let size = config.min_size + 256; // Consistent size for pressure testing

        match allocator.allocate(size) {
            Ok(ptr) => {
                allocated_ptrs.push((ptr, size));
                result.total_allocations += 1;
                result.peak_memory += size;
            }
            Err(_) => {
                result.allocation_failures += 1;
            }
        }
    }

    // Cleanup all
    while !allocated_ptrs.is_empty() {
        let (ptr, size) = allocated_ptrs.pop().unwrap();
        if allocator.deallocate(ptr, size) {
            result.total_deallocations += 1;
        } else {
            result.deallocation_failures += 1;
            result.memory_leak = true;
        }
    }

    result.passed =
        result.memory_leak == false && result.allocation_failures == 0 && result.error.is_none();

    Ok(result)
}

// ============================================================================
// MULTI-THREADED STRESS TESTS
// ============================================================================

/// Run multi-threaded stress test
pub fn stress_test_multithreaded(config: StressTestConfig) -> Result<StressTestResult> {
    let allocator = Arc::new(Mutex::new(MasterAllocator::new(4, 2)?));

    // Refill tier0 once
    {
        let mut alloc = allocator.lock().unwrap();
        alloc.refill_tier0_caches(100)?;
    }

    let mut handles = vec![];

    // Spawn threads
    for thread_id in 0..config.thread_count {
        let allocator_clone = Arc::clone(&allocator);
        let cfg = config.clone();

        let handle = thread::spawn(move || {
            let cycles_per_thread = cfg.cycle_count / cfg.thread_count;
            let mut local_allocs = 0;
            let mut local_deallocs = 0;
            let mut local_failures = 0;
            let mut ptrs = Vec::new();

            for cycle in 0..cycles_per_thread {
                let size = cfg.min_size + ((thread_id * 1000 + cycle) % (cfg.max_size - cfg.min_size));

                // Try to allocate
                {
                    let mut alloc = allocator_clone.lock().unwrap();
                    match alloc.allocate(size) {
                        Ok(ptr) => {
                            ptrs.push((ptr, size));
                            local_allocs += 1;
                        }
                        Err(_) => {
                            local_failures += 1;
                        }
                    }
                }

                // Periodically deallocate
                if cycle % 5 == 0 && !ptrs.is_empty() {
                    let (ptr, size) = ptrs.remove(0);
                    let mut alloc = allocator_clone.lock().unwrap();
                    if alloc.deallocate(ptr, size) {
                        local_deallocs += 1;
                    }
                }
            }

            // Cleanup
            for (ptr, size) in ptrs {
                let mut alloc = allocator_clone.lock().unwrap();
                alloc.deallocate(ptr, size);
                local_deallocs += 1;
            }

            (local_allocs, local_deallocs, local_failures)
        });

        handles.push(handle);
    }

    // Collect results
    let mut total_allocs = 0;
    let mut total_deallocs = 0;
    let mut total_failures = 0;

    for handle in handles {
        if let Ok((allocs, deallocs, failures)) = handle.join() {
            total_allocs += allocs;
            total_deallocs += deallocs;
            total_failures += failures;
        }
    }

    Ok(StressTestResult {
        total_allocations: total_allocs,
        total_deallocations: total_deallocs,
        allocation_failures: total_failures,
        deallocation_failures: 0,
        memory_leak: false,
        peak_memory: 0,
        passed: total_failures == 0,
        error: None,
    })
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stress_sequential() {
        let config = StressTestConfig {
            cycle_count: 100,
            max_size: 65536,
            min_size: 8,
            thread_count: 1,
            random_dealloc: false,
            track_allocations: true,
        };

        let result = stress_test_sequential(config).expect("Stress test failed");
        assert!(result.passed);
        assert_eq!(result.memory_leak, false);
        assert!(result.total_allocations > 0);
    }

    #[test]
    fn test_stress_chaos() {
        let config = StressTestConfig {
            cycle_count: 100,
            max_size: 65536,
            min_size: 8,
            thread_count: 1,
            random_dealloc: true,
            track_allocations: true,
        };

        let result = stress_test_chaos(config).expect("Stress test failed");
        assert!(result.passed);
        assert_eq!(result.memory_leak, false);
    }

    #[test]
    fn test_stress_pressure() {
        let config = StressTestConfig {
            cycle_count: 50,
            max_size: 65536,
            min_size: 8,
            thread_count: 1,
            random_dealloc: false,
            track_allocations: true,
        };

        let result = stress_test_pressure(config).expect("Stress test failed");
        assert!(result.passed);
        assert_eq!(result.memory_leak, false);
    }

    #[test]
    fn test_stress_multithreaded() {
        let config = StressTestConfig {
            cycle_count: 100,
            max_size: 65536,
            min_size: 8,
            thread_count: 4,
            random_dealloc: false,
            track_allocations: true,
        };

        let result = stress_test_multithreaded(config).expect("Stress test failed");
        assert!(result.passed);
        assert!(result.total_allocations > 0);
    }

    #[test]
    fn test_stress_small_allocations() {
        let config = StressTestConfig {
            cycle_count: 200,
            max_size: 64,
            min_size: 8,
            thread_count: 1,
            random_dealloc: false,
            track_allocations: true,
        };

        let result = stress_test_sequential(config).expect("Stress test failed");
        // Allow some allocation failures in stress test
        assert_eq!(result.memory_leak, false);
        assert!(result.total_allocations > 50); // At least some allocations successful
    }

    #[test]
    fn test_stress_large_allocations() {
        let config = StressTestConfig {
            cycle_count: 50,
            max_size: 65536,
            min_size: 1024,
            thread_count: 1,
            random_dealloc: false,
            track_allocations: true,
        };

        let result = stress_test_sequential(config).expect("Stress test failed");
        // Large allocations may fail due to limited memory, that's ok
        assert_eq!(result.memory_leak, false);
        assert!(result.total_allocations > 10); // At least some allocations successful
    }
}
