#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::hint::black_box;
use user_lib::{
    exit, fork_with_period, get_time, getpid, sched_stats_dump, sched_stats_reset, set_period,
    waitpid,
};

fn run_worker(duration_ms: isize) -> usize {
    let deadline = get_time() + duration_ms;
    let mut units = 0usize;
    let mut value = getpid() as usize + 1;
    while get_time() < deadline {
        for i in 0..25_000usize {
            value = value.rotate_left(7) ^ i.wrapping_mul(0x9e37);
        }
        units += 1;
    }
    black_box(value);
    units
}

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("TEST name=rms_rr_fairness phase=start");
    if set_period(1) != 0 {
        println!("FAIL name=rms_rr_fairness reason=parent_set_period");
        return 1;
    }

    sched_stats_reset();
    let mut children = [0isize; 3];
    for (index, slot) in children.iter_mut().enumerate() {
        let pid = fork_with_period(400);
        if pid < 0 {
            println!("FAIL name=rms_rr_fairness reason=fork index={}", index);
            return 2;
        }
        if pid == 0 {
            let start = get_time();
            println!("EVENT type=worker_start pid={} time={}", getpid(), start);
            let units = run_worker(1000);
            println!(
                "RESULT type=worker pid={} work_units={} elapsed_ms={}",
                getpid(),
                units,
                get_time() - start
            );
            exit(0);
        }
        *slot = pid;
    }

    let mut exit_code = -1;
    for &pid in &children {
        if waitpid(pid as usize, &mut exit_code) != pid || exit_code != 0 {
            println!(
                "FAIL name=rms_rr_fairness reason=wait pid={} code={}",
                pid, exit_code
            );
            return 3;
        }
    }
    sched_stats_dump();
    println!("PASS name=rms_rr_fairness metrics=derive_from_RESULT_lines");
    0
}
