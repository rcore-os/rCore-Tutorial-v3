use super::{load_reserved, store_conditional};
use core::cell::UnsafeCell;
use crate::task::TaskControlBlock;
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

// Futex的等待队列，由位于内核的进程控制块维护
pub struct FutexQ {
    pub guard: MutexSpin,
    queue: UnsafeCell<VecDeque<Arc<TaskControlBlock>>>,
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
    pub fn pop_front(&self) -> Option<Arc<TaskControlBlock>> {
        unsafe { &mut *self.queue.get() }.pop_front()
    }
}