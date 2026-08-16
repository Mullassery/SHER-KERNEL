//! SHER Kernel Driver Runtime
//!
//! Isolated execution environment for Linux drivers and native SHER drivers.
//! Every driver executes inside its own protected execution environment.

pub mod container;
pub mod hotplug_integration;
pub mod loader;
pub mod network;
pub mod sandbox;
pub mod translator;

#[cfg(test)]
mod tests;

// Re-export main types
pub use container::{
    ContainerPool, ContainerState, ContainerTelemetry, DriverCapability, DriverContainer,
    ResourceLimits,
};
pub use hotplug_integration::{
    DriverLifecycleManager, HotPlugEvent, HotPlugEventType, HotPlugIntegration,
};
pub use loader::{DriverLoader, DriverManifest, LinuxDriver};
pub use network::{
    BandwidthMetrics, BandwidthThrottler, DeviceIsolation, DeviceIsolationManager, IpProtocol,
    NetworkIsolationManager, NetworkPolicy, NetworkRule,
};
pub use sandbox::{
    CapabilitySet, FileDescriptorPolicy, NamespacePolicy, SandboxManager, SandboxPolicy,
    SecurityLevel, SyscallEntry, SyscallPolicy,
};
pub use translator::{
    LinuxApiCall, SherPrimitive, TranslationEngine, TranslationMapping, ValidationLevel,
};
