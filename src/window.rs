use std::{thread::sleep, time::Duration};

use anyhow::Result;
use minifb::{Key, KeyRepeat};

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;

const SCALE: usize = 8;
const WINDOW_WIDTH: usize = DISPLAY_WIDTH * SCALE;
const WINDOW_HEIGHT: usize = DISPLAY_HEIGHT * SCALE;

const FPS: u64 = 60;
const MSPF: u64 = 1000 / FPS;

#[derive(Debug)]
pub struct Window {
    win: minifb::Window,
    buffer: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    framebuffer: [u32; WINDOW_WIDTH * WINDOW_HEIGHT],
}

impl Window {
    pub fn new() -> Result<Window> {
        Ok(Window {
            win: minifb::Window::new(
                "CHIP-8",
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
                minifb::WindowOptions::default(),
            )?,
            buffer: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            framebuffer: [0; WINDOW_WIDTH * WINDOW_HEIGHT],
        })
    }

    pub fn clear(&mut self) {
        self.buffer = [false; DISPLAY_WIDTH * DISPLAY_HEIGHT];
    }

    pub fn is_key_down(&self, code: u8) -> bool {
        self.win.is_key_down(code_to_key(code))
    }

    pub fn get_pressed_key(&self) -> Option<u8> {
        self.win
            .get_keys_pressed(KeyRepeat::No)
            .first()
            .map(key_to_code)
    }

    pub fn xor_pixel(&mut self, x: usize, y: usize) -> bool {
        let index = y * DISPLAY_WIDTH + x;
        let was_set = self.buffer[index];
        self.buffer[index] ^= true;
        was_set
    }

    pub fn should_close(&self) -> bool {
        !self.win.is_open() || self.win.is_key_down(Key::Escape)
    }

    pub fn update(&mut self) -> Result<()> {
        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                let is_set = self.buffer[y * DISPLAY_WIDTH + x];
                self.draw_rectangle(x * SCALE, y * SCALE, SCALE, SCALE, is_set);
            }
        }

        self.win
            .update_with_buffer(&self.framebuffer, WINDOW_WIDTH, WINDOW_HEIGHT)?;

        sleep(Duration::from_millis(MSPF));

        Ok(())
    }

    fn draw_rectangle(&mut self, x: usize, y: usize, width: usize, height: usize, is_set: bool) {
        for py in y..y + height {
            for px in x..x + width {
                let index = py * WINDOW_WIDTH + px;
                self.framebuffer[index] = if is_set { 0xFFFFFFFF } else { 0 };
            }
        }
    }
}

fn code_to_key(code: u8) -> Key {
    match code {
        0x1 => Key::Key1,
        0x2 => Key::Key2,
        0x3 => Key::Key3,
        0x4 => Key::Key4,
        0x5 => Key::Key5,
        0x6 => Key::Key6,
        0x7 => Key::Key7,
        0x8 => Key::Key8,
        0x9 => Key::Key9,
        0xA => Key::A,
        0xB => Key::B,
        0xC => Key::C,
        0xD => Key::D,
        0xE => Key::E,
        0xF => Key::F,
        _ => Key::Key0,
    }
}

fn key_to_code(key: &Key) -> u8 {
    match key {
        Key::Key1 => 0x1,
        Key::Key2 => 0x2,
        Key::Key3 => 0x3,
        Key::Key4 => 0x4,
        Key::Key5 => 0x5,
        Key::Key6 => 0x6,
        Key::Key7 => 0x7,
        Key::Key8 => 0x8,
        Key::Key9 => 0x9,
        Key::A => 0xA,
        Key::B => 0xB,
        Key::C => 0xC,
        Key::D => 0xD,
        Key::E => 0xE,
        Key::F => 0xF,
        _ => 0x0,
    }
}
