use crate::tier::MemoryTier;
use crate::budget::ResourceBudget;
use sher_common::Result;

pub fn detect_memory_tier() -> Result<MemoryTier> {
    Ok(MemoryTier::Tier3Desktop) // Placeholder
}

pub fn calculate_budget(tier: &MemoryTier) -> Result<ResourceBudget> {
    Ok(ResourceBudget::for_tier(tier))
}
