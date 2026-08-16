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

    /// Classify a total-memory figure (in MB) into the tier whose band it
    /// falls into. Memory below Tier 0's floor still returns `Tier0Embedded`
    /// (there is nothing smaller to fall back to).
    pub fn from_mb(total_mb: u32) -> Self {
        if total_mb >= Self::Tier4Workstation.min_memory_mb() {
            Self::Tier4Workstation
        } else if total_mb >= Self::Tier3Desktop.min_memory_mb() {
            Self::Tier3Desktop
        } else if total_mb >= Self::Tier2Light.min_memory_mb() {
            Self::Tier2Light
        } else if total_mb >= Self::Tier1Iot.min_memory_mb() {
            Self::Tier1Iot
        } else {
            Self::Tier0Embedded
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_mb_classifies_each_band() {
        assert_eq!(MemoryTier::from_mb(64), MemoryTier::Tier0Embedded);
        assert_eq!(MemoryTier::from_mb(512), MemoryTier::Tier1Iot);
        assert_eq!(MemoryTier::from_mb(2048), MemoryTier::Tier2Light);
        assert_eq!(MemoryTier::from_mb(8192), MemoryTier::Tier3Desktop);
        assert_eq!(MemoryTier::from_mb(65536), MemoryTier::Tier4Workstation);
    }

    #[test]
    fn from_mb_is_monotonic_at_band_edges() {
        assert_eq!(MemoryTier::from_mb(8191), MemoryTier::Tier2Light);
        assert_eq!(MemoryTier::from_mb(8192), MemoryTier::Tier3Desktop);
    }
}
