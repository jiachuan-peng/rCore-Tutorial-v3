#![no_std]
#![no_main]

#[macro_use]

extern crate user_lib;

use user_lib::{exit, fork, getpid, set_period, waitpid, yield_};

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("rms_preempt start");
    let low = fork();
    if low == 0 {
        set_period(800);
        for i in 0..80000usize {
            if i % 8000 == 0 {
                println!("low pid={} cnt={}", getpid(), i);
            }
            yield_();
        }
        exit(0);
    }
    for _ in 0..50 {
        yield_();
    }
    let high = fork();
    if high == 0 {
        set_period(20);
        for j in 0..1000usize {
            if j % 300 == 0 {
                println!("high pid={} cnt={}", getpid(), j);
            }
            yield_();
        }
        exit(0);
    }
    let mut ec = 0i32;
    waitpid(high as usize, &mut ec);
    waitpid(low as usize, &mut ec);
    println!("rms_preempt done");
    0
}
