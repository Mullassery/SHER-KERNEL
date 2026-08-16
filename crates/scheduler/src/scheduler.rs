//! Priority-based heterogeneous task scheduler.
//!
//! This is a real, tested, userspace scheduling algorithm — but it is a
//! *simulation* of kernel-level scheduling. It decides which [`Task`] should
//! run next for a given [`ComputeTarget`]; it does not perform actual OS
//! thread/process context switches or CPU affinity pinning (that requires
//! privileged, platform-specific syscalls this crate deliberately does not
//! attempt).
//!
//! ## Algorithm
//! Per compute target, tasks are kept in a pending pool. `dispatch_next`
//! selects the pending task with the highest `priority`; ties are broken in
//! FIFO submission order (the task submitted earliest wins). This models a
//! classic static-priority scheduler with FIFO tie-breaking, which is a
//! reasonable default for a heterogeneous-compute kernel where each
//! accelerator class (CPU/GPU/NPU/DSP/...) is scheduled independently.

use crate::heterogeneous::ComputeTarget;
use crate::task::{Task, TaskState};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Scheduler {
    pending: HashMap<ComputeTarget, Vec<Task>>,
    running: HashMap<ComputeTarget, Vec<Task>>,
    completed: Vec<Task>,
    failed: Vec<(Task, String)>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Submit a task to be scheduled onto `target`. The task must currently
    /// be in `TaskState::Pending`.
    pub fn submit(&mut self, task: Task, target: ComputeTarget) {
        self.pending.entry(target).or_default().push(task);
    }

    /// Select and remove the highest-priority pending task for `target`,
    /// transition it to `Running`, and return it. Returns `None` if there is
    /// no pending work for that target.
    pub fn dispatch_next(&mut self, target: ComputeTarget) -> Option<Task> {
        let queue = self.pending.get_mut(&target)?;
        let idx = queue
            .iter()
            .enumerate()
            .max_by_key(|(i, t)| (t.priority, std::cmp::Reverse(*i)))
            .map(|(i, _)| i)?;
        let mut task = queue.remove(idx);
        task.state = TaskState::Running;
        self.running.entry(target).or_default().push(task.clone());
        Some(task)
    }

    /// Mark a running task as completed successfully.
    pub fn complete(&mut self, target: ComputeTarget, task_id: sher_common::ObjectId) -> bool {
        if let Some(running) = self.running.get_mut(&target) {
            if let Some(pos) = running.iter().position(|t| t.id == task_id) {
                let mut task = running.remove(pos);
                task.state = TaskState::Completed;
                self.completed.push(task);
                return true;
            }
        }
        false
    }

    /// Mark a running task as failed with a reason.
    pub fn fail(
        &mut self,
        target: ComputeTarget,
        task_id: sher_common::ObjectId,
        reason: impl Into<String>,
    ) -> bool {
        if let Some(running) = self.running.get_mut(&target) {
            if let Some(pos) = running.iter().position(|t| t.id == task_id) {
                let mut task = running.remove(pos);
                task.state = TaskState::Failed;
                self.failed.push((task, reason.into()));
                return true;
            }
        }
        false
    }

    pub fn pending_count(&self, target: ComputeTarget) -> usize {
        self.pending.get(&target).map(Vec::len).unwrap_or(0)
    }

    pub fn running_count(&self, target: ComputeTarget) -> usize {
        self.running.get(&target).map(Vec::len).unwrap_or(0)
    }

    pub fn completed_count(&self) -> usize {
        self.completed.len()
    }

    pub fn failed_count(&self) -> usize {
        self.failed.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatches_highest_priority_first() {
        let mut s = Scheduler::new();
        s.submit(Task::new("low", 1), ComputeTarget::Cpu);
        s.submit(Task::new("high", 10), ComputeTarget::Cpu);
        s.submit(Task::new("mid", 5), ComputeTarget::Cpu);

        let first = s.dispatch_next(ComputeTarget::Cpu).unwrap();
        assert_eq!(first.name, "high");
        assert_eq!(first.state, TaskState::Running);

        let second = s.dispatch_next(ComputeTarget::Cpu).unwrap();
        assert_eq!(second.name, "mid");

        let third = s.dispatch_next(ComputeTarget::Cpu).unwrap();
        assert_eq!(third.name, "low");

        assert!(s.dispatch_next(ComputeTarget::Cpu).is_none());
    }

    #[test]
    fn ties_broken_fifo() {
        let mut s = Scheduler::new();
        s.submit(Task::new("first", 5), ComputeTarget::Gpu);
        s.submit(Task::new("second", 5), ComputeTarget::Gpu);

        let first = s.dispatch_next(ComputeTarget::Gpu).unwrap();
        assert_eq!(first.name, "first");
        let second = s.dispatch_next(ComputeTarget::Gpu).unwrap();
        assert_eq!(second.name, "second");
    }

    #[test]
    fn targets_are_scheduled_independently() {
        let mut s = Scheduler::new();
        s.submit(Task::new("cpu-task", 1), ComputeTarget::Cpu);
        s.submit(Task::new("gpu-task", 1), ComputeTarget::Gpu);

        assert_eq!(s.pending_count(ComputeTarget::Cpu), 1);
        assert_eq!(s.pending_count(ComputeTarget::Gpu), 1);

        let dispatched = s.dispatch_next(ComputeTarget::Cpu).unwrap();
        assert_eq!(dispatched.name, "cpu-task");
        assert_eq!(s.pending_count(ComputeTarget::Gpu), 1);
    }

    #[test]
    fn complete_and_fail_move_task_out_of_running() {
        let mut s = Scheduler::new();
        s.submit(Task::new("task-a", 1), ComputeTarget::Cpu);
        s.submit(Task::new("task-b", 1), ComputeTarget::Cpu);

        let a = s.dispatch_next(ComputeTarget::Cpu).unwrap();
        let b = s.dispatch_next(ComputeTarget::Cpu).unwrap();
        assert_eq!(s.running_count(ComputeTarget::Cpu), 2);

        assert!(s.complete(ComputeTarget::Cpu, a.id));
        assert_eq!(s.completed_count(), 1);
        assert_eq!(s.running_count(ComputeTarget::Cpu), 1);

        assert!(s.fail(ComputeTarget::Cpu, b.id, "simulated fault"));
        assert_eq!(s.failed_count(), 1);
        assert_eq!(s.running_count(ComputeTarget::Cpu), 0);
    }

    #[test]
    fn completing_unknown_task_returns_false() {
        let mut s = Scheduler::new();
        assert!(!s.complete(ComputeTarget::Cpu, sher_common::ObjectId::new()));
    }
}
