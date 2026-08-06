//! SHER Kernel Unified Device Manager
//!
//! Responsible for:
//! - Hardware discovery (PCI, USB, ACPI, DeviceTree)
//! - Device registration and lifecycle management
//! - Driver matching with capability negotiation
//! - Hot-plug support for dynamic device management
//! - Policy enforcement and telemetry
//! - Health monitoring and error recovery

pub mod registry;
pub mod discovery;
pub mod policy;

#[cfg(test)]
mod tests;

// Re-export main types
pub use registry::{
    DeviceRegistry, RegisteredDevice, DeviceState, DeviceTelemetry, DeviceHierarchy,
};
pub use discovery::{
    DeviceDiscovery, PciEnumerator, PciDevice, PciCapability,
    UsbEnumerator, UsbDevice, UsbSpeed,
    FirmwareDiscovery, FirmwareDevice, FirmwareType,
};
pub use policy::{
    DevicePolicy, DriverPolicy, ErrorAction,
    DriverDatabase, DriverEntry, DriverMatch, MatchType, DriverMatcher,
};
