//! Lightweight counters for the unchanged baseline scheduler.

use crate::sync::UPSafeCell;
use lazy_static::lazy_static;

#[derive(Clone, Copy, Default)]
pub struct SchedStats {
    pub timer_interrupts: usize,
    pub schedule_calls: usize,
    pub context_switches: usize,
    pub fetch_calls: usize,
    pub queue_levels_scanned: usize,
    pub waitpid_not_ready_returns: usize,
    pub yield_calls_during_wait: usize,
}

lazy_static! {
    static ref SCHED_STATS: UPSafeCell<SchedStats> =
        unsafe { UPSafeCell::new(SchedStats::default()) };
}

fn update(f: impl FnOnce(&mut SchedStats)) {
    f(&mut SCHED_STATS.exclusive_access());
}

/// Reset all baseline experiment counters.
pub fn reset() {
    *SCHED_STATS.exclusive_access() = SchedStats::default();
}

/// Return a consistent copy of the baseline counters.
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

pub fn inc_fetch_calls() {
    update(|stats| {
        stats.fetch_calls += 1;
        stats.queue_levels_scanned += 1;
    });
}

pub fn inc_waitpid_not_ready_returns() {
    update(|stats| stats.waitpid_not_ready_returns += 1);
}

pub fn inc_yield_calls_during_wait() {
    update(|stats| stats.yield_calls_during_wait += 1);
}
