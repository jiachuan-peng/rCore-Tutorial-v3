#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::hint::black_box;
use user_lib::{
    exit, fork_with_period, get_time, getpid, sched_stats_dump, sched_stats_reset, set_period,
    waitpid,
};

fn busy_for(duration_ms: isize) -> usize {
    let deadline = get_time() + duration_ms;
    let mut value = 1usize;
    while get_time() < deadline {
        for i in 0..30_000usize {
            value = value.wrapping_add(i).rotate_left(5);
        }
    }
    black_box(value)
}

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("TEST name=rms_waitpid_block phase=start");
    if set_period(1) != 0 {
        println!("FAIL name=rms_waitpid_block reason=parent_set_period");
        return 1;
    }
    let child = fork_with_period(900);
    if child < 0 {
        println!("FAIL name=rms_waitpid_block reason=fork");
        return 2;
    }
    if child == 0 {
        println!(
            "EVENT type=child_start pid={} time={}",
            getpid(),
            get_time()
        );
        let checksum = busy_for(300);
        println!(
            "EVENT type=child_exit pid={} time={} checksum={}",
            getpid(),
            get_time(),
            checksum
        );
        exit(37);
    }

    sched_stats_reset();
    let wait_start = get_time();
    println!(
        "EVENT type=parent_wait pid={} child={} time={}",
        getpid(),
        child,
        wait_start
    );
    let mut exit_code = -1;
    let waited = waitpid(child as usize, &mut exit_code);
    let wait_end = get_time();
    println!(
        "RESULT waited_pid={} exit_code={} elapsed_ms={}",
        waited,
        exit_code,
        wait_end - wait_start
    );
    sched_stats_dump();
    if waited != child || exit_code != 37 {
        println!("FAIL name=rms_waitpid_block reason=wait_result");
        return 3;
    }
    println!("PASS name=rms_waitpid_block");
    0
}
