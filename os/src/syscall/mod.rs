pub const SYSCALL_DUP: usize = 24;
pub const SYSCALL_OPEN: usize = 56;
pub const SYSCALL_CLOSE: usize = 57;
pub const SYSCALL_PIPE: usize = 59;
pub const SYSCALL_READ: usize = 63;
pub const SYSCALL_WRITE: usize = 64;
pub const SYSCALL_EXIT: usize = 93;
pub const SYSCALL_SLEEP: usize = 101;
pub const SYSCALL_YIELD: usize = 124;
pub const SYSCALL_KILL: usize = 129;
pub const SYSCALL_GET_TIME: usize = 169;
pub const SYSCALL_GETPID: usize = 172;
pub const SYSCALL_FORK: usize = 220;
pub const SYSCALL_EXEC: usize = 221;
pub const SYSCALL_FUTEX: usize = 240;
pub const SYSCALL_WAITPID: usize = 260;
pub const SYSCALL_THREAD_CREATE: usize = 1000;
pub const SYSCALL_GETTID: usize = 1001;
pub const SYSCALL_WAITTID: usize = 1002;
pub const SYSCALL_MUTEX_CREATE: usize = 1010;
pub const SYSCALL_MUTEX_LOCK: usize = 1011;
pub const SYSCALL_MUTEX_UNLOCK: usize = 1012;
pub const SYSCALL_SEMAPHORE_CREATE: usize = 1020;
pub const SYSCALL_SEMAPHORE_UP: usize = 1021;
pub const SYSCALL_SEMAPHORE_DOWN: usize = 1022;
pub const SYSCALL_CONDVAR_CREATE: usize = 1030;
pub const SYSCALL_CONDVAR_SIGNAL: usize = 1031;
pub const SYSCALL_CONDVAR_WAIT: usize = 1032;

mod fs;
mod process;
mod sync;
mod thread;

use fs::*;
use process::*;
use sync::*;
use thread::*;

pub fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}

pub fn syscall_handler(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_DUP => sys_dup(args[0]),
        SYSCALL_OPEN => sys_open(args[0] as *const u8, args[1] as u32),
        SYSCALL_CLOSE => sys_close(args[0]),
        SYSCALL_PIPE => sys_pipe(args[0] as *mut usize),
        SYSCALL_READ => sys_read(args[0], args[1] as *const u8, args[2]),
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_SLEEP => sys_sleep(args[0]),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_KILL => sys_kill(args[0], args[1] as u32),
        SYSCALL_GET_TIME => sys_get_time(),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_FORK => sys_fork(),
        SYSCALL_EXEC => sys_exec(args[0] as *const u8, args[1] as *const usize),
        SYSCALL_FUTEX => sys_futex(args[0] as *const i32, args[1], args[2]),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),
        SYSCALL_THREAD_CREATE => sys_thread_create(args[0], args[1]),
        SYSCALL_GETTID => sys_gettid(),
        SYSCALL_WAITTID => sys_waittid(args[0]) as isize,
        SYSCALL_MUTEX_CREATE => sys_mutex_create(args[0] == 1),
        SYSCALL_MUTEX_LOCK => sys_mutex_lock(args[0]),
        SYSCALL_MUTEX_UNLOCK => sys_mutex_unlock(args[0]),
        SYSCALL_SEMAPHORE_CREATE => sys_semaphore_create(args[0]),
        SYSCALL_SEMAPHORE_UP => sys_semaphore_up(args[0]),
        SYSCALL_SEMAPHORE_DOWN => sys_semaphore_down(args[0]),
        SYSCALL_CONDVAR_CREATE => sys_condvar_create(),
        SYSCALL_CONDVAR_SIGNAL => sys_condvar_signal(args[0]),
        SYSCALL_CONDVAR_WAIT => sys_condvar_wait(args[0], args[1]),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
