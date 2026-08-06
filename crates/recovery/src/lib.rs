//! SHER Recovery: Immutable Bootable Images & Crash Recovery
//!
//! System A (Immutable, Active)
//! System B (Immutable, Standby)
//! User Data (Separate)
//!
//! Boot fails → Switch to previous version instantly
//! Driver crashes → Automatic recovery with exponential backoff
//! System degrades gracefully under failure

pub mod partition;
pub mod bootptr;
pub mod healthcheck;
pub mod crash_recovery;
pub mod watchdog;

pub use partition::ImmutablePartition;
pub use bootptr::BootPointer;
pub use crash_recovery::{CrashRecoveryManager, RecoveryPolicy, RecoveryState, CrashMetrics};
pub use watchdog::{Watchdog, HealthStatus, WatchdogStats, HeartbeatRecord};
