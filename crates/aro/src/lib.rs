//! SHER Adaptive Resource Orchestrator (ARO)
//!
//! Detects available hardware and dynamically enables/disables kernel features:
//! - 128 MB - 512 MB (Tier 0): Embedded/IoT only
//! - 512 MB - 2 GB (Tier 1): Minimal IoT
//! - 2 GB - 8 GB (Tier 2): Light desktop
//! - 8 GB - 32 GB (Tier 3): Desktop
//! - 32 GB+ (Tier 4): Workstation/AI
//!
//! Same kernel binary scales across entire spectrum.

pub mod adapter;
pub mod budget;
pub mod feature_matrix;
pub mod profiler;
pub mod tier;

use sher_common::Result;
use tracing::info;

pub use adapter::{AdaptationDecision, RuntimeAdapter, ThermalState};
pub use budget::ResourceBudget;
pub use feature_matrix::FeatureMatrix;
pub use tier::MemoryTier;

pub struct AroConfig {
    pub continuous_adaptation: bool,
    pub battery_aware: bool,
    pub thermal_aware: bool,
}

impl Default for AroConfig {
    fn default() -> Self {
        Self {
            continuous_adaptation: true,
            battery_aware: true,
            thermal_aware: true,
        }
    }
}

pub struct AdaptiveResourceOrchestrator {
    config: AroConfig,
    tier: MemoryTier,
    budget: ResourceBudget,
    features: FeatureMatrix,
    adapter: RuntimeAdapter,
    last_decision: Option<AdaptationDecision>,
}

impl AdaptiveResourceOrchestrator {
    pub async fn initialize(config: AroConfig) -> Result<Self> {
        info!("ARO: Profiling hardware...");
        let tier = profiler::detect_memory_tier()?;
        let budget = profiler::calculate_budget(&tier)?;
        let features = FeatureMatrix::for_tier(&tier);

        info!("ARO: Detected tier: {:?}", tier);
        info!("ARO: Allocated budget: {:?}", budget);

        Ok(Self {
            config,
            tier,
            budget,
            features,
            adapter: RuntimeAdapter::new(),
            last_decision: None,
        })
    }

    pub fn features(&self) -> &FeatureMatrix {
        &self.features
    }

    pub fn tier(&self) -> MemoryTier {
        self.tier
    }

    pub fn budget(&self) -> &ResourceBudget {
        &self.budget
    }

    pub fn last_decision(&self) -> Option<&AdaptationDecision> {
        self.last_decision.as_ref()
    }

    /// Feed a battery-power signal through to the runtime adapter, honoring
    /// `AroConfig::battery_aware`.
    pub fn adapt_to_battery(&mut self, charging: bool) {
        if self.config.battery_aware {
            let decision = self.adapter.adapt_to_battery(charging);
            info!("ARO: battery adaptation: {}", decision.reason);
            self.last_decision = Some(decision);
        }
    }

    /// Feed a thermal signal through to the runtime adapter, honoring
    /// `AroConfig::thermal_aware`.
    pub fn adapt_to_temperature(&mut self, celsius: f64) {
        if self.config.thermal_aware {
            let decision = self.adapter.adapt_to_temperature(celsius);
            info!("ARO: thermal adaptation: {}", decision.reason);
            self.last_decision = Some(decision);
        }
    }

    /// Shrink the cache budget under sustained memory pressure
    /// (`memory_pressure` in `0.0..=1.0`). Real bookkeeping: this actually
    /// mutates `self.budget`, it does not just log.
    pub async fn adapt_to_pressure(&mut self, memory_pressure: f64) -> Result<()> {
        if !self.config.continuous_adaptation {
            return Ok(());
        }
        let pressure = memory_pressure.clamp(0.0, 1.0);
        if pressure > 0.8 {
            let original = self.budget.cache_memory_mb;
            self.budget.cache_memory_mb = (original as f64 * 0.5) as u32;
            info!(
                "ARO: high memory pressure ({:.0}%), shrinking cache budget {}MB -> {}MB",
                pressure * 100.0,
                original,
                self.budget.cache_memory_mb
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn initialize_detects_a_tier_and_matching_budget() {
        let aro = AdaptiveResourceOrchestrator::initialize(AroConfig::default())
            .await
            .unwrap();
        assert!(aro.budget().total_memory_mb > 0);
        let expected = ResourceBudget::for_tier(&aro.tier());
        assert_eq!(aro.budget().total_memory_mb, expected.total_memory_mb);
    }

    #[tokio::test]
    async fn high_pressure_shrinks_cache_budget() {
        let mut aro = AdaptiveResourceOrchestrator::initialize(AroConfig::default())
            .await
            .unwrap();
        let before = aro.budget().cache_memory_mb;
        aro.adapt_to_pressure(0.95).await.unwrap();
        assert!(aro.budget().cache_memory_mb < before);
    }

    #[tokio::test]
    async fn low_pressure_leaves_budget_untouched() {
        let mut aro = AdaptiveResourceOrchestrator::initialize(AroConfig::default())
            .await
            .unwrap();
        let before = aro.budget().cache_memory_mb;
        aro.adapt_to_pressure(0.1).await.unwrap();
        assert_eq!(aro.budget().cache_memory_mb, before);
    }

    #[tokio::test]
    async fn battery_awareness_respects_config_flag() {
        let mut config = AroConfig::default();
        config.battery_aware = false;
        let mut aro = AdaptiveResourceOrchestrator::initialize(config)
            .await
            .unwrap();
        aro.adapt_to_battery(false);
        assert!(aro.last_decision().is_none());
    }

    #[tokio::test]
    async fn battery_awareness_records_decision_when_enabled() {
        let mut aro = AdaptiveResourceOrchestrator::initialize(AroConfig::default())
            .await
            .unwrap();
        aro.adapt_to_battery(false);
        assert!(aro.last_decision().unwrap().reduce_background_work);
    }
}
