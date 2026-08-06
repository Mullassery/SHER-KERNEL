//! SHER AI-Native Kernel Services
//!
//! Artificial Intelligence is part of the operating system:
//! - AI inference
//! - AI scheduling
//! - AI memory management
//! - AI security
//! - Semantic indexing
//! - Predictive resource allocation
//! - Autonomous optimization

pub mod inference;
pub mod monitoring;
pub mod optimization;
pub mod anomaly_detection;
pub mod predictive_allocation;
pub mod adaptive_scheduling;
pub mod continuous_learning;
pub mod inference_engine;
pub mod reinforcement_learning;

#[cfg(test)]
mod tests;

pub use inference::InferenceEngine as InferenceEngineOld;
pub use monitoring::AiMonitor;
pub use optimization::ResourceOptimizer;
pub use anomaly_detection::{AnomalyEngine, Anomaly, AnomalyType, AnomalySeverity};
pub use predictive_allocation::{PredictiveAllocator, ResourceProfile, AllocationRecommendation};
pub use adaptive_scheduling::{AdaptiveScheduler, SchedulingDecision, WorkloadClassifier, WorkloadType};
pub use continuous_learning::{ContinuousLearningEngine, DriverBehaviorModel, RuntimeObservation};
pub use inference_engine::{InferenceEngine, InferenceRequest, InferenceDecision, FeatureVector};
pub use reinforcement_learning::{ReinforcementLearner, DriverLearner, RewardEvent, RewardSignal};
