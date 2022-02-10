#![allow(unused)]

//! GPIOHS peripheral
//use k210_hal::pac;
use k210_pac as pac;

use super::gpio;
use super::utils::{set_bit,get_bit};

// TODO embedded-hal::digital::v2::{InputPin, OutputPin}

/** Set input/output direction for a GPIOHS pin */
pub fn set_direction(pin: u8, direction: gpio::direction) {
    unsafe {
        let ptr = pac::GPIOHS::ptr();
        (*ptr)
            .output_en
            .modify(|r, w| w.bits(set_bit(r.bits(), pin, direction == gpio::direction::OUTPUT)));
        (*ptr)
            .input_en
            .modify(|r, w| w.bits(set_bit(r.bits(), pin, direction == gpio::direction::INPUT)));
    }
}

/** Set output value for a GPIOHS pin */
pub fn set_pin(pin: u8, value: bool) {
    unsafe {
        let ptr = pac::GPIOHS::ptr();
        (*ptr)
            .output_val
            .modify(|r, w| w.bits(set_bit(r.bits(), pin, value)));
    }
}

/** Get input value for a GPIOHS pin */
pub fn get_pin(pin: u8) -> bool {
    unsafe {
        let ptr = pac::GPIOHS::ptr();
        get_bit((*ptr).input_val.read().bits(), pin)
    }
}