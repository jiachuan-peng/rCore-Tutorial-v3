//! Lightweight scheduler counters used by thesis experiments.

use crate::sync::UPSafeCell;
use lazy_static::lazy_static;

use super::ENABLE_SCHED_STATS;

#[derive(Clone, Copy, Default)]
pub struct SchedStats {
    pub timer_interrupts: usize,
    pub schedule_calls: usize,
    pub context_switches: usize,
    pub fetch_calls: usize,
    pub queue_levels_scanned: usize,
    pub timeslice_preemptions: usize,
    pub priority_preemptions: usize,
    pub waitpid_blocks: usize,
    pub task_wakeups: usize,
}

lazy_static! {
    static ref SCHED_STATS: UPSafeCell<SchedStats> =
        unsafe { UPSafeCell::new(SchedStats::default()) };
}

fn update(f: impl FnOnce(&mut SchedStats)) {
    if ENABLE_SCHED_STATS {
        f(&mut SCHED_STATS.exclusive_access());
    }
}

/// Reset every scheduler experiment counter to zero.
pub fn reset() {
    *SCHED_STATS.exclusive_access() = SchedStats::default();
}

/// Return a consistent copy of all scheduler experiment counters.
pub fn snapshot() -> SchedStats {
    *SCHED_STATS.exclusive_access()
}

pub fn inc_timer_interrupts() {
    update(|stats| stats.timer_interrupts += 1);
}

pub fn inc_schedule_calls() {
    update(|stats| stats.schedule_calls += 1);
}

pub fn inc_context_switches() {
    update(|stats| stats.context_switches += 1);
}

pub fn record_fetch(queue_levels_scanned: usize) {
    update(|stats| {
        stats.fetch_calls += 1;
        stats.queue_levels_scanned += queue_levels_scanned;
    });
}

pub fn inc_timeslice_preemptions() {
    update(|stats| stats.timeslice_preemptions += 1);
}

pub fn inc_priority_preemptions() {
    update(|stats| stats.priority_preemptions += 1);
}

pub fn inc_waitpid_blocks() {
    update(|stats| stats.waitpid_blocks += 1);
}

pub fn inc_task_wakeups() {
    update(|stats| stats.task_wakeups += 1);
}
