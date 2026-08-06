//! SHER Compatibility Layers
//! Linux: Load only if Linux driver encountered
//! POSIX: Load on first POSIX syscall

pub mod linux;
pub mod posix;
