use crate::loader::get_app_data_by_name;
use crate::mm::{translated_refmut, translated_str};
use crate::task::{
    MAX_PERIOD_TICKS, MIN_PERIOD_TICKS, add_task, block_current_and_run_next, current_task,
    current_user_token, exit_current_and_run_next, period_to_priority, reset_sched_stats,
    sched_stats_snapshot, suspend_current_and_run_next,
};
use crate::timer::get_time_ms;
use alloc::sync::Arc;

pub fn sys_exit(exit_code: i32) -> ! {
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

/// Fork a child and assign its RMS parameters before it enters a ready queue.
pub fn sys_fork_with_period(period_ticks: usize) -> isize {
    if !(MIN_PERIOD_TICKS..=MAX_PERIOD_TICKS).contains(&period_ticks) {
        return -1;
    }
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    {
        let mut inner = new_task.inner_exclusive_access();
        inner.period_ticks = period_ticks;
        inner.priority = period_to_priority(period_ticks);
        inner.remaining_slice = inner.time_slice;
        inner.get_trap_cx().x[10] = 0;
    }
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

/// Set RMS period for the current task (ticks); recomputes static priority.
///
/// Rejects values outside the documented `1..=1024` experiment range.
pub fn sys_set_period(period_ticks: usize) -> isize {
    if !(MIN_PERIOD_TICKS..=MAX_PERIOD_TICKS).contains(&period_ticks) {
        return -1;
    }
    let Some(task) = current_task() else {
        return -1;
    };
    let mut inner = task.inner_exclusive_access();
    inner.period_ticks = period_ticks;
    inner.priority = period_to_priority(period_ticks);
    inner.remaining_slice = inner.time_slice;
    0
}

pub fn sys_sched_stats_reset() -> isize {
    reset_sched_stats();
    0
}

pub fn sys_sched_stats_dump() -> isize {
    let stats = sched_stats_snapshot();
    println!(
        "SCHED_STATS timer={} schedule={} switch={} fetch={} scan={} ts_preempt={} prio_preempt={} wait_block={} wakeup={}",
        stats.timer_interrupts,
        stats.schedule_calls,
        stats.context_switches,
        stats.fetch_calls,
        stats.queue_levels_scanned,
        stats.timeslice_preemptions,
        stats.priority_preemptions,
        stats.waitpid_blocks,
        stats.task_wakeups,
    );
    0
}

/// Current task RMS static priority, or `-1` if none.
pub fn sys_get_priority() -> isize {
    current_task()
        .map(|t| t.inner_exclusive_access().priority as isize)
        .unwrap_or(-1)
}

/// Current task RMS period in ticks, or `-1` if none.
pub fn sys_get_period() -> isize {
    current_task()
        .map(|t| t.inner_exclusive_access().period_ticks as isize)
        .unwrap_or(-1)
}

/// Current task `remaining_slice` (timer decrements until 0, then reschedule), or `-1` if none.
pub fn sys_get_remaining_slice() -> isize {
    current_task()
        .map(|t| t.inner_exclusive_access().remaining_slice as isize)
        .unwrap_or(-1)
}

/// If there is not a child process whose pid is same as given, return -1.
/// If matching children exist but none is zombie yet, block until a child exits (event wakeup).
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    loop {
        let task = current_task().unwrap();
        let mut inner = task.inner_exclusive_access();
        if !inner
            .children
            .iter()
            .any(|p| pid == -1 || pid as usize == p.getpid())
        {
            return -1;
        }
        let pair = inner.children.iter().enumerate().find(|(_, p)| {
            p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        });
        if let Some((idx, _)) = pair {
            let child = inner.children.remove(idx);
            assert_eq!(Arc::strong_count(&child), 1);
            let found_pid = child.getpid();
            let exit_code = child.inner_exclusive_access().exit_code;
            *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
            return found_pid as isize;
        }
        let wait_spec = if pid == -1 { -1 } else { pid };
        drop(inner);
        block_current_and_run_next(wait_spec);
    }
}
