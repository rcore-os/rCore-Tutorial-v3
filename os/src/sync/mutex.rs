use super::{atomic_add_and_compare, atomic_decrement, atomic_increment, atomic_test_and_set, load_reserved, store_conditional};
use core::cell::UnsafeCell;
use crate::task::TaskControlBlock;
use crate::syscall::syscall;
use crate::syscall::SYSCALL_FUTEX;
use alloc::{collections::VecDeque, sync::Arc};

pub const FUTEX_WAIT: usize = 0;
pub const FUTEX_WAKE: usize = 1;
pub trait Mutex: Sync + Send {
    fn lock(&self);
    fn unlock(&self);
}

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
}

impl Mutex for MutexSpin {
    fn lock(&self) {
        let addr = self.locked.get() as *const u32;
        loop {
            while load_reserved(addr) == 1 {}
            if store_conditional(addr, 0) { break; }
        }
    }

    fn unlock(&self) {
        let addr = self.locked.get();
        unsafe { *addr = 0; }
    }
}

/// 只允许在用户态使用 Futex
pub struct Futex {
    flag: UnsafeCell<i32>,  
}

unsafe impl Send for Futex {}
unsafe impl Sync for Futex {}

// Futex的等待队列，由位于内核的进程控制块维护
pub struct FutexQ {
    pub guard: MutexSpin,
    queue: UnsafeCell<VecDeque<Arc<TaskControlBlock>>>,
}

impl Futex {
    pub fn new() -> Self {
        Self {
            flag: UnsafeCell::new(0),
        }
    }
}

impl Mutex for Futex {
    fn lock(&self) {
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
            syscall(SYSCALL_FUTEX, [addr as usize, FUTEX_WAIT, flag as usize]);
        }
    }

    fn unlock(&self) {
        let addr = self.flag.get();
        if atomic_add_and_compare(addr as *mut u32, 0x80000000, 0) {
            // no threads are waiting
            return;
        }         
        syscall(SYSCALL_FUTEX, [addr as usize, FUTEX_WAKE, 0]);
    }
}

impl FutexQ {
    pub fn new() -> Self {
        Self {
            guard: MutexSpin::new(),
            queue: UnsafeCell::new(VecDeque::new()), 
        }
    }

    /// 外部调用必须确保已拿到 FutexQ 的自旋锁
    pub fn push_back(&self, task: Arc<TaskControlBlock>) {
        unsafe { &mut *self.queue.get() }.push_back(task);        
    } 

    /// 外部调用必须确保已拿到 FutexQ 的自旋锁
    pub fn pop_front(&self) -> Arc<TaskControlBlock> {
        unsafe { &mut *self.queue.get() }.pop_front().unwrap()
    }
}