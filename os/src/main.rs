#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(fn_align)]

use crate::drivers::{GPU_DEVICE, KEYBOARD_DEVICE, MOUSE_DEVICE};

extern crate alloc;

#[macro_use]
extern crate bitflags;

#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
mod config;
mod drivers;
mod fs;
mod gui;
mod lang_items;
mod mm;
mod sbi;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;
//mod start;
mod riscvregs;
use riscvregs::registers::*;
use riscvregs::registers::pmpcfg0::*;
//use syscall::create_desktop; //for test

core::arch::global_asm!(include_str!("entry.asm"));

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

use lazy_static::*;
use sync::UPIntrFreeCell;

lazy_static! {
    pub static ref DEV_NON_BLOCKING_ACCESS: UPIntrFreeCell<bool> =
        unsafe { UPIntrFreeCell::new(false) };
}


#[repr(C, align(16))]
struct Stack([u8; 4096 * 4 * 1]);

#[no_mangle]
static mut STACK0: Stack = Stack([0; 4096 * 4 * 1]);

#[no_mangle]
pub unsafe fn rust_start() -> ! {
    // set MPP mode to Supervisor, for mret
    mstatus::set_mpp(mstatus::MPP::Supervisor);

    // set MEPC to main, for mret
    mepc::write(rust_main as usize);

    // disable paging for now.
    satp::write(0);

    // delegate all interrupts and exceptions to supervisor mode.
    medeleg::set_all();
    mideleg::set_all();
    sie::set_sext();
    sie::set_ssoft();
    sie::set_stimer();

    // configure Physical Memory Protection to give supervisor mode
    // access to all of physical memory.
    pmpaddr0::write(0x3fffffffffffff);
    pmpcfg0::set_pmp(0, Range::TOR, Permission::RWX, false); // 0 < addr < pmpaddr0

    // ask for clock interrupts.
    timerinit();

    // keep each CPU's hartid in its tp register, for cpuid().
    let id = mhartid::read();
    core::arch::asm!("mv tp, {0}", in(reg) id);

    // switch to supervisor mode and jump to main().
    core::arch::asm!("mret");

    extern "C" {
        fn rust_main() -> !;
    }
    core::hint::unreachable_unchecked();
}

// a scratch area per CPU for machine-mode timer interrupts.
static mut TIMER_SCRATCH: [[u64; 5]; 1] = [[0; 5]; 1];

unsafe fn timerinit() {
    // each CPU has a separate source of timer interrupts
    let id = mhartid::read();

    // ask the CLINT for a timer interrupts
    let interval = 1000000u64; // cycles; about 1/10th second in qemu.
    let mtimecmp = board::clint_mtimecmp(id) as *mut u64;
    let mtime = board::CLINT_MTIME as *const u64;
    mtimecmp.write_volatile(mtime.read_volatile() + interval);

    // prepare information in scratch[] for timervec.
    // scratch[0..2] : space for timervec to save registers.
    // scratch[3] : address of CLINT MTIMECMP register.
    // scratch[4] : desired interval (in cycles) between timer interrupts.
    let scratch = &mut TIMER_SCRATCH[id];
    scratch[3] = mtimecmp as u64;
    scratch[4] = interval;
    mscratch::write(scratch.as_mut_ptr() as usize);

    // set the machine-mode trap handler
    mtvec::write(board::timervec as usize, mtvec::TrapMode::Direct);

    // enable machine-mode interrupts.
    mstatus::set_mie();

    // enable machime-mode timer interrupts.
    mie::set_mtimer();
}


#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    mm::init();
    println!("KERN: init gpu");
    let _gpu = GPU_DEVICE.clone();
    println!("KERN: init keyboard");
    let _keyboard = KEYBOARD_DEVICE.clone();
    println!("KERN: init mouse");
    let _mouse = MOUSE_DEVICE.clone();
    println!("KERN: init trap");
    trap::init();
    //trap::enable_timer_interrupt();
    //timer::set_next_trigger();
    board::device_init();
    fs::list_apps();
    //gui::init_paint();
    task::add_initproc();
    *DEV_NON_BLOCKING_ACCESS.exclusive_access() = true;
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
