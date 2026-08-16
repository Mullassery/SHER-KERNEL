use serde::{Deserialize, Serialize};
use sher_common::ObjectId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterruptHandler {
    pub id: ObjectId,
    pub irq_number: u32,
    pub name: String,
    pub cpu_affinity: Option<u32>,
    /// Higher runs first among handlers sharing the same IRQ line.
    pub priority: u32,
    pub enabled: bool,
    pub invocation_count: u64,
}

impl InterruptHandler {
    pub fn new(irq_number: u32, name: impl Into<String>) -> Self {
        Self {
            id: ObjectId::new(),
            irq_number,
            name: name.into(),
            cpu_affinity: None,
            priority: 0,
            enabled: true,
            invocation_count: 0,
        }
    }

    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_affinity(mut self, cpu: u32) -> Self {
        self.cpu_affinity = Some(cpu);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_handler_starts_enabled_with_default_priority() {
        let h = InterruptHandler::new(9, "keyboard");
        assert!(h.enabled);
        assert_eq!(h.priority, 0);
        assert_eq!(h.invocation_count, 0);
        assert_eq!(h.cpu_affinity, None);
    }

    #[test]
    fn builder_methods_set_fields() {
        let h = InterruptHandler::new(16, "nic")
            .with_priority(5)
            .with_affinity(2);
        assert_eq!(h.priority, 5);
        assert_eq!(h.cpu_affinity, Some(2));
    }
}
