//! Fixed-capacity ring buffer for early-boot diagnostics.
//!
//! This is a real, generic, tested data structure (not a hardware facility):
//! it holds the most recent `capacity` diagnostic entries in memory and
//! silently overwrites the oldest entry once full, which is the behavior the
//! SHER staged-boot model wants before a full logging/telemetry subsystem is
//! available (see `crate::telemetry`).

use std::collections::VecDeque;

/// The default capacity used by [`RingBuffer::new`] when no explicit
/// capacity is supplied. Chosen to be small enough for a Stage 0/1 boot
/// budget while still holding a useful amount of history.
pub const DEFAULT_CAPACITY: usize = 256;

#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    capacity: usize,
    buf: VecDeque<T>,
}

impl<T> RingBuffer<T> {
    /// Create a ring buffer with the [`DEFAULT_CAPACITY`].
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    /// Create a ring buffer that holds at most `capacity` entries.
    ///
    /// # Panics
    /// Panics if `capacity` is zero.
    pub fn with_capacity(capacity: usize) -> Self {
        assert!(
            capacity > 0,
            "RingBuffer capacity must be greater than zero"
        );
        Self {
            capacity,
            buf: VecDeque::with_capacity(capacity),
        }
    }

    /// Push a new entry, evicting the oldest entry if the buffer is full.
    /// Returns the evicted entry, if any.
    pub fn push(&mut self, item: T) -> Option<T> {
        let evicted = if self.buf.len() == self.capacity {
            self.buf.pop_front()
        } else {
            None
        };
        self.buf.push_back(item);
        evicted
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.buf.len() == self.capacity
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Oldest-to-newest iteration order.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.buf.iter()
    }

    /// Remove and return all entries, oldest first, leaving the buffer empty.
    pub fn drain(&mut self) -> Vec<T> {
        self.buf.drain(..).collect()
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }

    pub fn newest(&self) -> Option<&T> {
        self.buf.back()
    }

    pub fn oldest(&self) -> Option<&T> {
        self.buf.front()
    }
}

impl<T> Default for RingBuffer<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_within_capacity_keeps_all_entries() {
        let mut rb = RingBuffer::with_capacity(3);
        assert_eq!(rb.push(1), None);
        assert_eq!(rb.push(2), None);
        assert_eq!(rb.push(3), None);
        assert_eq!(rb.len(), 3);
        assert!(rb.is_full());
        assert_eq!(rb.iter().copied().collect::<Vec<_>>(), vec![1, 2, 3]);
    }

    #[test]
    fn push_beyond_capacity_evicts_oldest() {
        let mut rb = RingBuffer::with_capacity(3);
        rb.push(1);
        rb.push(2);
        rb.push(3);
        assert_eq!(rb.push(4), Some(1));
        assert_eq!(rb.iter().copied().collect::<Vec<_>>(), vec![2, 3, 4]);
        assert_eq!(rb.len(), 3);
    }

    #[test]
    fn oldest_and_newest_track_correctly() {
        let mut rb = RingBuffer::with_capacity(2);
        rb.push("a");
        rb.push("b");
        assert_eq!(rb.oldest(), Some(&"a"));
        assert_eq!(rb.newest(), Some(&"b"));
        rb.push("c");
        assert_eq!(rb.oldest(), Some(&"b"));
        assert_eq!(rb.newest(), Some(&"c"));
    }

    #[test]
    fn drain_empties_buffer_oldest_first() {
        let mut rb = RingBuffer::with_capacity(4);
        for i in 0..3 {
            rb.push(i);
        }
        let drained = rb.drain();
        assert_eq!(drained, vec![0, 1, 2]);
        assert!(rb.is_empty());
    }

    #[test]
    #[should_panic(expected = "capacity must be greater than zero")]
    fn zero_capacity_panics() {
        let _: RingBuffer<u8> = RingBuffer::with_capacity(0);
    }

    #[test]
    fn default_capacity_matches_constant() {
        let rb: RingBuffer<u8> = RingBuffer::default();
        assert_eq!(rb.capacity(), DEFAULT_CAPACITY);
    }
}
