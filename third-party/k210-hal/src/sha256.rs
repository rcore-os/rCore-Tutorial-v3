//! Secure Hash Algorithm-256 (SHA256)

use crate::pac::SHA256;
use crate::sysctl::{self, APB0};

/// SHA256 module abstraction
pub struct Sha256 {
    sha256: SHA256,
}

impl Sha256 {
    pub fn sha256(sha256: SHA256, apb0: &mut APB0) -> Sha256 {
        apb0.enable();
        sysctl::clk_en_peri().modify(|_r, w| 
            w.sha_clk_en().set_bit());
        sysctl::peri_reset().modify(|_r, w| 
            w.sha_reset().set_bit());
        sysctl::peri_reset().modify(|_r, w| 
            w.sha_reset().clear_bit());
        Sha256 { sha256 }
    }

    pub fn new_digest(self) -> Digest {
        todo!()
    }

    pub fn release(self) -> SHA256 {
        sysctl::clk_en_peri().modify(|_r, w| 
            w.sha_clk_en().clear_bit());
        self.sha256
    }
}

pub struct Digest {
    sha256: SHA256,
}

impl Digest {
    pub fn write_u32(&mut self, n: u32) {
        let _todo = n;
        todo!()
    }

    pub fn finish(&self, out: &mut [u8; 32]) {
        let _todo = out;
        todo!()
    }

    pub fn free(self) -> Sha256 {
        Sha256 { sha256: self.sha256 }
    }
}
