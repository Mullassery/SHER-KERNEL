use sher_common::ObjectId;
use sher_security::isolation::Sandbox;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverContainer {
    pub id: ObjectId,
    pub driver_name: String,
    pub sandbox: Sandbox,
    pub memory_limit: u64,
    pub running: bool,
}

impl DriverContainer {
    pub fn new(driver_name: impl Into<String>, memory_limit: u64) -> Self {
        let id = ObjectId::new();
        Self {
            id,
            driver_name: driver_name.into(),
            sandbox: Sandbox::new("driver_sandbox"),
            memory_limit,
            running: false,
        }
    }

    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn stop(&mut self) {
        self.running = false;
    }
}
