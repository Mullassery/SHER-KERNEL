use crate::task::Task;
use std::collections::VecDeque;

#[derive(Debug, Clone, Default)]
pub struct TaskQueue {
    pub tasks: VecDeque<Task>,
}

impl TaskQueue {
    pub fn enqueue(&mut self, task: Task) {
        self.tasks.push_back(task);
    }

    pub fn dequeue(&mut self) -> Option<Task> {
        self.tasks.pop_front()
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enqueue_dequeue_is_fifo() {
        let mut q = TaskQueue::default();
        q.enqueue(Task::new("a", 1));
        q.enqueue(Task::new("b", 1));
        assert_eq!(q.len(), 2);
        assert_eq!(q.dequeue().unwrap().name, "a");
        assert_eq!(q.dequeue().unwrap().name, "b");
        assert!(q.dequeue().is_none());
    }

    #[test]
    fn is_empty_reflects_state() {
        let mut q = TaskQueue::default();
        assert!(q.is_empty());
        q.enqueue(Task::new("a", 1));
        assert!(!q.is_empty());
    }
}
