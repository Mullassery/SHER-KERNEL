use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryTier {
    Tier0Embedded,    // 128 MB - 512 MB
    Tier1Iot,         // 512 MB - 2 GB
    Tier2Light,       // 2 GB - 8 GB
    Tier3Desktop,     // 8 GB - 32 GB
    Tier4Workstation, // 32 GB+
}

impl MemoryTier {
    pub fn min_memory_mb(&self) -> u32 {
        match self {
            Self::Tier0Embedded => 128,
            Self::Tier1Iot => 512,
            Self::Tier2Light => 2048,
            Self::Tier3Desktop => 8192,
            Self::Tier4Workstation => 32768,
        }
    }
}
