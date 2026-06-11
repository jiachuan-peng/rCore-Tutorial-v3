#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::hint::black_box;
use user_lib::{
    exit, fork_with_period, get_time, getpid, sched_stats_dump, sched_stats_reset, set_period,
    waitpid,
};

fn busy_until(deadline: isize) -> usize {
    let mut units = 0usize;
    let mut value = 1usize;
    while get_time() < deadline {
        for i in 0..20_000usize {
            value = value.wrapping_mul(1664525).wrapping_add(i + 1013904223);
        }
        units += 1;
    }
    black_box(value);
    units
}

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("TEST name=rms_preempt_strict phase=start");
    if set_period(900) != 0 {
        println!("FAIL name=rms_preempt_strict reason=parent_set_period");
        return 1;
    }

    sched_stats_reset();
    let create_time = get_time();
    println!("EVENT type=high_ready_base time={}", create_time);
    let child = fork_with_period(100);
    if child < 0 {
        println!("FAIL name=rms_preempt_strict reason=fork");
        return 2;
    }
    if child == 0 {
        let start = get_time();
        let latency = start - create_time;
        println!(
            "EVENT type=high_first_run pid={} time={} latency_ms={}",
            getpid(),
            start,
            latency
        );
        exit(latency as i32);
    }

    let units = busy_until(create_time + 250);
    let low_end = get_time();
    println!(
        "EVENT type=low_finish pid={} time={} work_units={}",
        getpid(),
        low_end,
        units
    );
    let mut exit_code = -1;
    let waited = waitpid(child as usize, &mut exit_code);
    sched_stats_dump();
    let high_start = create_time + exit_code as isize;
    if waited != child || exit_code < 0 || high_start >= low_end {
        println!(
            "FAIL name=rms_preempt_strict reason=timing waited={} latency_ms={} high_start={} low_finish={}",
            waited, exit_code, high_start, low_end
        );
        return 3;
    }
    println!(
        "RESULT create_time_ms={} high_start_ms={} low_finish_ms={} latency_ms={}",
        create_time, high_start, low_end, exit_code
    );
    println!("PASS name=rms_preempt_strict");
    0
}
