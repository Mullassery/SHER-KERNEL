// SHER AI Services: Predictive Resource Allocation
// Machine learning-based prediction of resource needs for optimal scheduling

use sher_common::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// RESOURCE PREDICTION MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceProfile {
    pub driver_id: ObjectId,
    pub memory_usage_pattern: AllocationPattern,
    pub cpu_usage_pattern: CpuPattern,
    pub io_pattern: IoPattern,
    pub network_pattern: NetworkPattern,
    pub confidence: f64,
    pub samples: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationPattern {
    pub avg_allocation_size_bytes: u64,
    pub peak_allocation_bytes: u64,
    pub allocation_rate_per_second: f64,
    pub fragmentation_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuPattern {
    pub avg_utilization_percent: f64,
    pub peak_utilization_percent: f64,
    pub context_switch_rate: f64,
    pub cache_miss_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoPattern {
    pub avg_io_per_second: f64,
    pub peak_io_per_second: f64,
    pub read_write_ratio: f64,
    pub avg_io_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPattern {
    pub avg_bandwidth_mbps: f64,
    pub peak_bandwidth_mbps: f64,
    pub connection_churn_rate: f64,
    pub packet_loss_percent: f64,
}

// ============================================================================
// PREDICTION ENGINE
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct PredictiveAllocator {
    pub profiles: HashMap<ObjectId, ResourceProfile>,
    pub history_window: usize,
    pub prediction_horizon_ms: u64,
    pub learning_rate: f64,
}

impl PredictiveAllocator {
    pub fn new() -> Self {
        PredictiveAllocator {
            profiles: HashMap::new(),
            history_window: 100,
            prediction_horizon_ms: 1000,  // 1 second ahead
            learning_rate: 0.1,
        }
    }

    /// Update resource profile with new observations
    pub fn update_profile(&mut self, driver_id: ObjectId, observation: &ResourceObservation) {
        let profile = self
            .profiles
            .entry(driver_id)
            .or_insert_with(|| ResourceProfile {
                driver_id,
                memory_usage_pattern: AllocationPattern {
                    avg_allocation_size_bytes: 0,
                    peak_allocation_bytes: 0,
                    allocation_rate_per_second: 0.0,
                    fragmentation_ratio: 0.0,
                },
                cpu_usage_pattern: CpuPattern {
                    avg_utilization_percent: 0.0,
                    peak_utilization_percent: 0.0,
                    context_switch_rate: 0.0,
                    cache_miss_rate: 0.0,
                },
                io_pattern: IoPattern {
                    avg_io_per_second: 0.0,
                    peak_io_per_second: 0.0,
                    read_write_ratio: 0.5,
                    avg_io_size_bytes: 0,
                },
                network_pattern: NetworkPattern {
                    avg_bandwidth_mbps: 0.0,
                    peak_bandwidth_mbps: 0.0,
                    connection_churn_rate: 0.0,
                    packet_loss_percent: 0.0,
                },
                confidence: 0.0,
                samples: 0,
            });

        // Update memory pattern using exponential moving average
        profile.memory_usage_pattern.avg_allocation_size_bytes = (
            (profile.memory_usage_pattern.avg_allocation_size_bytes as f64 * (1.0 - self.learning_rate))
                + (observation.allocated_bytes as f64 * self.learning_rate)
        ) as u64;

        profile.memory_usage_pattern.peak_allocation_bytes =
            profile.memory_usage_pattern.peak_allocation_bytes.max(observation.allocated_bytes);

        profile.memory_usage_pattern.allocation_rate_per_second =
            profile.memory_usage_pattern.avg_allocation_size_bytes as f64 / 1024.0;

        // Update CPU pattern
        profile.cpu_usage_pattern.avg_utilization_percent =
            profile.cpu_usage_pattern.avg_utilization_percent * (1.0 - self.learning_rate)
                + observation.cpu_usage_percent * self.learning_rate;

        profile.cpu_usage_pattern.peak_utilization_percent =
            profile.cpu_usage_pattern.peak_utilization_percent.max(observation.cpu_usage_percent);

        // Update I/O pattern
        profile.io_pattern.avg_io_per_second =
            profile.io_pattern.avg_io_per_second * (1.0 - self.learning_rate) + observation.io_ops_per_sec * self.learning_rate;

        profile.io_pattern.peak_io_per_second = profile.io_pattern.peak_io_per_second.max(observation.io_ops_per_sec);

        // Update network pattern
        profile.network_pattern.avg_bandwidth_mbps = profile.network_pattern.avg_bandwidth_mbps * (1.0 - self.learning_rate)
            + observation.network_bandwidth_mbps * self.learning_rate;

        profile.network_pattern.peak_bandwidth_mbps =
            profile.network_pattern.peak_bandwidth_mbps.max(observation.network_bandwidth_mbps);

        // Increase confidence with more samples
        profile.samples += 1;
        profile.confidence = (profile.samples as f64 / 10.0).min(1.0);
    }

    /// Predict resource needs for driver in next prediction_horizon_ms
    pub fn predict_resources(&self, driver_id: ObjectId) -> Option<ResourcePrediction> {
        let profile = self.profiles.get(&driver_id)?;

        // Conservative prediction: use peak metrics with confidence discount
        let peak_memory = (profile.memory_usage_pattern.peak_allocation_bytes as f64 * profile.confidence) as u64;
        let peak_cpu = profile.cpu_usage_pattern.peak_utilization_percent * profile.confidence;
        let peak_io = profile.io_pattern.peak_io_per_second * profile.confidence;
        let peak_network = profile.network_pattern.peak_bandwidth_mbps * profile.confidence;

        Some(ResourcePrediction {
            driver_id,
            predicted_memory_bytes: peak_memory,
            predicted_cpu_percent: peak_cpu,
            predicted_io_ops_per_sec: peak_io,
            predicted_network_mbps: peak_network,
            confidence: profile.confidence,
            horizon_ms: self.prediction_horizon_ms,
        })
    }

    /// Get allocation recommendation based on predicted needs
    pub fn get_allocation_recommendation(&self, driver_id: ObjectId) -> Option<AllocationRecommendation> {
        let prediction = self.predict_resources(driver_id)?;
        let profile = self.profiles.get(&driver_id)?;

        // Allocate with 20% headroom for spikes
        let recommended_memory = (prediction.predicted_memory_bytes as f64 * 1.2) as u64;

        let priority = match prediction.predicted_cpu_percent {
            p if p > 80.0 => TaskPriority::High,
            p if p > 50.0 => TaskPriority::Normal,
            _ => TaskPriority::Low,
        };

        let cpu_affinity = if profile.cpu_usage_pattern.cache_miss_rate > 0.2 {
            AffinityPreference::SameL3Cache
        } else {
            AffinityPreference::AnyCore
        };

        Some(AllocationRecommendation {
            driver_id,
            recommended_memory_bytes: recommended_memory,
            priority,
            cpu_affinity,
            prefer_local_numa: prediction.predicted_memory_bytes > 1024 * 1024 * 100, // > 100MB
            io_scheduling_class: IoSchedulingClass::from_io_rate(prediction.predicted_io_ops_per_sec),
            confidence: prediction.confidence,
        })
    }

    /// Get all profiles sorted by CPU utilization
    pub fn get_profiles_by_cpu_load(&self) -> Vec<&ResourceProfile> {
        let mut profiles: Vec<_> = self.profiles.values().collect();
        profiles.sort_by(|a, b| b.cpu_usage_pattern.avg_utilization_percent.partial_cmp(&a.cpu_usage_pattern.avg_utilization_percent).unwrap());
        profiles
    }

    /// Get all profiles sorted by memory utilization
    pub fn get_profiles_by_memory_usage(&self) -> Vec<&ResourceProfile> {
        let mut profiles: Vec<_> = self.profiles.values().collect();
        profiles.sort_by(|a, b| b.memory_usage_pattern.peak_allocation_bytes.cmp(&a.memory_usage_pattern.peak_allocation_bytes));
        profiles
    }

    /// Get prediction confidence for driver
    pub fn get_prediction_confidence(&self, driver_id: ObjectId) -> f64 {
        self.profiles
            .get(&driver_id)
            .map(|p| p.confidence)
            .unwrap_or(0.0)
    }

    /// Get statistics
    pub fn get_stats(&self) -> PredictionStats {
        let total_drivers = self.profiles.len();
        let high_confidence = self.profiles.values().filter(|p| p.confidence > 0.8).count();
        let avg_confidence = self.profiles.values().map(|p| p.confidence).sum::<f64>() / total_drivers.max(1) as f64;

        PredictionStats {
            total_drivers: total_drivers as u64,
            high_confidence_profiles: high_confidence as u64,
            avg_confidence,
            total_samples_processed: self.profiles.values().map(|p| p.samples).sum(),
        }
    }
}

// ============================================================================
// RESOURCE OBSERVATION & PREDICTION
// ============================================================================

#[derive(Debug, Clone)]
pub struct ResourceObservation {
    pub allocated_bytes: u64,
    pub cpu_usage_percent: f64,
    pub io_ops_per_sec: f64,
    pub network_bandwidth_mbps: f64,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePrediction {
    pub driver_id: ObjectId,
    pub predicted_memory_bytes: u64,
    pub predicted_cpu_percent: f64,
    pub predicted_io_ops_per_sec: f64,
    pub predicted_network_mbps: f64,
    pub confidence: f64,
    pub horizon_ms: u64,
}

// ============================================================================
// ALLOCATION RECOMMENDATIONS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AffinityPreference {
    AnyCore,
    SameSocket,
    SameL3Cache,
    Specific(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IoSchedulingClass {
    Realtime,
    BestEffort,
    Idle,
}

impl IoSchedulingClass {
    pub fn from_io_rate(io_ops_per_sec: f64) -> Self {
        match io_ops_per_sec {
            rate if rate > 10_000.0 => IoSchedulingClass::Realtime,
            rate if rate > 1_000.0 => IoSchedulingClass::BestEffort,
            _ => IoSchedulingClass::Idle,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationRecommendation {
    pub driver_id: ObjectId,
    pub recommended_memory_bytes: u64,
    pub priority: TaskPriority,
    pub cpu_affinity: AffinityPreference,
    pub prefer_local_numa: bool,
    pub io_scheduling_class: IoSchedulingClass,
    pub confidence: f64,
}

// ============================================================================
// PREDICTION STATISTICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionStats {
    pub total_drivers: u64,
    pub high_confidence_profiles: u64,
    pub avg_confidence: f64,
    pub total_samples_processed: u64,
}
