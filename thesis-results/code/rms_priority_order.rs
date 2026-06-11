#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{exit, fork, get_period, get_priority, getpid, set_period, waitpid};

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("rms_priority_order start");
    let periods = [20usize, 100, 500];
    let mut pids = [0isize; 3];
    for i in 0..3 {
        let p = fork();
        if p == 0 {
            println!(
                "[warmup] child pid={} (default period/prio until set_period)",
                getpid()
            );
            if set_period(periods[i]) != 0 {
                println!("set_period failed");
                exit(-1);
            }
            let prio = get_priority();
            let per = get_period();
            println!(
                "[configured] child pid={} period={} prio={}",
                getpid(),
                per,
                prio
            );

            exit(0);
        }
        pids[i] = p;
    }
    let mut ec = 0i32;
    for i in 0..3 {
        waitpid(pids[i] as usize, &mut ec);
    }
    println!("rms_priority_order done");
    0
}
