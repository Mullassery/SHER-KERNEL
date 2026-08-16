//! Step 2 of the update sequence: verify signatures + hashes.
//!
//! This crate does not implement real cryptographic signature verification
//! (no key material/PKI exists anywhere in this workspace); it verifies
//! the staged partition's content checksum recorded by
//! `sher_recovery::ImmutablePartition`, which is the integrity check that is
//! actually implemented end-to-end.

use sher_recovery::ImmutablePartition;

pub fn verify(staged: &ImmutablePartition) -> Result<(), String> {
    if staged.is_empty() {
        return Err("nothing staged to verify".to_string());
    }
    if !staged.verify() {
        return Err("checksum verification failed".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sher_recovery::PartitionSlot;

    #[test]
    fn empty_partition_fails_verification() {
        let staged = ImmutablePartition::empty(PartitionSlot::B);
        assert!(verify(&staged).is_err());
    }

    #[test]
    fn written_partition_passes_verification() {
        let mut staged = ImmutablePartition::empty(PartitionSlot::B);
        staged.write_image("2.0.0", vec![1, 2, 3]);
        assert!(verify(&staged).is_ok());
    }
}
