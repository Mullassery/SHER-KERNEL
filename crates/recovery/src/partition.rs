//! Immutable bootable partitions (System A / System B).
//!
//! Models the A/B image scheme described in the crate root docs: each
//! partition holds one immutable, versioned system image plus a content
//! checksum used to detect corruption before it is ever booted. This is an
//! in-memory simulation of the bookkeeping a real updater would need —
//! writing/reading actual disk partitions is out of scope for a userspace
//! process.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PartitionSlot {
    A,
    B,
}

impl PartitionSlot {
    pub fn other(&self) -> PartitionSlot {
        match self {
            PartitionSlot::A => PartitionSlot::B,
            PartitionSlot::B => PartitionSlot::A,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImmutablePartition {
    pub slot: PartitionSlot,
    pub version: Option<String>,
    image: Vec<u8>,
    /// Non-cryptographic checksum of `image`, recorded at write time.
    checksum: u64,
}

impl ImmutablePartition {
    pub fn empty(slot: PartitionSlot) -> Self {
        Self {
            slot,
            version: None,
            image: Vec::new(),
            checksum: 0,
        }
    }

    /// Write a new immutable image into this partition. Once written the
    /// image is not mutated in place — a new write fully replaces it, which
    /// is how the A/B scheme keeps the previously-active system untouched
    /// while a new one is staged into the other slot.
    pub fn write_image(&mut self, version: impl Into<String>, image: Vec<u8>) {
        self.checksum = Self::checksum_of(&image);
        self.image = image;
        self.version = Some(version.into());
    }

    fn checksum_of(image: &[u8]) -> u64 {
        let mut hasher = DefaultHasher::new();
        image.hash(&mut hasher);
        hasher.finish()
    }

    pub fn is_empty(&self) -> bool {
        self.version.is_none()
    }

    /// Verify the stored image matches its recorded checksum (integrity
    /// check performed before this partition is ever made bootable).
    pub fn verify(&self) -> bool {
        !self.is_empty() && Self::checksum_of(&self.image) == self.checksum
    }

    pub fn image_len(&self) -> usize {
        self.image.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_partition_is_not_verified() {
        let p = ImmutablePartition::empty(PartitionSlot::A);
        assert!(p.is_empty());
        assert!(!p.verify());
    }

    #[test]
    fn written_image_verifies() {
        let mut p = ImmutablePartition::empty(PartitionSlot::B);
        p.write_image("1.2.0", vec![1, 2, 3, 4]);
        assert!(!p.is_empty());
        assert!(p.verify());
        assert_eq!(p.version.as_deref(), Some("1.2.0"));
    }

    #[test]
    fn other_slot_flips() {
        assert_eq!(PartitionSlot::A.other(), PartitionSlot::B);
        assert_eq!(PartitionSlot::B.other(), PartitionSlot::A);
    }
}
