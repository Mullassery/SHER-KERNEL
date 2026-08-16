use serde::{Deserialize, Serialize};
use sher_common::Result;

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
            return Err(sher_common::Error::Memory(format!(
                "Insufficient memory: requested {}, available {}",
                size, self.free
            )));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_reduces_free_and_increases_allocated() {
        let mut alloc = MemoryAllocator::new(1000);
        alloc.allocate(300).unwrap();
        assert_eq!(alloc.allocated, 300);
        assert_eq!(alloc.free, 700);
    }

    #[test]
    fn allocate_beyond_available_fails() {
        let mut alloc = MemoryAllocator::new(100);
        assert!(alloc.allocate(200).is_err());
        // Failed allocation must not mutate state.
        assert_eq!(alloc.allocated, 0);
    }

    #[test]
    fn deallocate_returns_memory_to_free_pool() {
        let mut alloc = MemoryAllocator::new(1000);
        alloc.allocate(400).unwrap();
        alloc.deallocate(400).unwrap();
        assert_eq!(alloc.allocated, 0);
        assert_eq!(alloc.free, 1000);
    }

    #[test]
    fn deallocate_saturates_rather_than_underflows() {
        let mut alloc = MemoryAllocator::new(1000);
        alloc.deallocate(500).unwrap();
        assert_eq!(alloc.allocated, 0);
    }

    #[test]
    fn usage_percent_reflects_allocation() {
        let mut alloc = MemoryAllocator::new(1000);
        alloc.allocate(250).unwrap();
        assert!((alloc.usage_percent() - 25.0).abs() < f64::EPSILON);
    }
}
