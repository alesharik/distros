use crate::sched::ctx::TaskContext;
use alloc::collections::BinaryHeap;
use crate::sched::{Task, TaskState};
use core::sync::atomic::{Ordering, AtomicBool};
use crossbeam_queue::SegQueue;

pub struct CpuTaskScheduler {
    // offtask_queue: BinaryHeap<Task>,

    // these queues have only `Ready` tasks
    queue: SegQueue<Task>,
    queue1: SegQueue<Task>,
    current_queue: bool,
}

impl CpuTaskScheduler {
    pub const fn new() -> Self {
        CpuTaskScheduler {
            queue: SegQueue::new(),
            queue1: SegQueue::new(),
            current_queue: false
        }
    }

    #[inline]
    pub fn next_task(&mut self, mut task: Task) -> Task {
        if task.state == TaskState::Running {
            task.state = TaskState::Ready;
        }
        self.push(task);

        let mut next = self.pop();
        next.state = TaskState::Running;
        next
    }

    #[inline]
    fn push(&mut self, task: Task) {
        if self.current_queue {
            self.queue.push(task);
        } else {
            self.queue1.push(task);
        }
    }

    #[inline]
    fn pop(&mut self) -> Task {
        let next = if self.current_queue {
            self.queue1.pop()
        } else {
            self.queue.pop()
        };
        if let Some(next) = next {
            next
        } else {
            self.current_queue = !self.current_queue;
            self.pop()
        }
    }
}