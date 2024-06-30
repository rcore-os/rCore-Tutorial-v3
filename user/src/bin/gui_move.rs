#![no_std]
#![no_main]

extern crate user_lib;
extern crate alloc;

use user_lib::console::getchar;
use user_lib::{Display, VIRTGPU_XRES, VIRTGPU_YRES};

use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::{Drawable, Point, RgbColor, Size};
use embedded_graphics::primitives::Primitive;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::draw_target::DrawTarget;

const INIT_X: i32 = 640;
const INIT_Y: i32 = 400;
const RECT_SIZE: u32 = 40;

pub struct DrawingBoard {
    disp: Display,
    latest_pos: Point,
}

impl DrawingBoard {
    pub fn new() -> Self {
        Self {
            disp: Display::new(Size::new(VIRTGPU_XRES, VIRTGPU_YRES)),
            latest_pos: Point::new(INIT_X, INIT_Y),
        }
    }
    fn paint(&mut self) {
        Rectangle::with_center(self.latest_pos, Size::new(RECT_SIZE, RECT_SIZE))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 1))
            .draw(&mut self.disp)
            .ok();
    }
    fn unpaint(&mut self) {
        Rectangle::with_center(self.latest_pos, Size::new(RECT_SIZE, RECT_SIZE))
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::BLACK, 1))
            .draw(&mut self.disp)
            .ok();
    }
    pub fn move_rect(&mut self, dx: i32, dy: i32) {
        let new_x = self.latest_pos.x + dx;
        let new_y = self.latest_pos.y + dy;
        let r = (RECT_SIZE / 2) as i32;
        if new_x > r && new_x + r < (VIRTGPU_XRES as i32) && new_y > r && new_y + r < (VIRTGPU_YRES as i32) {
            self.unpaint();
            self.latest_pos.x = new_x;
            self.latest_pos.y = new_y;
            self.paint();
        }
    }
}

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
#[no_mangle]
pub fn main() -> i32 {
    let mut board = DrawingBoard::new();
    let _ = board.disp.clear(Rgb888::BLACK).unwrap();
    board.disp.flush();
    loop {
        let c = getchar();
        if c == LF || c == CR {
            break;
        }
        let mut moved = true;
        match c {
            b'w' => board.move_rect(0, -10),
            b'a' => board.move_rect(-10, 0),
            b's' => board.move_rect(0, 10),
            b'd' => board.move_rect(10, 0),
            _ => moved = false,
        }
        if moved {
            board.disp.flush();
        }
    }
    0
}
