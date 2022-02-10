//! Stdout
pub use core::fmt::Write;
use nb::block;

/// Stdout implements the core::fmt::Write trait for hal::serial::Write
/// implementations.
pub struct Stdout<'p, T>(pub &'p mut T);

impl<'p, T> Write for Stdout<'p, T>
    where
    T: embedded_hal::serial::Write<u8>,
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.as_bytes() {
            if *byte == b'\n' {
                let res = block!(self.0.try_write(b'\r'));

                if res.is_err() {
                    return Err(core::fmt::Error);
                }
            }

            let res = block!(self.0.try_write(*byte));

            if res.is_err() {
                return Err(core::fmt::Error);
            }
        }
        Ok(())
    }
}
