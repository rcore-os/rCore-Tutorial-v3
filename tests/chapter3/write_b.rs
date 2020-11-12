/// rCore application without user_lib.
/// Load it manually to 0x80050000.

use user_lib::{sys_yield};

fn main() -> ! {
    for _ in 0..10 {
        for _ in 0..10 {
            print!("B");
        }
        println!();
        sys_yield();
    }
    loop {}
}