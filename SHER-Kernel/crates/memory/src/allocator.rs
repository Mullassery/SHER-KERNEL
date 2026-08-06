use sher_common::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAllocator {
    pub total_memory: u64,
    pub allocated: u64,
    pub free: u64,
}

impl MemoryAllocator {
    pub fn new(total_memory: u64) -> Self {
        Self {
            total_memory,
            allocated: 0,
            free: total_memory,
        }
    }

    pub fn allocate(&mut self, size: u64) -> Result<u64> {
        if size > self.free {
            return Err(sher_common::Error::Memory(
                format!("Insufficient memory: requested {}, available {}", size, self.free)
            ));
        }
        self.allocated += size;
        self.free -= size;
        Ok(self.allocated - size)
    }

    pub fn deallocate(&mut self, size: u64) -> Result<()> {
        self.allocated = self.allocated.saturating_sub(size);
        self.free += size;
        Ok(())
    }

    pub fn usage_percent(&self) -> f64 {
        (self.allocated as f64 / self.total_memory as f64) * 100.0
    }
}
