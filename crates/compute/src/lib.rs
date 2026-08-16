//! SHER Compute Schedulers (Lazy Loaded)
//!
//! Real, tested priority-queue scheduling *policy* for CPU/GPU/NPU/DSP work,
//! shared via [`queue::WorkQueue`]. GPU/NPU/DSP submission is a userspace
//! simulation only — see each module's doc comment — because dispatching to
//! real accelerator hardware requires a vendor driver this crate does not
//! have access to.
//!
//! CPU: loads at Stage 1
//! GPU/NPU/DSP: load when accelerator work arrives

pub mod cpu;
pub mod dsp;
pub mod gpu;
pub mod npu;
pub mod queue;

pub use cpu::CpuScheduler;
pub use dsp::DspScheduler;
pub use gpu::GpuScheduler;
pub use npu::NpuScheduler;
pub use queue::{Job, WorkQueue};
