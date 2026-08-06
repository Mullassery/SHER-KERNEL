use sher_common::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxMemoryApi {
    pub kmalloc_count: u32,
    pub kfree_count: u32,
}

impl Default for LinuxMemoryApi {
    fn default() -> Self {
        Self {
            kmalloc_count: 0,
            kfree_count: 0,
        }
    }
}

impl LinuxMemoryApi {
    pub fn kmalloc(&mut self, size: u64) -> Result<u64> {
        self.kmalloc_count += 1;
        Ok(0x1000 * self.kmalloc_count as u64 + size)
    }

    pub fn kfree(&mut self) -> Result<()> {
        self.kfree_count += 1;
        Ok(())
    }
}
