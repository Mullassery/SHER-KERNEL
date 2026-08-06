// SHER AI Services: Adaptive Scheduling Engine
// Real-time scheduling decisions based on anomaly detection and resource predictions

use sher_common::{ObjectId, Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// SCHEDULING DECISION ENGINE
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulingStrategy {
    Aggressive,      // Maximize throughput, accept higher latency variance
    Balanced,        // Default, trade throughput for predictability
    Conservative,    // Minimize latency spikes, may reduce throughput
    RealTime,        // Guarantee latency bounds, preemption enabled
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CPUAffinity {
    NoPreference = 0,
    PreferSocket = 1,
    RequireSocket = 2,
    PreferL3Cache = 3,
    RequireL3Cache = 4,
    SpecificCore = 5,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingDecision {
    pub driver_id: ObjectId,
    pub strategy: SchedulingStrategy,
    pub cpu_affinity: CPUAffinity,
    pub priority: u32,               // 0-255, higher = more important
    pub cpu_quota_percent: f64,      // 0-100
    pub latency_slo_ms: f64,         // Service level objective
    pub preempt_other: bool,
    pub confidence: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingMetrics {
    pub driver_id: ObjectId,
    pub decisions_made: u64,
    pub strategy_switches: u64,
    pub slo_violations: u64,
    pub slo_achievement_rate: f64,   // 0.0-1.0
    pub avg_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub throughput_ops_per_sec: f64,
}

// ============================================================================
// ADAPTIVE SCHEDULER
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct AdaptiveScheduler {
    pub decisions: HashMap<ObjectId, SchedulingDecision>,
    pub metrics: HashMap<ObjectId, SchedulingMetrics>,
    pub history: Vec<SchedulingDecision>,
    pub max_history: usize,
    pub total_decisions: u64,
}

impl AdaptiveScheduler {
    pub fn new() -> Self {
        AdaptiveScheduler {
            decisions: HashMap::new(),
            metrics: HashMap::new(),
            history: Vec::new(),
            max_history: 1000,
            total_decisions: 0,
        }
    }

    /// Make scheduling decision based on driver profile and anomalies
    pub fn decide_scheduling(
        &mut self,
        driver_id: ObjectId,
        current_strategy: SchedulingStrategy,
        cpu_usage: f64,
        memory_usage: f64,
        recent_anomalies: u32,
        latency_observed_ms: f64,
        target_latency_slo_ms: f64,
    ) -> SchedulingDecision {
        let mut decision = SchedulingDecision {
            driver_id,
            strategy: current_strategy,
            cpu_affinity: CPUAffinity::NoPreference,
            priority: 128,
            cpu_quota_percent: 50.0,
            latency_slo_ms: target_latency_slo_ms,
            preempt_other: false,
            confidence: 0.5,
            reason: String::new(),
        };

        // Determine strategy based on observed metrics
        decision.strategy = match (cpu_usage, memory_usage, recent_anomalies) {
            // High anomalies: switch to conservative for stability
            (_, _, anomalies) if anomalies > 5 => {
                decision.reason = format!("High anomaly count: {}", anomalies);
                SchedulingStrategy::Conservative
            }
            // CPU-bound workload with good latency: aggressive
            (cpu, _, anomalies) if cpu > 80.0 && latency_observed_ms < target_latency_slo_ms && anomalies <= 2 => {
                decision.reason = "CPU-bound with good latency".to_string();
                SchedulingStrategy::Aggressive
            }
            // Latency issues: switch to real-time if extreme
            (_, _, _) if latency_observed_ms > target_latency_slo_ms * 2.0 => {
                decision.reason = format!("Severe latency: {}ms vs {}ms SLO", latency_observed_ms, target_latency_slo_ms);
                SchedulingStrategy::RealTime
            }
            // Memory pressure: conservative to prevent swap
            (_, mem, _) if mem > 85.0 => {
                decision.reason = format!("High memory pressure: {}%", memory_usage);
                SchedulingStrategy::Conservative
            }
            // Default: balanced
            _ => {
                decision.reason = "Balanced operation".to_string();
                SchedulingStrategy::Balanced
            }
        };

        // Adjust priority based on strategy
        decision.priority = match decision.strategy {
            SchedulingStrategy::Aggressive => 200,
            SchedulingStrategy::Balanced => 128,
            SchedulingStrategy::Conservative => 64,
            SchedulingStrategy::RealTime => 255,
        };

        // Set CPU quota based on strategy and observed usage
        decision.cpu_quota_percent = match decision.strategy {
            SchedulingStrategy::Aggressive => (cpu_usage * 1.2).min(100.0),
            SchedulingStrategy::Balanced => (cpu_usage * 1.1).min(100.0),
            SchedulingStrategy::Conservative => (cpu_usage * 0.9).max(20.0),
            SchedulingStrategy::RealTime => cpu_usage,
        };

        // Determine preemption policy
        decision.preempt_other = decision.strategy == SchedulingStrategy::RealTime && recent_anomalies > 0;

        // Calculate confidence based on observed stability
        decision.confidence = if recent_anomalies == 0 &&
            (latency_observed_ms - target_latency_slo_ms).abs() < target_latency_slo_ms * 0.1 {
            0.95
        } else if recent_anomalies <= 2 {
            0.70
        } else {
            0.40
        };

        self.total_decisions += 1;

        // Store decision
        if let Some(old_decision) = self.decisions.insert(driver_id, decision.clone()) {
            if old_decision.strategy != decision.strategy {
                if let Some(metrics) = self.metrics.get_mut(&driver_id) {
                    metrics.strategy_switches += 1;
                }
            }
        }

        // Update history
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(decision.clone());

        decision
    }

    /// Record SLO achievement for a driver
    pub fn record_slo_result(
        &mut self,
        driver_id: ObjectId,
        latency_ms: f64,
        slo_ms: f64,
        throughput_ops: u64,
    ) {
        let metrics = self.metrics.entry(driver_id).or_insert_with(|| SchedulingMetrics {
            driver_id,
            decisions_made: 0,
            strategy_switches: 0,
            slo_violations: 0,
            slo_achievement_rate: 1.0,
            avg_latency_ms: latency_ms,
            p99_latency_ms: latency_ms,
            throughput_ops_per_sec: throughput_ops as f64,
        });

        metrics.decisions_made += 1;

        let is_violation = latency_ms > slo_ms;
        if is_violation {
            metrics.slo_violations += 1;
        }

        // Update exponential moving average
        let alpha = 0.1;
        metrics.avg_latency_ms = metrics.avg_latency_ms * (1.0 - alpha) + latency_ms * alpha;

        // Update P99 estimate (approximation)
        metrics.p99_latency_ms = metrics.p99_latency_ms * 0.95 + latency_ms * 0.05;

        // Calculate achievement rate
        metrics.slo_achievement_rate = if metrics.decisions_made > 0 {
            ((metrics.decisions_made - metrics.slo_violations) as f64 / metrics.decisions_made as f64).max(0.0)
        } else {
            1.0
        };

        metrics.throughput_ops_per_sec = throughput_ops as f64;
    }

    /// Get current decision for driver
    pub fn get_decision(&self, driver_id: ObjectId) -> Option<&SchedulingDecision> {
        self.decisions.get(&driver_id)
    }

    /// Get metrics for driver
    pub fn get_metrics(&self, driver_id: ObjectId) -> Option<&SchedulingMetrics> {
        self.metrics.get(&driver_id)
    }

    /// Get drivers sorted by SLO violation rate (worst first)
    pub fn get_problematic_drivers(&self) -> Vec<(ObjectId, f64)> {
        let mut drivers: Vec<_> = self.metrics
            .iter()
            .map(|(id, m)| (*id, 1.0 - m.slo_achievement_rate))
            .collect();

        drivers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        drivers
    }

    /// Get statistics
    pub fn get_stats(&self) -> SchedulingStats {
        let avg_achievement = if self.metrics.is_empty() {
            1.0
        } else {
            self.metrics.values().map(|m| m.slo_achievement_rate).sum::<f64>() / self.metrics.len() as f64
        };

        SchedulingStats {
            total_drivers: self.metrics.len() as u64,
            total_decisions: self.total_decisions,
            avg_slo_achievement: avg_achievement,
            strategy_distribution: self.get_strategy_distribution(),
            total_strategy_switches: self.metrics.values().map(|m| m.strategy_switches).sum(),
        }
    }

    fn get_strategy_distribution(&self) -> HashMap<String, u64> {
        let mut dist = HashMap::new();
        for decision in &self.history {
            let key = format!("{:?}", decision.strategy);
            *dist.entry(key).or_insert(0) += 1;
        }
        dist
    }
}

// ============================================================================
// SCHEDULING STATISTICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingStats {
    pub total_drivers: u64,
    pub total_decisions: u64,
    pub avg_slo_achievement: f64,
    pub strategy_distribution: HashMap<String, u64>,
    pub total_strategy_switches: u64,
}

// ============================================================================
// WORKLOAD CLASSIFIER
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkloadType {
    Interactive,     // Low latency sensitive, variable throughput
    Batch,           // Throughput optimized, flexible latency
    RealTime,        // Strict latency bounds, preemption required
    ML,              // High compute, memory intensive
    IO,              // I/O bound, network heavy
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadProfile {
    pub driver_id: ObjectId,
    pub workload_type: WorkloadType,
    pub cpu_intensity: f64,          // 0.0-1.0
    pub memory_intensity: f64,       // 0.0-1.0
    pub io_intensity: f64,           // 0.0-1.0
    pub latency_sensitivity: f64,    // 0.0-1.0
    pub confidence: f64,
    pub samples: u64,
}

#[derive(Debug, Clone, Default)]
pub struct WorkloadClassifier {
    pub profiles: HashMap<ObjectId, WorkloadProfile>,
}

impl WorkloadClassifier {
    pub fn new() -> Self {
        WorkloadClassifier {
            profiles: HashMap::new(),
        }
    }

    /// Classify workload based on observed characteristics
    pub fn classify(
        &mut self,
        driver_id: ObjectId,
        cpu_usage: f64,
        memory_usage: f64,
        io_ops_per_sec: f64,
        latency_variance: f64,
    ) -> WorkloadType {
        // Normalize to 0.0-1.0 scale
        let cpu_intensity = (cpu_usage / 100.0).min(1.0);
        let memory_intensity = (memory_usage / 100.0).min(1.0);
        let io_intensity = (io_ops_per_sec / 10000.0).min(1.0);
        let latency_sensitivity = (latency_variance / 100.0).min(1.0);

        let workload_type = match (cpu_intensity, memory_intensity, io_intensity, latency_sensitivity) {
            // Real-time: low variance, strict latency needs
            (_, _, _, sens) if sens < 0.2 => WorkloadType::RealTime,
            // ML: high CPU and memory
            (cpu, mem, _, _) if cpu > 0.7 && mem > 0.6 => WorkloadType::ML,
            // IO: high IO intensity
            (_, _, io, _) if io > 0.6 => WorkloadType::IO,
            // Batch: high CPU, moderate sensitivity
            (cpu, _, _, _) if cpu > 0.6 => WorkloadType::Batch,
            // Interactive: low CPU, sensitive to latency
            (cpu, _, _, _) if cpu < 0.3 => WorkloadType::Interactive,
            // Default
            _ => WorkloadType::Batch,
        };

        let profile = WorkloadProfile {
            driver_id,
            workload_type,
            cpu_intensity,
            memory_intensity,
            io_intensity,
            latency_sensitivity,
            confidence: 0.75,
            samples: 1,
        };

        if let Some(existing) = self.profiles.get_mut(&driver_id) {
            // Update existing profile with exponential moving average
            let alpha = 0.1;
            existing.cpu_intensity = existing.cpu_intensity * (1.0 - alpha) + cpu_intensity * alpha;
            existing.memory_intensity = existing.memory_intensity * (1.0 - alpha) + memory_intensity * alpha;
            existing.io_intensity = existing.io_intensity * (1.0 - alpha) + io_intensity * alpha;
            existing.latency_sensitivity = existing.latency_sensitivity * (1.0 - alpha) + latency_sensitivity * alpha;
            existing.samples += 1;
            existing.confidence = (existing.samples as f64 / 10.0).min(1.0);
        } else {
            self.profiles.insert(driver_id, profile);
        }

        workload_type
    }

    /// Get workload profile for driver
    pub fn get_profile(&self, driver_id: ObjectId) -> Option<&WorkloadProfile> {
        self.profiles.get(&driver_id)
    }
}
