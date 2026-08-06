use sher_common::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: ObjectId,
    pub name: String,
    pub state: TaskState,
    pub priority: u32,
    pub cpu_affinity: Option<u32>,
}

impl Task {
    pub fn new(name: impl Into<String>, priority: u32) -> Self {
        Self {
            id: ObjectId::new(),
            name: name.into(),
            state: TaskState::Pending,
            priority,
            cpu_affinity: None,
        }
    }
}
