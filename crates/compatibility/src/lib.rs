//! SHER Compatibility Layers
//!
//! Real, tested API-name → SHER-subsystem lookup tables for the Linux and
//! POSIX layers below. This crate does not implement a binary-compatible
//! Linux/POSIX ABI; `sher_lki` implements the fuller Linux Kernel Interface
//! translation used elsewhere in the workspace.
//!
//! Linux: Load only if Linux driver encountered
//! POSIX: Load on first POSIX syscall

pub mod linux;
pub mod posix;
