//! Interrupt registration and dispatch.
//!
//! This is a real, tested simulation of interrupt controller *policy*
//! (registration, shared-line handler ordering, enable/disable, dispatch
//! bookkeeping) — it does not touch actual CPU interrupt vectors, APIC/GIC
//! registers, or MSI/MSI-X hardware, which requires ring-0 privileges this
//! userspace crate does not have. `dispatch()` simulates "an interrupt
//! fired" by invoking the registered handler(s) in priority order and
//! recording that it happened.

use crate::handler::InterruptHandler;
use sher_common::ObjectId;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct InterruptController {
    handlers: HashMap<u32, Vec<InterruptHandler>>,
}

impl InterruptController {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a handler on its `irq_number`. Multiple handlers may share
    /// one IRQ line (shared interrupt support); they dispatch in descending
    /// `priority` order, ties broken by registration order.
    pub fn register_handler(&mut self, handler: InterruptHandler) {
        self.handlers
            .entry(handler.irq_number)
            .or_default()
            .push(handler);
    }

    /// Remove every handler registered on `irq_number` (the whole line).
    pub fn unregister_handler(&mut self, irq_number: u32) {
        self.handlers.remove(&irq_number);
    }

    /// Remove a single handler by id, leaving any other handlers sharing
    /// the line intact.
    pub fn unregister_by_id(&mut self, irq_number: u32, handler_id: ObjectId) -> bool {
        let Some(list) = self.handlers.get_mut(&irq_number) else {
            return false;
        };
        let before = list.len();
        list.retain(|h| h.id != handler_id);
        let removed = list.len() != before;
        if list.is_empty() {
            self.handlers.remove(&irq_number);
        }
        removed
    }

    /// Highest-priority handler registered on `irq_number`, if any.
    pub fn get_handler(&self, irq_number: u32) -> Option<&InterruptHandler> {
        self.handlers
            .get(&irq_number)?
            .iter()
            .max_by_key(|h| h.priority)
    }

    pub fn get_handlers(&self, irq_number: u32) -> &[InterruptHandler] {
        self.handlers
            .get(&irq_number)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn enable_irq(&mut self, irq_number: u32) {
        if let Some(list) = self.handlers.get_mut(&irq_number) {
            for h in list {
                h.enabled = true;
            }
        }
    }

    pub fn disable_irq(&mut self, irq_number: u32) {
        if let Some(list) = self.handlers.get_mut(&irq_number) {
            for h in list {
                h.enabled = false;
            }
        }
    }

    /// Simulate an interrupt firing on `irq_number`: run every *enabled*
    /// handler on that line, highest priority first, incrementing each
    /// handler's invocation count. Returns the ids of handlers that ran, in
    /// dispatch order.
    ///
    /// Errors if no handler is registered on the line at all (a spurious
    /// interrupt in real hardware terms). Returns an empty vec (not an
    /// error) if handlers exist but are all currently disabled — the line
    /// is masked, which is a normal, expected state.
    pub fn dispatch(&mut self, irq_number: u32) -> Result<Vec<ObjectId>, String> {
        let list = self.handlers.get_mut(&irq_number).ok_or_else(|| {
            format!("spurious interrupt: no handler registered for IRQ {irq_number}")
        })?;

        let mut order: Vec<usize> = (0..list.len()).filter(|&i| list[i].enabled).collect();
        order.sort_by(|&a, &b| list[b].priority.cmp(&list[a].priority));

        let mut dispatched = Vec::with_capacity(order.len());
        for idx in order {
            list[idx].invocation_count += 1;
            dispatched.push(list[idx].id);
        }
        Ok(dispatched)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_get_handler() {
        let mut ctrl = InterruptController::new();
        ctrl.register_handler(InterruptHandler::new(9, "keyboard"));
        assert!(ctrl.get_handler(9).is_some());
        assert!(ctrl.get_handler(99).is_none());
    }

    #[test]
    fn shared_irq_dispatches_in_priority_order() {
        let mut ctrl = InterruptController::new();
        ctrl.register_handler(InterruptHandler::new(16, "low").with_priority(1));
        ctrl.register_handler(InterruptHandler::new(16, "high").with_priority(9));

        let dispatched = ctrl.dispatch(16).unwrap();
        assert_eq!(dispatched.len(), 2);
        let high_id = ctrl
            .get_handlers(16)
            .iter()
            .find(|h| h.name == "high")
            .unwrap()
            .id;
        assert_eq!(dispatched[0], high_id);
    }

    #[test]
    fn dispatch_on_unregistered_irq_errors() {
        let mut ctrl = InterruptController::new();
        assert!(ctrl.dispatch(42).is_err());
    }

    #[test]
    fn disabled_irq_dispatches_nothing_but_does_not_error() {
        let mut ctrl = InterruptController::new();
        ctrl.register_handler(InterruptHandler::new(7, "timer"));
        ctrl.disable_irq(7);
        let dispatched = ctrl.dispatch(7).unwrap();
        assert!(dispatched.is_empty());

        ctrl.enable_irq(7);
        let dispatched = ctrl.dispatch(7).unwrap();
        assert_eq!(dispatched.len(), 1);
    }

    #[test]
    fn invocation_count_increments_per_dispatch() {
        let mut ctrl = InterruptController::new();
        ctrl.register_handler(InterruptHandler::new(3, "irq3"));
        ctrl.dispatch(3).unwrap();
        ctrl.dispatch(3).unwrap();
        assert_eq!(ctrl.get_handler(3).unwrap().invocation_count, 2);
    }

    #[test]
    fn unregister_handler_clears_whole_line() {
        let mut ctrl = InterruptController::new();
        ctrl.register_handler(InterruptHandler::new(5, "a"));
        ctrl.register_handler(InterruptHandler::new(5, "b"));
        ctrl.unregister_handler(5);
        assert!(ctrl.get_handler(5).is_none());
    }

    #[test]
    fn unregister_by_id_only_removes_that_handler() {
        let mut ctrl = InterruptController::new();
        ctrl.register_handler(InterruptHandler::new(5, "a"));
        let b = InterruptHandler::new(5, "b");
        let b_id = b.id;
        ctrl.register_handler(b);

        assert!(ctrl.unregister_by_id(5, b_id));
        assert_eq!(ctrl.get_handlers(5).len(), 1);
        assert_eq!(ctrl.get_handlers(5)[0].name, "a");
    }
}
