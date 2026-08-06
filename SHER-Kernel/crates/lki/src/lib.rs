//! Linux Kernel Interface (LKI)
//!
//! Translation layer that maps Linux kernel APIs to SHER primitives.
//! Every Linux driver API call is validated, translated, and tracked.
//! Supports 50+ Linux kernel APIs with full audit logging.

pub mod linux_api;
pub mod memory_compat;
pub mod device_compat;
pub mod validation;
pub mod memory_translation;
pub mod interrupt_translation;
pub mod device_translation;
pub mod audit;

#[cfg(test)]
mod tests;

pub use linux_api::LinuxKernelInterface;
pub use memory_compat::LinuxMemoryApi;
pub use device_compat::LinuxDeviceApi;
pub use validation::{ValidationResult, ValidationError, Validator};
pub use memory_translation::{LinuxMemoryAllocator, AllocationMode, MemoryAllocation};
pub use interrupt_translation::{InterruptHandler, InterruptManager};
pub use device_translation::{
    DeviceManager, PciDriver, PciDevice, PciDeviceId, DeviceBus, BusType,
    BlockDevice, BlockDeviceManager, NetworkDevice, NetworkDeviceManager,
};
pub use audit::{AuditLog, AuditEntry, AuditLevel};
