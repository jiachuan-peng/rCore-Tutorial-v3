#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::hint::black_box;
use user_lib::{exit, fork, get_time, getpid, sched_stats_dump, sched_stats_reset, waitpid};

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
    println!("TEST name=waitpid_common phase=start");
    let child = fork();
    if child < 0 {
        println!("FAIL name=waitpid_common reason=fork");
        return 1;
    }
    if child == 0 {
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
    let start = get_time();
    let mut exit_code = -1;
    let waited = waitpid(child as usize, &mut exit_code);
    println!(
        "RESULT waited_pid={} exit_code={} elapsed_ms={}",
        waited,
        exit_code,
        get_time() - start
    );
    sched_stats_dump();
    if waited != child || exit_code != 37 {
        println!("FAIL name=waitpid_common reason=wait_result");
        return 2;
    }
    println!("PASS name=waitpid_common");
    0
}
