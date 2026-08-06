//! SHER Digital Twins: Event Recording and Replay
//!
//! Captures kernel events for:
//! - Replay in controlled environments
//! - Debugging complex scenarios
//! - Reproducing bugs
//! - What-if analysis
//! - Performance testing without production impact

pub mod event_log;
pub mod replay_engine;

pub use event_log::{EventLog, KernelEvent, EventType};
pub use replay_engine::{ReplayEngine, ReplayMode, ReplayStats};
