use crate::tier::MemoryTier;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureMatrix {
    pub gui_enabled: bool,
    pub ai_runtime_enabled: bool,
    pub large_caches_enabled: bool,
    pub predictive_loading: bool,
    pub gpu_scheduler: bool,
    pub background_indexing: bool,
    pub driver_preloading: bool,
    pub memory_compression: bool,
    pub multi_gpu_support: bool,
}

impl FeatureMatrix {
    pub fn for_tier(tier: &MemoryTier) -> Self {
        match tier {
            MemoryTier::Tier0Embedded => Self {
                gui_enabled: false,
                ai_runtime_enabled: false,
                large_caches_enabled: false,
                predictive_loading: false,
                gpu_scheduler: false,
                background_indexing: false,
                driver_preloading: false,
                memory_compression: true,
                multi_gpu_support: false,
            },
            MemoryTier::Tier1Iot => Self {
                gui_enabled: false,
                ai_runtime_enabled: false,
                large_caches_enabled: false,
                predictive_loading: false,
                gpu_scheduler: false,
                background_indexing: false,
                driver_preloading: false,
                memory_compression: true,
                multi_gpu_support: false,
            },
            MemoryTier::Tier2Light => Self {
                gui_enabled: true,
                ai_runtime_enabled: false,
                large_caches_enabled: false,
                predictive_loading: false,
                gpu_scheduler: false,
                background_indexing: false,
                driver_preloading: false,
                memory_compression: true,
                multi_gpu_support: false,
            },
            MemoryTier::Tier3Desktop => Self {
                gui_enabled: true,
                ai_runtime_enabled: true,
                large_caches_enabled: true,
                predictive_loading: true,
                gpu_scheduler: true,
                background_indexing: false,
                driver_preloading: true,
                memory_compression: false,
                multi_gpu_support: false,
            },
            MemoryTier::Tier4Workstation => Self {
                gui_enabled: true,
                ai_runtime_enabled: true,
                large_caches_enabled: true,
                predictive_loading: true,
                gpu_scheduler: true,
                background_indexing: true,
                driver_preloading: true,
                memory_compression: false,
                multi_gpu_support: true,
            },
        }
    }
}
