use super::*;
use core::cell::UnsafeCell;
use crate::syscall::{sys_futex_wait, sys_futex_wake};

pub struct Semaphore {
    count: UnsafeCell<i32>,
}

unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}

impl Semaphore {
    pub fn new(count: i32) -> Self {
        Self {
            count: UnsafeCell::new(count)
        }
    }
    
    pub fn wait(&self) {
        let addr = self.count.get();
        atomic_decrement(addr as *mut u32);
        let val = unsafe { *addr };
        if val >= 0 { return; }
        sys_futex_wait(addr, val);
    }

    pub fn post(&self) {
        let addr = self.count.get();
        atomic_increment(addr as *mut u32);
        let val = unsafe { *addr };
        if val <= 0 {
            sys_futex_wake(addr);
        }
    }
}