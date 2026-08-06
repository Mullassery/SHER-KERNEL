//! SHER Kernel Scheduler
//!
//! Heterogeneous compute scheduler supporting:
//! - CPU
//! - GPU
//! - NPU
//! - DSP
//! - FPGA
//! - TPU
//! - Remote clusters

pub mod task;
pub mod queue;
pub mod heterogeneous;

pub use task::Task;
pub use queue::TaskQueue;
pub use heterogeneous::ComputeTarget;
