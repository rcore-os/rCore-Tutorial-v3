#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::vec::Vec;
use core::ptr::addr_of_mut;
use user_lib::{exit, get_time, thread_create, waittid, yield_};

static mut A: usize = 0;
static mut OCCUPIED: bool = false;
const PER_THREAD_DEFAULT: usize = 10000;
const THREAD_COUNT_DEFAULT: usize = 16;
static mut PER_THREAD: usize = 0;

fn critical_section(t: &mut usize) {
    let a = addr_of_mut!(A);
    let cur = unsafe { a.read_volatile() };
    for _ in 0..500 {
        *t = (*t) * (*t) % 10007;
    }
    unsafe {
        a.write_volatile(cur + 1);
    }
}

fn lock() {
    unsafe {
        while OCCUPIED {
            yield_();
        }
        OCCUPIED = true;
    }
}

fn unlock() {
    unsafe {
        OCCUPIED = false;
    }
}

fn f() -> ! {
    let mut t = 2usize;
    for _ in 0..unsafe { PER_THREAD } {
        lock();
        critical_section(&mut t);
        unlock();
    }
    exit(t as i32)
}

#[unsafe(no_mangle)]
pub fn main(argc: usize, argv: &[&str]) -> i32 {
    let mut thread_count = THREAD_COUNT_DEFAULT;
    let mut per_thread = PER_THREAD_DEFAULT;
    if argc >= 2 {
        thread_count = argv[1].parse().unwrap();
        if argc >= 3 {
            per_thread = argv[2].parse().unwrap();
        }
    }
    unsafe {
        PER_THREAD = per_thread;
    }
    let start = get_time();
    let mut v = Vec::new();
    for _ in 0..thread_count {
        v.push(thread_create(f as usize, 0) as usize);
    }
    for tid in v.into_iter() {
        waittid(tid);
    }
    println!("time cost is {}ms", get_time() - start);
    assert_eq!(unsafe { A }, unsafe { PER_THREAD } * thread_count);
    0
}
