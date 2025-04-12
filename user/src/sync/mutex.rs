use super::*;
use core::cell::UnsafeCell;
use crate::syscall::{sys_futex_wait, sys_futex_wake};

pub struct MutexSpin {
    locked: UnsafeCell<u32>,
}

unsafe impl Send for MutexSpin {}
unsafe impl Sync for MutexSpin {}

impl MutexSpin {
    pub fn new() -> Self {
        Self {
            locked: UnsafeCell::new(0),
        }
    }

    pub fn lock(&self) {
        let addr = self.locked.get() as *const u32;
        loop {
            while load_reserved(addr) == 1 {}
            if store_conditional(addr, 1) { return; }
        }
    }

    pub fn unlock(&self) {
        let addr = self.locked.get();
        unsafe { *addr = 0; }
    }
}

pub struct Futex {
    flag: UnsafeCell<i32>,  
}

unsafe impl Send for Futex {}
unsafe impl Sync for Futex {}

impl Futex {
    pub fn new() -> Self {
        Self {
            flag: UnsafeCell::new(0),
        }
    }

    pub fn lock(&self) {
        let addr = self.flag.get();
        if atomic_test_and_set(addr as *mut u32, 31) == 0 {
            // fastpath
            return;
        }
        // one more waiter
        atomic_increment(addr as *mut u32);
        loop {
            if atomic_test_and_set(addr as *mut u32, 31) == 0 {
                atomic_decrement(addr as *mut u32);
                return;
            }
            let flag = unsafe { *addr };
            if flag >= 0 { continue; }
            // we have to wait now
            sys_futex_wait(addr, flag);
        }
    }

    pub fn unlock(&self) {
        let addr = self.flag.get();
        if atomic_add_and_compare(addr as *mut u32, 0x80000000, 0) {
            // no threads are waiting
            return;
        }         
        sys_futex_wake(addr);
    }
}