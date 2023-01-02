//use crate::kernelvec::*;
//use crate::memlayout::*;
//use crate::param::NCPU;
//use super::main::*;
//use crate::riscv::registers::{pmpcfg0::*, *};
use core::arch::asm;
use core::hint::unreachable_unchecked;

mod riscv;

#[repr(C, align(16))]
struct Stack([u8; 4096 * 4 * NCPU]);

#[no_mangle]
static mut STACK0: Stack = Stack([0; 4096 * 4 * NCPU]);

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
    asm!("mv tp, {0}", in(reg) id);

    // switch to supervisor mode and jump to main().
    asm!("mret");

    extern "C" {
        fn rust_main() -> !;
    }
    unreachable_unchecked();
}

// a scratch area per CPU for machine-mode timer interrupts.
static mut TIMER_SCRATCH: [[u64; 5]; 1] = [[0; 5]; 1];

unsafe fn timerinit() {
    // each CPU has a separate source of timer interrupts
    let id = mhartid::read();

    // ask the CLINT for a timer interrupts
    let interval = 1000000u64; // cycles; about 1/10th second in qemu.
    let mtimecmp = clint_mtimecmp(id) as *mut u64;
    let mtime = CLINT_MTIME as *const u64;
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
    mtvec::write(timervec as usize, mtvec::TrapMode::Direct);

    // enable machine-mode interrupts.
    mstatus::set_mie();

    // enable machime-mode timer interrupts.
    mie::set_mtimer();
}