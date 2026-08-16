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

pub mod adaptive_scheduling;
pub mod anomaly_detection;
pub mod continuous_learning;
pub mod inference;
pub mod inference_engine;
pub mod monitoring;
pub mod optimization;
pub mod predictive_allocation;
pub mod reinforcement_learning;

#[cfg(test)]
mod tests;

pub use adaptive_scheduling::{
    AdaptiveScheduler, SchedulingDecision, WorkloadClassifier, WorkloadType,
};
pub use anomaly_detection::{Anomaly, AnomalyEngine, AnomalySeverity, AnomalyType};
pub use continuous_learning::{ContinuousLearningEngine, DriverBehaviorModel, RuntimeObservation};
pub use inference::InferenceEngine as InferenceEngineOld;
pub use inference_engine::{FeatureVector, InferenceDecision, InferenceEngine, InferenceRequest};
pub use monitoring::AiMonitor;
pub use optimization::ResourceOptimizer;
pub use predictive_allocation::{AllocationRecommendation, PredictiveAllocator, ResourceProfile};
pub use reinforcement_learning::{DriverLearner, ReinforcementLearner, RewardEvent, RewardSignal};
