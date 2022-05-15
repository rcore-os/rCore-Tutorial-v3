#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::{vec::Vec, string::ToString};
use alloc::string::String;

static SUCC_TESTS: &[&str] = &[
    "matrix\0",
    "exit\0",
    "fantastic_text\0",
    "filetest_simple\0",
    "forktest_simple\0",
    "forktest\0",
    "forktest2\0",
    "forktree\0",
    "hello_world\0",
    "huge_write\0",
    "mpsc_sem\0",
    "phil_din_mutex\0",
    "pipe_large_test\0",
    "pipetest\0",
    "race_adder_atomic\0",
    "race_adder_mutex_blocking\0",
    "race_adder_mutex_spin\0",
    "sleep_simple\0",
    "sleep\0",
    "sleep_simple\0",
    "sync_sem\0",
    "test_condvar\0",
    "threads_arg\0",
    "threads\0",
    "yield\0",
    "run_pipe_test\0",
];

static FAIL_TESTS: &[&str] = &[
    "stack_overflow\0",
    "race_adder_loop\0",
    "priv_csr\0",
    "priv_inst\0",
    "store_fault\0",
    "until_timeout\0",
    "stack_overflow\0",
    "race_adder\0",
    "race_adder_arg\0",
];

use user_lib::{exec, fork, waitpid};

fn run_one_test(test: &str) -> i32 {
    println!("Usertests: Running {}", test);
    let pid = fork();
    if pid == 0 {
        exec(test, &[core::ptr::null::<u8>()]);
        return -1
    } else {
        let mut exit_code: i32 = Default::default();
        let wait_pid = waitpid(pid as usize, &mut exit_code);
        assert_eq!(pid, wait_pid);
        println!(
            "\x1b[32mUsertests: Test {} in Process {} exited with code {}\x1b[0m",
            test, pid, exit_code
        );

        return exit_code
    }
}



#[no_mangle]
pub fn main() -> i32 {
    // tests should succ
    let mut succ_tests: Vec<String> = Vec::new();
    // tests should fail
    let mut fail_tests: Vec<String> = Vec::new();

    // other
    let mut fail_list: Vec<String> = Vec::new();

    for test in SUCC_TESTS {
        let result = run_one_test(test);
        if result == 0{
            succ_tests.push(test.to_string());
        }else{
            fail_list.push(test.to_string())
        }
    }
    for test in FAIL_TESTS {
        let result = run_one_test(test);
        if result != 0{
            fail_tests.push(test.to_string());
        }else{
            fail_list.push(test.to_string())
        }
    }

    if succ_tests.len() == SUCC_TESTS.len() && fail_tests.len() == FAIL_TESTS.len() {
        println!("\x1b[32mALL Usertests passed!\x1b[0m");
    }else{
        println!("\x1b[31mUsertests failed!\x1b[0m");
        println!("\x1b[32mPassed List\x1b[0m");
        for i in succ_tests.iter(){
            println!("{}",i);
        }
        for i in fail_tests.iter(){
            println!("{}",i);
        }
        println!("\x1b[31mFail List\x1b[0m");
        for i in fail_list.iter(){
            println!("{}",i);
        }
    }
    0
}
