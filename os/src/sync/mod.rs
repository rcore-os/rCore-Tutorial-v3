mod condvar;
mod mutex;
mod semaphore;
mod up;

pub use condvar::Condvar;
pub use mutex::{FUTEX_WAIT, FUTEX_WAKE};
pub use mutex::{Mutex, Futex, MutexSpin, FutexQ};
pub use semaphore::Semaphore;
pub use up::UPSafeCell;

#[inline(always)]
pub fn load_reserved(addr: *const u32) -> u32 {
    let val;
    unsafe {
        core::arch::asm!(
            "lr.w {}, ({})",
            out(reg) val,
            in(reg) addr
        );
    }
    val
}

/// return true if successfully modify `addr` in memory
#[inline(always)]
pub fn store_conditional(addr: *const u32, val: u32) -> bool {
    let res: u32;
    unsafe {
        core::arch::asm!(
            "sc.w {}, {}, ({})",
            out(reg) res,
            in(reg) val,
            in(reg) addr
        );
    }
    res == 0
}

#[inline(always)]
pub fn atomic_increment(addr: *mut u32) {
    loop {
        let val = load_reserved(addr);
        if store_conditional(addr, val + 1) { break; }    
    }
}

#[inline(always)]
pub fn atomic_decrement(addr: *mut u32) {
    loop {
        let val = load_reserved(addr);
        if store_conditional(addr, val - 1) { break; }
    }
}

#[inline(always)]
pub fn atomic_test_and_set(addr: *mut u32, bit: u32) -> u32 {
    loop {
        let val = load_reserved(addr);
        if store_conditional(addr, val | (1 << bit)) { return val; }
    }
}

/// 原子地使 `(addr)` 自加 `addend`，并比较结果与 `expected` 是否相等。
/// 相等则返回 `true`
#[inline(always)]
pub fn atomic_add_and_compare(addr: *mut u32, addend: u32, expected: u32) -> bool {
    loop {
        let val = load_reserved(addr);
        let res = val + addend;
        if store_conditional(addr, res) {
            return res == expected
        }
    }
}