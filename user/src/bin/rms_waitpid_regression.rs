#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{exit, fork, getpid, set_period, waitpid};

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("rms_waitpid_regression start");
    let periods = [20usize, 100, 500];
    let codes = [11i32, 22, 33];
    let mut kids = [0isize; 3];
    for i in 0..3 {
        let p = fork();
        if p == 0 {
            set_period(periods[i]);
            println!("child pid={} exit_code={}", getpid(), codes[i]);
            exit(codes[i]);
        }
        kids[i] = p;
    }
    let mut ec = 0i32;
    for i in 0..3 {
        let w = waitpid(kids[i] as usize, &mut ec);
        assert_eq!(w, kids[i]);
        assert_eq!(ec, codes[i]);
    }
    println!("rms_waitpid_regression done");
    0
}
