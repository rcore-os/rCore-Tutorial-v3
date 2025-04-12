#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

extern crate alloc;

use alloc::vec;
use user_lib::exit;
use user_lib::{
    MutexSpin, Condvar
};
use user_lib::{sleep, thread_create, waittid};
use lazy_static::lazy_static;

static mut A: usize = 0;

lazy_static! {
    static ref mutex: MutexSpin = MutexSpin::new();
    static ref condvar: Condvar = Condvar::new();
}

unsafe fn first() -> ! {
    sleep(10);
    println!("First work, Change A --> 1 and wakeup Second");
    mutex.lock();
    A = 1;
    condvar.notify_one();
    mutex.unlock();
    exit(0)
}

unsafe fn second() -> ! {
    println!("Second want to continue,but need to wait A=1");
    mutex.lock();
    while A == 0 {
        println!("Second: A is {}", &raw mut A as usize);
        condvar.wait(&mutex);
    }
    println!("A is {}, Second can work now", &raw mut A as usize);
    mutex.unlock();
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
    println!("test_condvar passed!");
    0
}
