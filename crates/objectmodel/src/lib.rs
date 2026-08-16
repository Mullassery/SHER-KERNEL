//! SHER Kernel Object Model
//!
//! Everything in SHER is represented as a managed object with:
//! - Unique identity
//! - Lifecycle management
//! - Capabilities
//! - Telemetry
//! - Security policy
//! - Dependency tracking

pub mod capabilities;
pub mod lifecycle;
pub mod object;
pub mod telemetry;

pub use capabilities::CapabilitySet;
pub use lifecycle::Lifecycle;
pub use object::KernelObject;
pub use telemetry::Telemetry;
