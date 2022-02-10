//! SPI peripherals handling
use core::cmp;
use core::convert::TryInto;
use core::ops::Deref;
//use k210_hal::pac;
use k210_pac as pac;
use pac::{SPI0,SPI1,spi0};
use pac::spi0::ctrlr0;
use pac::spi0::spi_ctrlr0;

//use super::sysctl;
use super::sysctl::{dma_channel, self};
use super::dmac::{DMAC, address_increment, burst_length, transfer_width};

/// Extension trait that constrains SPI peripherals
pub trait SPIExt: Sized {
    /// Constrains SPI peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> SPIImpl<Self>;
}

/// Trait for generalizing over SPI0 and SPI1 (SPI2 is slave-only and SPI3 is !!!special!!!)
pub trait SPI01: Deref<Target = spi0::RegisterBlock> {
    #[doc(hidden)]
    const CLK: sysctl::clock;
    #[doc(hidden)]
    const DIV: sysctl::threshold;
    #[doc(hidden)]
    const DMA_RX: sysctl::dma_select;
    #[doc(hidden)]
    const DMA_TX: sysctl::dma_select;
}

impl SPI01 for SPI0 {
    const CLK: sysctl::clock = sysctl::clock::SPI0;
    const DIV: sysctl::threshold = sysctl::threshold::SPI0;
    const DMA_RX: sysctl::dma_select = sysctl::dma_select::SSI0_RX_REQ;
    const DMA_TX: sysctl::dma_select = sysctl::dma_select::SSI0_TX_REQ;
}
impl SPI01 for SPI1 {
    const CLK: sysctl::clock = sysctl::clock::SPI1;
    const DIV: sysctl::threshold = sysctl::threshold::SPI1;
    const DMA_RX: sysctl::dma_select = sysctl::dma_select::SSI1_RX_REQ;
    const DMA_TX: sysctl::dma_select = sysctl::dma_select::SSI1_TX_REQ;
}

impl<SPI: SPI01> SPIExt for SPI {
    fn constrain(self) -> SPIImpl<SPI> {
        SPIImpl::<SPI>::new(self)
    }
}

pub struct SPIImpl<IF> {
    spi: IF,
}

/** Borrow work mode from pac */
pub use ctrlr0::WORK_MODE_A as work_mode;
/** Borrow frame format from pac */
pub use ctrlr0::FRAME_FORMAT_A as frame_format;
/** Borrow aitm from pac */
pub use spi_ctrlr0::AITM_A as aitm;
/** Borrow tmod from pac */
pub use ctrlr0::TMOD_A as tmod;

pub trait SPI {
    fn configure(
        &self,
        work_mode: work_mode,
        frame_format: frame_format,
        data_bit_length: u8,
        endian: u32,
        instruction_length: u8,
        address_length: u8,
        wait_cycles: u8,
        instruction_address_trans_mode: aitm,
        tmod: tmod,
    );
    fn set_clk_rate(&self, spi_clk: u32) -> u32;
    fn recv_data<X: TruncU32>(&self, chip_select: u32, rx: &mut [X]);
    fn recv_data_dma(&self, dmac: &DMAC, channel_num: dma_channel, chip_select: u32, rx: &mut [u32]);
    fn send_data<X: Into<u32> + Copy>(&self, chip_select: u32, tx: &[X]);
    fn send_data_dma(&self, dmac: &DMAC, channel_num: dma_channel, chip_select: u32, tx: &[u32]);
    fn fill_data(&self, chip_select: u32, value: u32, tx_len: usize);
    fn fill_data_dma(&self, dmac: &DMAC, channel_num: dma_channel, chip_select: u32, value: u32, tx_len: usize);
}

impl<IF: SPI01> SPIImpl<IF> {
    pub fn new(spi: IF) -> Self {
        Self { spi }
    }
}

/** Trait for trunction of a SPI frame from u32 register to other unsigned integer types. */
pub trait TruncU32 {
    fn trunc(val: u32)-> Self;
}
impl TruncU32 for u32 { fn  trunc(val: u32) -> u32 { return val; } }
impl TruncU32 for u16 { fn  trunc(val: u32) -> u16 { return (val & 0xffff) as u16; } }
impl TruncU32 for u8 { fn  trunc(val: u32) -> u8 { return (val & 0xff) as u8; } }

impl<IF: SPI01> SPI for SPIImpl<IF> {
    /// Configure SPI transaction
    fn configure(
        &self,
        work_mode: work_mode,
        frame_format: frame_format,
        data_bit_length: u8,
        endian: u32,
        instruction_length: u8,
        address_length: u8,
        wait_cycles: u8,
        instruction_address_trans_mode: aitm,
        tmod: tmod,
    ) {
        assert!(data_bit_length >= 4 && data_bit_length <= 32);
        assert!(wait_cycles < (1 << 5));
        let inst_l: u8 = match instruction_length {
            0 => 0,
            4 => 1,
            8 => 2,
            16 => 3,
            _ => panic!("unhandled intruction length"),
        };

        assert!(address_length % 4 == 0 && address_length <= 60);
        let addr_l: u8 = address_length / 4;

        unsafe {
            self.spi.imr.write(|w| w.bits(0x00));
            self.spi.dmacr.write(|w| w.bits(0x00));
            self.spi.dmatdlr.write(|w| w.bits(0x10));
            self.spi.dmardlr.write(|w| w.bits(0x00));
            self.spi.ser.write(|w| w.bits(0x00));
            self.spi.ssienr.write(|w| w.bits(0x00));
            self.spi.ctrlr0.write(|w| {
                w.work_mode()
                    .variant(work_mode)
                    .tmod()
                    .variant(tmod)
                    .frame_format()
                    .variant(frame_format)
                    .data_length()
                    .bits(data_bit_length - 1)
            });
            self.spi.spi_ctrlr0.write(|w| {
                w.aitm()
                    .variant(instruction_address_trans_mode)
                    .addr_length()
                    .bits(addr_l)
                    .inst_length()
                    .bits(inst_l)
                    .wait_cycles()
                    .bits(wait_cycles)
            });
            self.spi.endian.write(|w| w.bits(endian));
        }
    }

    /// Set SPI clock rate
    fn set_clk_rate(&self, spi_clk: u32) -> u32 {
        sysctl::clock_enable(IF::CLK);
        sysctl::clock_set_threshold(IF::DIV, 0);
        let clock_freq: u32 = sysctl::clock_get_freq(sysctl::clock::SPI0);
        let spi_baudr = clock_freq / spi_clk;
        // Clamp baudrate divider to valid range
        let spi_baudr = cmp::min(cmp::max(spi_baudr, 2), 65534);
        unsafe {
            self.spi.baudr.write(|w| w.bits(spi_baudr));
        }
        clock_freq / spi_baudr
    }

    /// Receive arbitrary data
    // make sure to set tmod to tmod::RECV
    fn recv_data<X: TruncU32>(&self, chip_select: u32, rx: &mut [X]) {
        if rx.len() == 0 {
            return;
        }
        unsafe {
            self.spi.ctrlr1.write(|w| w.bits((rx.len() - 1).try_into().unwrap()));
            self.spi.ssienr.write(|w| w.bits(0x01));
            self.spi.dr[0].write(|w| w.bits(0xffffffff));
            self.spi.ser.write(|w| w.bits(1 << chip_select));

            let mut fifo_len = 0;
            for val in rx.iter_mut() {
                while fifo_len == 0 {
                    fifo_len = self.spi.rxflr.read().bits();
                }
                *val = X::trunc(self.spi.dr[0].read().bits());
                fifo_len -= 1;
            }

            self.spi.ser.write(|w| w.bits(0x00));
            self.spi.ssienr.write(|w| w.bits(0x00));
        }
    }

    /// Receive 32-bit data using DMA.
    // make sure to set tmod to tmod::RECV
    fn recv_data_dma(&self, dmac: &DMAC, channel_num: dma_channel, chip_select: u32, rx: &mut [u32]) {
        if rx.len() == 0 {
            return;
        }
        unsafe {
            self.spi.ctrlr1.write(|w| w.bits((rx.len() - 1).try_into().unwrap()));
            self.spi.ssienr.write(|w| w.bits(0x01));
            self.spi.dmacr.write(|w| w.bits(0x3));    /*enable dma receive */

            sysctl::dma_select(channel_num, IF::DMA_RX);
            dmac.set_single_mode(channel_num, self.spi.dr.as_ptr() as u64, rx.as_ptr() as u64,
                                 address_increment::NOCHANGE, address_increment::INCREMENT,
                                 burst_length::LENGTH_1, transfer_width::WIDTH_32, rx.len() as u32);
            self.spi.dr[0].write(|w| w.bits(0xffffffff));
            self.spi.ser.write(|w| w.bits(1 << chip_select));
            dmac.wait_done(channel_num);

            self.spi.ser.write(|w| w.bits(0x00));
            self.spi.ssienr.write(|w| w.bits(0x00));
        }
    }

    /// Send arbitrary data
    fn send_data<X: Into<u32> + Copy>(&self, chip_select: u32, tx: &[X]) {
        unsafe {
            self.spi.ser.write(|w| w.bits(1 << chip_select));
            self.spi.ssienr.write(|w| w.bits(0x01));

            let mut fifo_len = 0;
            for &val in tx {
                while fifo_len == 0 {
                    fifo_len = 32 - self.spi.txflr.read().bits();
                }
                self.spi.dr[0].write(|f| f.bits(val.into()));
                fifo_len -= 1;
            }

            while (self.spi.sr.read().bits() & 0x05) != 0x04 {
                // IDLE
            }
            self.spi.ser.write(|w| w.bits(0x00));
            self.spi.ssienr.write(|w| w.bits(0x00));
        }
    }

    /// Send 32-bit data using DMA.
    /// If you want to use this function to send 8-bit or 16-bit data, you need to wrap each
    /// data unit in a 32-bit word.
    /// This is intentionally left to the caller: the difficulty here is to avoid the need for alloc/freeing()
    /// buffers every time as the SDK does because this is highly undesirable!
    fn send_data_dma(&self, dmac: &DMAC, channel_num: dma_channel, chip_select: u32, tx: &[u32]) {
        unsafe {
            self.spi.dmacr.write(|w| w.bits(0x2));    /*enable dma transmit*/
            self.spi.ssienr.write(|w| w.bits(0x01));

            sysctl::dma_select(channel_num, IF::DMA_TX);
            dmac.set_single_mode(channel_num, tx.as_ptr() as u64, self.spi.dr.as_ptr() as u64,
                                 address_increment::INCREMENT, address_increment::NOCHANGE,
                                 burst_length::LENGTH_4, transfer_width::WIDTH_32, tx.len() as u32);
            self.spi.ser.write(|w| w.bits(1 << chip_select));
            dmac.wait_done(channel_num);

            while (self.spi.sr.read().bits() & 0x05) != 0x04 {
                // IDLE
            }
            self.spi.ser.write(|w| w.bits(0x00));
            self.spi.ssienr.write(|w| w.bits(0x00));
        }
    }

    /// Send repeated data
    fn fill_data(&self, chip_select: u32, value: u32, mut tx_len: usize) {
        unsafe {
            self.spi.ser.write(|w| w.bits(1 << chip_select));
            self.spi.ssienr.write(|w| w.bits(0x01));

            while tx_len != 0 {
                let fifo_len = (32 - self.spi.txflr.read().bits()) as usize;
                let fifo_len = cmp::min(fifo_len, tx_len);
                for _ in 0..fifo_len {
                    self.spi.dr[0].write(|f| f.bits(value));
                }
                tx_len -= fifo_len;
            }

            while (self.spi.sr.read().bits() & 0x05) != 0x04 {
                // IDLE
            }
            self.spi.ser.write(|w| w.bits(0x00));
            self.spi.ssienr.write(|w| w.bits(0x00));
        }
    }

    /// Send repeated data (using DMA)
    fn fill_data_dma(&self, dmac: &DMAC, channel_num: dma_channel, chip_select: u32, value: u32, tx_len: usize) {
        unsafe {
            self.spi.dmacr.write(|w| w.bits(0x2));    /*enable dma transmit*/
            self.spi.ssienr.write(|w| w.bits(0x01));

            sysctl::dma_select(channel_num, IF::DMA_TX);
            let val = [value];
            // simple trick to repeating a value: don't increment the source address
            dmac.set_single_mode(channel_num, val.as_ptr() as u64, self.spi.dr.as_ptr() as u64,
                                 address_increment::NOCHANGE, address_increment::NOCHANGE,
                                 burst_length::LENGTH_4, transfer_width::WIDTH_32, tx_len.try_into().unwrap());
            self.spi.ser.write(|w| w.bits(1 << chip_select));
            dmac.wait_done(channel_num);

            while (self.spi.sr.read().bits() & 0x05) != 0x04 {
                // IDLE
            }
            self.spi.ser.write(|w| w.bits(0x00));
            self.spi.ssienr.write(|w| w.bits(0x00));
        }
    }
}