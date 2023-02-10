#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

const TESTS: &[&str] = &[
    "hello_world\0",
    "yield\0",
    "exit\0",
    "forktest_simple\0",
    "forktest\0",
    "forktest2\0",
    "forktree\0",
    "filetest_simple\0",
    "cat\0",
    "pipetest\0",
    "mpsc_sem\0",
    "phil_din_mutex\0",
    "adder_mutex_spin\0",
    "condsync_sem\0",
    "condsync_condvar\0",
    "threads\0",
    "threads_arg\0",
];

const TEST_NUM: usize = TESTS.len();

use user_lib::{exec, fork, waitpid};

#[no_mangle]
pub fn main() -> i32 {
    let mut pids = [0; TEST_NUM];
    for (i, &test) in TESTS.iter().enumerate() {
        println!("Usertests: Running {}", test);
        let pid = fork();
        if pid == 0 {
            exec(&*test, &[core::ptr::null::<u8>()]);
            panic!("unreachable!");
        } else {
            pids[i] = pid;
        }
    }
    let mut xstate: i32 = Default::default();
    for (i, &test) in TESTS.iter().enumerate() {
        let wait_pid = waitpid(pids[i] as usize, &mut xstate);
        assert_eq!(pids[i], wait_pid);
        println!(
            "\x1b[32mUsertests: Test {} in Process {} exited with code {}\x1b[0m",
            test, pids[i], xstate
        );
    }
    println!("Basic usertests passed!");
    0
}
