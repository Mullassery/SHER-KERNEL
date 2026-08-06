//! SHER Kernel Driver Runtime
//!
//! Isolated execution environment for Linux drivers and native SHER drivers.
//! Every driver executes inside its own protected execution environment.

pub mod container;
pub mod loader;
pub mod translator;

pub use container::DriverContainer;
pub use loader::DriverLoader;
pub use translator::TranslationEngine;
