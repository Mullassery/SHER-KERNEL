use crate::tier::MemoryTier;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceBudget {
    pub total_memory_mb: u32,
    pub kernel_memory_mb: u32,
    pub cache_memory_mb: u32,
    pub reserve_memory_mb: u32,
}

impl ResourceBudget {
    pub fn for_tier(tier: &MemoryTier) -> Self {
        match tier {
            MemoryTier::Tier0Embedded => Self {
                total_memory_mb: 256,
                kernel_memory_mb: 64,
                cache_memory_mb: 32,
                reserve_memory_mb: 64,
            },
            MemoryTier::Tier1Iot => Self {
                total_memory_mb: 1024,
                kernel_memory_mb: 128,
                cache_memory_mb: 256,
                reserve_memory_mb: 128,
            },
            MemoryTier::Tier2Light => Self {
                total_memory_mb: 4096,
                kernel_memory_mb: 256,
                cache_memory_mb: 1024,
                reserve_memory_mb: 256,
            },
            MemoryTier::Tier3Desktop => Self {
                total_memory_mb: 16384,
                kernel_memory_mb: 512,
                cache_memory_mb: 4096,
                reserve_memory_mb: 1024,
            },
            MemoryTier::Tier4Workstation => Self {
                total_memory_mb: 65536,
                kernel_memory_mb: 1024,
                cache_memory_mb: 16384,
                reserve_memory_mb: 4096,
            },
        }
    }
}
