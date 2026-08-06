use sher_common::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterruptHandler {
    pub id: ObjectId,
    pub irq_number: u32,
    pub name: String,
    pub cpu_affinity: Option<u32>,
}

impl InterruptHandler {
    pub fn new(irq_number: u32, name: impl Into<String>) -> Self {
        Self {
            id: ObjectId::new(),
            irq_number,
            name: name.into(),
            cpu_affinity: None,
        }
    }
}
