/// rCore application without user_lib.
/// Load it manually to 0x80040000.

use user_lib::{sys_yield};

fn main() -> ! {
    for _ in 0..10 {
        for _ in 0..10 {
            print!("A");
        }
        println!();
        sys_yield();
    }
    loop {}
}