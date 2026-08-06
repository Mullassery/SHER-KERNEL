//! Linux Kernel Interface (LKI)
//!
//! Implements an advanced Linux Kernel Interface capable of executing Linux kernel drivers
//! through compatibility translation rather than requiring Linux internals.
//!
//! LKI emulates Linux kernel APIs while mapping every operation to SHER Kernel primitives.

pub mod linux_api;
pub mod memory_compat;
pub mod device_compat;

pub use linux_api::LinuxKernelInterface;
pub use memory_compat::LinuxMemoryApi;
pub use device_compat::LinuxDeviceApi;
