//! The main module and entrypoint
//!
//! Various facilities of the kernels are implemented as submodules. The most
//! important ones are:
//!
//! - [`trap`]: Handles all cases of switching from userspace to the kernel
//! - [`syscall`]: System call handling and implementation
//!
//! The operating system also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality. (See its source code for
//! details.)
//!
//! We then call [`batch::run_next_app()`] and for the first time go to
//! userspace.

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![no_main]

use core::arch::global_asm;

use log::*;
#[macro_use]
mod console;
pub mod batch;
mod lang_items;
mod logging;
mod sbi;
mod sync;
pub mod syscall;
pub mod trap;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

/// clear BSS segment
fn clear_bss() {
    unsafe extern "C" {
        safe fn sbss();
        safe fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

/// the rust entry-point of os
#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    unsafe extern "C" {
        safe fn stext(); // begin addr of text segment
        safe fn etext(); // end addr of text segment
        safe fn srodata(); // start addr of Read-Only data segment
        safe fn erodata(); // end addr of Read-Only data ssegment
        safe fn sdata(); // start addr of data segment
        safe fn edata(); // end addr of data segment
        safe fn sbss(); // start addr of BSS segment
        safe fn ebss(); // end addr of BSS segment
        safe fn boot_stack_lower_bound(); // stack lower bound
        safe fn boot_stack_top(); // stack top
    }
    clear_bss();
    logging::init();
    println!("[kernel] Hello, world!");
    trace!(
        "[kernel] .text [{:#x}, {:#x})",
        stext as usize, etext as usize
    );
    debug!(
        "[kernel] .rodata [{:#x}, {:#x})",
        srodata as usize, erodata as usize
    );
    info!(
        "[kernel] .data [{:#x}, {:#x})",
        sdata as usize, edata as usize
    );
    warn!(
        "[kernel] boot_stack top=bottom={:#x}, lower_bound={:#x}",
        boot_stack_top as usize, boot_stack_lower_bound as usize
    );
    error!("[kernel] .bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
    trap::init();
    batch::init();
    batch::run_next_app();
}
