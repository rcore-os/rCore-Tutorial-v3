#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::console::getchar;

const N: usize = 10;

#[no_mangle]
pub fn main() -> i32 {
    println!("I will receive {} letters.", N);
    let mut line = [0u8; N];
    for idx in 0..N {
        let c = getchar();
        line[idx] = c;
    }
    println!("{} letters entered", N);
    for idx in 0..N {
        print!("{}", line[idx] as char);
    }
    println!("");
    println!("You got it!");
    0
}