//!Implementation of [`TaskManager`]
use super::{LOWEST_PRIORITY, MAX_PRIORITY, TaskControlBlock, TaskStatus};
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
/// RMS-style multi-level ready queues (`MAX_PRIORITY` FIFO queues).
pub struct TaskManager {
    ready_queues: [VecDeque<Arc<TaskControlBlock>>; MAX_PRIORITY],
}

impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queues: core::array::from_fn(|_| VecDeque::new()),
        }
    }
    /// True when every priority queue is empty.
    pub fn is_empty(&self) -> bool {
        self.ready_queues.iter().all(|q| q.is_empty())
    }
    /// Total tasks sitting in all ready queues.
    pub fn len(&self) -> usize {
        self.ready_queues.iter().map(VecDeque::len).sum()
    }
    /// Debug: print length of each priority queue.
    pub fn debug_print_ready_lens(&self) {
        for (p, q) in self.ready_queues.iter().enumerate() {
            let n = q.len();
            if n != 0 {
                println!(
                    "[TaskManager] ready_queues[{}] len = {}",
                    p, n
                );
            }
        }
    }
    /// Add a task to the tail of its priority queue if it is [`TaskStatus::Ready`].
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        let queue_idx = {
            let task_inner = task.inner_exclusive_access();
            if task_inner.task_status != TaskStatus::Ready {
                return;
            }
            if task_inner.priority >= MAX_PRIORITY {
                LOWEST_PRIORITY
            } else {
                task_inner.priority
            }
        };
        self.ready_queues[queue_idx].push_back(task);
    }
    /// Pop the next runnable task: highest precedence first (`priority` ascending), FIFO within level.
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        for priority in 0..MAX_PRIORITY {
            if let Some(task) = self.ready_queues[priority].pop_front() {
                return Some(task);
            }
        }
        None
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}
///Interface offered to add task
pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.exclusive_access().add(task);
}
///Interface offered to pop the first task
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch()
}
