//! SHER Kernel Interrupt Management Subsystem
//!
//! Real, tested interrupt-controller *policy* simulation:
//! - Interrupt registration (including shared IRQ lines, priority-ordered)
//! - Interrupt handler dispatch bookkeeping (invocation counts)
//! - Interrupt affinity (recorded, not enforced — see below)
//! - enable_irq/disable_irq
//!
//! **Not implemented for real** (would require ring-0 access this userspace
//! crate does not have): actual CPU interrupt vector programming,
//! APIC/GIC register access, MSI/MSI-X hardware setup, and real CPU
//! affinity pinning. `cpu_affinity` on a handler is metadata only.

pub mod controller;
pub mod handler;

pub use controller::InterruptController;
pub use handler::InterruptHandler;
