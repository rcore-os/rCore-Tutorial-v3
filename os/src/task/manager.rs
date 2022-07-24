//!Implementation of [`TaskManager`]
use core::borrow::Borrow;
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;

///A array of `TaskControlBlock` that is thread-safe
#[repr(C)]
pub struct TaskManager {
    pub upper_border:usize,
    pub ready_queue: VecDeque<Arc<TaskControlBlock>>,
    pub lower_border:usize,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            upper_border:0xAAAAAAAA,
            ready_queue: VecDeque::new(),
            lower_border:0xBBBBBBBB
        }
    }
    ///Add a task to `TaskManager`
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    ///Remove the first task and return it,or `None` if `TaskManager` is empty
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }
    
    pub fn get_ready_queue_pointer(&mut self) -> &VecDeque<Arc<TaskControlBlock>> {
        &self.ready_queue
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}
///Interface offered to add task
pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.exclusive_access().add(task);
    //println!("TASK_MANAGER in {:p}\n",&TASK_MANAGER);
    unsafe{
        TM_RQ=&TASK_MANAGER.exclusive_access().ready_queue as *const _ as usize;
    }    
}
///Interface offered to pop the first task
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch()
}
