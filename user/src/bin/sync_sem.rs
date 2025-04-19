#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

extern crate alloc;

use alloc::vec;
use user_lib::exit;
use user_lib::Semaphore;
use user_lib::{sleep, thread_create, waittid};
use lazy_static::lazy_static;
lazy_static! {
    static ref sem: Semaphore = Semaphore::new(0);
}

unsafe fn first() -> ! {
    sleep(10);
    println!("First work and wakeup Second");
    sem.post();
    exit(0)
}

unsafe fn second() -> ! {
    println!("Second want to continue,but need to wait first");
    sem.wait();
    println!("Second can work now");
    exit(0)
}

#[no_mangle]
pub fn main() -> i32 {
    // create threads
    let threads = vec![
        thread_create(first as usize, 0),
        thread_create(second as usize, 0),
    ];
    // wait for all threads to complete
    for thread in threads.iter() {
        waittid(*thread as usize);
    }
    println!("sync_sem passed!");
    0
}
