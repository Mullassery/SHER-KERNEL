//! SHER Kernel Hardening: Security and Attack Surface Reduction
//!
//! Comprehensive hardening including:
//! - Memory safety auditing and validation
//! - Syscall whitelisting and filtering
//! - Parameter and return value validation
//! - Use-after-free and double-free detection
//! - Buffer overflow detection
//! - Rate limiting and anomaly detection

pub mod memory_safety;
pub mod syscall_hardening;

pub use memory_safety::{AuditResult, MemorySafetyAudit, MemorySafetyValidator};
pub use syscall_hardening::{SyscallAudit, SyscallHardener, SyscallPolicy, SyscallType};
