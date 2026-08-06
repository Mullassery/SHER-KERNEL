//! SHER Snapshots: Versioned Components
//!
//! Browser v12, v13, v14 all coexist
//! Switching is pointer change, not reinstall
//! Rollback is instant

pub mod version;
pub mod store;
pub mod restore;

pub use version::Snapshot;
pub use store::SnapshotStore;
