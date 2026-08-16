// SHER AI Services: Anomaly Detection Engine
// ML-based detection of memory leaks, interrupt storms, DMA abuse, and resource exhaustion

use serde::{Deserialize, Serialize};
use sher_common::ObjectId;
use std::collections::{HashMap, VecDeque};

// ============================================================================
// ANOMALY TYPES & DETECTION
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalyType {
    MemoryLeak,         // Unbounded allocation growth
    InterruptStorm,     // Excessive interrupt rate
    DmaAbuse,           // Suspicious DMA patterns
    ResourceExhaustion, // Rapid resource depletion
    ExcessiveCpuUsage,  // CPU spinning or loops
    NetworkFlood,       // Unusual network traffic
    FileDescriptorLeak, // FD accumulation
    DeadlockRisk,       // Circular resource dependencies
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub anomaly_id: ObjectId,
    pub anomaly_type: AnomalyType,
    pub driver_id: ObjectId,
    pub severity: AnomalySeverity,
    pub confidence: f64, // 0.0-1.0
    pub detected_at_ms: u64,
    pub metrics: HashMap<String, f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,      // Monitor
    Medium,   // Alert
    High,     // Throttle
    Critical, // Isolate
}

// ============================================================================
// MEMORY LEAK DETECTOR
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct MemoryLeakDetector {
    pub allocation_history: HashMap<ObjectId, VecDeque<MemorySnapshot>>,
    pub threshold_mb_per_second: u64,
    pub history_window_seconds: u64,
    pub detections: u64,
}

#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    timestamp_ms: u64,
    total_allocated_bytes: u64,
}

impl MemoryLeakDetector {
    pub fn new() -> Self {
        MemoryLeakDetector {
            allocation_history: HashMap::new(),
            threshold_mb_per_second: 50, // 50 MB/s is suspicious
            history_window_seconds: 10,
            detections: 0,
        }
    }

    pub fn record_allocation(
        &mut self,
        driver_id: ObjectId,
        total_bytes: u64,
        current_time_ms: u64,
    ) {
        let history = self.allocation_history.entry(driver_id).or_default();

        // Remove old entries outside window
        let window_ms = self.history_window_seconds * 1000;
        while let Some(front) = history.front() {
            if current_time_ms - front.timestamp_ms > window_ms {
                history.pop_front();
            } else {
                break;
            }
        }

        history.push_back(MemorySnapshot {
            timestamp_ms: current_time_ms,
            total_allocated_bytes: total_bytes,
        });
    }

    pub fn detect_leak(&mut self, driver_id: ObjectId, current_time_ms: u64) -> Option<Anomaly> {
        if let Some(history) = self.allocation_history.get(&driver_id) {
            if history.len() < 2 {
                return None;
            }

            let first = &history[0];
            let last = &history[history.len() - 1];

            let time_delta_ms = last.timestamp_ms - first.timestamp_ms;
            if time_delta_ms == 0 {
                return None;
            }

            let bytes_delta = last
                .total_allocated_bytes
                .saturating_sub(first.total_allocated_bytes);
            let mb_delta = bytes_delta / (1024 * 1024);
            let time_delta_seconds = time_delta_ms / 1000;

            if let Some(rate_mb_per_sec) = mb_delta.checked_div(time_delta_seconds) {
                if rate_mb_per_sec > self.threshold_mb_per_second {
                    self.detections += 1;

                    let confidence =
                        (rate_mb_per_sec as f64 / self.threshold_mb_per_second as f64).min(1.0);
                    let severity = match rate_mb_per_sec {
                        0..=100 => AnomalySeverity::Medium,
                        101..=500 => AnomalySeverity::High,
                        _ => AnomalySeverity::Critical,
                    };

                    let mut metrics = HashMap::new();
                    metrics.insert("mb_per_second".to_string(), rate_mb_per_sec as f64);
                    metrics.insert(
                        "total_allocated_mb".to_string(),
                        (last.total_allocated_bytes / (1024 * 1024)) as f64,
                    );
                    metrics.insert(
                        "growth_percentage".to_string(),
                        (rate_mb_per_sec as f64 / self.threshold_mb_per_second as f64) * 100.0,
                    );

                    return Some(Anomaly {
                        anomaly_id: ObjectId::new(),
                        anomaly_type: AnomalyType::MemoryLeak,
                        driver_id,
                        severity,
                        confidence,
                        detected_at_ms: current_time_ms,
                        metrics,
                    });
                }
            }
        }

        None
    }
}

// ============================================================================
// INTERRUPT STORM DETECTOR
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct InterruptStormDetector {
    pub interrupt_counts: HashMap<u32, VecDeque<u64>>, // irq -> timestamps
    pub threshold_per_second: u32,
    pub history_window_seconds: u64,
    pub detections: u64,
}

impl InterruptStormDetector {
    pub fn new() -> Self {
        InterruptStormDetector {
            interrupt_counts: HashMap::new(),
            threshold_per_second: 10_000, // > 10k interrupts/sec is a storm
            history_window_seconds: 1,
            detections: 0,
        }
    }

    pub fn record_interrupt(&mut self, irq: u32, timestamp_ms: u64) {
        let history = self.interrupt_counts.entry(irq).or_default();

        let window_ms = self.history_window_seconds * 1000;
        while let Some(front) = history.front() {
            if timestamp_ms - front > window_ms {
                history.pop_front();
            } else {
                break;
            }
        }

        history.push_back(timestamp_ms);
    }

    pub fn detect_storm(&mut self, irq: u32, current_time_ms: u64) -> Option<Anomaly> {
        if let Some(history) = self.interrupt_counts.get(&irq) {
            let count = history.len() as u32;

            if count > self.threshold_per_second {
                self.detections += 1;

                let confidence = (count as f64 / self.threshold_per_second as f64).min(1.0);
                let severity = match count {
                    0..=20_000 => AnomalySeverity::High,
                    20_001..=50_000 => AnomalySeverity::Critical,
                    _ => AnomalySeverity::Critical,
                };

                let mut metrics = HashMap::new();
                metrics.insert("interrupts_per_second".to_string(), count as f64);
                metrics.insert("threshold".to_string(), self.threshold_per_second as f64);
                metrics.insert(
                    "excess_percentage".to_string(),
                    ((count as f64 / self.threshold_per_second as f64) - 1.0) * 100.0,
                );

                return Some(Anomaly {
                    anomaly_id: ObjectId::new(),
                    anomaly_type: AnomalyType::InterruptStorm,
                    driver_id: ObjectId::new(), // Would be irq number in real scenario
                    severity,
                    confidence,
                    detected_at_ms: current_time_ms,
                    metrics,
                });
            }
        }

        None
    }
}

// ============================================================================
// DMA ABUSE DETECTOR
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct DmaAbuseDetector {
    pub dma_operations: HashMap<ObjectId, VecDeque<DmaOperation>>,
    pub threshold_concurrent: u32,
    pub threshold_bytes_per_second: u64,
    pub detections: u64,
}

#[derive(Debug, Clone)]
pub struct DmaOperation {
    timestamp_ms: u64,
    bytes: u64,
    is_write: bool,
}

impl DmaAbuseDetector {
    pub fn new() -> Self {
        DmaAbuseDetector {
            dma_operations: HashMap::new(),
            threshold_concurrent: 100,
            threshold_bytes_per_second: 1024 * 1024 * 1024, // 1GB/s
            detections: 0,
        }
    }

    pub fn record_dma(
        &mut self,
        driver_id: ObjectId,
        bytes: u64,
        is_write: bool,
        timestamp_ms: u64,
    ) {
        let ops = self.dma_operations.entry(driver_id).or_default();

        // Keep last 1 second
        while let Some(front) = ops.front() {
            if timestamp_ms - front.timestamp_ms > 1000 {
                ops.pop_front();
            } else {
                break;
            }
        }

        ops.push_back(DmaOperation {
            timestamp_ms,
            bytes,
            is_write,
        });
    }

    pub fn detect_abuse(&mut self, driver_id: ObjectId, current_time_ms: u64) -> Option<Anomaly> {
        if let Some(ops) = self.dma_operations.get(&driver_id) {
            let concurrent = ops.len() as u32;
            let total_bytes: u64 = ops.iter().map(|op| op.bytes).sum();
            let write_count = ops.iter().filter(|op| op.is_write).count();
            let write_ratio = if ops.is_empty() {
                0.0
            } else {
                write_count as f64 / ops.len() as f64
            };

            let mut should_alert = false;
            // A DMA-write-heavy burst (e.g. > 80% writes) is more
            // consistent with memory corruption/exfiltration attempts than
            // read-heavy traffic, so it escalates severity a tier.
            let write_heavy = write_ratio > 0.8;
            let mut severity = AnomalySeverity::Low;

            if concurrent > self.threshold_concurrent {
                should_alert = true;
                severity = if write_heavy {
                    AnomalySeverity::Critical
                } else {
                    AnomalySeverity::High
                };
            }

            if total_bytes > self.threshold_bytes_per_second {
                should_alert = true;
                severity = AnomalySeverity::Critical;
            }

            if should_alert {
                self.detections += 1;

                let confidence = (concurrent as f64 / self.threshold_concurrent as f64).min(1.0);

                let mut metrics = HashMap::new();
                metrics.insert("concurrent_operations".to_string(), concurrent as f64);
                metrics.insert("bytes_per_second".to_string(), total_bytes as f64);
                metrics.insert(
                    "threshold_bytes".to_string(),
                    self.threshold_bytes_per_second as f64,
                );
                metrics.insert("write_ratio".to_string(), write_ratio);

                return Some(Anomaly {
                    anomaly_id: ObjectId::new(),
                    anomaly_type: AnomalyType::DmaAbuse,
                    driver_id,
                    severity,
                    confidence,
                    detected_at_ms: current_time_ms,
                    metrics,
                });
            }
        }

        None
    }
}

// ============================================================================
// ANOMALY ENGINE
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct AnomalyEngine {
    pub memory_detector: MemoryLeakDetector,
    pub interrupt_detector: InterruptStormDetector,
    pub dma_detector: DmaAbuseDetector,
    pub total_anomalies: u64,
    pub recent_anomalies: VecDeque<Anomaly>,
    pub max_history: usize,
}

impl AnomalyEngine {
    pub fn new() -> Self {
        AnomalyEngine {
            memory_detector: MemoryLeakDetector::new(),
            interrupt_detector: InterruptStormDetector::new(),
            dma_detector: DmaAbuseDetector::new(),
            total_anomalies: 0,
            recent_anomalies: VecDeque::new(),
            max_history: 100,
        }
    }

    pub fn record_memory_state(
        &mut self,
        driver_id: ObjectId,
        total_bytes: u64,
        current_time_ms: u64,
    ) {
        self.memory_detector
            .record_allocation(driver_id, total_bytes, current_time_ms);

        if let Some(anomaly) = self.memory_detector.detect_leak(driver_id, current_time_ms) {
            self.add_anomaly(anomaly);
        }
    }

    pub fn record_interrupt(&mut self, irq: u32, timestamp_ms: u64) {
        self.interrupt_detector.record_interrupt(irq, timestamp_ms);

        if let Some(anomaly) = self.interrupt_detector.detect_storm(irq, timestamp_ms) {
            self.add_anomaly(anomaly);
        }
    }

    pub fn record_dma_operation(
        &mut self,
        driver_id: ObjectId,
        bytes: u64,
        is_write: bool,
        timestamp_ms: u64,
    ) {
        self.dma_detector
            .record_dma(driver_id, bytes, is_write, timestamp_ms);

        if let Some(anomaly) = self.dma_detector.detect_abuse(driver_id, timestamp_ms) {
            self.add_anomaly(anomaly);
        }
    }

    fn add_anomaly(&mut self, anomaly: Anomaly) {
        self.total_anomalies += 1;

        if self.recent_anomalies.len() >= self.max_history {
            self.recent_anomalies.pop_front();
        }

        self.recent_anomalies.push_back(anomaly);
    }

    pub fn get_critical_anomalies(&self) -> Vec<&Anomaly> {
        self.recent_anomalies
            .iter()
            .filter(|a| a.severity == AnomalySeverity::Critical)
            .collect()
    }

    pub fn get_anomalies_by_type(&self, anomaly_type: AnomalyType) -> Vec<&Anomaly> {
        self.recent_anomalies
            .iter()
            .filter(|a| a.anomaly_type == anomaly_type)
            .collect()
    }

    pub fn get_anomalies_by_driver(&self, driver_id: ObjectId) -> Vec<&Anomaly> {
        self.recent_anomalies
            .iter()
            .filter(|a| a.driver_id == driver_id)
            .collect()
    }

    pub fn get_stats(&self) -> AnomalyStats {
        let critical_count = self
            .recent_anomalies
            .iter()
            .filter(|a| a.severity == AnomalySeverity::Critical)
            .count();
        let high_count = self
            .recent_anomalies
            .iter()
            .filter(|a| a.severity == AnomalySeverity::High)
            .count();

        AnomalyStats {
            total_detected: self.total_anomalies,
            currently_active: self.recent_anomalies.len() as u64,
            critical_count: critical_count as u64,
            high_count: high_count as u64,
            memory_leaks: self.memory_detector.detections,
            interrupt_storms: self.interrupt_detector.detections,
            dma_abuse: self.dma_detector.detections,
        }
    }
}

// ============================================================================
// ANOMALY STATISTICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyStats {
    pub total_detected: u64,
    pub currently_active: u64,
    pub critical_count: u64,
    pub high_count: u64,
    pub memory_leaks: u64,
    pub interrupt_storms: u64,
    pub dma_abuse: u64,
}
