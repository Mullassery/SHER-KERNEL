//! Kernel heap bring-up (Stage 0).
//!
//! **Simulation notice**: a real kernel heap allocator manages physical
//! pages this process does not own. What's implemented here for real is
//! the *bookkeeping* a bump allocator needs (current offset vs. capacity) —
//! useful for testing higher layers against, not for backing real
//! allocations.

use sher_common::{Error, Result};

#[derive(Debug, Clone)]
pub struct HeapState {
    pub capacity: u64,
    offset: u64,
}

impl HeapState {
    pub fn new(capacity: u64) -> Self {
        Self {
            capacity,
            offset: 0,
        }
    }

    /// Bump-allocate `size` bytes, returning the offset it was placed at.
    pub fn bump(&mut self, size: u64) -> Result<u64> {
        let new_offset = self.offset.checked_add(size).ok_or(Error::OutOfMemory)?;
        if new_offset > self.capacity {
            return Err(Error::OutOfMemory);
        }
        let placed_at = self.offset;
        self.offset = new_offset;
        Ok(placed_at)
    }

    pub fn used(&self) -> u64 {
        self.offset
    }

    pub fn remaining(&self) -> u64 {
        self.capacity - self.offset
    }
}

pub fn initialize(size: u64) -> Result<HeapState> {
    // Real kernel heap allocator initialization (simulated — see module
    // docs). This process cannot own physical memory pages, so this
    // returns purely in-process bookkeeping.
    Ok(HeapState::new(size))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_returns_empty_heap_of_requested_capacity() {
        let heap = initialize(1024).unwrap();
        assert_eq!(heap.capacity, 1024);
        assert_eq!(heap.used(), 0);
        assert_eq!(heap.remaining(), 1024);
    }

    #[test]
    fn bump_allocates_sequentially() {
        let mut heap = HeapState::new(100);
        assert_eq!(heap.bump(40).unwrap(), 0);
        assert_eq!(heap.bump(40).unwrap(), 40);
        assert_eq!(heap.used(), 80);
    }

    #[test]
    fn bump_beyond_capacity_errors() {
        let mut heap = HeapState::new(100);
        heap.bump(90).unwrap();
        assert!(matches!(heap.bump(20), Err(Error::OutOfMemory)));
        // Failed allocation must not consume capacity.
        assert_eq!(heap.used(), 90);
    }
}
