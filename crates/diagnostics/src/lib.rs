//! SHER Diagnostics
//! Stage 0-1: Ring buffer only
//! Later: Persistent logs, telemetry, crash analytics, profiling

pub mod ringbuffer;
pub mod telemetry;

pub use ringbuffer::RingBuffer;
