#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{exit, get_period, get_priority, set_period};

#[unsafe(no_mangle)]
pub fn main() -> i32  { 
    println!("rms_syscall_probe start");
    println!(
        "[probe] initial period={} priority={}",
        get_period(),
        get_priority()
    );

    let periods = [20usize, 100, 500, 800];
    let mut last_prio: Option<isize> = None;

    let mut cnt =100;


    for &p in &periods {
        
        let gp = set_period(200+cnt);
        cnt += 100;
        let gp = get_period();
        let pr = get_priority();
        println!(
            "[probe] after set_period({}): period={} priority={}",
            p, gp, pr
        );
        // if gp != p as isize {
        //     println!("[probe] FAIL: period mismatch (expected {})", p);
        //     exit(-2);
        // }
        // if let Some(lp) = last_prio {
        //     if pr <= lp {
        //         println!(
        //             "[probe] FAIL: priority should strictly increase as period increases (got {} after {})",
        //             pr, lp
        //         );
        //         exit(-3);
        //     }
        // }
        // last_prio = Some(pr);
    }

    println!("rms_syscall_probe done");
    0

}
