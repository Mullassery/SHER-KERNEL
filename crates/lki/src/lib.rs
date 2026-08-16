//! Linux Kernel Interface (LKI)
//!
//! Translation layer that maps Linux kernel APIs to SHER primitives.
//! Every Linux driver API call is validated, translated, and tracked.
//! Supports 50+ Linux kernel APIs with full audit logging.

pub mod audit;
pub mod device_compat;
pub mod device_translation;
pub mod enforcement;
pub mod interrupt_translation;
pub mod linux_api;
pub mod memory_compat;
pub mod memory_translation;
pub mod security;
pub mod validation;

#[cfg(test)]
mod tests;

pub use audit::{AuditEntry, AuditLevel, AuditLog};
pub use device_compat::LinuxDeviceApi;
pub use device_translation::{
    BlockDevice, BlockDeviceManager, BusType, DeviceBus, DeviceManager, NetworkDevice,
    NetworkDeviceManager, PciDevice, PciDeviceId, PciDriver,
};
pub use enforcement::{PermissionChecker, SecurityContext, SecurityEnforcer};
pub use interrupt_translation::{InterruptHandler, InterruptManager};
pub use linux_api::LinuxKernelInterface;
pub use memory_compat::LinuxMemoryApi;
pub use memory_translation::{AllocationMode, LinuxMemoryAllocator, MemoryAllocation};
pub use security::{
    Capability, CapabilityGrant, CapabilityManager, PermissionTier, ReauthMethod, SecurityLevel,
    SecurityPolicy,
};
pub use validation::{ValidationError, ValidationResult, Validator};
