#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use core::hint::spin_loop;

use user_lib::{exit, fork, getpid, set_period, waitpid, yield_};

/// Busy-spin in user mode so the timer can fire between syscall polls.
fn spin_delay() {
    for _ in 0..80_000usize {
        spin_loop();
    }
}

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("rms_remaining_slice start");

    // --- A: watch remaining_slice count down (needs user-mode time between reads) ---
    println!("--- A: remaining_slice should decrease across timer ticks ---");
    if set_period(512) != 0 {
        println!("set_period failed");
        exit(-1);
    }
    let mut last = -1isize;
    for poll in 0..24usize {
        spin_delay();
        let r = user_lib::get_remaining_slice();
        if r < 0 {
            println!("get_remaining_slice error");
            exit(-2);
        }
        if r != last {
            println!(
                "[A] pid={} poll={} remaining_slice={}",
                getpid(),
                poll,
                r
            );
            last = r;
        }
    }

    // --- B: same priority round-robin (parent + two children, identical period) ---
    println!("--- B: same-priority RR (two children + parent, period=400) ---");
    if set_period(400) != 0 {
        println!("set_period B failed");
        exit(-3);
    }

    let mut children = [0isize; 2];
    for i in 0..2 {
        let p = fork();
        if p == 0 {
            if set_period(400) != 0 {
                exit(-4);
            }
            for _ in 0..8usize {
                println!(
                    "[B] child pid={} remaining={}",
                    getpid(),
                    user_lib::get_remaining_slice()
                );
                spin_delay();
                yield_();
            }
            exit(0);
        }
        children[i] = p;
    }

    for _ in 0..6usize {
        println!(
            "[B] parent pid={} remaining={}",
            getpid(),
            user_lib::get_remaining_slice()
        );
        spin_delay();
        yield_();
    }

    let mut ec = 0i32;
    for pid in children {
        waitpid(pid as usize, &mut ec);
    }

    // --- C: fork higher-priority child while low-priority parent runs ---
    println!("--- C: preempt after fork: parent slow period, child fast period ---");
    if set_period(900) != 0 {
        println!("set_period C parent failed");
        exit(-5);
    }

    let high_pid = fork();
    if high_pid == 0 {
        if set_period(10) != 0 {
            exit(-6);
        }
        for i in 0..12usize {
            println!(
                "[C][HIGH prio] pid={} i={} remaining={}",
                getpid(),
                i,
                user_lib::get_remaining_slice()
            );
            spin_delay();
            yield_();
        }
        exit(0);
    }

    println!(
        "[C][LOW prio] parent pid={} forked high-priority child pid={}",
        getpid(),
        high_pid
    );
    for i in 0..20usize {
        println!(
            "[C][LOW prio] pid={} i={} remaining={}",
            getpid(),
            i,
            user_lib::get_remaining_slice()
        );
        spin_delay();
        // intentional: mostly no yield so parent stays runnable until timer preempt / higher task
        if i % 7 == 6 {
            yield_();
        }
    }

    waitpid(high_pid as usize, &mut ec);

    println!("rms_remaining_slice done");
    0
}
