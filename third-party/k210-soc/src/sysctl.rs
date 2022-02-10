#![allow(unused)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

//! SYSCTL peripheral
//use k210_hal::pac;
use k210_pac as pac;

use core::convert::TryInto;

use super::utils::set_bit;
use super::sleep::usleep;

mod pll_compute;

const SYSCTRL_CLOCK_FREQ_IN0: u32 = 26000000;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum pll {
    /** PLL0 can usually be selected as alternative to IN0, for example the CPU
     * clock speed. It can be used as source for PLL2. */
    PLL0,
    /** PLL1 is used for the KPU clock, and can be used as source for PLL2. */
    PLL1,
    /** PLL2 is used for I2S clocks. */
    PLL2,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum clock_source {
    IN0,
    PLL0,
    PLL1,
    PLL2,
    ACLK,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum clock {
    PLL0,
    PLL1,
    PLL2,
    CPU,
    SRAM0,
    SRAM1,
    APB0,
    APB1,
    APB2,
    ROM,
    DMA,
    AI,
    DVP,
    FFT,
    GPIO,
    SPI0,
    SPI1,
    SPI2,
    SPI3,
    I2S0,
    I2S1,
    I2S2,
    I2C0,
    I2C1,
    I2C2,
    UART1,
    UART2,
    UART3,
    AES,
    FPIOA,
    TIMER0,
    TIMER1,
    TIMER2,
    WDT0,
    WDT1,
    SHA,
    OTP,
    RTC,
    ACLK,
    HCLK,
    IN0,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum threshold {
    ACLK,
    APB0,
    APB1,
    APB2,
    SRAM0,
    SRAM1,
    AI,
    DVP,
    ROM,
    SPI0,
    SPI1,
    SPI2,
    SPI3,
    TIMER0,
    TIMER1,
    TIMER2,
    I2S0,
    I2S1,
    I2S2,
    I2S0_M,
    I2S1_M,
    I2S2_M,
    I2C0,
    I2C1,
    I2C2,
    WDT0,
    WDT1,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum clock_select {
    PLL0_BYPASS,
    PLL1_BYPASS,
    PLL2_BYPASS,
    PLL2,
    ACLK,
    SPI3,
    TIMER0,
    TIMER1,
    TIMER2,
    SPI3_SAMPLE,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum io_power_mode {
    V33,
    V18,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum power_bank {
    BANK0 = 0,
    BANK1,
    BANK2,
    BANK3,
    BANK4,
    BANK5,
    BANK6,
    BANK7,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum reset {
    SOC,
    ROM,
    DMA,
    AI,
    DVP,
    FFT,
    GPIO,
    SPI0,
    SPI1,
    SPI2,
    SPI3,
    I2S0,
    I2S1,
    I2S2,
    I2C0,
    I2C1,
    I2C2,
    UART1,
    UART2,
    UART3,
    AES,
    FPIOA,
    TIMER0,
    TIMER1,
    TIMER2,
    WDT0,
    WDT1,
    SHA,
    RTC,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum dma_channel {
    CHANNEL0 = 0,
    CHANNEL1 = 1,
    CHANNEL2 = 2,
    CHANNEL3 = 3,
    CHANNEL4 = 4,
    CHANNEL5 = 5,
}

impl dma_channel {
    pub fn idx(self) -> usize { self as usize }
}

pub type dma_select = pac::sysctl::dma_sel0::DMA_SEL0_A;

fn clock_bus_en(clock: clock, en: bool) {
    /*
     * The timer is under APB0, to prevent apb0_clk_en1 and apb0_clk_en0
     * on same register, we split it to peripheral and central two
     * registers, to protect CPU close apb0 clock accidentally.
     *
     * The apb0_clk_en0 and apb0_clk_en1 have same function,
     * one of them set, the APB0 clock enable.
     */

    /* The APB clock should carefully disable */
    if en {
        match clock {
            /*
             * These peripheral devices are under APB0
             * GPIO, UART1, UART2, UART3, SPI_SLAVE, I2S0, I2S1,
             * I2S2, I2C0, I2C1, I2C2, FPIOA, SHA256, TIMER0,
             * TIMER1, TIMER2
             */
            clock::GPIO
            | clock::SPI2
            | clock::I2S0
            | clock::I2S1
            | clock::I2S2
            | clock::I2C0
            | clock::I2C1
            | clock::I2C2
            | clock::UART1
            | clock::UART2
            | clock::UART3
            | clock::FPIOA
            | clock::TIMER0
            | clock::TIMER1
            | clock::TIMER2
            | clock::SHA => unsafe {
                (*pac::SYSCTL::ptr())
                    .clk_en_cent
                    .modify(|_, w| w.apb0_clk_en().bit(en));
            },

            /*
             * These peripheral devices are under APB1
             * WDT, AES, OTP, DVP, SYSCTL
             */
            clock::AES | clock::WDT0 | clock::WDT1 | clock::OTP | clock::RTC => unsafe {
                (*pac::SYSCTL::ptr())
                    .clk_en_cent
                    .modify(|_, w| w.apb1_clk_en().bit(en));
            },

            /*
             * These peripheral devices are under APB2
             * SPI0, SPI1
             */
            clock::SPI0 | clock::SPI1 => unsafe {
                (*pac::SYSCTL::ptr())
                    .clk_en_cent
                    .modify(|_, w| w.apb2_clk_en().bit(en));
            },

            _ => {}
        }
    }
}

fn clock_device_en(clock: clock, en: bool) {
    unsafe {
        let ptr = pac::SYSCTL::ptr();
        match clock {
            clock::PLL0 => (*ptr).pll0.modify(|_, w| w.out_en().bit(en)),
            clock::PLL1 => (*ptr).pll1.modify(|_, w| w.out_en().bit(en)),
            clock::PLL2 => (*ptr).pll2.modify(|_, w| w.out_en().bit(en)),
            clock::CPU => (*ptr).clk_en_cent.modify(|_, w| w.cpu_clk_en().bit(en)),
            clock::SRAM0 => (*ptr).clk_en_cent.modify(|_, w| w.sram0_clk_en().bit(en)),
            clock::SRAM1 => (*ptr).clk_en_cent.modify(|_, w| w.sram1_clk_en().bit(en)),
            clock::APB0 => (*ptr).clk_en_cent.modify(|_, w| w.apb0_clk_en().bit(en)),
            clock::APB1 => (*ptr).clk_en_cent.modify(|_, w| w.apb1_clk_en().bit(en)),
            clock::APB2 => (*ptr).clk_en_cent.modify(|_, w| w.apb2_clk_en().bit(en)),
            clock::ROM => (*ptr).clk_en_peri.modify(|_, w| w.rom_clk_en().bit(en)),
            clock::DMA => (*ptr).clk_en_peri.modify(|_, w| w.dma_clk_en().bit(en)),
            clock::AI => (*ptr).clk_en_peri.modify(|_, w| w.ai_clk_en().bit(en)),
            clock::DVP => (*ptr).clk_en_peri.modify(|_, w| w.dvp_clk_en().bit(en)),
            clock::FFT => (*ptr).clk_en_peri.modify(|_, w| w.fft_clk_en().bit(en)),
            clock::SPI3 => (*ptr).clk_en_peri.modify(|_, w| w.spi3_clk_en().bit(en)),
            clock::GPIO => (*ptr).clk_en_peri.modify(|_, w| w.gpio_clk_en().bit(en)),
            clock::SPI2 => (*ptr).clk_en_peri.modify(|_, w| w.spi2_clk_en().bit(en)),
            clock::I2S0 => (*ptr).clk_en_peri.modify(|_, w| w.i2s0_clk_en().bit(en)),
            clock::I2S1 => (*ptr).clk_en_peri.modify(|_, w| w.i2s1_clk_en().bit(en)),
            clock::I2S2 => (*ptr).clk_en_peri.modify(|_, w| w.i2s2_clk_en().bit(en)),
            clock::I2C0 => (*ptr).clk_en_peri.modify(|_, w| w.i2c0_clk_en().bit(en)),
            clock::I2C1 => (*ptr).clk_en_peri.modify(|_, w| w.i2c1_clk_en().bit(en)),
            clock::I2C2 => (*ptr).clk_en_peri.modify(|_, w| w.i2c2_clk_en().bit(en)),
            clock::UART1 => (*ptr).clk_en_peri.modify(|_, w| w.uart1_clk_en().bit(en)),
            clock::UART2 => (*ptr).clk_en_peri.modify(|_, w| w.uart2_clk_en().bit(en)),
            clock::UART3 => (*ptr).clk_en_peri.modify(|_, w| w.uart3_clk_en().bit(en)),
            clock::FPIOA => (*ptr).clk_en_peri.modify(|_, w| w.fpioa_clk_en().bit(en)),
            clock::TIMER0 => (*ptr).clk_en_peri.modify(|_, w| w.timer0_clk_en().bit(en)),
            clock::TIMER1 => (*ptr).clk_en_peri.modify(|_, w| w.timer1_clk_en().bit(en)),
            clock::TIMER2 => (*ptr).clk_en_peri.modify(|_, w| w.timer2_clk_en().bit(en)),
            clock::SHA => (*ptr).clk_en_peri.modify(|_, w| w.sha_clk_en().bit(en)),
            clock::AES => (*ptr).clk_en_peri.modify(|_, w| w.aes_clk_en().bit(en)),
            clock::WDT0 => (*ptr).clk_en_peri.modify(|_, w| w.wdt0_clk_en().bit(en)),
            clock::WDT1 => (*ptr).clk_en_peri.modify(|_, w| w.wdt1_clk_en().bit(en)),
            clock::OTP => (*ptr).clk_en_peri.modify(|_, w| w.otp_clk_en().bit(en)),
            clock::RTC => (*ptr).clk_en_peri.modify(|_, w| w.rtc_clk_en().bit(en)),
            clock::SPI0 => (*ptr).clk_en_peri.modify(|_, w| w.spi0_clk_en().bit(en)),
            clock::SPI1 => (*ptr).clk_en_peri.modify(|_, w| w.spi1_clk_en().bit(en)),
            clock::ACLK | clock::HCLK | clock::IN0 => { /* no separate enables */ }
        }
    }
}

pub fn clock_enable(clock: clock) {
    clock_bus_en(clock, true);
    clock_device_en(clock, true);
}

pub fn sysctl_clock_disable(clock: clock) {
    clock_bus_en(clock, false);
    clock_device_en(clock, false);
}

/// Set clock divider
pub fn clock_set_threshold(which: threshold, threshold: u32) {
    // TODO: this should take a multiplier directly, not a peripheral specific value
    unsafe {
        let ptr = pac::SYSCTL::ptr();
        match which {
            /* 2 bit wide */
            threshold::ACLK => (*ptr).clk_sel0.modify(|_, w| w.aclk_divider_sel().bits(threshold as u8)),

            /* 3 bit wide */
            threshold::APB0 => (*ptr).clk_sel0.modify(|_, w| w.apb0_clk_sel().bits(threshold as u8)),
            threshold::APB1 => (*ptr).clk_sel0.modify(|_, w| w.apb1_clk_sel().bits(threshold as u8)),
            threshold::APB2 => (*ptr).clk_sel0.modify(|_, w| w.apb2_clk_sel().bits(threshold as u8)),

            /* 4 bit wide */
            threshold::SRAM0 => (*ptr).clk_th0.modify(|_, w| w.sram0_gclk().bits(threshold as u8)),
            threshold::SRAM1 => (*ptr).clk_th0.modify(|_, w| w.sram1_gclk().bits(threshold as u8)),
            threshold::AI => (*ptr).clk_th0.modify(|_, w| w.ai_gclk().bits(threshold as u8)),
            threshold::DVP => (*ptr).clk_th0.modify(|_, w| w.dvp_gclk().bits(threshold as u8)),
            threshold::ROM => (*ptr).clk_th0.modify(|_, w| w.rom_gclk().bits(threshold as u8)),

            /* 8 bit wide */
            threshold::SPI0 => (*ptr).clk_th1.modify(|_, w| w.spi0_clk().bits(threshold as u8)),
            threshold::SPI1 => (*ptr).clk_th1.modify(|_, w| w.spi1_clk().bits(threshold as u8)),
            threshold::SPI2 => (*ptr).clk_th1.modify(|_, w| w.spi2_clk().bits(threshold as u8)),
            threshold::SPI3 => (*ptr).clk_th1.modify(|_, w| w.spi3_clk().bits(threshold as u8)),
            threshold::TIMER0 => (*ptr).clk_th2.modify(|_, w| w.timer0_clk().bits(threshold as u8)),
            threshold::TIMER1 => (*ptr).clk_th2.modify(|_, w| w.timer1_clk().bits(threshold as u8)),
            threshold::TIMER2 => (*ptr).clk_th2.modify(|_, w| w.timer2_clk().bits(threshold as u8)),
            threshold::I2S0_M => (*ptr).clk_th4.modify(|_, w| w.i2s0_mclk().bits(threshold as u8)),
            threshold::I2S1_M => (*ptr).clk_th4.modify(|_, w| w.i2s1_mclk().bits(threshold as u8)),
            threshold::I2S2_M => (*ptr).clk_th5.modify(|_, w| w.i2s2_mclk().bits(threshold as u8)),
            threshold::I2C0 => (*ptr).clk_th5.modify(|_, w| w.i2c0_clk().bits(threshold as u8)),
            threshold::I2C1 => (*ptr).clk_th5.modify(|_, w| w.i2c1_clk().bits(threshold as u8)),
            threshold::I2C2 => (*ptr).clk_th5.modify(|_, w| w.i2c2_clk().bits(threshold as u8)),
            threshold::WDT0 => (*ptr).clk_th6.modify(|_, w| w.wdt0_clk().bits(threshold as u8)),
            threshold::WDT1 => (*ptr).clk_th6.modify(|_, w| w.wdt1_clk().bits(threshold as u8)),

            /* 16 bit wide */
            threshold::I2S0 => (*ptr).clk_th3.modify(|_, w| w.i2s0_clk().bits(threshold as u16)),
            threshold::I2S1 => (*ptr).clk_th3.modify(|_, w| w.i2s1_clk().bits(threshold as u16)),
            threshold::I2S2 => (*ptr).clk_th4.modify(|_, w| w.i2s2_clk().bits(threshold as u16)),
        }
    }
}

/// Get clock divider
pub fn clock_get_threshold(which: threshold) -> u32 {
    unsafe {
        // TODO: this should return a multiplier directly, not a peripheral specific value
        let ptr = pac::SYSCTL::ptr();
        match which {
            /* 2 bit wide */
            threshold::ACLK => (*ptr).clk_sel0.read().aclk_divider_sel().bits().into(),

            /* 3 bit wide */
            threshold::APB0 => (*ptr).clk_sel0.read().apb0_clk_sel().bits().into(),
            threshold::APB1 => (*ptr).clk_sel0.read().apb1_clk_sel().bits().into(),
            threshold::APB2 => (*ptr).clk_sel0.read().apb2_clk_sel().bits().into(),

            /* 4 bit wide */
            threshold::SRAM0 => (*ptr).clk_th0.read().sram0_gclk().bits().into(),
            threshold::SRAM1 => (*ptr).clk_th0.read().sram1_gclk().bits().into(),
            threshold::AI => (*ptr).clk_th0.read().ai_gclk().bits().into(),
            threshold::DVP => (*ptr).clk_th0.read().dvp_gclk().bits().into(),
            threshold::ROM => (*ptr).clk_th0.read().rom_gclk().bits().into(),

            /* 8 bit wide */
            threshold::SPI0 => (*ptr).clk_th1.read().spi0_clk().bits().into(),
            threshold::SPI1 => (*ptr).clk_th1.read().spi1_clk().bits().into(),
            threshold::SPI2 => (*ptr).clk_th1.read().spi2_clk().bits().into(),
            threshold::SPI3 => (*ptr).clk_th1.read().spi3_clk().bits().into(),
            threshold::TIMER0 => (*ptr).clk_th2.read().timer0_clk().bits().into(),
            threshold::TIMER1 => (*ptr).clk_th2.read().timer1_clk().bits().into(),
            threshold::TIMER2 => (*ptr).clk_th2.read().timer2_clk().bits().into(),
            threshold::I2S0_M => (*ptr).clk_th4.read().i2s0_mclk().bits().into(),
            threshold::I2S1_M => (*ptr).clk_th4.read().i2s1_mclk().bits().into(),
            threshold::I2S2_M => (*ptr).clk_th5.read().i2s2_mclk().bits().into(),
            threshold::I2C0 => (*ptr).clk_th5.read().i2c0_clk().bits().into(),
            threshold::I2C1 => (*ptr).clk_th5.read().i2c1_clk().bits().into(),
            threshold::I2C2 => (*ptr).clk_th5.read().i2c2_clk().bits().into(),
            threshold::WDT0 => (*ptr).clk_th6.read().wdt0_clk().bits().into(),
            threshold::WDT1 => (*ptr).clk_th6.read().wdt1_clk().bits().into(),

            /* 16 bit wide */
            threshold::I2S0 => (*ptr).clk_th3.read().i2s0_clk().bits().into(),
            threshold::I2S1 => (*ptr).clk_th3.read().i2s1_clk().bits().into(),
            threshold::I2S2 => (*ptr).clk_th4.read().i2s2_clk().bits().into(),
        }
    }
}

pub fn set_power_mode(power_bank: power_bank, mode: io_power_mode) {
    unsafe {
        (*pac::SYSCTL::ptr()).power_sel.modify(|r, w| {
            w.bits(set_bit(
                r.bits(),
                power_bank as u8,
                match mode {
                    io_power_mode::V33 => false,
                    io_power_mode::V18 => true,
                },
            ))
        });
    }
}

/** Route SPI0_D0-D7 DVP_D0-D7 functions to SPI and DVP data pins (bypassing FPIOA). */
pub fn set_spi0_dvp_data(status: bool) {
    unsafe {
        (*pac::SYSCTL::ptr())
            .misc
            .modify(|_, w| w.spi_dvp_data_enable().bit(status));
    }
}

/** Map PLL2 cksel value to clock source */
fn pll2_cksel_to_source(bits: u8) -> clock_source {
    match bits {
        0 => clock_source::IN0,
        1 => clock_source::PLL0,
        2 => clock_source::PLL1,
        _ => panic!("invalid value for PLL2 ckin_sel"),
    }
}

/** Map clock source to PLL2 cksel value */
fn pll2_source_to_cksel(source: clock_source) -> u8 {
    match source {
        clock_source::IN0 => 0,
        clock_source::PLL0 => 1,
        clock_source::PLL1 => 2,
        _ => panic!("unsupported clock source for PLL2"),
    }
}

pub fn pll_get_freq(pll: pll) -> u32 {
    let freq_in;
    let nr;
    let nf;
    let od;

    match pll {
        pll::PLL0 => {
            freq_in = clock_source_get_freq(clock_source::IN0);
            unsafe {
                let val = (*pac::SYSCTL::ptr()).pll0.read();
                nr = val.clkr().bits() + 1;
                nf = val.clkf().bits() + 1;
                od = val.clkod().bits() + 1;
            }
        }
        pll::PLL1 => {
            freq_in = clock_source_get_freq(clock_source::IN0);
            unsafe {
                let val = (*pac::SYSCTL::ptr()).pll1.read();
                nr = val.clkr().bits() + 1;
                nf = val.clkf().bits() + 1;
                od = val.clkod().bits() + 1;
            }
        }
        pll::PLL2 => {
            /* Get input freq accrording to select register. */
            freq_in = clock_source_get_freq(pll2_cksel_to_source(clock_get_clock_select(clock_select::PLL2)));
            unsafe {
                let val = (*pac::SYSCTL::ptr()).pll2.read();
                nr = val.clkr().bits() + 1;
                nf = val.clkf().bits() + 1;
                od = val.clkod().bits() + 1;
            }
        }
    }

    /*
     * Get final PLL output freq
     * FOUT = FIN / NR * NF / OD
     * (rewritten as integer expression)
     */
    ((u64::from(freq_in) * u64::from(nf)) / (u64::from(nr) * u64::from(od))).try_into().unwrap()
}

pub fn clock_source_get_freq(source: clock_source) -> u32 {
    match source {
        clock_source::IN0 => SYSCTRL_CLOCK_FREQ_IN0,
        clock_source::PLL0 => pll_get_freq(pll::PLL0),
        clock_source::PLL1 => pll_get_freq(pll::PLL1),
        clock_source::PLL2 => pll_get_freq(pll::PLL2),
        clock_source::ACLK => clock_get_freq(clock::ACLK),
    }
}

pub fn clock_set_clock_select(which: clock_select, select: u8) {
    unsafe {
        let ptr = pac::SYSCTL::ptr();
        // Seems that PLL2 is the only one that takes a non-boolean clock select
        // TODO:  take a clock_source directly when we know the meanings of these bits
        match which {
            clock_select::PLL0_BYPASS => (*ptr).pll0.modify(|_, w| w.bypass().bit(select != 0)),
            clock_select::PLL1_BYPASS => (*ptr).pll1.modify(|_, w| w.bypass().bit(select != 0)),
            clock_select::PLL2_BYPASS => (*ptr).pll2.modify(|_, w| w.bypass().bit(select != 0)),
            clock_select::PLL2 => (*ptr).pll2.modify(|_, w| w.ckin_sel().bits(select)),
            clock_select::ACLK => (*ptr).clk_sel0.modify(|_, w| w.aclk_sel().bit(select != 0)),
            clock_select::SPI3 => (*ptr).clk_sel0.modify(|_, w| w.spi3_clk_sel().bit(select != 0)),
            clock_select::TIMER0 => (*ptr).clk_sel0.modify(|_, w| w.timer0_clk_sel().bit(select != 0)),
            clock_select::TIMER1 => (*ptr).clk_sel0.modify(|_, w| w.timer1_clk_sel().bit(select != 0)),
            clock_select::TIMER2 => (*ptr).clk_sel0.modify(|_, w| w.timer2_clk_sel().bit(select != 0)),
            clock_select::SPI3_SAMPLE => (*ptr).clk_sel1.modify(|_, w| w.spi3_sample_clk_sel().bit(select != 0)),
        }
    }
}

pub fn clock_get_clock_select(which: clock_select) -> u8 {
    unsafe {
        let ptr = pac::SYSCTL::ptr();
        // Seems that PLL2 is the only one that has a non-boolean clock select
        // TODO: return a clock_source directly when we know the meanings of these bits
        //   meaning seems to be usually:
        //     0  IN0
        //     1  PLL0
        //     (2  PLL1)
        //   it's likely different for _BYPASS, which, I suspect, wires the PLL output to the
        //   input (IN0 for PLL0 and PLL1, selectable for PLL2)
        match which {
            clock_select::PLL0_BYPASS => (*ptr).pll0.read().bypass().bit().into(),
            clock_select::PLL1_BYPASS => (*ptr).pll1.read().bypass().bit().into(),
            clock_select::PLL2_BYPASS => (*ptr).pll2.read().bypass().bit().into(),
            clock_select::PLL2 => (*ptr).pll2.read().ckin_sel().bits().into(),
            clock_select::ACLK => (*ptr).clk_sel0.read().aclk_sel().bit().into(),
            clock_select::SPI3 => (*ptr).clk_sel0.read().spi3_clk_sel().bit().into(),
            clock_select::TIMER0 => (*ptr).clk_sel0.read().timer0_clk_sel().bit().into(),
            clock_select::TIMER1 => (*ptr).clk_sel0.read().timer1_clk_sel().bit().into(),
            clock_select::TIMER2 => (*ptr).clk_sel0.read().timer2_clk_sel().bit().into(),
            clock_select::SPI3_SAMPLE => (*ptr).clk_sel1.read().spi3_sample_clk_sel().bit().into(),
        }
    }
}

pub fn clock_get_freq(clock: clock) -> u32 {
    // TODO: all of these are source / threshold, where source can depend on clock_select: generalize this
    //       to some kind of clock tree
    // TODO: clock_source_get_freq(ACLK) calls back into here, don't do this
    match clock {
        clock::IN0 => clock_source_get_freq(clock_source::IN0),
        clock::PLL0 => clock_source_get_freq(clock_source::PLL0),
        clock::PLL1 => clock_source_get_freq(clock_source::PLL1),
        clock::PLL2 => clock_source_get_freq(clock_source::PLL2),
        clock::CPU | clock::DMA | clock::FFT | clock::ACLK | clock::HCLK => match clock_get_clock_select(clock_select::ACLK) {
            0 => clock_source_get_freq(clock_source::IN0),
            1 => {
                clock_source_get_freq(clock_source::PLL0)
                    / (2 << clock_get_threshold(threshold::ACLK))
            }
            _ => panic!("invalid cpu clock select"),
        },
        clock::SRAM0 => clock_source_get_freq(clock_source::ACLK) / (clock_get_threshold(threshold::SRAM0) + 1),
        clock::SRAM1 => clock_source_get_freq(clock_source::ACLK) / (clock_get_threshold(threshold::SRAM1) + 1),
        clock::ROM => clock_source_get_freq(clock_source::ACLK) / (clock_get_threshold(threshold::ROM) + 1),
        clock::DVP => clock_source_get_freq(clock_source::ACLK) / (clock_get_threshold(threshold::DVP) + 1),
        clock::APB0 | clock::GPIO | clock::UART1 | clock::UART2 | clock::UART3 | clock::FPIOA | clock::SHA =>
            clock_source_get_freq(clock_source::ACLK) / (clock_get_threshold(threshold::APB0) + 1),
        clock::APB1 | clock::AES | clock::OTP =>
            clock_source_get_freq(clock_source::ACLK) / (clock_get_threshold(threshold::APB1) + 1),
        clock::APB2 => clock_source_get_freq(clock_source::ACLK) / (clock_get_threshold(threshold::APB2) + 1),
        clock::AI => clock_source_get_freq(clock_source::PLL1) / (clock_get_threshold(threshold::AI) + 1),
        clock::I2S0 => clock_source_get_freq(clock_source::PLL2) / ((clock_get_threshold(threshold::I2S0) + 1) * 2),
        clock::I2S1 => clock_source_get_freq(clock_source::PLL2) / ((clock_get_threshold(threshold::I2S1) + 1) * 2),
        clock::I2S2 => clock_source_get_freq(clock_source::PLL2) / ((clock_get_threshold(threshold::I2S2) + 1) * 2),
        clock::WDT0 => clock_source_get_freq(clock_source::IN0) / ((clock_get_threshold(threshold::WDT0) + 1) * 2),
        clock::WDT1 => clock_source_get_freq(clock_source::IN0) / ((clock_get_threshold(threshold::WDT1) + 1) * 2),
        clock::SPI0 => clock_source_get_freq(clock_source::PLL0) / ((clock_get_threshold(threshold::SPI0) + 1) * 2),
        clock::SPI1 => clock_source_get_freq(clock_source::PLL0) / ((clock_get_threshold(threshold::SPI1) + 1) * 2),
        clock::SPI2 => clock_source_get_freq(clock_source::PLL0) / ((clock_get_threshold(threshold::SPI2) + 1) * 2),
        clock::I2C0 => clock_source_get_freq(clock_source::PLL0) / ((clock_get_threshold(threshold::I2C0) + 1) * 2),
        clock::I2C1 => clock_source_get_freq(clock_source::PLL0) / ((clock_get_threshold(threshold::I2C1) + 1) * 2),
        clock::I2C2 => clock_source_get_freq(clock_source::PLL0) / ((clock_get_threshold(threshold::I2C2) + 1) * 2),
        clock::SPI3 => {
            let source = match clock_get_clock_select(clock_select::SPI3) {
                0 => clock_source_get_freq(clock_source::IN0),
                1 => clock_source_get_freq(clock_source::PLL0),
                _ => panic!("unimplemented clock source"),
            };
            source / ((clock_get_threshold(threshold::SPI3) + 1) * 2)
        }
        clock::TIMER0 => {
            let source = match clock_get_clock_select(clock_select::TIMER0) {
                0 => clock_source_get_freq(clock_source::IN0),
                1 => clock_source_get_freq(clock_source::PLL0),
                _ => panic!("unimplemented clock source"),
            };
            source / ((clock_get_threshold(threshold::TIMER0) + 1) * 2)
        }
        clock::TIMER1 => {
            let source = match clock_get_clock_select(clock_select::TIMER1) {
                0 => clock_source_get_freq(clock_source::IN0),
                1 => clock_source_get_freq(clock_source::PLL0),
                _ => panic!("unimplemented clock source"),
            };
            source / ((clock_get_threshold(threshold::TIMER1) + 1) * 2)
        }
        clock::TIMER2 => {
            let source = match clock_get_clock_select(clock_select::TIMER2) {
                0 => clock_source_get_freq(clock_source::IN0),
                1 => clock_source_get_freq(clock_source::PLL0),
                _ => panic!("unimplemented clock source"),
            };
            source / ((clock_get_threshold(threshold::TIMER2) + 1) * 2)
        }
        clock::RTC => clock_source_get_freq(clock_source::IN0),
    }
}

fn reset_ctl(reset: reset, rst_value: bool) {
    unsafe {
        let ptr = pac::SYSCTL::ptr();
        match reset {
            reset::SOC => (*ptr).soft_reset.modify(|_, w| w.soft_reset().bit(rst_value)),
            reset::ROM => (*ptr).peri_reset.modify(|_, w| w.rom_reset().bit(rst_value)),
            reset::DMA => (*ptr).peri_reset.modify(|_, w| w.dma_reset().bit(rst_value)),
            reset::AI => (*ptr).peri_reset.modify(|_, w| w.ai_reset().bit(rst_value)),
            reset::DVP => (*ptr).peri_reset.modify(|_, w| w.dvp_reset().bit(rst_value)),
            reset::FFT => (*ptr).peri_reset.modify(|_, w| w.fft_reset().bit(rst_value)),
            reset::GPIO => (*ptr).peri_reset.modify(|_, w| w.gpio_reset().bit(rst_value)),
            reset::SPI0 => (*ptr).peri_reset.modify(|_, w| w.spi0_reset().bit(rst_value)),
            reset::SPI1 => (*ptr).peri_reset.modify(|_, w| w.spi1_reset().bit(rst_value)),
            reset::SPI2 => (*ptr).peri_reset.modify(|_, w| w.spi2_reset().bit(rst_value)),
            reset::SPI3 => (*ptr).peri_reset.modify(|_, w| w.spi3_reset().bit(rst_value)),
            reset::I2S0 => (*ptr).peri_reset.modify(|_, w| w.i2s0_reset().bit(rst_value)),
            reset::I2S1 => (*ptr).peri_reset.modify(|_, w| w.i2s1_reset().bit(rst_value)),
            reset::I2S2 => (*ptr).peri_reset.modify(|_, w| w.i2s2_reset().bit(rst_value)),
            reset::I2C0 => (*ptr).peri_reset.modify(|_, w| w.i2c0_reset().bit(rst_value)),
            reset::I2C1 => (*ptr).peri_reset.modify(|_, w| w.i2c1_reset().bit(rst_value)),
            reset::I2C2 => (*ptr).peri_reset.modify(|_, w| w.i2c2_reset().bit(rst_value)),
            reset::UART1 => (*ptr).peri_reset.modify(|_, w| w.uart1_reset().bit(rst_value)),
            reset::UART2 => (*ptr).peri_reset.modify(|_, w| w.uart2_reset().bit(rst_value)),
            reset::UART3 => (*ptr).peri_reset.modify(|_, w| w.uart3_reset().bit(rst_value)),
            reset::AES => (*ptr).peri_reset.modify(|_, w| w.aes_reset().bit(rst_value)),
            reset::FPIOA => (*ptr).peri_reset.modify(|_, w| w.fpioa_reset().bit(rst_value)),
            reset::TIMER0 => (*ptr).peri_reset.modify(|_, w| w.timer0_reset().bit(rst_value)),
            reset::TIMER1 => (*ptr).peri_reset.modify(|_, w| w.timer1_reset().bit(rst_value)),
            reset::TIMER2 => (*ptr).peri_reset.modify(|_, w| w.timer2_reset().bit(rst_value)),
            reset::WDT0 => (*ptr).peri_reset.modify(|_, w| w.wdt0_reset().bit(rst_value)),
            reset::WDT1 => (*ptr).peri_reset.modify(|_, w| w.wdt1_reset().bit(rst_value)),
            reset::SHA => (*ptr).peri_reset.modify(|_, w| w.sha_reset().bit(rst_value)),
            reset::RTC => (*ptr).peri_reset.modify(|_, w| w.rtc_reset().bit(rst_value)),
        }
    }
}

pub fn reset(reset: reset) {
    reset_ctl(reset, true);
    usleep(10);
    reset_ctl(reset, false);
}

/** Select DMA handshake for a channel */
pub fn dma_select(channel: dma_channel, select: dma_select)
{
    unsafe {
        use dma_channel::*;
        let ptr = pac::SYSCTL::ptr();
        match channel {
            CHANNEL0 => (*ptr).dma_sel0.modify(|_,w| w.dma_sel0().variant(select)),
            CHANNEL1 => (*ptr).dma_sel0.modify(|_,w| w.dma_sel1().variant(select)),
            CHANNEL2 => (*ptr).dma_sel0.modify(|_,w| w.dma_sel2().variant(select)),
            CHANNEL3 => (*ptr).dma_sel0.modify(|_,w| w.dma_sel3().variant(select)),
            CHANNEL4 => (*ptr).dma_sel0.modify(|_,w| w.dma_sel4().variant(select)),
            CHANNEL5 => (*ptr).dma_sel1.modify(|_,w| w.dma_sel5().variant(select)),
        }
    }
}

/** Return whether the selected PLL has achieved lock */
fn pll_is_lock(pll: pll) -> bool {
    let ptr = pac::SYSCTL::ptr();
    let pll_lock = unsafe { (*ptr).pll_lock.read() };
    match pll {
        pll::PLL0 => pll_lock.pll_lock0().bits() == 3,
        pll::PLL1 => (pll_lock.pll_lock1().bits() & 1) == 1,
        pll::PLL2 => (pll_lock.pll_lock2().bits() & 1) == 1,
    }
}

/** Clear PLL slip, this is done repeatedly until lock is achieved */
fn pll_clear_slip(pll: pll) -> bool {
    let ptr = pac::SYSCTL::ptr();
    unsafe {
        (*ptr).pll_lock.modify(|_,w|
            match pll {
                pll::PLL0 => w.pll_slip_clear0().set_bit(),
                pll::PLL1 => w.pll_slip_clear1().set_bit(),
                pll::PLL2 => w.pll_slip_clear2().set_bit(),
            }
        );
    }
    pll_is_lock(pll)
}

fn pll_source_set_freq(pll: pll, source: clock_source, freq: u32) -> Result<u32,()> {
    use pll::*;
    /* PLL0 and 1 can only source from IN0 */
    if (pll == PLL0 || pll == PLL1) && source != clock_source::IN0 {
        return Err(());
    }
    let freq_in = clock_source_get_freq(source);
    if freq_in == 0 {
        return Err(());
    }
    if let Some(found) = pll_compute::compute_params(freq_in, freq) {
        let ptr = pac::SYSCTL::ptr();
        unsafe {
            match pll {
                PLL0 => {
                    (*ptr).pll0.modify(|_,w|
                        w.clkr().bits(found.clkr)
                            .clkf().bits(found.clkf)
                            .clkod().bits(found.clkod)
                            .bwadj().bits(found.bwadj)
                    );
                }
                PLL1 => {
                    (*ptr).pll1.modify(|_,w|
                        w.clkr().bits(found.clkr)
                            .clkf().bits(found.clkf)
                            .clkod().bits(found.clkod)
                            .bwadj().bits(found.bwadj)
                    );
                }
                PLL2 => {
                    (*ptr).pll2.modify(|_,w|
                        w.ckin_sel().bits(pll2_source_to_cksel(source))
                            .clkr().bits(found.clkr)
                            .clkf().bits(found.clkf)
                            .clkod().bits(found.clkod)
                            .bwadj().bits(found.bwadj)
                    );
                }
            }
        }
        Ok(pll_get_freq(pll))
    } else {
        Err(())
    }
}

/**
 * @brief       Init PLL freqency
 * @param[in]   pll            The PLL id
 * @param[in]   pll_freq       The desired frequency in Hz

 */
pub fn pll_set_freq(pll: pll, freq: u32) -> Result<u32,()> {
    let ptr = pac::SYSCTL::ptr();
    use pll::*;

    /* 1. Change CPU CLK to XTAL */
    if pll == PLL0 {
        clock_set_clock_select(clock_select::ACLK, 0 /* clock_source::IN0 */);
    }

    /* 2. Disable PLL output */
    unsafe {
        match pll {
            PLL0 => (*ptr).pll0.modify(|_,w| w.out_en().clear_bit()),
            PLL1 => (*ptr).pll1.modify(|_,w| w.out_en().clear_bit()),
            PLL2 => (*ptr).pll2.modify(|_,w| w.out_en().clear_bit()),
        };
    }
    /* 3. Turn off PLL */
    unsafe {
        match pll {
            PLL0 => (*ptr).pll0.modify(|_,w| w.pwrd().clear_bit()),
            PLL1 => (*ptr).pll1.modify(|_,w| w.pwrd().clear_bit()),
            PLL2 => (*ptr).pll2.modify(|_,w| w.pwrd().clear_bit()),
        };
    }
    /* 4. Set PLL to new value */
    let result = if pll == PLL2 {
        pll_source_set_freq(pll, pll2_cksel_to_source(clock_get_clock_select(clock_select::PLL2)), freq)
    } else {
        pll_source_set_freq(pll, clock_source::IN0, freq)
    };

    /* 5. Power on PLL */
    unsafe {
        match pll {
            PLL0 => (*ptr).pll0.modify(|_,w| w.pwrd().set_bit()),
            PLL1 => (*ptr).pll1.modify(|_,w| w.pwrd().set_bit()),
            PLL2 => (*ptr).pll2.modify(|_,w| w.pwrd().set_bit()),
        };
    }


    /* wait >100ns */
    usleep(1);

    /* 6. Reset PLL then Release Reset*/
    unsafe {
        match pll {
            PLL0 => (*ptr).pll0.modify(|_,w| w.reset().clear_bit()),
            PLL1 => (*ptr).pll1.modify(|_,w| w.reset().clear_bit()),
            PLL2 => (*ptr).pll2.modify(|_,w| w.reset().clear_bit()),
        };
        match pll {
            PLL0 => (*ptr).pll0.modify(|_,w| w.reset().set_bit()),
            PLL1 => (*ptr).pll1.modify(|_,w| w.reset().set_bit()),
            PLL2 => (*ptr).pll2.modify(|_,w| w.reset().set_bit()),
        };
    }
    /* wait >100ns */
    usleep(1);
    unsafe {
        match pll {
            PLL0 => (*ptr).pll0.modify(|_,w| w.reset().clear_bit()),
            PLL1 => (*ptr).pll1.modify(|_,w| w.reset().clear_bit()),
            PLL2 => (*ptr).pll2.modify(|_,w| w.reset().clear_bit()),
        };
    }

    /* 7. Get lock status, wait PLL stable */
    while !pll_is_lock(pll) {
        pll_clear_slip(pll);
    }

    /* 8. Enable PLL output */
    unsafe {
        match pll {
            PLL0 => (*ptr).pll0.modify(|_,w| w.out_en().set_bit()),
            PLL1 => (*ptr).pll1.modify(|_,w| w.out_en().set_bit()),
            PLL2 => (*ptr).pll2.modify(|_,w| w.out_en().set_bit()),
        };
    }

    /* 9. Change CPU CLK to PLL */
    if pll == PLL0 {
        clock_set_clock_select(clock_select::ACLK, 1 /*clock_source::PLL0*/);
    }
    result
}
