//! SHER Diagnostics
//!
//! Userspace simulation of a kernel diagnostics subsystem: a real, tested
//! ring buffer for bounded-memory event history plus an in-memory telemetry
//! collector (counters/gauges/events). There is no persistent log store or
//! crash-analytics pipeline behind this yet — see module docs for what's
//! implemented vs. still a placeholder.
//!
//! Stage 0-1: Ring buffer only
//! Later: Persistent logs, telemetry, crash analytics, profiling

pub mod ringbuffer;
pub mod telemetry;

pub use ringbuffer::RingBuffer;
pub use telemetry::Telemetry;
