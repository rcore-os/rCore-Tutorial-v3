/// rCore application without user_lib.
/// Load it manually to 0x80040000.
/// It should OK.

const SIZE: usize = 10;
const P: u32 = 3;
const STEP: usize = 10000000;
const MOD: u32 = 10007;
fn main() -> ! {
    let mut pow = [0u32; SIZE];
    let mut index: usize = 0;
    pow[index] = 1;
    for i in 1..=STEP {
        let last = pow[index];
        index = (index + 1) % SIZE;
        pow[index] = last * P % MOD;
        if i % 10000 == 0 {
            println!("{}^{}={}", P, i, pow[index]);
        }
    }
    println!("Test power OK!");
    loop {}
}