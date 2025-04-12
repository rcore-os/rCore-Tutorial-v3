use crate::sync::{Mutex, Semaphore, FutexQ};
use crate::task::{TaskStatus, TaskContext};
use crate::task::{block_current_and_run_next, current_process, current_task, schedule, take_current_task, wakeup_task};
use crate::timer::{add_timer, get_time_ms};
use crate::sync::{FUTEX_WAIT, FUTEX_WAKE};
use alloc::sync::Arc;

pub fn sys_sleep(ms: usize) -> isize {
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}

pub fn sys_semaphore_create(res_count: usize) -> isize {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        process_inner.semaphore_list.len() - 1
    };
    id as isize
}

pub fn sys_semaphore_up(sem_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    sem.up();
    0
}

pub fn sys_semaphore_down(sem_id: usize) -> isize {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    sem.down();
    0
}

/// 封装在 Futex 方法中，不应直接调用
pub fn sys_futex(uaddr: *const i32, futex_op: usize, val: usize) -> isize{
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let phys_addr = process_inner
        .memory_set
        .translate_va((uaddr as usize).into())
        .unwrap()
        .0 as *const i32;
    // 如果 Futex 第一次调用 sys_futex，则插入新的队列
    let futex_q = process_inner.futex_queues
        .entry(uaddr as usize)
        .or_insert(FutexQ::new());
    match futex_op {
        FUTEX_WAIT => {
            futex_q.guard.lock();
            if unsafe { *phys_addr == (val as i32)} {
                // *addr等于预期值，标记 task 为阻塞，加入 futex 等待队列
                let task = take_current_task().unwrap();
                let mut task_inner = task.inner_exclusive_access();    
                task_inner.task_status = TaskStatus::Blocked;
                let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
                drop(task_inner);
                futex_q.push_back(task);
                futex_q.guard.unlock();
                // switch to other task
                drop(process_inner);
                schedule(task_cx_ptr);
            } else {
                futex_q.guard.unlock();
                return 0;
            }
        },
        FUTEX_WAKE => {
            futex_q.guard.lock();
            if let Some(task) = futex_q.pop_front() {
                wakeup_task(task);
            } 
            futex_q.guard.unlock();
        },
        _ => panic!("Unsupported futex_op: {}", futex_op)
    };
    0
}