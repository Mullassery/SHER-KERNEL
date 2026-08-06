//! SHER Kernel Unified Device Manager
//!
//! Responsible for:
//! - Hardware discovery
//! - Driver matching
//! - Capability negotiation
//! - Firmware management
//! - Telemetry
//! - Health monitoring
//! - Power optimization
//! - Policy enforcement
//! - Security validation

pub mod registry;
pub mod discovery;
pub mod policy;

pub use registry::DeviceRegistry;
pub use discovery::DeviceDiscovery;
pub use policy::DevicePolicy;
