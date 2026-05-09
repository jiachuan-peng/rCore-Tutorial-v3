//!Implementation of [`TaskControlBlock`]
use super::TaskContext;
use super::{KernelStack, PidHandle, pid_alloc};
use crate::config::TRAP_CONTEXT;
use crate::mm::{KERNEL_SPACE, MemorySet, PhysPageNum, VirtAddr};
use crate::sync::UPSafeCell;
use crate::trap::{TrapContext, trap_handler};
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::cell::RefMut;

/// Number of RMS static priority levels (0..= [`LOWEST_PRIORITY`]).
pub const MAX_PRIORITY: usize = 32;
/// Smallest `priority` value (highest precedence).
pub const HIGHEST_PRIORITY: usize = 0;
/// Largest `priority` value (lowest precedence).
pub const LOWEST_PRIORITY: usize = MAX_PRIORITY - 1;

/// Minimum valid RMS period (inclusive), in timer ticks.
pub const MIN_PERIOD_TICKS: usize = 1;
/// Maximum valid RMS period (inclusive), in timer ticks.
pub const MAX_PERIOD_TICKS: usize = 1024;
/// Default RMS period for a newly created task, in timer ticks.
pub const DEFAULT_PERIOD_TICKS: usize = 1;
/// Default length of one scheduling time slice, in timer ticks.
pub const DEFAULT_TIME_SLICE: usize = 5;

/// Sentinel for [`TaskControlBlockInner::waiting_pid`] when not blocked in `waitpid`.
pub const WAITPID_NONE: isize = -2;

/// Map an RMS period in timer ticks to a static priority in `0..= LOWEST_PRIORITY`.
///
/// Shorter periods yield smaller (higher-precedence) priorities. `period_ticks` is
/// clamped to [`MIN_PERIOD_TICKS`]..=[`MAX_PERIOD_TICKS`] before mapping.
pub fn period_to_priority(period_ticks: usize) -> usize {
    let period = if period_ticks < MIN_PERIOD_TICKS {
        MIN_PERIOD_TICKS
    } else if period_ticks > MAX_PERIOD_TICKS {
        MAX_PERIOD_TICKS
    } else {
        period_ticks
    };
    (period - MIN_PERIOD_TICKS) * (MAX_PRIORITY - 1) / (MAX_PERIOD_TICKS - MIN_PERIOD_TICKS)
}

pub struct TaskControlBlock {
    // immutable
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    // mutable
    inner: UPSafeCell<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub trap_cx_ppn: PhysPageNum,
    #[allow(unused)]
    pub base_size: usize,
    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,
    pub parent: Option<Weak<TaskControlBlock>>,
    pub children: Vec<Arc<TaskControlBlock>>,
    pub exit_code: i32,
    /// `waitpid` block state: [`WAITPID_NONE`], `-1` (any child), or a specific child pid.
    pub waiting_pid: isize,

    /// RMS period in timer ticks (shorter period => higher RMS priority).
    pub period_ticks: usize,
    /// Static priority: smaller is higher; range `HIGHEST_PRIORITY`..=`LOWEST_PRIORITY`.
    pub priority: usize,
    /// Time slice length for same-priority round-robin.
    pub time_slice: usize,
    /// Remaining time in the current slice.
    pub remaining_slice: usize,
}

impl TaskControlBlockInner {
    /*
    pub fn get_task_cx_ptr2(&self) -> *const usize {
        &self.task_cx_ptr as *const usize
    }
    */
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
    }
}

impl TaskControlBlock {
    pub fn inner_exclusive_access(&self) -> RefMut<'_, TaskControlBlockInner> {
        self.inner.exclusive_access()
    }
    pub fn new(elf_data: &[u8]) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
        let period_ticks = DEFAULT_PERIOD_TICKS;
        let priority = period_to_priority(period_ticks);
        let time_slice = DEFAULT_TIME_SLICE;
        // push a task context which goes to trap_return to the top of kernel stack
        let task_control_block = Self {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx_ppn,
                    base_size: user_sp,
                    task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    memory_set,
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0,
                    waiting_pid: WAITPID_NONE,

                    period_ticks,
                    priority,
                    time_slice,
                    remaining_slice: time_slice,
                })
            },
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }
    pub fn exec(&self, elf_data: &[u8]) {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();

        // **** access inner exclusively
        let mut inner = self.inner_exclusive_access();
        // substitute memory_set
        inner.memory_set = memory_set;
        // update trap_cx ppn
        inner.trap_cx_ppn = trap_cx_ppn;
        // initialize base_size
        inner.base_size = user_sp;
        // initialize trap_cx
        let trap_cx = inner.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            self.kernel_stack.get_top(),
            trap_handler as usize,
        );
        // **** release inner automatically
    }
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        // ---- access parent PCB exclusively
        let mut parent_inner = self.inner_exclusive_access();
        // copy user space(include trap context)
        let memory_set = MemorySet::from_existed_user(&parent_inner.memory_set);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        // alloc a pid and a kernel stack in kernel space
        let pid_handle = pid_alloc();
        let kernel_stack = KernelStack::new(&pid_handle);
        let kernel_stack_top = kernel_stack.get_top();
        let period_ticks = parent_inner.period_ticks;
        let priority = parent_inner.priority;
        let time_slice = parent_inner.time_slice;
        let task_control_block = Arc::new(TaskControlBlock {
            pid: pid_handle,
            kernel_stack,
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    trap_cx_ppn,
                    base_size: parent_inner.base_size,
                    task_cx: TaskContext::goto_trap_return(kernel_stack_top),
                    task_status: TaskStatus::Ready,
                    memory_set,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    exit_code: 0,
                    waiting_pid: WAITPID_NONE,
                    period_ticks,
                    priority,
                    time_slice,
                    remaining_slice: time_slice,
                })
            },
        });
        // add child
        parent_inner.children.push(task_control_block.clone());
        // modify kernel_sp in trap_cx
        // **** access children PCB exclusively
        let trap_cx = task_control_block.inner_exclusive_access().get_trap_cx();
        trap_cx.kernel_sp = kernel_stack_top;
        // return
        task_control_block
        // ---- release parent PCB automatically
        // **** release children PCB automatically
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// Runnable and may be selected by the scheduler.
    Ready,
    /// Currently executing on a CPU.
    Running,
    /// Not runnable (e.g. I/O wait); not used by the current FIFO scheduler.
    Blocked,
    /// Explicitly suspended; not used by the current FIFO scheduler.
    Suspended,
    /// Exited; TCB kept until parent `waitpid` reclaims it.
    Zombie,
}
