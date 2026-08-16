//! SHER Core - Stage 1 (< 200ms)
//!
//! Essential primitives that enable the kernel to run. Unlike
//! `sher_bootstrap` (Stage 0, which needs real ring-0 access this
//! userspace crate does not have), everything here is a real, tested,
//! in-process implementation — object tracking, IPC mailboxes, capability
//! enforcement, a timer wheel, and a basic FIFO CPU scheduler are all
//! things a userspace process legitimately can implement:
//! - Object Manager (create, track, lifecycle)
//! - IPC (inter-process communication)
//! - Capability Manager (grant, revoke, enforce)
//! - Timer (scheduling primitive)
//! - Basic CPU Scheduler (only)
//!
//! After Stage 1, applications can already execute.
//! No drivers. No services. No AI. Nothing else.

pub mod capability_manager;
pub mod cpu_scheduler;
pub mod ipc;
pub mod object_manager;
pub mod timer;

use capability_manager::CapabilityManager;
use cpu_scheduler::BasicCpuScheduler;
use ipc::IpcBus;
use object_manager::ObjectManager;
use sher_common::Result;
use timer::TimerWheel;
use tracing::info;

pub struct CoreKernel {
    pub objects: ObjectManager,
    pub ipc: IpcBus,
    pub capabilities: CapabilityManager,
    pub timers: TimerWheel,
    pub scheduler: BasicCpuScheduler,
}

impl CoreKernel {
    pub async fn initialize() -> Result<Self> {
        info!("Stage 1: Object Manager Initialization");
        let objects = object_manager::initialize()?;

        info!("Stage 1: IPC Setup");
        let ipc = ipc::initialize()?;

        info!("Stage 1: Capability Manager");
        let capabilities = capability_manager::initialize()?;

        info!("Stage 1: Timer Initialization");
        let timers = timer::initialize()?;

        info!("Stage 1: CPU Scheduler Only");
        let scheduler = cpu_scheduler::initialize()?;

        info!("Stage 1 complete: System ready to execute applications");
        Ok(Self {
            objects,
            ipc,
            capabilities,
            timers,
            scheduler,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn initialize_wires_up_all_subsystems() {
        let core = CoreKernel::initialize().await.unwrap();
        assert!(core.objects.root().is_some());
        assert_eq!(core.ipc.pending_count("nonexistent"), 0);
        assert_eq!(core.timers.pending_count(), 0);
        assert!(core.scheduler.running().is_none());
        assert!(core
            .capabilities
            .enforce(sher_common::ObjectId::new(), sher_common::Capability::Read)
            .is_err());
    }
}
