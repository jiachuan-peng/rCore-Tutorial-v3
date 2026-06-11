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

    println!("TEST name=rms_priority_map phase=start");
    for &(period, expected) in &CASES {
        if set_period(period) != 0 {
            println!(
                "FAIL name=rms_priority_map reason=set_failed period={}",
                period
            );
            return 1;
        }
        let got_period = get_period();
        let got_priority = get_priority();
        println!(
            "DATA period={} priority={} expected={}",
            got_period, got_priority, expected
        );
        if got_period != period as isize || got_priority != expected {
            println!(
                "FAIL name=rms_priority_map reason=mismatch period={} got={} expected={}",
                period, got_priority, expected
            );
            return 2;
        }
    }

    let saved_period = get_period();
    let saved_priority = get_priority();
    for invalid in [0usize, 1025] {
        let result = set_period(invalid);
        println!(
            "DATA period={} result={} retained_period={} retained_priority={}",
            invalid,
            result,
            get_period(),
            get_priority()
        );
        if result != -1 || get_period() != saved_period || get_priority() != saved_priority {
            println!(
                "FAIL name=rms_priority_map reason=invalid_changed_state period={}",
                invalid
            );
            return 3;
        }
    }

    println!("PASS name=rms_priority_map");
    0
}
