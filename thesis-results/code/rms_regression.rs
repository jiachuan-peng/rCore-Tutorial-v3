#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{exit, fork_with_period, waitpid};

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    const PERIODS: [usize; 8] = [1, 34, 100, 256, 400, 512, 900, 1024];
    const ROUNDS: usize = 200;

    println!("TEST name=rms_regression phase=start rounds={}", ROUNDS);
    for round in 0..ROUNDS {
        let period = PERIODS[round % PERIODS.len()];
        let pid = fork_with_period(period);
        if pid < 0 {
            println!(
                "FAIL name=rms_regression reason=fork round={} period={}",
                round, period
            );
            return 1;
        }
        if pid == 0 {
            exit((round % 100) as i32);
        }
        let mut exit_code = -1;
        let waited = waitpid(pid as usize, &mut exit_code);
        if waited != pid || exit_code != (round % 100) as i32 {
            println!(
                "FAIL name=rms_regression reason=wait round={} pid={} got={} code={}",
                round, pid, waited, exit_code
            );
            return 2;
        }
        if round % 25 == 24 {
            println!("DATA completed_rounds={}", round + 1);
        }
    }
    println!("PASS name=rms_regression rounds={}", ROUNDS);
    0
}
