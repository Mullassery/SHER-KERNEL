//! SHER Core - Stage 1 (< 200ms)
//!
//! Essential primitives that enable the kernel to run:
//! - Object Manager (create, track, lifecycle)
//! - IPC (inter-process communication)
//! - Capability Manager (grant, revoke, enforce)
//! - Timer (scheduling primitive)
//! - Basic CPU Scheduler (only)
//!
//! After Stage 1, applications can already execute.
//! No drivers. No services. No AI. Nothing else.

pub mod object_manager;
pub mod ipc;
pub mod capability_manager;
pub mod timer;
pub mod cpu_scheduler;

use sher_common::Result;
use tracing::info;

pub struct CoreKernel;

impl CoreKernel {
    pub async fn initialize() -> Result<Self> {
        info!("Stage 1: Object Manager Initialization");
        object_manager::initialize()?;

        info!("Stage 1: IPC Setup");
        ipc::initialize()?;

        info!("Stage 1: Capability Manager");
        capability_manager::initialize()?;

        info!("Stage 1: Timer Initialization");
        timer::initialize()?;

        info!("Stage 1: CPU Scheduler Only");
        cpu_scheduler::initialize()?;

        info!("Stage 1 complete: System ready to execute applications");
        Ok(Self)
    }
}
