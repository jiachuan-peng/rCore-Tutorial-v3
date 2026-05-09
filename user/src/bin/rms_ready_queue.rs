#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{exit, fork, get_time, get_period, get_priority, getpid, set_period, waitpid, yield_};

/// 用户态可运行自检（`make run` 后在 shell 里执行 `rms_ready_queue`）。
///
/// 内核就绪队列无法直接访问，这里用等价可观测行为覆盖：
/// - **查找**：`set_period` 后用 `get_period` / `get_priority` 读回 RMS 参数（与入队所用 priority 一致）。
/// - **增加**：`fork` 后子进程成为可调度任务，由内核加入对应优先级就绪队列。
/// - **删除**：子进程 `exit` 后父进程 `waitpid` 回收；僵尸不会重新入队（与调度器约定一致）。
#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("rms_ready_queue start");

   

    let periods = [150usize, 150, 40];
    let exit_codes = [31i32, 32, 33];
    let mut kids = [0isize; 3];

    for i in 0..3 {
        let p = fork();
        let start_time = get_time();
        println!("fork pid={} start_time={}", getpid(), start_time);
        if p == 0 {
            if set_period(periods[i]) != 0 {
                println!("set_period failed");
                exit(-1);
            }
            let prio = get_priority();
            let per = get_period();
            //let time_slice = get_time_slice();
            //let remaining_slice = get_remaining_slice();
            println!("lookup: pid period prio time_slice remaining_slice:");
            println!("{}, {}, {}", getpid(), per, prio);

            let rounds = if periods[i] == 40 {
                120usize
            } else {
                300usize
            };
            for c in 0..rounds {
                if c % 50 == 0 {
                    let current_time = get_time();
                    println!("                               run pid={} current_time={} times={}", getpid(), current_time, c);
                }
                yield_();
            }
            exit(exit_codes[i]);
        }
        kids[i] = p;
    }

    let mut ec = 0i32;
    for i in 0..3 {
        let w = waitpid(kids[i] as usize, &mut ec);
        assert_eq!(w, kids[i]);
        assert_eq!(ec, exit_codes[i]);
    }

    println!("rms_ready_queue done");
    0
}
