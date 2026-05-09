#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{exit, fork, getpid, set_period, waitpid, yield_};

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("rms_round_robin start");
    let mut pids = [0isize; 3];
    for i in 0..3 {
        let p = fork();
        if p == 0 {
            set_period(100);
            for c in 0..6000usize {
                if c % 400 == 0 {
                    println!("rr pid={} cnt={}", getpid(), c);
                }
                yield_();
            }
            exit(0);
        }
        pids[i] = p;
    }
    let mut ec = 0i32;
    for i in 0..3 {
        waitpid(pids[i] as usize, &mut ec);
    }
    println!("rms_round_robin done");
    0
}
