//! DSP work scheduler (lazy-loaded on first accelerator workload).
//!
//! **Simulation notice**: no real Digital Signal Processor driver exists in
//! this workspace. This module simulates scheduling policy for DSP-class
//! jobs (e.g. audio processing pipelines) using the shared
//! [`crate::queue::WorkQueue`].

use crate::queue::{Job, WorkQueue};

#[derive(Debug, Default)]
pub struct DspScheduler {
    queue: WorkQueue,
}

impl DspScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn submit(&mut self, name: impl Into<String>, priority: u32) {
        self.queue.submit(Job::new(name, priority));
    }

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
        let mut sched = DspScheduler::new();
        sched.submit("echo-cancellation", 4);
        sched.submit("low-latency-monitor", 7);
        assert_eq!(sched.schedule().unwrap().name, "low-latency-monitor");
        assert_eq!(sched.completed(), 1);
    }
}
