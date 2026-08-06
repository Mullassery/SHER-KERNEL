use crate::handler::InterruptHandler;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct InterruptController {
    pub handlers: HashMap<u32, InterruptHandler>,
}

impl InterruptController {
    pub fn register_handler(&mut self, handler: InterruptHandler) {
        self.handlers.insert(handler.irq_number, handler);
    }

    pub fn unregister_handler(&mut self, irq_number: u32) {
        self.handlers.remove(&irq_number);
    }

    pub fn get_handler(&self, irq_number: u32) -> Option<&InterruptHandler> {
        self.handlers.get(&irq_number)
    }
}
