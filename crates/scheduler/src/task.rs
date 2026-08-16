use serde::{Deserialize, Serialize};
use sher_common::ObjectId;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_task_starts_pending_with_unique_id() {
        let a = Task::new("a", 3);
        let b = Task::new("b", 3);
        assert_eq!(a.state, TaskState::Pending);
        assert_eq!(a.priority, 3);
        assert_ne!(a.id, b.id);
        assert_eq!(a.cpu_affinity, None);
    }
}
