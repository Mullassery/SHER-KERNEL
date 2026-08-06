// SHER AI Services: Inference Engine
// Real-time decision making based on learned patterns and predictive models

use sher_common::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// DECISION TYPES
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InferenceType {
    ResourceAllocation,
    SchedulingStrategy,
    AnomalyResponse,
    OptimizationAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub request_id: ObjectId,
    pub driver_id: ObjectId,
    pub inference_type: InferenceType,
    pub context: HashMap<String, f64>,  // Key metrics for inference
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceDecision {
    pub request_id: ObjectId,
    pub driver_id: ObjectId,
    pub action: String,
    pub confidence: f64,
    pub reasoning: String,
    pub parameters: HashMap<String, f64>,
    pub latency_ms: u64,
}

// ============================================================================
// FEATURE EXTRACTION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureVector {
    pub driver_id: ObjectId,
    pub cpu_usage_normalized: f64,
    pub memory_usage_normalized: f64,
    pub io_intensity: f64,
    pub network_intensity: f64,
    pub anomaly_count: f64,
    pub latency_ms: f64,
    pub slo_violation_rate: f64,
    pub strategy_switches: f64,
}

impl FeatureVector {
    pub fn new(driver_id: ObjectId) -> Self {
        FeatureVector {
            driver_id,
            cpu_usage_normalized: 0.0,
            memory_usage_normalized: 0.0,
            io_intensity: 0.0,
            network_intensity: 0.0,
            anomaly_count: 0.0,
            latency_ms: 0.0,
            slo_violation_rate: 0.0,
            strategy_switches: 0.0,
        }
    }

    /// Extract features from context
    pub fn from_context(driver_id: ObjectId, context: &HashMap<String, f64>) -> Self {
        let mut features = FeatureVector::new(driver_id);

        features.cpu_usage_normalized = (context.get("cpu_usage").copied().unwrap_or(0.0) / 100.0).min(1.0);
        features.memory_usage_normalized = (context.get("memory_usage").copied().unwrap_or(0.0) / 100.0).min(1.0);
        features.io_intensity = (context.get("io_ops_per_sec").copied().unwrap_or(0.0) / 10000.0).min(1.0);
        features.network_intensity = (context.get("network_throughput").copied().unwrap_or(0.0) / 1000.0).min(1.0);
        features.anomaly_count = context.get("anomalies").copied().unwrap_or(0.0).min(10.0) / 10.0;
        features.latency_ms = context.get("latency_ms").copied().unwrap_or(0.0);
        features.slo_violation_rate = context.get("slo_violation_rate").copied().unwrap_or(0.0);
        features.strategy_switches = context.get("strategy_switches").copied().unwrap_or(0.0).min(10.0) / 10.0;

        features
    }

    /// Compute L2 norm of feature vector for similarity
    pub fn norm(&self) -> f64 {
        (
            self.cpu_usage_normalized.powi(2) +
            self.memory_usage_normalized.powi(2) +
            self.io_intensity.powi(2) +
            self.network_intensity.powi(2) +
            self.anomaly_count.powi(2) +
            (self.latency_ms / 100.0).min(1.0).powi(2) +
            self.slo_violation_rate.powi(2) +
            self.strategy_switches.powi(2)
        ).sqrt()
    }
}

// ============================================================================
// INFERENCE ENGINE
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct InferenceEngine {
    pub decisions_made: u64,
    pub total_latency_ms: u64,
    pub high_confidence_decisions: u64,
    pub history: Vec<InferenceDecision>,
    pub max_history: usize,
}

impl InferenceEngine {
    pub fn new() -> Self {
        InferenceEngine {
            decisions_made: 0,
            total_latency_ms: 0,
            high_confidence_decisions: 0,
            history: Vec::new(),
            max_history: 1000,
        }
    }

    /// Make inference decision based on request and features
    pub fn infer(
        &mut self,
        request: &InferenceRequest,
        features: &FeatureVector,
        baseline_confidence: f64,
    ) -> InferenceDecision {
        let start_time = std::time::SystemTime::now();

        let (action, confidence, reasoning, parameters) = match request.inference_type {
            InferenceType::ResourceAllocation => {
                self.infer_resource_allocation(features, baseline_confidence)
            }
            InferenceType::SchedulingStrategy => {
                self.infer_scheduling_strategy(features, baseline_confidence)
            }
            InferenceType::AnomalyResponse => {
                self.infer_anomaly_response(features, baseline_confidence)
            }
            InferenceType::OptimizationAction => {
                self.infer_optimization_action(features, baseline_confidence)
            }
        };

        let latency = start_time.elapsed()
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let decision = InferenceDecision {
            request_id: request.request_id,
            driver_id: request.driver_id,
            action,
            confidence,
            reasoning,
            parameters,
            latency_ms: latency,
        };

        self.decisions_made += 1;
        self.total_latency_ms += latency;
        if confidence > 0.7 {
            self.high_confidence_decisions += 1;
        }

        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(decision.clone());

        decision
    }

    fn infer_resource_allocation(
        &self,
        features: &FeatureVector,
        baseline_confidence: f64,
    ) -> (String, f64, String, HashMap<String, f64>) {
        let mut params = HashMap::new();

        // Allocate more memory if needed
        let memory_allocation = if features.memory_usage_normalized > 0.8 {
            features.memory_usage_normalized * 1.3
        } else if features.memory_usage_normalized > 0.6 {
            features.memory_usage_normalized * 1.15
        } else {
            features.memory_usage_normalized
        };

        params.insert("memory_scale".to_string(), memory_allocation);

        // CPU quota based on current usage and anomalies
        let cpu_quota = if features.anomaly_count > 0.5 {
            (features.cpu_usage_normalized * 0.8).min(1.0)  // Reduce if unstable
        } else {
            (features.cpu_usage_normalized * 1.1).min(1.0)
        };

        params.insert("cpu_quota".to_string(), cpu_quota);

        let confidence = (baseline_confidence * 0.95).max(0.5);
        let reasoning = format!(
            "Allocating {}% CPU quota, {}x memory scale based on {} anomalies",
            (cpu_quota * 100.0) as u32,
            (memory_allocation * 100.0) as u32 / 100,
            (features.anomaly_count * 10.0) as u32
        );

        ("allocate_resources".to_string(), confidence, reasoning, params)
    }

    fn infer_scheduling_strategy(
        &self,
        features: &FeatureVector,
        baseline_confidence: f64,
    ) -> (String, f64, String, HashMap<String, f64>) {
        let mut params = HashMap::new();

        let (strategy, reason) = if features.anomaly_count > 0.6 {
            ("conservative", "High anomaly rate detected")
        } else if features.latency_ms > 100.0 && features.slo_violation_rate > 0.2 {
            ("realtime", "SLO violations and high latency")
        } else if features.cpu_usage_normalized > 0.75 && features.anomaly_count < 0.3 {
            ("aggressive", "High CPU utilization with stability")
        } else {
            ("balanced", "Normal operation")
        };

        params.insert("strategy_score".to_string(), features.cpu_usage_normalized + features.io_intensity);
        params.insert("latency_priority".to_string(), (features.latency_ms / 100.0).min(1.0));

        let confidence = (baseline_confidence * 0.92).max(0.5);

        (
            format!("use_{}_strategy", strategy),
            confidence,
            reason.to_string(),
            params,
        )
    }

    fn infer_anomaly_response(
        &self,
        features: &FeatureVector,
        baseline_confidence: f64,
    ) -> (String, f64, String, HashMap<String, f64>) {
        let mut params = HashMap::new();

        let (action, severity) = match (features.anomaly_count, features.slo_violation_rate) {
            (a, _) if a > 0.8 => ("isolate_driver", 3),
            (a, slo) if a > 0.6 && slo > 0.3 => ("throttle_driver", 2),
            (a, _) if a > 0.3 => ("monitor_closely", 1),
            _ => ("allow_normal", 0),
        };

        params.insert("severity_level".to_string(), severity as f64);
        params.insert("isolation_threshold".to_string(), 0.8);

        let confidence = (baseline_confidence * (1.0 - features.anomaly_count)).max(0.5);
        let reasoning = format!(
            "Anomaly severity: {} (confidence: {:.2})",
            severity,
            confidence
        );

        (action.to_string(), confidence, reasoning, params)
    }

    fn infer_optimization_action(
        &self,
        features: &FeatureVector,
        baseline_confidence: f64,
    ) -> (String, f64, String, HashMap<String, f64>) {
        let mut params = HashMap::new();

        // Determine optimization priority
        let optimization_score = features.cpu_usage_normalized +
                                features.memory_usage_normalized +
                                features.slo_violation_rate;

        let (action, priority) = if optimization_score > 2.0 {
            ("urgent_optimization", 3)
        } else if optimization_score > 1.5 {
            ("schedule_optimization", 2)
        } else if optimization_score > 1.0 {
            ("continuous_optimization", 1)
        } else {
            ("standard_tuning", 0)
        };

        params.insert("optimization_priority".to_string(), priority as f64);
        params.insert("improvement_target".to_string(), (optimization_score * 0.15).min(0.5));

        let confidence = (baseline_confidence * 0.90).max(0.5);
        let reasoning = format!(
            "Optimization priority {} with target {}% improvement",
            priority,
            (params.get("improvement_target").unwrap_or(&0.0) * 100.0) as u32
        );

        (action.to_string(), confidence, reasoning, params)
    }

    /// Get average decision latency
    pub fn get_avg_latency(&self) -> f64 {
        if self.decisions_made == 0 {
            0.0
        } else {
            self.total_latency_ms as f64 / self.decisions_made as f64
        }
    }

    /// Get statistics
    pub fn get_stats(&self) -> InferenceStats {
        InferenceStats {
            total_decisions: self.decisions_made,
            avg_latency_ms: self.get_avg_latency(),
            high_confidence_decisions: self.high_confidence_decisions,
            high_confidence_rate: if self.decisions_made > 0 {
                self.high_confidence_decisions as f64 / self.decisions_made as f64
            } else {
                0.0
            },
        }
    }
}

// ============================================================================
// STATISTICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceStats {
    pub total_decisions: u64,
    pub avg_latency_ms: f64,
    pub high_confidence_decisions: u64,
    pub high_confidence_rate: f64,
}
