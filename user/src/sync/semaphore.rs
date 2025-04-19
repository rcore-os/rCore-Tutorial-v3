use super::*;
use core::cell::UnsafeCell;

pub struct Semaphore {
    count: UnsafeCell<i32>,
    cond: Condvar,
    mutex: MutexSpin,
}

unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}

impl Semaphore {
    pub fn new(count: i32) -> Self {
        Self {
            count: UnsafeCell::new(count),
            cond: Condvar::new(),
            mutex: MutexSpin::new(),
        }
    }
    
    pub fn wait(&self) {
        self.mutex.lock();
        let cnt_p = self.count.get();
        while unsafe { *cnt_p <= 0 } {
            self.cond.wait(&self.mutex);
        }
        unsafe { *cnt_p -= 1; }
        self.mutex.unlock();
    }

    pub fn post(&self) {
        self.mutex.lock();
        let cnt_p = self.count.get();
        unsafe { *cnt_p += 1; }
        self.cond.notify_one();
        self.mutex.unlock();
    }
}