//! Performance Profiler for Kernel Subsystems
//!
//! Measures and analyzes performance of kernel operations with:
//! - Latency tracking (min, max, average, percentiles)
//! - Throughput measurement
//! - Resource utilization monitoring
//! - Bottleneck identification
//! - Comparative analysis

use std::collections::HashMap;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct LatencyMetrics {
    pub operation_name: String,
    pub samples: usize,
    pub min_ns: u64,
    pub max_ns: u64,
    pub avg_ns: u64,
    pub p50_ns: u64,
    pub p95_ns: u64,
    pub p99_ns: u64,
}

#[derive(Clone, Debug)]
pub struct ThroughputMetrics {
    pub operation_name: String,
    pub total_operations: u64,
    pub duration_ms: u64,
    pub ops_per_second: f64,
    pub avg_latency_us: f64,
}

#[derive(Clone, Debug)]
pub struct ResourceMetrics {
    pub memory_used_mb: f64,
    pub cpu_utilization: f64,
    pub io_operations: u64,
    pub cache_hit_rate: f64,
}

pub struct Profiler {
    measurements: HashMap<String, Vec<u64>>,
    operations_completed: HashMap<String, u64>,
    total_time_ms: HashMap<String, u64>,
    active_timers: HashMap<String, Instant>,
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Profiler {
    pub fn new() -> Self {
        Profiler {
            measurements: HashMap::new(),
            operations_completed: HashMap::new(),
            total_time_ms: HashMap::new(),
            active_timers: HashMap::new(),
        }
    }

    pub fn start_timer(&mut self, operation: &str) {
        self.active_timers
            .insert(operation.to_string(), Instant::now());
    }

    pub fn end_timer(&mut self, operation: &str) {
        if let Some(start_time) = self.active_timers.remove(operation) {
            let elapsed_ns = start_time.elapsed().as_nanos() as u64;
            self.measurements
                .entry(operation.to_string())
                .or_default()
                .push(elapsed_ns);
        }
    }

    pub fn record_operation(&mut self, operation: &str, latency_ns: u64) {
        self.measurements
            .entry(operation.to_string())
            .or_default()
            .push(latency_ns);
        *self
            .operations_completed
            .entry(operation.to_string())
            .or_insert(0) += 1;
    }

    pub fn get_latency_metrics(&self, operation: &str) -> Option<LatencyMetrics> {
        let samples = self.measurements.get(operation)?;
        if samples.is_empty() {
            return None;
        }

        let mut sorted = samples.clone();
        sorted.sort_unstable();

        let min_ns = sorted[0];
        let max_ns = sorted[sorted.len() - 1];
        let avg_ns = sorted.iter().sum::<u64>() / sorted.len() as u64;

        let p50_idx = sorted.len() / 2;
        let p95_idx = (sorted.len() * 95) / 100;
        let p99_idx = (sorted.len() * 99) / 100;

        Some(LatencyMetrics {
            operation_name: operation.to_string(),
            samples: sorted.len(),
            min_ns,
            max_ns,
            avg_ns,
            p50_ns: sorted[p50_idx],
            p95_ns: sorted[p95_idx.min(sorted.len() - 1)],
            p99_ns: sorted[p99_idx.min(sorted.len() - 1)],
        })
    }

    pub fn get_throughput_metrics(
        &self,
        operation: &str,
        duration_ms: u64,
    ) -> Option<ThroughputMetrics> {
        let total_ops = self.operations_completed.get(operation)?;
        let total_time_ms = if duration_ms == 0 {
            *self.total_time_ms.get(operation).unwrap_or(&1)
        } else {
            duration_ms
        };

        let ops_per_second = (*total_ops as f64 / total_time_ms as f64) * 1000.0;
        let metrics = self.get_latency_metrics(operation)?;
        let avg_latency_us = metrics.avg_ns as f64 / 1000.0;

        Some(ThroughputMetrics {
            operation_name: operation.to_string(),
            total_operations: *total_ops,
            duration_ms: total_time_ms,
            ops_per_second,
            avg_latency_us,
        })
    }

    pub fn identify_bottlenecks(&self, threshold_percentile: f64) -> Vec<(String, u64)> {
        let mut bottlenecks = Vec::new();

        for (operation, samples) in &self.measurements {
            if samples.is_empty() {
                continue;
            }

            let mut sorted = samples.clone();
            sorted.sort_unstable();

            let threshold_idx = ((sorted.len() as f64) * (threshold_percentile / 100.0)) as usize;
            let threshold_value = sorted[threshold_idx.min(sorted.len() - 1)];

            let outliers = sorted.iter().filter(|&&s| s > threshold_value).count();

            if outliers > 0 {
                bottlenecks.push((operation.clone(), threshold_value));
            }
        }

        bottlenecks.sort_by_key(|b| std::cmp::Reverse(b.1));
        bottlenecks
    }

    pub fn reset(&mut self) {
        self.measurements.clear();
        self.operations_completed.clear();
        self.total_time_ms.clear();
        self.active_timers.clear();
    }

    pub fn generate_report(&self) -> String {
        let mut report = String::from("SHER Kernel Performance Report\n");
        report.push_str("================================\n\n");

        for operation in self.operations_completed.keys() {
            if let Some(metrics) = self.get_latency_metrics(operation) {
                report.push_str(&format!(
                    "{}: min={:.2}μs, avg={:.2}μs, p95={:.2}μs, p99={:.2}μs, max={:.2}μs ({} samples)\n",
                    metrics.operation_name,
                    metrics.min_ns as f64 / 1000.0,
                    metrics.avg_ns as f64 / 1000.0,
                    metrics.p95_ns as f64 / 1000.0,
                    metrics.p99_ns as f64 / 1000.0,
                    metrics.max_ns as f64 / 1000.0,
                    metrics.samples
                ));
            }
        }

        report.push_str("\nBottlenecks (p99+):\n");
        let bottlenecks = self.identify_bottlenecks(99.0);
        for (op, latency) in bottlenecks {
            report.push_str(&format!("  {}: {}ns\n", op, latency));
        }

        report
    }

    pub fn clear_operation(&mut self, operation: &str) {
        self.measurements.remove(operation);
        self.operations_completed.remove(operation);
        self.total_time_ms.remove(operation);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_profiler_creation() {
        let profiler = Profiler::new();
        assert_eq!(profiler.measurements.len(), 0);
    }

    #[test]
    fn test_timer_measurement() {
        let mut profiler = Profiler::new();
        profiler.start_timer("test_op");
        thread::sleep(Duration::from_micros(100));
        profiler.end_timer("test_op");

        let metrics = profiler.get_latency_metrics("test_op");
        assert!(metrics.is_some());
        assert!(metrics.unwrap().avg_ns > 50000);
    }

    #[test]
    fn test_latency_percentiles() {
        let mut profiler = Profiler::new();

        for i in 1..=100 {
            profiler.record_operation("test_op", i * 1000);
        }

        let metrics = profiler.get_latency_metrics("test_op").unwrap();
        assert_eq!(metrics.min_ns, 1000);
        assert_eq!(metrics.max_ns, 100000);
        assert!(metrics.p50_ns > 40000 && metrics.p50_ns < 60000);
        assert!(metrics.p95_ns > 90000);
    }

    #[test]
    fn test_throughput_calculation() {
        let mut profiler = Profiler::new();

        for _ in 0..1000 {
            profiler.record_operation("test_op", 1000);
        }

        let metrics = profiler.get_throughput_metrics("test_op", 1000).unwrap();
        assert_eq!(metrics.total_operations, 1000);
        assert!(metrics.ops_per_second > 500.0);
    }

    #[test]
    fn test_bottleneck_identification() {
        let mut profiler = Profiler::new();

        for _ in 1..=99 {
            profiler.record_operation("test_op", 1000);
        }
        profiler.record_operation("test_op", 100000);

        let bottlenecks = profiler.identify_bottlenecks(95.0);
        assert!(!bottlenecks.is_empty());
    }

    #[test]
    fn test_report_generation() {
        let mut profiler = Profiler::new();
        profiler.record_operation("test_op", 1000);
        profiler.record_operation("test_op", 2000);

        let report = profiler.generate_report();
        assert!(report.contains("test_op"));
        assert!(report.contains("Performance Report"));
    }

    #[test]
    fn test_reset() {
        let mut profiler = Profiler::new();
        profiler.record_operation("test_op", 1000);
        assert!(!profiler.measurements.is_empty());

        profiler.reset();
        assert!(profiler.measurements.is_empty());
    }
}
