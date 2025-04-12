use super::*;
use core::cell::UnsafeCell;
use crate::syscall::{sys_futex_wait, sys_futex_wake};

pub struct Condvar {
    flag: UnsafeCell<i32>,
}

unsafe impl Send for Condvar {}
unsafe impl Sync for Condvar {}

impl Condvar {
    pub fn new() -> Self {
        Self {
            flag: UnsafeCell::new(0)
        }
    }

    pub fn wait(&self, mutex: &MutexSpin) {
        mutex.unlock();
        let addr = self.flag.get();
        let val = unsafe { *addr }; 
        sys_futex_wait(addr, val);
        mutex.lock();
    }

    pub fn notify_one(&self) {
        let addr = self.flag.get();
        atomic_increment(addr as *mut u32);
        sys_futex_wake(addr);
    }
}