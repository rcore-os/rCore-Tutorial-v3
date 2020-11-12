/// rCore application without user_lib.
/// Load it manually to 0x80040000.
/// It should not OK. Kernel should catch the page fault.

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
        if i == STEP / 2 {
            // TODO: Modify satp to a malicious value in order to destroy memory access mechanism.
        }
    }
    println!("Test power_bad OK!");
    loop {}
}