//! (TODO) Direct Memory Access Controller (DMAC)
use crate::{pac, sysctl};

pub fn dmac_id() -> u64 {
    unsafe { (*pac::DMAC::ptr()).id.read().bits() }
}

pub fn dmac_version() -> u64 {
    unsafe { (*pac::DMAC::ptr()).compver.read().bits() }
}

pub trait DmacExt {
    fn configure(self, /* sysctl ACLK clock */) -> Dmac;
}

impl DmacExt for pac::DMAC {
    fn configure(self, /* sysctl ACLK clock */) -> Dmac {
        // enable
        sysctl::clk_en_peri().modify(|_, w| w.dma_clk_en().set_bit());
        // todo: reset
        Dmac {} // todo
    }
}

pub struct Dmac {

}

// pub struct C0 {
//     // todo
//     pub async fn poll() {
        
//     }
// }
