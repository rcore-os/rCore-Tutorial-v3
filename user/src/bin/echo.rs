#![no_std]
#![no_main]

extern crate alloc;

#[macro_use]
extern crate user_lib;

#[macro_use]
use alloc::string::String;
use user_lib::console::getchar;

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const BS: u8 = 0x08u8;
const DL: u8 = 0x7fu8;
const ET: u8 = 0x04u8;
const NU: u8 = 0x00u8;

#[no_mangle]
pub fn main(argc: usize, argv: &[&str]) -> i32 {
    if argc == 2 {
        println!("{}\0", argv[1]);
    } else {
        let mut line: String = String::new();
        loop {
            let c = getchar();
            match c {
                LF | CR => {
                    println!("");
                    println!("{}", line);
                    line.clear();
                }
                BS | DL => {
                    if !line.is_empty() {
                        print!("{}", BS as char);
                        print!(" ");
                        print!("{}", BS as char);
                        line.pop();
                    }
                }
                ET | NU => {
                    println!("{}\0", line);
                    return 0;
                }
                _ => {
                    // print!("{}", c as char);
                    line.push(c as char);
                }
            }
        }
    }
    0
}
