pub use ksync::{UPIntrFreeCell, UPIntrRefMut};
use riscv::register::sstatus;

// extern "C" {
//     fn swap_and_disable_intr() -> bool;
//     fn enable_intr();
// }

#[no_mangle]
extern "C" fn swap_and_disable_intr() -> bool {
    let sie = sstatus::read().sie();
    unsafe {
        sstatus::clear_sie();
    }
    sie
}

#[no_mangle]
extern "C" fn enable_intr() {
    unsafe {
        sstatus::set_sie();
    }
}