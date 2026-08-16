//! Shared work-queue implementation backing each accelerator scheduler in
//! this crate (`cpu`, `gpu`, `npu`, `dsp`).
//!
//! This models *queueing and dispatch policy only* — it does not submit real
//! work to a GPU/NPU/DSP device (that would require vendor driver access
//! this userspace crate does not have). Each accelerator module wraps this
//! queue with a type name that documents which class of hardware it stands
//! in for.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub name: String,
    pub priority: u32,
}

impl Job {
    pub fn new(name: impl Into<String>, priority: u32) -> Self {
        Self {
            name: name.into(),
            priority,
        }
    }
}

/// A simple priority work queue: highest `priority` runs first, FIFO among
/// ties.
#[derive(Debug, Default, Clone)]
pub struct WorkQueue {
    jobs: Vec<Job>,
    completed: u64,
}

impl WorkQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn submit(&mut self, job: Job) {
        self.jobs.push(job);
    }

    pub fn pending(&self) -> usize {
        self.jobs.len()
    }

    pub fn completed(&self) -> u64 {
        self.completed
    }

    /// Remove and return the highest-priority pending job, simulating one
    /// unit of dispatch/execution by incrementing the completed counter.
    pub fn run_next(&mut self) -> Option<Job> {
        if self.jobs.is_empty() {
            return None;
        }
        let idx = self
            .jobs
            .iter()
            .enumerate()
            .max_by_key(|(i, j)| (j.priority, std::cmp::Reverse(*i)))
            .map(|(i, _)| i)?;
        let job = self.jobs.remove(idx);
        self.completed += 1;
        Some(job)
    }

    /// Run every pending job in priority order, returning them in dispatch
    /// order.
    pub fn drain_all(&mut self) -> Vec<Job> {
        let mut out = Vec::with_capacity(self.jobs.len());
        while let Some(job) = self.run_next() {
            out.push(job);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_next_respects_priority() {
        let mut q = WorkQueue::new();
        q.submit(Job::new("low", 1));
        q.submit(Job::new("high", 9));
        assert_eq!(q.run_next().unwrap().name, "high");
        assert_eq!(q.run_next().unwrap().name, "low");
        assert!(q.run_next().is_none());
    }

    #[test]
    fn completed_counter_tracks_dispatches() {
        let mut q = WorkQueue::new();
        q.submit(Job::new("a", 1));
        q.submit(Job::new("b", 1));
        assert_eq!(q.completed(), 0);
        q.run_next();
        assert_eq!(q.completed(), 1);
        q.run_next();
        assert_eq!(q.completed(), 2);
    }

    #[test]
    fn drain_all_returns_priority_order() {
        let mut q = WorkQueue::new();
        q.submit(Job::new("mid", 5));
        q.submit(Job::new("high", 10));
        q.submit(Job::new("low", 1));
        let order: Vec<String> = q.drain_all().into_iter().map(|j| j.name).collect();
        assert_eq!(order, vec!["high", "mid", "low"]);
        assert_eq!(q.pending(), 0);
    }
}
