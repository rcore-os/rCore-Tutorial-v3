// RISC-V registers
pub mod registers {
    // hart (core) id registers
    pub mod mhartid {
        use core::arch::asm;

        #[inline]
        pub fn read() -> usize {
            let id: usize;
            unsafe {
                asm!("csrr {}, mhartid", out(reg) id);
            }
            id
        }
    }

    // Machine Status Register, mstatus
    pub mod mstatus {
        use core::arch::asm;

        // Machine Status Register bit
        const MPP_MASK: usize = 3 << 11;
        const MIE: usize = 1 << 3;

        // Machine Previous Privilege mode
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum MPP {
            Machine = 3,
            Supervisor = 1,
            User = 0,
        }

        #[inline]
        unsafe fn _read() -> usize {
            let bits: usize;
            asm!("csrr {}, mstatus", out(reg) bits);
            bits
        }

        #[inline]
        unsafe fn _write(bits: usize) {
            asm!("csrw mstatus, {}", in(reg) bits);
        }

        // Machine Previous Privilege Mode
        #[inline]
        pub fn set_mpp(mpp: MPP) {
            unsafe {
                let mut value = _read();
                value &= !MPP_MASK;
                value |= (mpp as usize) << 11;
                _write(value);
            }
        }

        #[inline]
        pub fn set_mie() {
            unsafe {
                asm!("csrs mstatus, {}", in(reg) MIE);
            }
        }
    }

    // machine exception program counter, holds the
    // instruction address to which a return from
    // exception will go.
    pub mod mepc {
        use core::arch::asm;

        #[inline]
        pub fn write(x: usize) {
            unsafe {
                asm!("csrw mepc, {}", in(reg) x);
            }
        }
    }

    // Supervisor Status Register, sstatus
    pub mod sstatus {
        use core::arch::asm;

        // Supervisor Status Register bit
        const SPP: usize = 1 << 8; // Previous mode, 1=Supervisor, 0=user
        const SPIE: usize = 1 << 5; // Supervisor Previous Interrupt Enable
        const SIE: usize = 1 << 1; // Supervisor Interrupt Enable

        #[derive(Clone, Copy, Debug)]
        pub struct Sstatus {
            bits: usize,
        }

        impl Sstatus {
            // Supervisor Interrupt Enable
            #[inline]
            pub(in crate::riscvregs) fn sie(&self) -> bool {
                self.bits & SIE != 0
            }

            // Supervisor Previous Privilege mode
            #[inline]
            pub fn spp(&self) -> SPP {
                match self.bits & SPP {
                    0 => SPP::User,
                    _ => SPP::Supervisor,
                }
            }

            // restore status bits
            #[inline]
            pub fn restore(&self) {
                unsafe {
                    _write(self.bits);
                }
            }
        }

        // Supervisor Previous Privilege Mode
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum SPP {
            Supervisor = 1,
            User = 0,
        }

        #[inline]
        pub fn read() -> Sstatus {
            let bits: usize;
            unsafe { asm!("csrr {}, sstatus", out(reg) bits) }
            Sstatus { bits }
        }

        #[inline]
        unsafe fn _write(bits: usize) {
            asm!("csrw sstatus, {}", in(reg) bits);
        }

        // bit set
        #[inline]
        unsafe fn _set(bits: usize) {
            asm!("csrs sstatus, {}", in(reg) bits);
        }

        // bit clear
        #[inline]
        unsafe fn _clear(bits: usize) {
            asm!("csrc sstatus, {}", in(reg) bits);
        }

        #[inline]
        pub(in crate::riscvregs) unsafe fn set_sie() {
            _set(SIE)
        }

        #[inline]
        pub(in crate::riscvregs) unsafe fn clear_sie() {
            _clear(SIE)
        }

        #[inline]
        pub unsafe fn set_spie() {
            _set(SPIE);
        }

        #[inline]
        pub unsafe fn set_spp(spp: SPP) {
            match spp {
                SPP::Supervisor => _set(SPP),
                SPP::User => _clear(SPP),
            }
        }
    }

    // Supervisor Interrupt Pending
    pub mod sip {
        use core::arch::asm;

        const SSIP: usize = 1 << 1;

        // Supervisor Software Interrupt Pending
        #[inline]
        pub unsafe fn clear_ssoft() {
            asm!("csrc sip, {}", in(reg) SSIP);
        }
    }

    // Supervisor Interrupt Enable
    pub mod sie {
        use core::arch::asm;

        const SEIE: usize = 1 << 9; // external
        const STIE: usize = 1 << 5; // timer
        const SSIE: usize = 1 << 1; // software

        #[inline]
        unsafe fn _set(bits: usize) {
            asm!("csrs sie, {}", in(reg) bits);
        }

        #[inline]
        pub unsafe fn set_sext() {
            _set(SEIE);
        }

        #[inline]
        pub unsafe fn set_stimer() {
            _set(STIE);
        }

        #[inline]
        pub unsafe fn set_ssoft() {
            _set(SSIE);
        }
    }

    // Machine-mode Interrupt Enable
    pub mod mie {
        use core::arch::asm;

        const MTIE: usize = 1 << 7;

        #[inline]
        pub unsafe fn set_mtimer() {
            asm!("csrs mie, {}", in(reg) MTIE);
        }
    }

    // supervisor exceptions program counter, holds the
    // instruction address to which a return from
    // exception will go.
    pub mod sepc {
        use core::arch::asm;

        #[inline]
        pub fn read() -> usize {
            let bits: usize;
            unsafe {
                asm!("csrr {}, sepc", out(reg) bits);
            }
            bits
        }

        #[inline]
        pub fn write(bits: usize) {
            unsafe {
                asm!("csrw sepc, {}", in(reg) bits);
            }
        }
    }

    // Machine Exception Delegation
    pub mod medeleg {
        use core::arch::asm;

        pub unsafe fn set_all() {
            asm!("csrw medeleg, {}", in(reg) 0xffff);
        }
    }

    // Machine Interrupt Delegation
    pub mod mideleg {
        use core::arch::asm;

        #[inline]
        pub unsafe fn set_all() {
            asm!("csrw mideleg, {}", in(reg) 0xffff);
        }
    }

    // Supervisor Trap-Vector Base Address
    // low two bits are mode.
    pub mod stvec {
        pub use super::mtvec::TrapMode;
        use core::arch::asm;

        #[inline]
        pub unsafe fn write(addr: usize, mode: TrapMode) {
            asm!("csrw stvec, {}", in(reg) addr + mode as usize);
        }
    }

    // Machine-mode interrupt vector
    pub mod mtvec {
        use core::arch::asm;

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum TrapMode {
            Direct = 0,
            Vectored = 1,
        }

        #[inline]
        pub unsafe fn write(addr: usize, mode: TrapMode) {
            asm!("csrw mtvec, {}", in(reg) addr + mode as usize);
        }
    }

    // Physical Memory Protection Configuration
    pub mod pmpcfg0 {
        use core::arch::asm;

        // Permission enum contains all possible permission modes for pmp registers
        #[derive(Clone, Copy, Debug)]
        pub enum Permission {
            NONE = 0b000,
            R = 0b001,
            W = 0b010,
            RW = 0b011,
            X = 0b100,
            RX = 0b101,
            WX = 0b110,
            RWX = 0b111,
        }

        // Range enum contains all possible addressing modes for pmp registers
        pub enum Range {
            OFF = 0b00,
            TOR = 0b01,
            NA4 = 0b10,
            NAPOT = 0b11,
        }

        // Set the pmp configuration corresponging to the index
        #[inline]
        pub unsafe fn set_pmp(index: usize, range: Range, permission: Permission, locked: bool) {
            assert!(index < 8);
            let mut value = _read();
            let byte = (locked as usize) << 7 | (range as usize) << 3 | (permission as usize);
            value |= byte << (8 * index);
            _write(value);
        }

        #[inline]
        unsafe fn _read() -> usize {
            let bits: usize;
            asm!("csrr {}, pmpcfg0", out(reg) bits);
            bits
        }

        #[inline]
        unsafe fn _write(bits: usize) {
            asm!("csrw pmpcfg0, {}", in(reg) bits);
        }
    }

    // Physical memory protection address register
    pub mod pmpaddr0 {
        use core::arch::asm;

        pub fn write(bits: usize) {
            unsafe {
                asm!("csrw pmpaddr0, {}", in(reg) bits);
            }
        }
    }

    // Supervisor address translation and protection;
    // holds the address of the page table.
    pub mod satp {
        use core::arch::asm;

        // stap register
        #[derive(Clone, Copy, Debug)]
        pub struct Satp {
            bits: usize,
        }

        // 64-bit satp mode
        pub enum Mode {
            // No translation or protection
            Bare = 0,
            // Page-based 39-bit virtual addressing
            Sv39 = 8,
            // Page-based 48-bit virtual addressing
            Sv48 = 9,
            // Page-based 57-bit virtual addressing
            Sv57 = 10,
            // Page-based 64-bit virtual addressing
            Sv64 = 11,
        }

        impl Satp {
            // Return the contents of the register as raw bits
            #[inline]
            pub fn bits(&self) -> usize {
                self.bits
            }
        }

        #[inline]
        pub unsafe fn read() -> Satp {
            let bits: usize;
            asm!("csrr {}, satp", out(reg) bits);
            Satp { bits }
        }

        #[inline]
        pub unsafe fn write(bits: usize) {
            asm!("csrw satp, {}", in(reg) bits);
        }

        #[inline]
        pub fn make(mode: Mode, asid: usize, ppn: usize) -> usize {
            let mut bits: usize = 0;
            bits |= (mode as usize) << 60;
            bits |= asid << 44;
            bits |= ppn >> 12;
            bits
        }
    }

    // mscratch register
    pub mod mscratch {
        use core::arch::asm;

        #[inline]
        pub fn write(bits: usize) {
            unsafe {
                asm!("csrw mscratch, {}", in(reg) bits);
            }
        }
    }

    // Supervisor Trap Cause
    pub mod scause {
        use core::{arch::asm, mem::size_of};

        // scause register
        #[derive(Clone, Copy)]
        pub struct Scause {
            bits: usize,
        }

        // Trap Cause
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Trap {
            Interrupt(Interrupt),
            Exception(Exception),
        }

        // Interrupt
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Interrupt {
            UserSoft,
            SupervisorSoft,
            UserTimer,
            SupervisorTimer,
            UserExternal,
            SupervisorExternal,
            Unknown,
        }

        // Exception
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        pub enum Exception {
            InstructionMisaligned,
            InstructionFault,
            IllegalInstruction,
            Breakpoint,
            LoadFault,
            StoreMisaligned,
            StoreFault,
            UserEnvCall,
            InstructionPageFault,
            LoadPageFault,
            StorePageFault,
            Unknown,
        }

        impl Interrupt {
            #[inline]
            pub fn from(nr: usize) -> Self {
                match nr {
                    0 => Interrupt::UserSoft,
                    1 => Interrupt::SupervisorSoft,
                    4 => Interrupt::UserTimer,
                    5 => Interrupt::SupervisorTimer,
                    8 => Interrupt::UserExternal,
                    9 => Interrupt::SupervisorExternal,
                    _ => Interrupt::Unknown,
                }
            }
        }

        impl Exception {
            #[inline]
            pub fn from(nr: usize) -> Self {
                match nr {
                    0 => Exception::InstructionMisaligned,
                    1 => Exception::InstructionFault,
                    2 => Exception::IllegalInstruction,
                    3 => Exception::Breakpoint,
                    5 => Exception::LoadFault,
                    6 => Exception::StoreMisaligned,
                    7 => Exception::StoreFault,
                    8 => Exception::UserEnvCall,
                    12 => Exception::InstructionPageFault,
                    13 => Exception::LoadPageFault,
                    15 => Exception::StorePageFault,
                    _ => Exception::Unknown,
                }
            }
        }

        impl Scause {
            // Returns the contents of the register as raw bits
            #[inline]
            pub fn bits(&self) -> usize {
                self.bits
            }

            // Returns the code field
            #[inline]
            pub fn code(&self) -> usize {
                let bit = 1 << (size_of::<usize>() * 8 - 1);
                self.bits & !bit
            }

            // Trap cause
            #[inline]
            pub fn cause(&self) -> Trap {
                if self.is_interrupt() {
                    Trap::Interrupt(Interrupt::from(self.code()))
                } else {
                    Trap::Exception(Exception::from(self.code()))
                }
            }

            // Is trap cause an interrupt.
            #[inline]
            pub fn is_interrupt(&self) -> bool {
                self.bits & (1 << (size_of::<usize>() * 8 - 1)) != 0
            }

            // Is trap cause an exception.
            #[inline]
            pub fn is_exception(&self) -> bool {
                !self.is_interrupt()
            }
        }

        #[inline]
        pub fn read() -> Scause {
            let bits: usize;
            unsafe {
                asm!("csrr {}, scause", out(reg) bits);
            }
            Scause { bits }
        }
    }

    // Supervisor Trap Value
    pub mod stval {
        use core::arch::asm;

        #[inline]
        pub fn read() -> usize {
            let bits: usize;
            unsafe { asm!("csrr {}, stval", out(reg) bits) }
            bits
        }
    }
}

use core::arch::asm;

use registers::*;

// enable device interrupts
#[inline]
pub fn intr_on() {
    unsafe {
        sstatus::set_sie();
    }
}

// disable device interrupts
#[inline]
pub fn intr_off() {
    unsafe {
        sstatus::clear_sie();
    }
}

// are device interrupts enabled?
#[inline]
pub fn intr_get() -> bool {
    sstatus::read().sie()
}

// flush the TLB.
#[inline]
pub unsafe fn sfence_vma() {
    // the zero, zero means flush all TLB entries
    asm!("sfence.vma zero, zero");
}

pub const PGSIZE: usize = 4096; // bytes per page
pub const PGSHIFT: usize = 12; // bits of offset within a page

pub const fn pgroundup(sz: usize) -> usize {
    (sz + PGSIZE - 1) & !(PGSIZE - 1)
}

pub const fn pgrounddown(sz: usize) -> usize {
    sz & !(PGSIZE - 1)
}

// PTE flags
pub mod pteflags {
    pub const PTE_V: usize = 1 << 0; // valid
    pub const PTE_R: usize = 1 << 1;
    pub const PTE_W: usize = 1 << 2;
    pub const PTE_X: usize = 1 << 3;
    pub const PTE_U: usize = 1 << 4; // user can access
}
