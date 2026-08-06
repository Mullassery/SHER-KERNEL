use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    Initializing,
    Ready,
    Running,
    Paused,
    Stopping,
    Stopped,
    Failed,
    Recovering,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lifecycle {
    pub state: State,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub stopped_at: Option<u64>,
    pub restart_count: u32,
    pub last_error: Option<String>,
}

impl Default for Lifecycle {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            state: State::Initializing,
            created_at: now,
            started_at: None,
            stopped_at: None,
            restart_count: 0,
            last_error: None,
        }
    }
}

impl Lifecycle {
    pub fn start(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.state = State::Running;
        self.started_at = Some(now);
    }

    pub fn stop(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.state = State::Stopped;
        self.stopped_at = Some(now);
    }

    pub fn mark_failed(&mut self, error: impl Into<String>) {
        self.state = State::Failed;
        self.last_error = Some(error.into());
    }

    pub fn is_running(&self) -> bool {
        self.state == State::Running
    }
}
