//! SHER Kernel Object Model
//!
//! Everything in SHER is represented as a managed object with:
//! - Unique identity
//! - Lifecycle management
//! - Capabilities
//! - Telemetry
//! - Security policy
//! - Dependency tracking

pub mod object;
pub mod lifecycle;
pub mod capabilities;
pub mod telemetry;

pub use object::KernelObject;
pub use lifecycle::Lifecycle;
pub use capabilities::CapabilitySet;
pub use telemetry::Telemetry;
