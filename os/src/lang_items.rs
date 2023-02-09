use crate::sbi::shutdown;
use crate::task::current_kstack_top;
use core::arch::asm;
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
    backtrace();
    shutdown(255)
}

#[no_mangle]
fn backtrace() {
    println!("---START BACKTRACE---");
    let info = crate::trace::init_kernel_trace();
    let func_info = unsafe { trace_lib::my_trace(info) };
    func_info.iter().for_each(|x| {
        println!("{}", x);
    });
    println!("---END   BACKTRACE---");
}
