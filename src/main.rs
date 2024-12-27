#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use std::{fs, path::PathBuf, thread::sleep, time::Duration};

use clap::Parser;
use display::{Display, DISPLAY_HEIGHT, DISPLAY_WIDTH, WINDOW_HEIGHT, WINDOW_WIDTH};
use raylib::{ffi::KeyboardKey, RaylibHandle};

mod display;

/// A tiny CHIP-8 emulator.
#[derive(Parser, Debug)]
struct Args {
    /// Path to the program to emulate.
    ///
    /// This typically ends in `.ch8`.
    program_path: PathBuf,
}

fn main() {
    let args = Args::parse();

    let mut emu = Emulator::new();
    emu.load_program(&fs::read(&args.program_path).unwrap());
    emu.run();
}

const MEMORY_SIZE: usize = 0xFFF;
const MEMORY_PROGRAM_START: usize = 0x200;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

struct Emulator {
    memory: [u8; MEMORY_SIZE],
    program_len: usize,
    registers: [u8; 16],
    i: usize,
    stack: Vec<usize>,
    display: Display,
    pc: usize,
    dt: u8,
    st: u8,
}

impl Emulator {
    pub fn new() -> Emulator {
        let mut emu = Emulator {
            memory: [0; MEMORY_SIZE],
            program_len: 0,
            registers: [0; 16],
            i: 0,
            stack: Vec::new(),
            display: Display::new(),
            pc: MEMORY_PROGRAM_START,
            dt: 0,
            st: 0,
        };

        emu.memory[0..FONTSET_SIZE].copy_from_slice(&FONTSET);

        emu
    }

    pub fn load_program(&mut self, program: &[u8]) {
        self.program_len = program.len();

        self.memory[MEMORY_PROGRAM_START..MEMORY_PROGRAM_START + program.len()]
            .copy_from_slice(program);
    }

    pub fn run(&mut self) {
        let (mut rl, rl_thread) = raylib::init()
            .size(WINDOW_WIDTH, WINDOW_HEIGHT)
            .title("CHIP-8")
            .build();

        while self.should_run() {
            for _ in 0..11 {
                self.execute_cycle(&mut rl);
            }

            self.display.draw(&mut rl.begin_drawing(&rl_thread));

            sleep(Duration::from_millis(16));
        }
    }

    fn should_run(&self) -> bool {
        self.pc < MEMORY_PROGRAM_START + self.program_len
    }

    #[allow(clippy::too_many_lines)]
    fn execute_cycle(&mut self, rl: &mut RaylibHandle) {
        let opcode = self.fetch_opcode();

        let v = &mut self.registers;

        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n = opcode & 0x000F;
        let nn = (opcode & 0x00FF) as u8;
        let nnn = (opcode & 0x0FFF) as usize;

        if self.dt > 0 {
            self.dt -= 1;
            return;
        }

        match (opcode & 0xF000) >> 12 {
            0x0 if nn == 0x00 => {}
            0x0 if nn == 0xE0 => self.display.clear(),
            0x0 if nn == 0xEE => match self.stack.pop() {
                Some(addr) => self.pc = addr,
                None => println!("RET outside subroutine"),
            },
            0x1 => self.pc = nnn - 2,
            0x2 => {
                self.stack.push(self.pc);
                self.pc = nnn - 2;
            }
            0x3 if v[x] == nn => self.pc += 2,
            0x3 if v[x] != nn => {}
            0x4 if v[x] != nn => self.pc += 2,
            0x4 if v[x] == nn => {}
            0x5 if n == 0 && v[x] == v[y] => self.pc += 2,
            0x5 if n == 0 && v[x] != v[y] => {}
            0x6 => v[x] = nn,
            0x7 => v[x] = v[x].wrapping_add(nn),
            0x8 if n == 0x0 => v[x] = v[y],
            0x8 if n == 0x1 => v[x] |= v[y],
            0x8 if n == 0x2 => v[x] &= v[y],
            0x8 if n == 0x3 => v[x] ^= v[y],
            0x8 if n == 0x4 => {
                let (vx, carry) = v[x].overflowing_add(v[y]);
                v[x] = vx;
                v[0xF] = carry.into();
            }
            0x8 if n == 0x5 => {
                let vf = (v[x] >= v[y]).into();
                v[x] = v[x].wrapping_sub(v[y]);
                v[0xF] = vf;
            }
            0x8 if n == 0x6 => {
                let vf = v[y] & 1;
                v[y] >>= 1;
                v[0xF] = vf;
            }
            0x8 if n == 0x7 => {
                let vf = (v[y] >= v[x]).into();
                v[x] = v[y].wrapping_sub(v[x]);
                v[0xF] = vf;
            }
            0x8 if n == 0xE => {
                let vf = v[y] >> 7;
                v[y] <<= 1;
                v[0xF] = vf;
            }
            0x9 if n == 0x0 && v[x] != v[y] => self.pc += 2,
            0x9 if n == 0x0 && v[x] == v[y] => {}
            0xA => self.i = nnn,
            0xB => self.pc = (nnn + v[0] as usize) - 2,
            0xC => v[x] = fastrand::u8(0..u8::MAX) & nn,
            0xD => {
                v[0xF] = 0;

                let x = v[x] as usize;
                let y = v[y] as usize;

                for row in 0..n {
                    if (y + row as usize) >= DISPLAY_HEIGHT {
                        break;
                    }

                    let sprite_byte = self.memory[self.i + row as usize];
                    for bit in 0..8 {
                        if (x + bit) >= DISPLAY_WIDTH {
                            continue;
                        }

                        if (sprite_byte & (0x80 >> bit)) != 0
                            && self.display.xor_pixel(x + bit, y + row as usize)
                        {
                            v[0xF] = 1;
                        }
                    }
                }
            }
            0xE if nn == 0x9E && rl.is_key_down(code_to_key(v[x])) => self.pc += 2,
            0xE if nn == 0x9E && !rl.is_key_down(code_to_key(v[x])) => {}
            0xE if nn == 0xA1 && !rl.is_key_down(code_to_key(v[x])) => self.pc += 2,
            0xE if nn == 0xA1 && rl.is_key_down(code_to_key(v[x])) => {}
            0xF if nn == 0x0A => {
                if let Some(key) = rl.get_key_pressed() {
                    v[x] = key_to_code(key);
                    return;
                }
            }
            0xF if nn == 0x07 => v[x] = self.dt,
            0xF if nn == 0x15 => self.dt = v[x],
            0xF if nn == 0x18 => self.st = v[x],
            0xF if nn == 0x1E => self.i += v[x] as usize,
            0xF if nn == 0x33 => {
                let value = v[x];
                let hundreds = value / 100;
                let tens = (value / 10) % 10;
                let ones = value % 10;

                self.memory[self.i] = hundreds;
                self.memory[self.i + 1] = tens;
                self.memory[self.i + 2] = ones;
            }
            0xF if nn == 0x55 => self.memory[self.i..=self.i + x].copy_from_slice(&v[0..=x]),
            0xF if nn == 0x65 => v[0..=x].copy_from_slice(&self.memory[self.i..=self.i + x]),
            0xF if nn == 0x29 => self.i = v[x] as usize * 5,
            _ => println!("Unknown instruction: {opcode:#X}"),
        }

        self.pc += 2;
    }

    fn fetch_opcode(&mut self) -> u16 {
        // Combine next two bytes
        u16::from_be_bytes([self.memory[self.pc], self.memory[self.pc + 1]])
    }
}

fn code_to_key(code: u8) -> KeyboardKey {
    match code {
        0x1 => KeyboardKey::KEY_ONE,
        0x2 => KeyboardKey::KEY_TWO,
        0x3 => KeyboardKey::KEY_THREE,
        0x4 => KeyboardKey::KEY_FOUR,
        0x5 => KeyboardKey::KEY_FIVE,
        0x6 => KeyboardKey::KEY_SIX,
        0x7 => KeyboardKey::KEY_SEVEN,
        0x8 => KeyboardKey::KEY_EIGHT,
        0x9 => KeyboardKey::KEY_NINE,
        0xA => KeyboardKey::KEY_A,
        0xB => KeyboardKey::KEY_B,
        0xC => KeyboardKey::KEY_C,
        0xD => KeyboardKey::KEY_D,
        0xE => KeyboardKey::KEY_E,
        0xF => KeyboardKey::KEY_F,
        _ => KeyboardKey::KEY_ZERO,
    }
}

fn key_to_code(key: KeyboardKey) -> u8 {
    match key {
        KeyboardKey::KEY_ONE => 0x1,
        KeyboardKey::KEY_TWO => 0x2,
        KeyboardKey::KEY_THREE => 0x3,
        KeyboardKey::KEY_FOUR => 0x4,
        KeyboardKey::KEY_FIVE => 0x5,
        KeyboardKey::KEY_SIX => 0x6,
        KeyboardKey::KEY_SEVEN => 0x7,
        KeyboardKey::KEY_EIGHT => 0x8,
        KeyboardKey::KEY_NINE => 0x9,
        KeyboardKey::KEY_A => 0xA,
        KeyboardKey::KEY_B => 0xB,
        KeyboardKey::KEY_C => 0xC,
        KeyboardKey::KEY_D => 0xD,
        KeyboardKey::KEY_E => 0xE,
        KeyboardKey::KEY_F => 0xF,
        _ => 0x0,
    }
}
