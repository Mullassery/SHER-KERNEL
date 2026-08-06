//! SHER Kernel Memory Management Subsystem
//!
//! Provides memory allocation, translation, and management for both
//! native SHER applications and Linux-compatible drivers.

pub mod allocator;
pub mod paging;
pub mod dma;

pub use allocator::MemoryAllocator;
pub use paging::PageTable;
pub use dma::DmaManager;
