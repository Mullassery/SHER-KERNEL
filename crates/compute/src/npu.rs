//! NPU work scheduler (lazy-loaded on first accelerator workload).
//!
//! **Simulation notice**: no real Neural Processing Unit driver exists in
//! this workspace. This module simulates scheduling policy for NPU-class
//! inference jobs using the shared [`crate::queue::WorkQueue`].

use crate::queue::{Job, WorkQueue};

#[derive(Debug, Default)]
pub struct NpuScheduler {
    queue: WorkQueue,
}

impl NpuScheduler {
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
        let mut sched = NpuScheduler::new();
        sched.submit("embedding", 3);
        sched.submit("realtime-inference", 10);
        assert_eq!(sched.schedule().unwrap().name, "realtime-inference");
        assert_eq!(sched.completed(), 1);
    }
}
