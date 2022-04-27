#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{exit, stackless_coroutine_test};

#[no_mangle]
pub fn main() -> i32 {
    
    stackless_coroutine_test();
    
    0
}