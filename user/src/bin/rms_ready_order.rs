#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{
    exit, fork_with_period, get_period, get_priority, get_time, getpid, set_period, waitpid,
};

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    const PERIODS: [usize; 3] = [900, 400, 100];

    println!("TEST name=rms_ready_order phase=start");
    if set_period(1) != 0 {
        println!("FAIL name=rms_ready_order reason=parent_set_period");
        return 1;
    }

    let mut children = [0isize; 3];
    for (index, &period) in PERIODS.iter().enumerate() {
        let pid = fork_with_period(period);
        if pid < 0 {
            println!("FAIL name=rms_ready_order reason=fork period={}", period);
            return 2;
        }
        if pid == 0 {
            println!(
                "EVENT type=first_run pid={} period={} priority={} time={}",
                getpid(),
                get_period(),
                get_priority(),
                get_time()
            );
            exit(index as i32);
        }
        children[index] = pid;
        println!("DATA type=created pid={} period={}", pid, period);
    }

    let mut exit_code = -1;
    for (index, &pid) in children.iter().enumerate() {
        let waited = waitpid(pid as usize, &mut exit_code);
        if waited != pid || exit_code != index as i32 {
            println!(
                "FAIL name=rms_ready_order reason=wait pid={} got={} code={}",
                pid, waited, exit_code
            );
            return 3;
        }
    }
    println!("PASS name=rms_ready_order expected_period_order=100,400,900");
    0
}
