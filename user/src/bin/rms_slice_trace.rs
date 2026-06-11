#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::hint::black_box;
use user_lib::{
    exit, fork_with_period, get_remaining_slice, get_time, getpid, sched_stats_dump,
    sched_stats_reset, set_period, waitpid,
};

fn work_chunk(seed: usize) -> usize {
    let mut value = seed;
    for i in 0..40_000usize {
        value = value.rotate_left(3).wrapping_add(i ^ 0x5a5a);
    }
    black_box(value)
}

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("TEST name=rms_slice_trace phase=start");
    if set_period(1) != 0 {
        println!("FAIL name=rms_slice_trace reason=parent_set_period");
        return 1;
    }

    sched_stats_reset();
    let mut children = [0isize; 2];
    for slot in &mut children {
        let pid = fork_with_period(400);
        if pid < 0 {
            println!("FAIL name=rms_slice_trace reason=fork");
            return 2;
        }
        if pid == 0 {
            let deadline = get_time() + 250;
            let mut last = -1;
            let mut seed = getpid() as usize;
            while get_time() < deadline {
                seed = work_chunk(seed);
                let remaining = get_remaining_slice();
                if remaining != last {
                    println!(
                        "EVENT type=slice pid={} time={} remaining={}",
                        getpid(),
                        get_time(),
                        remaining
                    );
                    last = remaining;
                }
            }
            println!("RESULT type=worker_done pid={} checksum={}", getpid(), seed);
            exit(0);
        }
        *slot = pid;
    }

    let mut exit_code = -1;
    for &pid in &children {
        if waitpid(pid as usize, &mut exit_code) != pid || exit_code != 0 {
            println!("FAIL name=rms_slice_trace reason=wait pid={}", pid);
            return 3;
        }
    }
    sched_stats_dump();
    println!("PASS name=rms_slice_trace");
    0
}
