// SHER LKI: Interrupt Translation
// Maps Linux request_irq/free_irq to SHER interrupt primitives

use crate::validation::Validator;
use serde::{Deserialize, Serialize};
use sher_common::{Error, ObjectId, Result};
use std::collections::HashMap;

// ============================================================================
// INTERRUPT TYPES & FLAGS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IrqTrigger {
    Rising,    // Rising edge
    Falling,   // Falling edge
    HighLevel, // High level
    LowLevel,  // Low level
    Shared,    // Can be shared with other devices
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IrqPriority {
    Low,
    Normal,
    High,
    Critical,
}

// ============================================================================
// INTERRUPT HANDLER
// ============================================================================

pub type IrqHandlerFn = fn(irq: u32, data: *mut u8) -> IrqReturnValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IrqReturnValue {
    NotHandled,
    Handled,
    WakeThread,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterruptHandler {
    pub handler_id: ObjectId,
    pub driver_id: ObjectId,
    pub irq_number: u32,
    pub trigger_type: IrqTrigger,
    pub priority: IrqPriority,
    pub enabled: bool,
    pub call_count: u64,
    pub error_count: u64,
    pub avg_latency_us: u32,
    pub peak_latency_us: u32,
}

impl InterruptHandler {
    pub fn new(driver_id: ObjectId, irq_number: u32, trigger_type: IrqTrigger) -> Self {
        InterruptHandler {
            handler_id: ObjectId::new(),
            driver_id,
            irq_number,
            trigger_type,
            priority: IrqPriority::Normal,
            enabled: false,
            call_count: 0,
            error_count: 0,
            avg_latency_us: 0,
            peak_latency_us: 0,
        }
    }

    pub fn with_priority(mut self, priority: IrqPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn record_call(&mut self, latency_us: u32) {
        self.call_count += 1;
        self.peak_latency_us = self.peak_latency_us.max(latency_us);
        self.avg_latency_us = ((self.avg_latency_us as u64 * (self.call_count - 1)
            + latency_us as u64)
            / self.call_count) as u32;
    }

    pub fn record_error(&mut self) {
        self.error_count += 1;
    }
}

// ============================================================================
// INTERRUPT MANAGER
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct InterruptManager {
    pub validator: Validator,
    pub handlers: HashMap<u32, InterruptHandler>, // irq_number -> handler
    pub total_interrupts: u64,
    pub shared_irqs: HashMap<u32, Vec<ObjectId>>, // irq_number -> [driver_ids]
}

impl InterruptManager {
    pub fn new() -> Self {
        InterruptManager {
            validator: Validator::new(),
            handlers: HashMap::new(),
            total_interrupts: 0,
            shared_irqs: HashMap::new(),
        }
    }

    /// Translate request_irq(irq, handler, flags, name, dev_id) to SHER interrupt
    pub fn request_irq(
        &mut self,
        driver_id: ObjectId,
        irq: u32,
        trigger_type: IrqTrigger,
        flags: u32,
    ) -> Result<ObjectId> {
        // Validate IRQ number
        self.validator.validate_irq(irq)?;

        // Check if IRQ already registered
        match self.handlers.entry(irq) {
            std::collections::hash_map::Entry::Occupied(_) => {
                // Check if sharable (IRQF_SHARED flag)
                const IRQF_SHARED: u32 = 0x00000080;
                if (flags & IRQF_SHARED) == 0 {
                    return Err(Error::Driver("IRQ already in use (not shared)".to_string()));
                }

                // Add to shared list
                self.shared_irqs.entry(irq).or_default().push(driver_id);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                // Register new handler
                let mut handler = InterruptHandler::new(driver_id, irq, trigger_type);

                // Set priority based on flags
                const IRQF_HIGH_PRIO: u32 = 0x00000001;
                const IRQF_CRITICAL: u32 = 0x00000002;
                if (flags & IRQF_CRITICAL) != 0 {
                    handler.priority = IrqPriority::Critical;
                } else if (flags & IRQF_HIGH_PRIO) != 0 {
                    handler.priority = IrqPriority::High;
                }

                handler.enable();
                entry.insert(handler);
                self.shared_irqs.insert(irq, vec![driver_id]);
            }
        }

        self.total_interrupts += 1;

        Ok(self.handlers.get(&irq).unwrap().handler_id)
    }

    /// Translate free_irq(irq, dev_id) to deregistration
    pub fn free_irq(&mut self, irq: u32, driver_id: ObjectId) -> Result<()> {
        // Check if driver is registered
        let is_registered = if let Some(handler) = self.handlers.get(&irq) {
            handler.driver_id == driver_id || self.is_shared_irq(irq, driver_id)
        } else {
            false
        };

        if !is_registered {
            return Err(Error::Driver(
                "Driver did not register this IRQ".to_string(),
            ));
        }

        if let Some(handler) = self.handlers.get_mut(&irq) {
            handler.disable();
        }

        // Remove from shared list
        if let Some(drivers) = self.shared_irqs.get_mut(&irq) {
            drivers.retain(|&d| d != driver_id);
            if drivers.is_empty() {
                self.handlers.remove(&irq);
                self.shared_irqs.remove(&irq);
            }
        }

        Ok(())
    }

    /// Check if IRQ is shared
    pub fn is_shared_irq(&self, irq: u32, driver_id: ObjectId) -> bool {
        if let Some(drivers) = self.shared_irqs.get(&irq) {
            drivers.contains(&driver_id)
        } else {
            false
        }
    }

    /// Enable interrupt
    pub fn enable_irq(&mut self, irq: u32) -> Result<()> {
        if let Some(handler) = self.handlers.get_mut(&irq) {
            handler.enable();
            Ok(())
        } else {
            Err(Error::Driver("IRQ not found".to_string()))
        }
    }

    /// Disable interrupt
    pub fn disable_irq(&mut self, irq: u32) -> Result<()> {
        if let Some(handler) = self.handlers.get_mut(&irq) {
            handler.disable();
            Ok(())
        } else {
            Err(Error::Driver("IRQ not found".to_string()))
        }
    }

    /// Get handler for IRQ
    pub fn get_handler(&self, irq: u32) -> Option<&InterruptHandler> {
        self.handlers.get(&irq)
    }

    /// Get active interrupt count
    pub fn active_interrupts(&self) -> usize {
        self.handlers.iter().filter(|(_, h)| h.enabled).count()
    }

    /// Get registered interrupt count
    pub fn registered_interrupts(&self) -> usize {
        self.handlers.len()
    }

    /// Get interrupt statistics
    pub fn get_stats(&self) -> InterruptStats {
        InterruptStats {
            total_registered: self.registered_interrupts() as u64,
            active_interrupts: self.active_interrupts() as u64,
            total_interrupt_calls: self.handlers.values().map(|h| h.call_count).sum(),
            total_errors: self.handlers.values().map(|h| h.error_count).sum(),
            avg_latency_us: if self.handlers.is_empty() {
                0
            } else {
                self.handlers
                    .values()
                    .map(|h| h.avg_latency_us as u64)
                    .sum::<u64>()
                    / self.handlers.len() as u64
            } as u32,
            peak_latency_us: self
                .handlers
                .values()
                .map(|h| h.peak_latency_us)
                .max()
                .unwrap_or(0),
        }
    }

    /// Find high-latency interrupts (> threshold)
    pub fn find_high_latency_irqs(&self, threshold_us: u32) -> Vec<&InterruptHandler> {
        self.handlers
            .values()
            .filter(|h| h.peak_latency_us > threshold_us)
            .collect()
    }
}

// ============================================================================
// INTERRUPT STATISTICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterruptStats {
    pub total_registered: u64,
    pub active_interrupts: u64,
    pub total_interrupt_calls: u64,
    pub total_errors: u64,
    pub avg_latency_us: u32,
    pub peak_latency_us: u32,
}
