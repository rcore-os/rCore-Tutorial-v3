#![no_std]
#![no_main]

use user_lib::get_taskinfo;

extern crate user_lib;

#[unsafe(no_mangle)]
fn main() -> i32 {
    get_taskinfo(0, "task".as_bytes());
    0
}
