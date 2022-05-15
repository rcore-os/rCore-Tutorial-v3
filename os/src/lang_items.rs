//! The panic handler

//use crate::sbi::shutdown;
use crate::qemu_exit::{QEMUExit,QEMU_EXIT_HANDLE};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "[kernel] Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("[kernel] Panicked: {}", info.message().unwrap());
    }
    //shutdown()
   // QEMU_EXIT_HANDLE.exit(13)
    QEMU_EXIT_HANDLE.exit_success()
}
