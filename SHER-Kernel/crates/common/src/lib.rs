//! SHER Kernel Common - Shared utilities and types
//!
//! Provides core types, errors, and utilities used across all SHER subsystems.

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;
