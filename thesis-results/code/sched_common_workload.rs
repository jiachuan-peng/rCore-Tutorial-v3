#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::hint::black_box;
use user_lib::{exit, fork, get_time, getpid, sched_stats_dump, sched_stats_reset, waitpid};

fn fixed_work() -> usize {
    let mut value = getpid() as usize + 1;
    for i in 0..80_000_000usize {
        value = value.wrapping_mul(1664525).wrapping_add(i ^ 1013904223);
    }
    black_box(value)
}

fn run_case(tasks: usize) -> bool {
    let mut children = [0isize; 8];
    sched_stats_reset();
    let start = get_time();
    for slot in children.iter_mut().take(tasks) {
        let pid = fork();
        if pid < 0 {
            println!(
                "FAIL name=sched_common_workload reason=fork tasks={}",
                tasks
            );
            return false;
        }
        if pid == 0 {
            let checksum = fixed_work();
            println!(
                "RESULT type=worker tasks={} pid={} checksum={}",
                tasks,
                getpid(),
                checksum
            );
            exit(0);
        }
        *slot = pid;
    }

    let mut exit_code = -1;
    for &pid in children.iter().take(tasks) {
        if waitpid(pid as usize, &mut exit_code) != pid || exit_code != 0 {
            println!(
                "FAIL name=sched_common_workload reason=wait tasks={} pid={}",
                tasks, pid
            );
            return false;
        }
    }
    println!(
        "RESULT type=case tasks={} elapsed_ms={}",
        tasks,
        get_time() - start
    );
    sched_stats_dump();
    true
}

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("TEST name=sched_common_workload phase=start");
    for tasks in [1usize, 2, 4, 8] {
        if !run_case(tasks) {
            return 1;
        }
    }
    println!("PASS name=sched_common_workload");
    0
}
