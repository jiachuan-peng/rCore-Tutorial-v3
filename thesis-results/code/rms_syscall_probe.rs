#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{get_period, get_priority, set_period};

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    const CASES: [(usize, isize); 8] = [
        (1, 0),
        (2, 0),
        (33, 0),
        (34, 1),
        (100, 3),
        (512, 15),
        (1023, 30),
        (1024, 31),
    ];

    println!("TEST name=rms_syscall_probe phase=start");
    for &(period, expected) in &CASES {
        let result = set_period(period);
        let actual_period = get_period();
        let actual_priority = get_priority();
        println!(
            "DATA period={} priority={} expected={} result={}",
            actual_period, actual_priority, expected, result
        );
        if result != 0 || actual_period != period as isize || actual_priority != expected {
            println!(
                "FAIL name=rms_syscall_probe reason=mismatch period={} got={} expected={}",
                period, actual_priority, expected
            );
            return 1;
        }
    }

    let saved_period = get_period();
    let saved_priority = get_priority();
    for invalid in [0usize, 1025] {
        if set_period(invalid) != -1
            || get_period() != saved_period
            || get_priority() != saved_priority
        {
            println!(
                "FAIL name=rms_syscall_probe reason=invalid_changed_state period={}",
                invalid
            );
            return 2;
        }
    }
    println!("PASS name=rms_syscall_probe");
    0
}
