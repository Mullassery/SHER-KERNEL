//! SHER Kernel Profiling & Stress Testing
//!
//! Performance analysis and stress testing framework for:
//! - Latency profiling and percentile analysis
//! - Throughput measurement
//! - Bottleneck identification
//! - Memory stress testing
//! - Concurrency testing
//! - Cascade failure analysis

pub mod profiler;
pub mod stress_test;

pub use profiler::{LatencyMetrics, Profiler, ResourceMetrics, ThroughputMetrics};
pub use stress_test::{StressTest, StressTestConfig, StressTestResults};
