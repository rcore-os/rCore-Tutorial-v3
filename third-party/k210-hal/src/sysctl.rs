//! (TODO) System Controller (SYSCTL)

use crate::pac::{sysctl, SYSCTL};

pub(crate) fn sysctl<'a>() -> &'a sysctl::RegisterBlock {
    unsafe { &*(SYSCTL::ptr()) }
}

pub(crate) fn clk_en_cent<'a>() -> &'a sysctl::CLK_EN_CENT {
    &sysctl().clk_en_cent
}

pub(crate) fn clk_en_peri<'a>() -> &'a sysctl::CLK_EN_PERI {
    &sysctl().clk_en_peri
}

pub(crate) fn peri_reset<'a>() -> &'a sysctl::PERI_RESET {
    &sysctl().peri_reset
}

pub trait SysctlExt {
    fn constrain(self) -> Parts;
}

impl SysctlExt for SYSCTL {
    fn constrain(self) -> Parts {
        Parts { 
            apb0: APB0 { _ownership: () },
        }
    }
}

// ref: sysctl.c
pub struct Parts {
    // todo: PLL0, PLL1, PLL2
    // todo: CPU, SRAM, APB-bus, ROM, DMA, AI
    pub apb0: APB0,
    // pub apb1: APB1,
    // pub apb2: APB2,
}

pub struct APB0 {
    _ownership: ()
}

impl APB0 {
    pub(crate) fn enable(&mut self) {
        clk_en_cent().modify(
            |_r, w|
                w.apb0_clk_en().set_bit()
        );
    }
}

// pub struct APB1 {
//     _ownership: ()
// }

// pub struct APB2 {
//     _ownership: ()
// }
