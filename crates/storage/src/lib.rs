//! SHER Kernel Storage Subsystem
//!
//! Device/block descriptors plus a real, tested, bounds-checked in-memory
//! block device simulation (`blockdevice`) and registry (`registry`).
//! There is no real SATA/NVMe/USB/eMMC/SD/RAID/persistent-memory I/O here —
//! talking to actual storage controllers needs privileged/raw-device access
//! this userspace crate does not have.

pub mod blockdevice;
pub mod device;
pub mod registry;

pub use blockdevice::BlockDevice;
pub use device::StorageDevice;
pub use registry::StorageRegistry;
