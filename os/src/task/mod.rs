mod context;
mod id;
mod manager;
mod process;
mod processor;
mod signal;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use self::id::TaskUserRes;
use crate::fs::{open_file, OpenFlags};
use alloc::{sync::Arc, vec::Vec};
use lazy_static::*;
use manager::fetch_task;
use process::ProcessControlBlock;
use switch::__switch;

pub use context::TaskContext;
pub use id::{kstack_alloc, pid_alloc, KernelStack, PidHandle, IDLE_PID};
pub use manager::{add_task, pid2process, remove_from_pid2process};
pub use processor::{
    current_kstack_top, current_process, current_task, current_trap_cx, current_trap_cx_user_va,
    current_user_token, run_tasks, schedule, take_current_task,
};
pub use signal::SignalFlags;
pub use task::{TaskControlBlock, TaskStatus};

pub fn suspend_current_and_run_next() {
    // There must be an application running.
    //kprintln!("[KERN] task::suspend_current_and_run_next() begin");
    let task = take_current_task().unwrap();

    // ---- access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    // ---- release current TCB

    // push back to ready queue.
    add_task(task);
    // jump to scheduling cycle
    //kprintln!("[KERN] task::suspend_current_and_run_next() end");
    schedule(task_cx_ptr);
}

/// This function must be followed by a schedule
pub fn block_current_task() -> *mut TaskContext {
    //kprintln!("[KERN] task::block_current_task() begin");
    let task = take_current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    task_inner.task_status = TaskStatus::Blocking;
    //kprintln!("[KERN] task::block_current_task() end");
    &mut task_inner.task_cx as *mut TaskContext
}

pub fn block_current_and_run_next() {
    let task_cx_ptr = block_current_task();
    schedule(task_cx_ptr);
}

use crate::board::QEMUExit;

pub fn exit_current_and_run_next(exit_code: i32) {
    kprintln!("[KERN] task::exit_current_and_run_next() begin");
    let task = take_current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    let process = task.process.upgrade().unwrap();
    let tid = task_inner.res.as_ref().unwrap().tid;
    // record exit code
    kprintln!("[KERN] task::exit_current_and_run_next(): record exit code in task_inner");
    task_inner.exit_code = Some(exit_code);
    kprintln!("[KERN] task::exit_current_and_run_next(): TaskUserRes =>None");
    task_inner.res = None;
    // here we do not remove the thread since we are still using the kstack
    // it will be deallocated when sys_waittid is called
    kprintln!("[KERN] task::exit_current_and_run_next(): drop task_inner");
    drop(task_inner);
    kprintln!("[KERN] task::exit_current_and_run_next(): drop task");
    drop(task);
    // however, if this is the main thread of current process
    // the process should terminate at once
    if tid == 0 {
        kprintln!("[KERN] task::exit_current_and_run_next(): it's main thread, process should terminate at once");
        let pid = process.getpid();
        if pid == IDLE_PID {
            println!(
                "[kernel] Idle process exit with exit_code {} ...",
                exit_code
            );
            if exit_code != 0 {
                //crate::sbi::shutdown(255); //255 == -1 for err hint
                crate::board::QEMU_EXIT_HANDLE.exit_failure();
            } else {
                //crate::sbi::shutdown(0); //0 for success hint
                crate::board::QEMU_EXIT_HANDLE.exit_success();
            }
        }
        remove_from_pid2process(pid);
        let mut process_inner = process.inner_exclusive_access();
        // mark this process as a zombie process
        kprintln!(
            "[KERN] task::exit_current_and_run_next(): mark this process as a zombie process"
        );
        process_inner.is_zombie = true;
        // record exit code of main process
        kprintln!("[KERN] task::exit_current_and_run_next(): record exit code in process_inner");
        process_inner.exit_code = exit_code;

        {
            // move all child processes under init process
            kprintln!(
                "[KERN] task::exit_current_and_run_next(): move all child processes under INITPROC"
            );
            let mut initproc_inner = INITPROC.inner_exclusive_access();
            for child in process_inner.children.iter() {
                child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
                initproc_inner.children.push(child.clone());
            }
        }

        // deallocate user res (including tid/trap_cx/ustack) of all threads
        // it has to be done before we dealloc the whole memory_set
        // otherwise they will be deallocated twice
        kprintln!("[KERN] task::exit_current_and_run_next(): deallocate user res (tid/trap_cx/ustack) of all threads");
        let mut recycle_res = Vec::<TaskUserRes>::new();
        for task in process_inner.tasks.iter().filter(|t| t.is_some()) {
            let task = task.as_ref().unwrap();
            let mut task_inner = task.inner_exclusive_access();
            if let Some(res) = task_inner.res.take() {
                recycle_res.push(res);
            }
        }

        kprintln!(
            "[KERN] task::exit_current_and_run_next(): clear children Vector in process_inner"
        );
        // dealloc_tid and dealloc_user_res require access to PCB inner, so we
        // need to collect those user res first, then release process_inner
        // for now to avoid deadlock/double borrow problem.
        drop(process_inner);
        recycle_res.clear();

        let mut process_inner = process.inner_exclusive_access();
        process_inner.children.clear();
        // deallocate other data in user space i.e. program code/data section
        kprintln!("[KERN] task::exit_current_and_run_next(): deallocate code/data in user space");
        process_inner.memory_set.recycle_data_pages();
        // drop file descriptors
        kprintln!("[KERN] task::exit_current_and_run_next(): drop file descriptors");
        process_inner.fd_table.clear();
    }
    kprintln!("[KERN] task::exit_current_and_run_next(): drop process");
    drop(process);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    kprintln!("[KERN] task::exit_current_and_run_next() end, sched next task");
    schedule(&mut _unused as *mut _);
}

lazy_static! {
    pub static ref INITPROC: Arc<ProcessControlBlock> = {
        kprintln!("[KERN] task::lazy_static!INITPROC begin");
        let inode = open_file("initproc", OpenFlags::RDONLY).unwrap();
        let v = inode.read_all();
        ProcessControlBlock::new(v.as_slice())
    };
}

pub fn add_initproc() {
    kprintln!("[KERN] task::add_initproc() begin");
    let _initproc = INITPROC.clone();
    kprintln!("[KERN] task::add_initproc() end");
}

pub fn check_signals_of_current() -> Option<(i32, &'static str)> {
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    process_inner.signals.check_error()
}

pub fn current_add_signal(signal: SignalFlags) {
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    process_inner.signals |= signal;
}
