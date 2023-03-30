#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

extern crate alloc;

use alloc::vec;
use user_lib::exit;
use user_lib::{
    mutex_blocking_create, mutex_lock, mutex_unlock, semaphore_create, semaphore_down, semaphore_up,
};
use user_lib::{sleep, thread_create, waittid};

static mut A: usize = 0;

const SEM_ID: usize = 0;
const MUTEX_ID: usize = 0;

unsafe fn first() -> ! {
    sleep(10);
    println!("First work, Change A --> 1 and wakeup Second");
    mutex_lock(MUTEX_ID);
    A = 1;
    semaphore_up(SEM_ID);
    mutex_unlock(MUTEX_ID);
    exit(0)
}

unsafe fn second() -> ! {
    println!("Second want to continue,but need to wait A=1");
    loop {
        mutex_lock(MUTEX_ID);
        if A == 0 {
            println!("Second: A is {}", A);
            mutex_unlock(MUTEX_ID);
            semaphore_down(SEM_ID);
        } else {
            mutex_unlock(MUTEX_ID);
            break;
        }
    }
    println!("A is {}, Second can work now", A);
    exit(0)
}

#[no_mangle]
pub fn main() -> i32 {
    // create semaphore & mutex
    assert_eq!(semaphore_create(0) as usize, SEM_ID);
    assert_eq!(mutex_blocking_create() as usize, MUTEX_ID);
    // create threads
    let threads = vec![
        thread_create(first as usize, 0),
        thread_create(second as usize, 0),
    ];
    // wait for all threads to complete
    for thread in threads.iter() {
        waittid(*thread as usize);
    }
    println!("test_condvar passed!");
    0
}
