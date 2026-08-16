//! SHER Recovery: Immutable Bootable Images & Crash Recovery
//!
//! System A (Immutable, Active)
//! System B (Immutable, Standby)
//! User Data (Separate)
//!
//! Boot fails → Switch to previous version instantly
//! Driver crashes → Automatic recovery with exponential backoff
//! System degrades gracefully under failure

pub mod bootptr;
pub mod crash_recovery;
pub mod healthcheck;
pub mod partition;
pub mod watchdog;

pub use bootptr::{BootPointer, BootSwitch};
pub use crash_recovery::{CrashMetrics, CrashRecoveryManager, RecoveryPolicy, RecoveryState};
pub use healthcheck::{check as run_healthcheck, check_default, HealthCheckReport, ProbeResult};
pub use partition::{ImmutablePartition, PartitionSlot};
pub use watchdog::{HealthStatus, HeartbeatRecord, Watchdog, WatchdogStats};
