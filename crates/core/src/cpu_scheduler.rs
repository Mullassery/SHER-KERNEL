//! Stage 1's basic CPU scheduler: a plain FIFO run queue. Deliberately
//! simpler than `sher_scheduler::Scheduler` (priority-based, multi-target)
//! — Stage 1 only needs enough to run applications before richer
//! scheduling policy loads later in boot.

use sher_common::{ObjectId, Result};
use std::collections::VecDeque;

#[derive(Default)]
pub struct BasicCpuScheduler {
    ready_queue: VecDeque<ObjectId>,
    running: Option<ObjectId>,
}

impl BasicCpuScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enqueue(&mut self, task: ObjectId) {
        self.ready_queue.push_back(task);
    }

    /// Move the next queued task into the running slot, returning it. If a
    /// task was already running, it is pushed back to the tail of the
    /// ready queue (cooperative round robin).
    pub fn run_next(&mut self) -> Option<ObjectId> {
        if let Some(prev) = self.running.take() {
            self.ready_queue.push_back(prev);
        }
        let next = self.ready_queue.pop_front();
        self.running = next;
        next
    }

    pub fn running(&self) -> Option<ObjectId> {
        self.running
    }

    pub fn ready_count(&self) -> usize {
        self.ready_queue.len()
    }
}

pub fn initialize() -> Result<BasicCpuScheduler> {
    Ok(BasicCpuScheduler::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_next_on_empty_queue_returns_none() {
        let mut sched = BasicCpuScheduler::new();
        assert!(sched.run_next().is_none());
    }

    #[test]
    fn fifo_ordering_is_respected() {
        let mut sched = BasicCpuScheduler::new();
        let a = ObjectId::new();
        let b = ObjectId::new();
        sched.enqueue(a);
        sched.enqueue(b);

        assert_eq!(sched.run_next(), Some(a));
        assert_eq!(sched.running(), Some(a));
    }

    #[test]
    fn round_robin_requeues_previous_task() {
        let mut sched = BasicCpuScheduler::new();
        let a = ObjectId::new();
        let b = ObjectId::new();
        sched.enqueue(a);
        sched.enqueue(b);

        sched.run_next(); // a runs
        sched.run_next(); // b runs, a goes back to tail
        assert_eq!(sched.running(), Some(b));
        assert_eq!(sched.ready_count(), 1);

        sched.run_next(); // a runs again
        assert_eq!(sched.running(), Some(a));
    }
}
