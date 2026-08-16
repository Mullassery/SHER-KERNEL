//! Immutable root image verification (Stage 0).
//!
//! **Simulation notice**: this crate has no PKI/signing infrastructure, so
//! "verify signature" is not implemented — there is no cryptographic key
//! material anywhere in this workspace. What *is* real: `verify_image`
//! computes a content checksum (std `DefaultHasher`, explicitly
//! non-cryptographic) and compares it against an expected value, which is
//! the same integrity-check pattern used by `sher_recovery::ImmutablePartition`
//! and `sher_snapshot::Snapshot`.

use sher_common::{Error, Result};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn checksum(image: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    image.hash(&mut hasher);
    hasher.finish()
}

pub fn verify_image(image: &[u8], expected_checksum: u64) -> Result<()> {
    let actual = checksum(image);
    if actual != expected_checksum {
        return Err(Error::Security(format!(
            "root image checksum mismatch: expected {expected_checksum:#x}, got {actual:#x}"
        )));
    }
    Ok(())
}

/// Placeholder root image bytes used when no real image is supplied (e.g.
/// by `Bootstrap::execute`, which has no image source in a userspace
/// context). In a real Stage 0 this would be the actual immutable root
/// filesystem image.
const PLACEHOLDER_ROOT_IMAGE: &[u8] = b"SHER-ROOT-IMAGE-PLACEHOLDER";

pub fn verify_root_image() -> Result<()> {
    let expected = checksum(PLACEHOLDER_ROOT_IMAGE);
    verify_image(PLACEHOLDER_ROOT_IMAGE, expected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_checksum_verifies() {
        let image = b"hello kernel";
        let expected = checksum(image);
        assert!(verify_image(image, expected).is_ok());
    }

    #[test]
    fn mismatched_checksum_fails() {
        let image = b"hello kernel";
        assert!(verify_image(image, 0xDEAD_BEEF).is_err());
    }

    #[test]
    fn placeholder_root_image_always_verifies() {
        assert!(verify_root_image().is_ok());
    }

    #[test]
    fn different_images_produce_different_checksums() {
        assert_ne!(checksum(b"image-a"), checksum(b"image-b"));
    }
}
