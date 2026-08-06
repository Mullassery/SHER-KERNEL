//! SHER Updater: Transactional System Updates
//!
//! Update sequence:
//! 1. Download into System B (System A untouched)
//! 2. Verify signatures + hashes
//! 3. Boot test System B
//! 4. If OK, switch boot pointer
//! 5. Old version still bootable
//!
//! Power fails at any point → Just boot previous version

pub mod transaction;
pub mod verify;
pub mod commit;

pub use transaction::Transaction;
