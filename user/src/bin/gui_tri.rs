#![no_std]
#![no_main]

extern crate user_lib;

use embedded_graphics::prelude::Size;
use user_lib::{Display, VIRTGPU_XRES, VIRTGPU_YRES};

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    let mut disp = Display::new(Size::new(VIRTGPU_XRES, VIRTGPU_YRES));
    disp.paint_on_framebuffer(|fb| {
        for y in 0..VIRTGPU_YRES as usize {
            for x in 0..VIRTGPU_XRES as usize {
                let idx = (y * VIRTGPU_XRES as usize + x) * 4;
                fb[idx] = x as u8;
                fb[idx + 1] = y as u8;
                fb[idx + 2] = (x + y) as u8;
            }
        }
    });
    disp.flush();
    0
}
