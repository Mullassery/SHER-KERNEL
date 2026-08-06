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
