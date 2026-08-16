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

pub mod heterogeneous;
pub mod queue;
pub mod scheduler;
pub mod task;

pub use heterogeneous::ComputeTarget;
pub use queue::TaskQueue;
pub use scheduler::Scheduler;
pub use task::Task;
