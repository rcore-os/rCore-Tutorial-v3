//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.

// const SYSCALL_WRITE: usize = 64;
// const SYSCALL_EXIT: usize = 93;
// const SYSCALL_GETTASKINFO: usize = 100;

mod fs;
pub mod info;
mod process;
mod tool;

use fs::*;
use info::*;
use process::*;
use tool::*;

/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_WRITE => {
            SYSTALL_TABLE.exclusive_access().count(SYSCALL_WRITE);
            sys_write(args[0], args[1] as *const u8, args[2])
        }
        SYSCALL_EXIT => {
            SYSTALL_TABLE.exclusive_access().count(SYSCALL_EXIT);
            sys_exit(args[0] as i32)
        }
        SYSCALL_GETTASKINFO => {
            SYSTALL_TABLE.exclusive_access().count(SYSCALL_GETTASKINFO);
            sys_get_taskinfo(args[0], args[1] as *const u8, args[2])
        }
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
