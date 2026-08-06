//! SHER Compute Schedulers (Lazy Loaded)
//! CPU: loads at Stage 1
//! GPU/NPU/DSP: load when accelerator work arrives

pub mod cpu;
pub mod gpu;
pub mod npu;
pub mod dsp;

pub use cpu::CpuScheduler;
