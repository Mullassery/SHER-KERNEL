//! SHER Kernel Unified Device Manager
//!
//! Responsible for:
//! - Hardware discovery (PCI, USB, ACPI, DeviceTree)
//! - Device registration and lifecycle management
//! - Driver matching with capability negotiation
//! - Hot-plug support for dynamic device management
//! - Policy enforcement and telemetry
//! - Health monitoring and error recovery

pub mod discovery;
pub mod hotplug;
pub mod policy;
pub mod registry;

#[cfg(test)]
mod tests;

// Re-export main types
pub use discovery::{
    DeviceDiscovery, FirmwareDevice, FirmwareDiscovery, FirmwareType, PciCapability, PciDevice,
    PciEnumerator, UsbDevice, UsbEnumerator, UsbSpeed,
};
pub use hotplug::{
    DeviceEvent, DeviceEventType, EventCallback, EventSubscription, HotPlugController,
    HotPlugManager, RecoveryManager, RecoveryPolicy,
};
pub use policy::{
    DevicePolicy, DriverDatabase, DriverEntry, DriverMatch, DriverMatcher, DriverPolicy,
    ErrorAction, MatchType,
};
pub use registry::{
    DeviceHierarchy, DeviceRegistry, DeviceState, DeviceTelemetry, RegisteredDevice,
};
