//! GPU work scheduler (lazy-loaded on first accelerator workload).
//!
//! **Simulation notice**: this crate has no access to a real GPU driver or
//! command queue (that lives in `sher_gpu_driver` / `sher_hal`, which talk to
//! actual DRM/KMS devices). This module only simulates scheduling *policy*
//! for GPU-class jobs using the shared [`crate::queue::WorkQueue`] — it is
//! useful for testing scheduler fairness/priority logic, not for submitting
//! real GPU commands.

use crate::queue::{Job, WorkQueue};

#[derive(Debug, Default)]
pub struct GpuScheduler {
    queue: WorkQueue,
}

impl GpuScheduler {
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
        let mut sched = GpuScheduler::new();
        sched.submit("shadow-pass", 2);
        sched.submit("present", 9);
        assert_eq!(sched.schedule().unwrap().name, "present");
        assert_eq!(sched.completed(), 1);
    }
}
