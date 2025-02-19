#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

const LEN: usize = 100;

#[unsafe(no_mangle)]
fn main() -> i32 {
    let p = 5u64;
    let m = 998244353u64;
    let iter: usize = 210000;
    let mut cur = 0usize;
    let mut s = [0u64; LEN];
    s[cur] = 1;
    for i in 1..=iter {
        let next = if cur + 1 == LEN { 0 } else { cur + 1 };
        s[next] = s[cur] * p % m;
        cur = next;
        if i % 10000 == 0 {
            println!("power_5 [{}/{}]", i, iter);
        }
    }
    println!("{}^{} = {}(MOD {})", p, iter, s[cur], m);
    println!("Test power_5 OK!");
    0
}
