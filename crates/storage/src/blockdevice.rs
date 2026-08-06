use sher_common::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDevice {
    pub id: ObjectId,
    pub sector_size: u64,
    pub sector_count: u64,
}

impl BlockDevice {
    pub fn new(sector_size: u64, sector_count: u64) -> Self {
        Self {
            id: ObjectId::new(),
            sector_size,
            sector_count,
        }
    }

    pub fn total_size(&self) -> u64 {
        self.sector_size * self.sector_count
    }
}
