//! App management syscalls
use crate::batch::run_next_app;
use crate::syscall::info::*;
/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    let mut execute_time = EXECUTE_TIME.exclusive_access();
    execute_time.end();
    let interval_ns = execute_time.interval_ns();
    println!("Execution time: {} ns", interval_ns);
    drop(execute_time);
    println!("[kernel] Application exited with code {}", exit_code);
    run_next_app()
}
