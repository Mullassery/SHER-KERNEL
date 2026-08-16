//! SHER Bootstrap - Stage 0 (< 50ms)
//!
//! **Userspace simulation**: this models the *sequencing* of a Stage 0 boot
//! (CPU bring-up -> memory map -> MMU -> heap -> integrity check) as a
//! userspace process would run it for testing purposes. It does not, and
//! cannot, perform real ring-0 CPU/MMU bring-up — see each submodule's doc
//! comment for exactly what's simulated vs. what real bookkeeping logic
//! backs it (e.g. `heap::HeapState` bump allocation, `mmu::setup`'s
//! overlap validation, `verification`'s checksum check).
//!
//! Absolute minimum to bring the system to a state where the kernel can run:
//! - CPU initialization
//! - Memory map discovery
//! - MMU setup
//! - Kernel heap allocation
//! - Immutable root object creation
//! - Integrity verification
//!
//! Nothing else. No services. No drivers. No compatibility layers.

pub mod cpu;
pub mod heap;
pub mod memory_map;
pub mod mmu;
pub mod verification;

use heap::HeapState;
use sher_common::Result;
use tracing::info;

pub struct BootstrapConfig {
    pub kernel_heap_size: u64,
    pub verify_root_image: bool,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            kernel_heap_size: 2 * 1024 * 1024, // 2MB minimum
            verify_root_image: true,
        }
    }
}

pub struct Bootstrap {
    config: BootstrapConfig,
    boot_time_ms: u64,
    heap: Option<HeapState>,
}

impl Bootstrap {
    pub fn new(config: BootstrapConfig) -> Self {
        Self {
            config,
            boot_time_ms: 0,
            heap: None,
        }
    }

    pub async fn execute(&mut self) -> Result<()> {
        let start = std::time::Instant::now();

        info!("Stage 0: CPU Initialization");
        cpu::initialize()?;

        info!("Stage 0: Memory Map Discovery");
        let mem_map = memory_map::discover()?;

        info!("Stage 0: MMU Setup");
        mmu::setup(&mem_map)?;

        info!("Stage 0: Kernel Heap Allocation");
        self.heap = Some(heap::initialize(self.config.kernel_heap_size)?);

        if self.config.verify_root_image {
            info!("Stage 0: Immutable Root Verification");
            verification::verify_root_image()?;
        }

        self.boot_time_ms = start.elapsed().as_millis() as u64;
        info!("Stage 0 complete in {}ms", self.boot_time_ms);

        Ok(())
    }

    pub fn boot_time_ms(&self) -> u64 {
        self.boot_time_ms
    }

    pub fn heap(&self) -> Option<&HeapState> {
        self.heap.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn execute_completes_and_records_heap_state() {
        let mut bootstrap = Bootstrap::new(BootstrapConfig::default());
        bootstrap.execute().await.unwrap();
        assert!(bootstrap.heap().is_some());
        assert_eq!(bootstrap.heap().unwrap().capacity, 2 * 1024 * 1024);
    }

    #[tokio::test]
    async fn execute_skips_verification_when_disabled() {
        let mut bootstrap = Bootstrap::new(BootstrapConfig {
            kernel_heap_size: 4096,
            verify_root_image: false,
        });
        assert!(bootstrap.execute().await.is_ok());
    }
}
