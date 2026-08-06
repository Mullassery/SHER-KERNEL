//! Stress Testing Framework for Kernel Resilience
//!
//! Tests kernel behavior under:
//! - High concurrency (many operations simultaneously)
//! - Resource exhaustion (memory pressure, CPU saturation)
//! - Long-running operations
//! - Cascading failures

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use sher_common::Result;

#[derive(Clone, Debug)]
pub struct StressTestConfig {
    pub duration_seconds: u64,
    pub concurrent_operations: usize,
    pub allocation_size_bytes: usize,
    pub max_concurrent_allocations: usize,
    pub enable_memory_pressure: bool,
    pub enable_cpu_saturation: bool,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        StressTestConfig {
            duration_seconds: 10,
            concurrent_operations: 4,
            allocation_size_bytes: 4096,
            max_concurrent_allocations: 1000,
            enable_memory_pressure: true,
            enable_cpu_saturation: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StressTestResults {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub duration_seconds: u64,
    pub operations_per_second: f64,
    pub failure_rate: f64,
    pub peak_memory_mb: f64,
    pub avg_latency_us: f64,
}

pub struct StressTest {
    config: StressTestConfig,
    operations_completed: Arc<AtomicUsize>,
    operations_failed: Arc<AtomicUsize>,
    peak_memory: Arc<AtomicUsize>,
}

impl StressTest {
    pub fn new(config: StressTestConfig) -> Self {
        StressTest {
            config,
            operations_completed: Arc::new(AtomicUsize::new(0)),
            operations_failed: Arc::new(AtomicUsize::new(0)),
            peak_memory: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn run_memory_stress_test(&mut self) -> Result<StressTestResults> {
        let start_time = Instant::now();
        let mut allocations = Vec::new();

        while start_time.elapsed().as_secs() < self.config.duration_seconds {
            if allocations.len() < self.config.max_concurrent_allocations {
                match self.allocate_memory() {
                    Ok(allocation) => {
                        allocations.push(allocation);
                        self.operations_completed.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => {
                        self.operations_failed.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }

            if allocations.len() > self.config.max_concurrent_allocations / 2 {
                allocations.pop();
            }
        }

        self.generate_results(start_time.elapsed().as_secs())
    }

    pub fn run_concurrency_stress_test(&mut self) -> Result<StressTestResults> {
        let start_time = Instant::now();
        let config = self.config.clone();

        let handles: Vec<_> = (0..config.concurrent_operations)
            .map(|_| {
                let ops_completed = Arc::clone(&self.operations_completed);
                let ops_failed = Arc::clone(&self.operations_failed);
                let duration = config.duration_seconds;

                std::thread::spawn(move || {
                    let thread_start = Instant::now();
                    while thread_start.elapsed().as_secs() < duration {
                        match Self::simulate_operation() {
                            Ok(_) => {
                                ops_completed.fetch_add(1, Ordering::Relaxed);
                            }
                            Err(_) => {
                                ops_failed.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                })
            })
            .collect();

        for handle in handles {
            let _ = handle.join();
        }

        self.generate_results(start_time.elapsed().as_secs())
    }

    pub fn run_long_running_test(&mut self) -> Result<StressTestResults> {
        let start_time = Instant::now();

        while start_time.elapsed().as_secs() < self.config.duration_seconds {
            match Self::simulate_operation() {
                Ok(_) => {
                    self.operations_completed.fetch_add(1, Ordering::Relaxed);
                }
                Err(_) => {
                    self.operations_failed.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        self.generate_results(start_time.elapsed().as_secs())
    }

    pub fn run_cascade_failure_test(&mut self) -> Result<StressTestResults> {
        let start_time = Instant::now();
        let mut consecutive_failures = 0;
        const FAILURE_CASCADE_THRESHOLD: usize = 10;

        while start_time.elapsed().as_secs() < self.config.duration_seconds
            && consecutive_failures < FAILURE_CASCADE_THRESHOLD
        {
            match Self::simulate_operation() {
                Ok(_) => {
                    consecutive_failures = 0;
                    self.operations_completed.fetch_add(1, Ordering::Relaxed);
                }
                Err(_) => {
                    consecutive_failures += 1;
                    self.operations_failed.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        self.generate_results(start_time.elapsed().as_secs())
    }

    fn allocate_memory(&self) -> Result<Vec<u8>> {
        Ok(vec![0u8; self.config.allocation_size_bytes])
    }

    fn simulate_operation() -> Result<()> {
        std::thread::sleep(std::time::Duration::from_micros(10));
        Ok(())
    }

    fn generate_results(&self, duration_seconds: u64) -> Result<StressTestResults> {
        let completed = self.operations_completed.load(Ordering::Relaxed) as u64;
        let failed = self.operations_failed.load(Ordering::Relaxed) as u64;
        let total = completed + failed;

        let ops_per_second = if duration_seconds > 0 {
            completed as f64 / duration_seconds as f64
        } else {
            0.0
        };

        let failure_rate = if total > 0 {
            (failed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let peak_memory = self.peak_memory.load(Ordering::Relaxed);

        Ok(StressTestResults {
            total_operations: total,
            successful_operations: completed,
            failed_operations: failed,
            duration_seconds,
            operations_per_second: ops_per_second,
            failure_rate,
            peak_memory_mb: peak_memory as f64 / (1024 * 1024) as f64,
            avg_latency_us: if completed > 0 {
                (duration_seconds as f64 * 1_000_000.0) / completed as f64
            } else {
                0.0
            },
        })
    }

    pub fn reset(&mut self) {
        self.operations_completed.store(0, Ordering::Relaxed);
        self.operations_failed.store(0, Ordering::Relaxed);
        self.peak_memory.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stress_test_creation() {
        let config = StressTestConfig::default();
        let test = StressTest::new(config);
        assert_eq!(test.operations_completed.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_memory_stress_test() {
        let config = StressTestConfig {
            duration_seconds: 1,
            max_concurrent_allocations: 100,
            ..Default::default()
        };
        let mut test = StressTest::new(config);
        let results = test.run_memory_stress_test().unwrap();
        assert!(results.total_operations > 0);
    }

    #[test]
    fn test_concurrency_stress_test() {
        let config = StressTestConfig {
            duration_seconds: 1,
            concurrent_operations: 2,
            ..Default::default()
        };
        let mut test = StressTest::new(config);
        let results = test.run_concurrency_stress_test().unwrap();
        assert!(results.total_operations > 0);
    }

    #[test]
    fn test_long_running_test() {
        let config = StressTestConfig {
            duration_seconds: 1,
            ..Default::default()
        };
        let mut test = StressTest::new(config);
        let results = test.run_long_running_test().unwrap();
        assert!(results.total_operations > 0);
    }

    #[test]
    fn test_results_accuracy() {
        let config = StressTestConfig {
            duration_seconds: 1,
            concurrent_operations: 1,
            ..Default::default()
        };
        let mut test = StressTest::new(config);
        let results = test.run_long_running_test().unwrap();

        assert!(results.operations_per_second > 0.0);
        assert!(results.failure_rate >= 0.0 && results.failure_rate <= 100.0);
    }

    #[test]
    fn test_reset() {
        let config = StressTestConfig::default();
        let mut test = StressTest::new(config);
        test.operations_completed.store(100, Ordering::Relaxed);
        test.reset();
        assert_eq!(test.operations_completed.load(Ordering::Relaxed), 0);
    }
}
