//! SHER Kernel Driver Runtime
//!
//! Isolated execution environment for Linux drivers and native SHER drivers.
//! Every driver executes inside its own protected execution environment.

pub mod container;
pub mod loader;
pub mod translator;
pub mod sandbox;
pub mod network;

#[cfg(test)]
mod tests;

// Re-export main types
pub use container::{
    DriverContainer, ContainerState, ContainerPool, ContainerTelemetry,
    ResourceLimits, DriverCapability,
};
pub use loader::{DriverLoader, DriverManifest, LinuxDriver};
pub use translator::{TranslationEngine, LinuxApiCall, SherPrimitive, TranslationMapping, ValidationLevel};
pub use sandbox::{
    SandboxManager, SandboxPolicy, SyscallPolicy, SyscallEntry,
    SecurityLevel, NamespacePolicy, FileDescriptorPolicy, CapabilitySet,
};
pub use network::{
    NetworkIsolationManager, NetworkPolicy, NetworkRule, IpProtocol,
    BandwidthThrottler, BandwidthMetrics, DeviceIsolationManager, DeviceIsolation,
};
