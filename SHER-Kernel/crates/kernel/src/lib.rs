//! SHER Kernel: AI-Native Operating System Kernel
//!
//! A completely new operating system kernel engineered from first principles for the AI era.
//! Not a Linux fork, not a microkernel clone, but a new architecture that:
//!
//! - Is AI-native: intelligence is part of the OS, not an application
//! - Preserves ecosystem: Linux hardware drivers work through compatibility
//! - Modular by design: every subsystem is independently replaceable
//! - Security by architecture: capability-based, zero trust
//! - Self-healing: components can be monitored, restarted, rolled back, migrated without reboot

pub mod config;
pub mod kernel;

pub use config::KernelConfig;
pub use kernel::SherKernel;
