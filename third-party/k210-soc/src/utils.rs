#![allow(unused)]

//! Miscelleneous utilities for SoC functions (private)

pub fn set_bit(inval: u32, bit: u8, state: bool) -> u32 {
    if state {
        inval | (1 << u32::from(bit))
    } else {
        inval & !(1 << u32::from(bit))
    }
}

pub fn get_bit(inval: u32, bit: u8) -> bool {
    (inval & (1 << u32::from(bit))) != 0
}