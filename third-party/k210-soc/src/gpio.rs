//! GPIO peripheral

#![allow(non_camel_case_types)]

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum direction {
    INPUT,
    OUTPUT,
}

// TODO