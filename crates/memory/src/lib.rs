//! SHER Kernel Memory Management Subsystem
//!
//! Provides memory allocation, translation, and management for both
//! native SHER applications and Linux-compatible drivers.
//!
//! Four-Tier Allocator Architecture:
//! - Tier 0: Per-CPU slab caching (8-64B, <50ns)
//! - Tier 1: Per-socket slab caching (65B-64KB, <100ns)
//! - Tier 2: NUMA-aware buddy allocator (64KB-1GB)
//! - Tier 3: Direct huge page allocation (>1GB)

pub mod allocator;
pub mod dma;
pub mod master_allocator;
pub mod paging;
pub mod stress_tests;
pub mod tier0_slab;
pub mod tier1_slab;

pub use allocator::MemoryAllocator;
pub use dma::DmaManager;
pub use master_allocator::{AllocatorStats, MasterAllocator};
pub use paging::PageTable;
pub use tier0_slab::{CpuSlabCache, Tier0Allocator};
pub use tier1_slab::{SlabPage, SocketSlabCache, Tier1Allocator};

// Re-export size classes with namespace prefix to avoid conflicts
pub use tier0_slab::SizeClass as SizeClassTier0;
pub use tier1_slab::SizeClass as SizeClassTier1;
