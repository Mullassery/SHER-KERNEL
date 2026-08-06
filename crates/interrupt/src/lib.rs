//! SHER Kernel Interrupt Management Subsystem
//!
//! Handles hardware and software interrupts with:
//! - Interrupt registration
//! - Interrupt handlers
//! - Threaded interrupts
//! - Interrupt affinity
//! - MSI/MSI-X support

pub mod handler;
pub mod controller;

pub use handler::InterruptHandler;
pub use controller::InterruptController;
