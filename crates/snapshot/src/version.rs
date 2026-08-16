//! A single immutable, versioned snapshot of a component.

use sher_common::ObjectId;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub id: ObjectId,
    pub component: String,
    pub version: u32,
    pub label: String,
    pub created_at: u64,
    pub data: Vec<u8>,
    /// Non-cryptographic content checksum (std `DefaultHasher`), used only
    /// to detect accidental corruption between snapshots in this in-memory
    /// simulation — not a security guarantee.
    pub checksum: u64,
}

impl Snapshot {
    pub fn new(
        component: impl Into<String>,
        version: u32,
        label: impl Into<String>,
        data: Vec<u8>,
    ) -> Self {
        let checksum = Self::checksum_of(&data);
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id: ObjectId::new(),
            component: component.into(),
            version,
            label: label.into(),
            created_at,
            data,
            checksum,
        }
    }

    fn checksum_of(data: &[u8]) -> u64 {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    /// Verify the snapshot's stored data has not been mutated since capture.
    pub fn is_intact(&self) -> bool {
        Self::checksum_of(&self.data) == self.checksum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_snapshot_is_intact() {
        let snap = Snapshot::new("browser", 12, "v12", vec![1, 2, 3]);
        assert!(snap.is_intact());
        assert_eq!(snap.version, 12);
    }

    #[test]
    fn tampering_breaks_integrity_check() {
        let mut snap = Snapshot::new("browser", 12, "v12", vec![1, 2, 3]);
        snap.data.push(4);
        assert!(!snap.is_intact());
    }
}
