#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(fn_align)]

use alloc::string::ToString;
use crate::drivers::{GPU_DEVICE, KEYBOARD_DEVICE, MOUSE_DEVICE};

extern crate alloc;

#[macro_use]
extern crate bitflags;

#[path = "boards/qemu.rs"]
mod board;
use board::*;
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

use riscv::register::*;
// mod riscvreg;
// use riscvreg::{
//     mstatus, mepc, satp, medeleg, mideleg, sie, mhartid, tp, clint, 
//     mscratch, mtvec, mie, sstatus
// };
// use riscvregs::registers::*;
// use riscvregs::registers::pmpcfg0::*;
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

#[inline]
pub unsafe fn medeleg_write(medeleg: usize){
    core::arch::asm!("csrw medeleg, {}",in(reg)medeleg);
}

pub unsafe fn mideleg_write(mideleg: usize) {
    core::arch::asm!("csrw mideleg, {}", in(reg)mideleg);
}

pub enum SIE {
    SEIE = 1 << 9, // external
    STIE = 1 << 5, // timer
    SSIE = 1 << 1, // software
}

#[inline]
pub unsafe fn sie_read() -> usize {
    let ret:usize;
    core::arch::asm!("csrr {}, sie", out(reg)ret);
    ret
}

#[inline]
pub unsafe fn sie_write(x:usize) {
    core::arch::asm!("csrw sie, {}", in(reg)x);
}

/// enable all software interrupts
/// still need to set SIE bit in sstatus
pub unsafe fn intr_on() {
    let mut sie = sie_read();
    sie |= SIE::SSIE as usize | SIE::STIE as usize | SIE::SEIE as usize;
    sie_write(sie);
}

#[no_mangle]
pub unsafe fn rust_start() -> ! {
    // set MPP mode to Supervisor, for mret
    mstatus::set_mpp(mstatus::MPP::Supervisor);

    // set MEPC to main, for mret
    mepc::write(rust_main as usize);

    // disable paging for now.
    satp::write(0);

    // delegate all interrupts and exceptions to supervisor mode.
    medeleg_write(0xffff);
    mideleg_write(0xffff);
    intr_on();

    // configure Physical Memory Protection to give supervisor mode
    // access to all of physical memory.
    pmpaddr0::write(0x3fffffffffffff);
    pmpcfg0::write(0xf);
    //pmpcfg0::set_pmp(0, Range::TOR, Permission::RWX, false); // 0 < addr < pmpaddr0

    // ask for clock interrupts.
    timer_init();

    // keep each CPU's hartid in its tp register, for cpuid().
    // let id = mhartid::read();
    // core::arch::asm!("mv tp, {0}", in(reg) id);

    // switch to supervisor mode and jump to main().
    core::arch::asm!("mret");

    extern "C" {
        fn rust_main() -> !;
    }
    core::hint::unreachable_unchecked();
}

use core::convert::Into;
use core::ptr;

// a scratch area per CPU for machine-mode timer interrupts.
static mut TIMER_SCRATCH: [u64; 5] = [0; 5];

#[inline]
unsafe fn read_mtime() -> u64 {
    ptr::read_volatile(Into::<usize>::into(CLINT_MTIME) as *const u64)
}

unsafe fn write_mtimecmp(value: u64) {
    let offset = Into::<usize>::into(CLINT_MTIMECMP);
    ptr::write_volatile(offset as *mut u64, value);
}

pub unsafe fn add_mtimecmp(interval:u64){
    let value = read_mtime();
    write_mtimecmp(value+interval);
}

pub fn count_mtiecmp() -> usize{
    let ret:usize;
    ret = Into::<usize>::into(CLINT) + 0x4000;
    ret
}

#[inline]
pub unsafe fn mtvec_write(x:usize){
    core::arch::asm!("csrw mtvec, {}",in(reg)x);
}

use bit_field::BitField;

#[inline]
unsafe fn mstatus_read() -> usize {
    let ret:usize;
    core::arch::asm!("csrr {}, mstatus",out(reg)ret);
    ret
}

#[inline]
unsafe fn mstatus_write(x: usize) {
    core::arch::asm!("csrw mstatus, {}",in(reg)x);
}

// enable machine-mode interrupts.
pub unsafe fn mstatus_enable_interrupt(){
    let mut mstatus = mstatus_read();
    mstatus.set_bit(3, true);
    mstatus_write(mstatus);
}


pub enum MIE {
    MEIE = 1 << 11, // external
    MTIE = 1 << 7,  // timer
    MSIE = 1 << 3  // software
}

#[inline]
pub unsafe fn mie_read() -> usize {
    let ret:usize;
    core::arch::asm!("csrr {}, mie", out(reg)ret);
    ret
}

#[inline]
pub unsafe fn mie_write(x:usize){
    core::arch::asm!("csrw mie, {}",in(reg)x);
}

unsafe fn timer_init() {
    clear_bss();
    // each CPU has a separate source of timer interrupts
    //let id = mhartid::read();

    // ask the CLINT for a timer interrupts
    let interval = 1000000u64; // cycles; about 1/10th second in qemu.
    add_mtimecmp(interval);
    // let mtimecmp = board::clint_mtimecmp(0) as *mut u64;
    // let mtime = board::CLINT_MTIME as *const u64;
    // mtimecmp.write_volatile(mtime.read_volatile() + interval);

    // prepare information in scratch[] for timervec.
    // scratch[0..2] : space for timervec to save registers.
    // scratch[3] : address of CLINT MTIMECMP register.
    // scratch[4] : desired interval (in cycles) between timer interrupts.
    let scratch = &mut TIMER_SCRATCH;
    scratch[3] = count_mtiecmp() as u64;
    scratch[4] = interval;
    mscratch::write(scratch.as_mut_ptr() as usize);

    // set the machine-mode trap handler
    mtvec_write(timervec as usize);
    //mtvec::write(board::timervec as usize, mtvec::TrapMode::Direct);

    // enable machine-mode interrupts.
    mstatus_enable_interrupt();
    //mstatus::set_mie();

    // enable machine-mode timer interrupts.
    mie_write(mie_read() | MIE::MTIE as usize);
    //mie::set_mtimer();
}

use crate::drivers::chardev::CharDevice;
use crate::drivers::chardev::UART;
use crate::drivers::rtc::{enable_rtc_irq, get_rtc_time, set_rtc_alarm_with_next_second};

#[no_mangle]
pub fn rust_main() -> ! {

    //clear_bss();
    mm::init();
    UART.init();
    println!("KERN: begin");
    //loop{};
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
    println!("-----------Time:{}-----------", get_rtc_time().to_string());
    fs::list_apps();
    //gui::init_paint();
    task::add_initproc();
    *DEV_NON_BLOCKING_ACCESS.exclusive_access() = true;
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
