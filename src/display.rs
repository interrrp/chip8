use std::ops::{Index, IndexMut};

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const DISPLAY_PIXELS: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

#[derive(Debug, Clone, Copy)]
pub struct Display {
    pixels: [bool; DISPLAY_PIXELS],
    pub width: usize,
    pub height: usize,
}

impl Display {
    pub fn new() -> Display {
        Display {
            pixels: [false; DISPLAY_PIXELS],
            width: 64,
            height: 32,
        }
    }

    pub fn xor_pixel(&mut self, x: usize, y: usize) -> bool {
        let was_set = self[(x, y)];
        self[(x, y)] ^= true;
        was_set
    }

    pub fn clear(&mut self) {
        self.pixels = [false; DISPLAY_PIXELS];
    }

    pub fn render(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel = self[(x, y)];
                print!("{}", if pixel { "█" } else { " " });
            }
            println!();
        }
    }
}

impl Index<(usize, usize)> for Display {
    type Output = bool;
    fn index(&self, index: (usize, usize)) -> &bool {
        &self.pixels[index.1 * DISPLAY_WIDTH + index.0]
    }
}
impl IndexMut<(usize, usize)> for Display {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut bool {
        &mut self.pixels[index.1 * DISPLAY_WIDTH + index.0]
    }
}
