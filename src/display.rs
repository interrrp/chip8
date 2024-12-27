// I hate how Raylib requires `i32`s for everything
#![allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]

use raylib::{color::Color, prelude::RaylibDraw};

const SCALE: usize = 8;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;

pub const WINDOW_WIDTH: i32 = (DISPLAY_WIDTH * SCALE) as i32;
pub const WINDOW_HEIGHT: i32 = (DISPLAY_HEIGHT * SCALE) as i32;

type DisplayBuffer = [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
const EMPTY_DISPLAY_BUFFER: DisplayBuffer = [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT];

pub struct Display {
    buf: DisplayBuffer,
}

impl Display {
    pub fn new() -> Display {
        Display {
            buf: EMPTY_DISPLAY_BUFFER,
        }
    }

    pub fn clear(&mut self) {
        self.buf = EMPTY_DISPLAY_BUFFER;
    }

    pub fn xor_pixel(&mut self, x: usize, y: usize) -> bool {
        let was_set = self.buf[y][x];
        self.buf[y][x] ^= true;
        was_set
    }

    pub fn draw(&self, d: &mut impl RaylibDraw) {
        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                self.draw_pixel(d, x, y);
            }
        }
    }

    fn draw_pixel(&self, d: &mut impl RaylibDraw, x: usize, y: usize) {
        let is_set = self.buf[y][x];

        let color = if is_set { Color::WHITE } else { Color::BLACK };

        let x = (x * SCALE) as i32;
        let y = (y * SCALE) as i32;
        let width = SCALE as i32;
        let height = SCALE as i32;

        d.draw_rectangle(x, y, width, height, color);
    }
}
