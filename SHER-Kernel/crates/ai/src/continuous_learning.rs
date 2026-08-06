// SHER AI Services: Continuous Learning System
// Real-time learning from runtime observations and optimization feedback

use sher_common::{ObjectId, Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

// ============================================================================
// OBSERVATION TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeObservation {
    pub driver_id: ObjectId,
    pub timestamp_ms: u64,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub io_throughput: f64,
    pub network_throughput: f64,
    pub latency_ms: f64,
    pub anomalies_detected: u32,
    pub task_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub driver_id: ObjectId,
    pub change_type: String,           // e.g., "cpu_affinity", "memory_limit"
    pub before_metric: f64,
    pub after_metric: f64,
    pub improvement_percent: f64,
    pub applied_at_ms: u64,
}

// ============================================================================
// LEARNING MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverBehaviorModel {
    pub driver_id: ObjectId,
    pub observations: VecDeque<RuntimeObservation>,
    pub max_observations: usize,

    // Learned patterns
    pub peak_cpu_usage: f64,
    pub avg_cpu_usage: f64,
    pub peak_memory_usage: f64,
    pub avg_memory_usage: f64,
    pub peak_latency_ms: f64,
    pub avg_latency_ms: f64,

    // Correlation coefficients
    pub cpu_memory_correlation: f64,
    pub cpu_latency_correlation: f64,
    pub io_latency_correlation: f64,

    // Trend analysis
    pub cpu_trend: f64,                 // -1.0 to 1.0, negative = decreasing
    pub memory_trend: f64,
    pub latency_trend: f64,

    pub samples: u64,
}

impl DriverBehaviorModel {
    pub fn new(driver_id: ObjectId) -> Self {
        DriverBehaviorModel {
            driver_id,
            observations: VecDeque::new(),
            max_observations: 100,
            peak_cpu_usage: 0.0,
            avg_cpu_usage: 0.0,
            peak_memory_usage: 0.0,
            avg_memory_usage: 0.0,
            peak_latency_ms: 0.0,
            avg_latency_ms: 0.0,
            cpu_memory_correlation: 0.0,
            cpu_latency_correlation: 0.0,
            io_latency_correlation: 0.0,
            cpu_trend: 0.0,
            memory_trend: 0.0,
            latency_trend: 0.0,
            samples: 0,
        }
    }

    /// Add observation and update model
    pub fn observe(&mut self, observation: RuntimeObservation) {
        // Maintain circular buffer
        if self.observations.len() >= self.max_observations {
            self.observations.pop_front();
        }
        self.observations.push_back(observation.clone());

        // Update peaks
        self.peak_cpu_usage = self.peak_cpu_usage.max(observation.cpu_usage);
        self.peak_memory_usage = self.peak_memory_usage.max(observation.memory_usage);
        self.peak_latency_ms = self.peak_latency_ms.max(observation.latency_ms);

        // Update moving averages
        let alpha = 0.1;
        self.avg_cpu_usage = self.avg_cpu_usage * (1.0 - alpha) + observation.cpu_usage * alpha;
        self.avg_memory_usage = self.avg_memory_usage * (1.0 - alpha) + observation.memory_usage * alpha;
        self.avg_latency_ms = self.avg_latency_ms * (1.0 - alpha) + observation.latency_ms * alpha;

        self.samples += 1;

        // Recalculate correlations if we have enough samples
        if self.observations.len() > 10 {
            self.recalculate_correlations();
            self.recalculate_trends();
        }
    }

    fn recalculate_correlations(&mut self) {
        let obs: Vec<_> = self.observations.iter().cloned().collect();
        if obs.len() < 2 {
            return;
        }

        // Calculate correlation between CPU and memory
        self.cpu_memory_correlation = calculate_correlation(
            obs.iter().map(|o| o.cpu_usage).collect(),
            obs.iter().map(|o| o.memory_usage).collect(),
        );

        // Calculate correlation between CPU and latency
        self.cpu_latency_correlation = calculate_correlation(
            obs.iter().map(|o| o.cpu_usage).collect(),
            obs.iter().map(|o| o.latency_ms).collect(),
        );

        // Calculate correlation between I/O and latency
        self.io_latency_correlation = calculate_correlation(
            obs.iter().map(|o| o.io_throughput).collect(),
            obs.iter().map(|o| o.latency_ms).collect(),
        );
    }

    fn recalculate_trends(&mut self) {
        let obs: Vec<_> = self.observations.iter().cloned().collect();
        if obs.len() < 3 {
            return;
        }

        // Simple linear trend: compare first third vs last third
        let third = obs.len() / 3;
        let first_third_avg = obs[..third].iter().map(|o| o.cpu_usage).sum::<f64>() / third as f64;
        let last_third_avg = obs[obs.len() - third..].iter().map(|o| o.cpu_usage).sum::<f64>() / third as f64;
        self.cpu_trend = ((last_third_avg - first_third_avg) / first_third_avg).clamp(-1.0, 1.0);

        let first_third_avg = obs[..third].iter().map(|o| o.memory_usage).sum::<f64>() / third as f64;
        let last_third_avg = obs[obs.len() - third..].iter().map(|o| o.memory_usage).sum::<f64>() / third as f64;
        self.memory_trend = ((last_third_avg - first_third_avg) / first_third_avg).clamp(-1.0, 1.0);

        let first_third_avg = obs[..third].iter().map(|o| o.latency_ms).sum::<f64>() / third as f64;
        let last_third_avg = obs[obs.len() - third..].iter().map(|o| o.latency_ms).sum::<f64>() / third as f64;
        self.latency_trend = ((last_third_avg - first_third_avg) / first_third_avg).clamp(-1.0, 1.0);
    }

    /// Predict next CPU usage based on trend
    pub fn predict_cpu_usage(&self) -> f64 {
        self.avg_cpu_usage * (1.0 + self.cpu_trend * 0.1)
    }

    /// Predict next memory usage based on trend
    pub fn predict_memory_usage(&self) -> f64 {
        self.avg_memory_usage * (1.0 + self.memory_trend * 0.1)
    }

    /// Check if behavior is anomalous compared to learned model
    pub fn is_anomalous(&self, observation: &RuntimeObservation) -> bool {
        if self.samples < 10 {
            return false; // Not enough data
        }

        let cpu_deviation = (observation.cpu_usage - self.avg_cpu_usage).abs();
        let mem_deviation = (observation.memory_usage - self.avg_memory_usage).abs();
        let lat_deviation = (observation.latency_ms - self.avg_latency_ms).abs();

        // Consider anomalous if deviation is > 2x average
        cpu_deviation > self.avg_cpu_usage * 2.0 ||
            mem_deviation > self.avg_memory_usage * 2.0 ||
            lat_deviation > self.avg_latency_ms * 2.0
    }
}

fn calculate_correlation(x: Vec<f64>, y: Vec<f64>) -> f64 {
    if x.len() < 2 || x.len() != y.len() {
        return 0.0;
    }

    let n = x.len() as f64;
    let x_mean = x.iter().sum::<f64>() / n;
    let y_mean = y.iter().sum::<f64>() / n;

    let covariance = x.iter()
        .zip(y.iter())
        .map(|(xi, yi)| (xi - x_mean) * (yi - y_mean))
        .sum::<f64>() / n;

    let x_var = x.iter().map(|xi| (xi - x_mean).powi(2)).sum::<f64>() / n;
    let y_var = y.iter().map(|yi| (yi - y_mean).powi(2)).sum::<f64>() / n;

    let correlation = covariance / (x_var * y_var).sqrt();
    correlation.clamp(-1.0, 1.0)
}

// ============================================================================
// CONTINUOUS LEARNING ENGINE
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct ContinuousLearningEngine {
    pub behavior_models: HashMap<ObjectId, DriverBehaviorModel>,
    pub optimization_history: Vec<OptimizationResult>,
    pub max_history: usize,
    pub total_observations: u64,
    pub optimizations_applied: u64,
}

impl ContinuousLearningEngine {
    pub fn new() -> Self {
        ContinuousLearningEngine {
            behavior_models: HashMap::new(),
            optimization_history: Vec::new(),
            max_history: 500,
            total_observations: 0,
            optimizations_applied: 0,
        }
    }

    /// Process runtime observation
    pub fn observe(&mut self, observation: RuntimeObservation) {
        let driver_id = observation.driver_id;

        let model = self.behavior_models
            .entry(driver_id)
            .or_insert_with(|| DriverBehaviorModel::new(driver_id));

        model.observe(observation);
        self.total_observations += 1;
    }

    /// Record optimization result for learning
    pub fn record_optimization(&mut self, result: OptimizationResult) {
        if self.optimization_history.len() >= self.max_history {
            self.optimization_history.remove(0);
        }
        self.optimization_history.push(result);
        self.optimizations_applied += 1;
    }

    /// Get behavior model for driver
    pub fn get_model(&self, driver_id: ObjectId) -> Option<&DriverBehaviorModel> {
        self.behavior_models.get(&driver_id)
    }

    /// Get drivers sorted by anomaly risk (based on learned deviations)
    pub fn get_high_risk_drivers(&self) -> Vec<(ObjectId, f64)> {
        let mut drivers: Vec<_> = self.behavior_models
            .iter()
            .filter_map(|(id, model)| {
                if model.samples > 10 {
                    let risk = (model.cpu_trend.abs() + model.memory_trend.abs() + model.latency_trend.abs()) / 3.0;
                    Some((*id, risk))
                } else {
                    None
                }
            })
            .collect();

        drivers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        drivers
    }

    /// Get optimization effectiveness
    pub fn get_optimization_stats(&self) -> OptimizationStats {
        let total_optimizations = self.optimization_history.len() as u64;
        let successful = self.optimization_history
            .iter()
            .filter(|o| o.improvement_percent > 0.0)
            .count() as u64;

        let avg_improvement = if self.optimization_history.is_empty() {
            0.0
        } else {
            self.optimization_history
                .iter()
                .map(|o| o.improvement_percent)
                .sum::<f64>() / self.optimization_history.len() as f64
        };

        OptimizationStats {
            total_optimizations,
            successful_optimizations: successful,
            success_rate: if total_optimizations > 0 {
                successful as f64 / total_optimizations as f64
            } else {
                0.0
            },
            avg_improvement_percent: avg_improvement,
            total_observations: self.total_observations,
        }
    }

    /// Get learning summary
    pub fn get_stats(&self) -> LearningStats {
        LearningStats {
            total_drivers_observed: self.behavior_models.len() as u64,
            total_observations: self.total_observations,
            optimizations_applied: self.optimizations_applied,
            avg_model_confidence: if self.behavior_models.is_empty() {
                0.0
            } else {
                self.behavior_models.values()
                    .map(|m| (m.samples as f64 / 100.0).min(1.0))
                    .sum::<f64>() / self.behavior_models.len() as f64
            },
        }
    }
}

// ============================================================================
// STATISTICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationStats {
    pub total_optimizations: u64,
    pub successful_optimizations: u64,
    pub success_rate: f64,
    pub avg_improvement_percent: f64,
    pub total_observations: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStats {
    pub total_drivers_observed: u64,
    pub total_observations: u64,
    pub optimizations_applied: u64,
    pub avg_model_confidence: f64,
}
