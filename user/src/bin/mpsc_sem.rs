#![no_std]
#![no_main]
#![allow(clippy::println_empty_string)]

#[macro_use]
extern crate user_lib;

extern crate alloc;

use alloc::vec::Vec;
use user_lib::exit;
use user_lib::Semaphore;
use user_lib::{thread_create, waittid};
use lazy_static::lazy_static;

const BUFFER_SIZE: usize = 8;
static mut BUFFER: [usize; BUFFER_SIZE] = [0; BUFFER_SIZE];
static mut FRONT: usize = 0;
static mut TAIL: usize = 0;
const PRODUCER_COUNT: usize = 4;
const NUMBER_PER_PRODUCER: usize = 100;

lazy_static! {
    static ref mutex: Semaphore = Semaphore::new(1);
    static ref empty: Semaphore = Semaphore::new(BUFFER_SIZE as i32);
    static ref avail: Semaphore = Semaphore::new(0);
}

unsafe fn producer(id: *const usize) -> ! {
    let id = *id;
    for _ in 0..NUMBER_PER_PRODUCER {
        empty.wait();
        mutex.wait();
        BUFFER[TAIL] = id;
        TAIL = (TAIL + 1) % BUFFER_SIZE;
        mutex.post();
        avail.post();
    }
    exit(0)
}

unsafe fn consumer() -> ! {
    for _ in 0..PRODUCER_COUNT * NUMBER_PER_PRODUCER {
        avail.wait();
        mutex.wait();
        print!("{} ", BUFFER[FRONT]);
        FRONT = (FRONT + 1) % BUFFER_SIZE;
        mutex.post();
        empty.post();
    }
    println!("");
    exit(0)
}

#[no_mangle]
pub fn main() -> i32 {
    // create threads
    let ids: Vec<_> = (0..PRODUCER_COUNT).collect();
    let mut threads = Vec::new();
    for i in 0..PRODUCER_COUNT {
        threads.push(thread_create(
            producer as usize,
            &ids.as_slice()[i] as *const _ as usize,
        ));
    }
    threads.push(thread_create(consumer as usize, 0));
    // wait for all threads to complete
    for thread in threads.iter() {
        waittid(*thread as usize);
    }
    println!("mpsc_sem passed!");
    0
}
