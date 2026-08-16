//! SHER Kernel Security Subsystem
//!
//! Capability-based security architecture:
//! - No component receives unrestricted access
//! - Every subsystem operates using capability-based permissions
//! - Every driver is isolated
//! - Every service is observable
//! - Every object is authenticated

pub mod audit;
pub mod capabilities;
pub mod isolation;

pub use audit::AuditLog;
pub use capabilities::SecurityContext;
pub use isolation::Sandbox;
