//! SHER Snapshots: Versioned Components
//!
//! Browser v12, v13, v14 all coexist
//! Switching is pointer change, not reinstall
//! Rollback is instant

pub mod restore;
pub mod store;
pub mod version;

pub use restore::{restore, restore_previous};
pub use store::SnapshotStore;
pub use version::Snapshot;
