//! SHER Driver Runtime
//! Hardware discovery finds devices but doesn't load drivers
//! Drivers load on first access via sandbox isolation

pub mod discovery;
pub mod registry;
pub mod sandbox;

pub use discovery::HardwareDiscovery;
pub use registry::DriverRegistry;
