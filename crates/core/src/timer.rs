//! Timer subsystem: register named deadlines and poll for the ones that
//! have come due. Real scheduling-primitive bookkeeping — this is not tied
//! to a real hardware timer interrupt (see `sher_interrupt` for the
//! interrupt-controller-shaped simulation); callers supply "now" so the
//! logic is deterministic and testable.

use sher_common::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimerEntry {
    pub name: String,
    pub deadline_ms: u64,
}

#[derive(Default)]
pub struct TimerWheel {
    entries: Vec<TimerEntry>,
}

impl TimerWheel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn schedule(&mut self, name: impl Into<String>, deadline_ms: u64) {
        self.entries.push(TimerEntry {
            name: name.into(),
            deadline_ms,
        });
    }

    pub fn cancel(&mut self, name: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| e.name != name);
        self.entries.len() != before
    }

    pub fn pending_count(&self) -> usize {
        self.entries.len()
    }

    /// Remove and return every entry whose deadline is `<= now_ms`, sorted
    /// by deadline ascending (earliest first).
    pub fn poll(&mut self, now_ms: u64) -> Vec<TimerEntry> {
        let (due, remaining): (Vec<_>, Vec<_>) = self
            .entries
            .drain(..)
            .partition(|e| e.deadline_ms <= now_ms);
        self.entries = remaining;
        let mut due = due;
        due.sort_by_key(|e| e.deadline_ms);
        due
    }
}

pub fn initialize() -> Result<TimerWheel> {
    Ok(TimerWheel::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poll_returns_only_due_entries_in_order() {
        let mut wheel = TimerWheel::new();
        wheel.schedule("late", 100);
        wheel.schedule("early", 10);
        wheel.schedule("future", 1000);

        let due = wheel.poll(50);
        let names: Vec<&str> = due.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["early"]);
        assert_eq!(wheel.pending_count(), 2);
    }

    #[test]
    fn poll_at_exact_deadline_fires() {
        let mut wheel = TimerWheel::new();
        wheel.schedule("t", 100);
        assert_eq!(wheel.poll(100).len(), 1);
    }

    #[test]
    fn cancel_removes_pending_timer() {
        let mut wheel = TimerWheel::new();
        wheel.schedule("t", 100);
        assert!(wheel.cancel("t"));
        assert_eq!(wheel.poll(1000).len(), 0);
        assert!(!wheel.cancel("t"));
    }
}
