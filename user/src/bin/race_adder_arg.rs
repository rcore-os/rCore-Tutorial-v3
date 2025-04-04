#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;
extern crate alloc;

use crate::alloc::string::ToString;
use alloc::vec::Vec;
use core::ptr::addr_of_mut;
use user_lib::{exit, get_time, thread_create, waittid};

static mut A: usize = 0;
const PER_THREAD: usize = 1000;
const THREAD_COUNT: usize = 16;

fn f(count: usize) -> ! {
    let mut t = 2usize;
    for _ in 0..PER_THREAD {
        let a = addr_of_mut!(A);
        let cur = unsafe { a.read_volatile() };
        for _ in 0..count {
            t = t * t % 10007;
        }
        unsafe {
            a.write_volatile(cur + 1);
        }
    }
    exit(t as i32)
}

#[unsafe(no_mangle)]
pub fn main(argc: usize, argv: &[&str]) -> i32 {
    let count: usize;
    if argc == 1 {
        count = THREAD_COUNT;
    } else if argc == 2 {
        count = argv[1].to_string().parse::<usize>().unwrap();
    } else {
        println!(
            "ERROR in argv, argc is {}, argv[0] {} , argv[1] {} , argv[2] {}",
            argc, argv[0], argv[1], argv[2]
        );
        exit(-1);
    }

    let start = get_time();
    let mut v = Vec::new();
    for _ in 0..THREAD_COUNT {
        v.push(thread_create(f as usize, count) as usize);
    }
    let mut time_cost = Vec::new();
    for tid in v.iter() {
        time_cost.push(waittid(*tid));
    }
    println!("time cost is {}ms", get_time() - start);
    assert_eq!(unsafe { A }, PER_THREAD * THREAD_COUNT);
    0
}
