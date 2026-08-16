//! SHER Driver Runtime (early prototype)
//!
//! Hardware discovery finds devices but doesn't load drivers.
//! Drivers load on first access via sandbox isolation.
//!
//! **Relationship to `sher_device_manager` / `sher_driver_runtime`**: this
//! crate is a small, self-contained early prototype of discovery →
//! matching → sandboxed-load policy. The kernel binary (`sher_kernel`)
//! actually wires up `sher_device_manager::registry::DeviceRegistry` and
//! `sher_driver_runtime::container::DriverContainer`, which implement the
//! same idea with more depth (hot-plug, telemetry, real container
//! lifecycle). This crate is kept as a minimal, independently testable
//! reference implementation of the concept, not as a second copy that's
//! wired into the kernel.

pub mod discovery;
pub mod registry;
pub mod sandbox;

pub use discovery::{DiscoveredDevice, HardwareDiscovery};
pub use registry::DriverRegistry;
