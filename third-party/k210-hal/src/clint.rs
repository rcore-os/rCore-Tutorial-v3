//! Core Local Interruptor (CLINT)
//!
//! TODO: Should this module designed in a somehow IP-core peripheral create?

/// mtime register
pub mod mtime {
    use crate::pac;
    /// Read mtime register.
    pub fn read() -> u64 {
        unsafe { (*pac::CLINT::ptr()).mtime.read().bits() }
    }
}

/// msip register
pub mod msip {
    use crate::pac;
    
    /// set IPI interrupt flag for one given hart
    pub fn set_ipi(hart_id: usize) {
        unsafe {
            (*pac::CLINT::ptr()).msip[hart_id].write(|w| 
                w.bits(1))
        }
    }
    /// clear IPI interrupt flag for one given hart
    pub fn clear_ipi(hart_id: usize) {
        unsafe {
            (*pac::CLINT::ptr()).msip[hart_id].write(|w| 
                w.bits(0))
        }
    }
}

/// mtimecmp register
pub mod mtimecmp {
    use crate::pac;
    
    /// Read 64-bit mtimecmp register for certain hart id
    pub fn read(hart_id: usize) -> u64 {
        unsafe { (*pac::CLINT::ptr()).mtimecmp[hart_id].read().bits() }
    }

    /// Write 64-bit mtimecmp register for certain hart id
    pub fn write(hart_id: usize, bits: u64) {
        // Volume II: RISC-V Privileged Architectures V1.10 p.31, figure 3.15
        unsafe { (*pac::CLINT::ptr()).mtimecmp[hart_id].write(|w| 
            w.bits(bits)) };
    }
}
