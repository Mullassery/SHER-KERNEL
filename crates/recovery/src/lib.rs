//! SHER Recovery: Immutable Bootable Images
//!
//! System A (Immutable, Active)
//! System B (Immutable, Standby)
//! User Data (Separate)
//!
//! Boot fails → Switch to previous version instantly
//! No recovery media needed. No repair mode. Just switch.

pub mod partition;
pub mod bootptr;
pub mod healthcheck;

pub use partition::ImmutablePartition;
pub use bootptr::BootPointer;
