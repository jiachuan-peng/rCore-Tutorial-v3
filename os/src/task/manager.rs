//!Implementation of [`TaskManager`]
use super::{ENABLE_SCHED_TRACE, LOWEST_PRIORITY, MAX_PRIORITY, TaskControlBlock, TaskStatus};
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
        if ENABLE_SCHED_TRACE {
            let pid = task.getpid();
            let inn = task.inner_exclusive_access();
            println!(
                "[sched] enqueue pid={} prio={} period={}",
                pid,
                inn.priority,
                inn.period_ticks
            );
        }
        self.ready_queues[queue_idx].push_back(task);
    }
    /// Pop the next runnable task: highest precedence first (`priority` ascending), FIFO within level.
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        for priority in 0..MAX_PRIORITY {
            if let Some(task) = self.ready_queues[priority].pop_front() {
                if ENABLE_SCHED_TRACE {
                    let pid = task.getpid();
                    let inn = task.inner_exclusive_access();
                    println!("[sched] fetch pid={} prio={}", pid, inn.priority);
                }
                return Some(task);
            }
        }
        None
    }
    /// Read-only check: any ready task strictly higher precedence than `current_priority`?
    ///
    /// Does not modify queues or task state. When `current_priority == 0`, nothing is higher.
    pub fn has_higher_priority_task(&self, current_priority: usize) -> bool {
        if current_priority == 0 {
            return false;
        }
        let upper = if current_priority > MAX_PRIORITY {
            MAX_PRIORITY
        } else {
            current_priority
        };
        for priority in 0..upper {
            if !self.ready_queues[priority].is_empty() {
                return true;
            }
        }
        false
    }

    /// Returns whether any ready queue holds a task with the given PID (read-only).
    #[allow(dead_code)]
    pub fn ready_contains_pid(&self, pid: usize) -> bool {
        self.ready_priority_of_pid(pid).is_some()
    }

    /// If a task with `pid` is in some ready queue, returns that queue index (`priority` level).
    #[allow(dead_code)]
    pub fn ready_priority_of_pid(&self, pid: usize) -> Option<usize> {
        for (prio, q) in self.ready_queues.iter().enumerate() {
            if q.iter().any(|t| t.getpid() == pid) {
                return Some(prio);
            }
        }
        None
    }

    /// Removes the first queued task with `pid` from whichever priority queue it sits in.
    ///
    /// Does not change [`TaskControlBlock`] state; only detaches it from ready queues.
    /// Returns `None` if not present.
    #[allow(dead_code)]
    pub fn remove_ready_by_pid(&mut self, pid: usize) -> Option<Arc<TaskControlBlock>> {
        for queue in self.ready_queues.iter_mut() {
            let mut kept = VecDeque::new();
            let mut found = None;
            while let Some(t) = queue.pop_front() {
                if found.is_none() && t.getpid() == pid {
                    found = Some(t);
                } else {
                    kept.push_back(t);
                }
            }
            *queue = kept;
            if found.is_some() {
                return found;
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
