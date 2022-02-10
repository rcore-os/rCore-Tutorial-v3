//! (TODO) Serial Peripheral Interface (SPI)

use crate::pac::SPI0;
use crate::clock::Clocks;
use crate::sysctl::{self, APB0};
pub use embedded_hal::spi::{Mode, Polarity, Phase};
use core::convert::Infallible;

/// 
pub struct Spi<SPI> {
    spi: SPI
}

impl Spi<SPI0> {
    pub fn spi0(
        spi: SPI0, 
        mode: Mode, 
        frame_format: FrameFormat, 
        endian: Endian, 
        clock: &Clocks, 
        apb0: &mut APB0
    ) -> Self {
        let work_mode = hal_mode_to_pac(mode);
        let frame_format = frame_format_to_pac(frame_format);
        let tmod = crate::pac::spi0::ctrlr0::TMOD_A::TRANS_RECV; // todo other modes
        let endian = endian as u32;
        let data_bit_length = 8; // todo more length
        let _ = clock; // todo
        unsafe {
            // no interrupts for now
            spi.imr.write(|w| w.bits(0x00));
            // no dma for now
            spi.dmacr.write(|w| w.bits(0x00));
            spi.dmatdlr.write(|w| w.bits(0x10));
            spi.dmardlr.write(|w| w.bits(0x00));
            // no slave access for now
            spi.ser.write(|w| w.bits(0x00));
            spi.ssienr.write(|w| w.bits(0x00));
            // set control registers
            spi.ctrlr0.write(|w| {
                w.work_mode()
                    .variant(work_mode)
                    .tmod()
                    .variant(tmod)
                    .frame_format()
                    .variant(frame_format)
                    .data_length()
                    .bits(data_bit_length - 1)
            });
            spi.spi_ctrlr0.reset(); // standard
            spi.endian.write(|w| w.bits(endian));
        }
        // enable APB0 bus
        apb0.enable();
        // enable peripheral via sysctl
        sysctl::clk_en_peri().modify(|_r, w| 
            w.spi0_clk_en().set_bit());
        Spi { spi }
    }

    pub fn release(self) -> SPI0 {
        // power off
        sysctl::clk_en_peri().modify(|_r, w| 
            w.spi0_clk_en().clear_bit());
        self.spi
    }
}

impl embedded_hal::spi::FullDuplex<u8> for Spi<SPI0> {
    /// An enumeration of SPI errors
    type Error = Infallible;

    /// Reads the word stored in the shift register
    ///
    /// **NOTE** A word must be sent to the slave before attempting to call this
    /// method.
    fn try_read(&mut self) -> nb::Result<u8, Self::Error> {
        todo!()
    }

    /// Sends a word to the slave
    fn try_send(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        todo!("{}", word)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FrameFormat {
    Standard,
    Dual,
    Quad,
    Octal,
}
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Endian {
    Little = 0,
    Big = 1,
}

#[inline]
fn hal_mode_to_pac(mode: Mode) -> crate::pac::spi0::ctrlr0::WORK_MODE_A {
    use crate::pac::spi0::ctrlr0::WORK_MODE_A;
    use {Polarity::*, Phase::*};
    match (mode.polarity, mode.phase) {
        (IdleLow, CaptureOnFirstTransition) => WORK_MODE_A::MODE0,
        (IdleLow, CaptureOnSecondTransition) => WORK_MODE_A::MODE1,
        (IdleHigh, CaptureOnFirstTransition) => WORK_MODE_A::MODE2,
        (IdleHigh, CaptureOnSecondTransition) => WORK_MODE_A::MODE3,
    }
}

#[inline]
fn frame_format_to_pac(frame_format: FrameFormat) -> crate::pac::spi0::ctrlr0::FRAME_FORMAT_A {
    use crate::pac::spi0::ctrlr0::FRAME_FORMAT_A;
    match frame_format {
        FrameFormat::Standard => FRAME_FORMAT_A::STANDARD,
        FrameFormat::Dual => FRAME_FORMAT_A::DUAL,
        FrameFormat::Quad => FRAME_FORMAT_A::QUAD,
        FrameFormat::Octal => FRAME_FORMAT_A::OCTAL,
    }
}
