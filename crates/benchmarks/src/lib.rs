//! SHER Kernel Performance Benchmarking Suite
//!
//! Comprehensive benchmarks measuring SHER kernel subsystem performance
//! with comparison against Linux kernel baselines.

use std::time::Duration;

pub struct BenchmarkMetrics {
    pub operation: String,
    pub iterations: usize,
    pub total_duration: Duration,
    pub avg_nanoseconds: u64,
    pub min_nanoseconds: u64,
    pub max_nanoseconds: u64,
}

impl BenchmarkMetrics {
    pub fn new(operation: &str, iterations: usize, durations: Vec<Duration>) -> Self {
        let total_duration: Duration = durations.iter().sum();
        let avg_ns = total_duration.as_nanos() as u64 / iterations as u64;
        let min_ns = durations.iter().min().unwrap().as_nanos() as u64;
        let max_ns = durations.iter().max().unwrap().as_nanos() as u64;

        BenchmarkMetrics {
            operation: operation.to_string(),
            iterations,
            total_duration,
            avg_nanoseconds: avg_ns,
            min_nanoseconds: min_ns,
            max_nanoseconds: max_ns,
        }
    }

    pub fn to_microseconds(nanos: u64) -> f64 {
        nanos as f64 / 1000.0
    }
}

impl std::fmt::Display for BenchmarkMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: avg {:.2}μs, min {:.2}μs, max {:.2}μs",
            self.operation,
            Self::to_microseconds(self.avg_nanoseconds),
            Self::to_microseconds(self.min_nanoseconds),
            Self::to_microseconds(self.max_nanoseconds),
        )
    }
}

pub struct ComparisonRow {
    pub operation: String,
    pub sher_microseconds: f64,
    pub linux_baseline_microseconds: f64,
    pub overhead_percent: f64,
    pub status: String,
}

impl ComparisonRow {
    pub fn to_markdown(&self) -> String {
        format!(
            "| {} | {:.2}μs | {:.2}μs | {:+.1}% | {} |",
            self.operation,
            self.sher_microseconds,
            self.linux_baseline_microseconds,
            self.overhead_percent,
            self.status
        )
    }
}

pub fn calculate_overhead(sher_us: f64, linux_us: f64) -> (f64, String) {
    let overhead_percent = ((sher_us - linux_us) / linux_us) * 100.0;
    let status = if overhead_percent < 20.0 {
        "✓ Excellent".to_string()
    } else if overhead_percent < 50.0 {
        "✓ Acceptable".to_string()
    } else if overhead_percent < 100.0 {
        "⚠ Notable".to_string()
    } else {
        "⚠ High".to_string()
    };
    (overhead_percent, status)
}
