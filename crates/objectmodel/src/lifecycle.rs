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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_is_initializing() {
        let lc = Lifecycle::default();
        assert_eq!(lc.state, State::Initializing);
        assert!(lc.started_at.is_none());
        assert!(!lc.is_running());
    }

    #[test]
    fn start_transitions_to_running_and_stamps_time() {
        let mut lc = Lifecycle::default();
        lc.start();
        assert!(lc.is_running());
        assert!(lc.started_at.is_some());
    }

    #[test]
    fn stop_transitions_to_stopped_and_stamps_time() {
        let mut lc = Lifecycle::default();
        lc.start();
        lc.stop();
        assert_eq!(lc.state, State::Stopped);
        assert!(lc.stopped_at.is_some());
        assert!(!lc.is_running());
    }

    #[test]
    fn mark_failed_records_error_message() {
        let mut lc = Lifecycle::default();
        lc.mark_failed("driver panicked");
        assert_eq!(lc.state, State::Failed);
        assert_eq!(lc.last_error.as_deref(), Some("driver panicked"));
    }
}
