//! SHER Kernel Storage Subsystem
//!
//! Support for SATA, NVMe, USB, eMMC, SD, RAID, Persistent Memory

pub mod device;
pub mod blockdevice;

pub use device::StorageDevice;
pub use blockdevice::BlockDevice;
