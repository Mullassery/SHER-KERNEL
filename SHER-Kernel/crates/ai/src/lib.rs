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

#[cfg(test)]
mod tests;

pub use inference::InferenceEngine;
pub use monitoring::AiMonitor;
pub use optimization::ResourceOptimizer;
pub use anomaly_detection::{AnomalyEngine, Anomaly, AnomalyType, AnomalySeverity};
pub use predictive_allocation::{PredictiveAllocator, ResourceProfile, AllocationRecommendation};
