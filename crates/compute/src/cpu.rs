//! CPU work scheduler (Stage 1, loads at boot).
//!
//! Backed by the shared [`crate::queue::WorkQueue`] priority queue. This
//! models scheduling *policy* for CPU-bound jobs; it does not perform real
//! OS thread scheduling or CPU pinning.

use crate::queue::{Job, WorkQueue};

#[derive(Debug, Default)]
pub struct CpuScheduler {
    queue: WorkQueue,
}

impl CpuScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn submit(&mut self, name: impl Into<String>, priority: u32) {
        self.queue.submit(Job::new(name, priority));
    }

    /// Dispatch the next highest-priority job. Kept as the crate's original
    /// public entry point name for backward compatibility.
    pub fn schedule(&mut self) -> Option<Job> {
        self.queue.run_next()
    }

    pub fn pending(&self) -> usize {
        self.queue.pending()
    }

    pub fn completed(&self) -> u64 {
        self.queue.completed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_dispatches_highest_priority_job() {
        let mut sched = CpuScheduler::new();
        sched.submit("background-sync", 1);
        sched.submit("interactive-input", 8);
        let job = sched.schedule().unwrap();
        assert_eq!(job.name, "interactive-input");
        assert_eq!(sched.completed(), 1);
        assert_eq!(sched.pending(), 1);
    }

    #[test]
    fn schedule_on_empty_queue_returns_none() {
        let mut sched = CpuScheduler::new();
        assert!(sched.schedule().is_none());
    }
}
