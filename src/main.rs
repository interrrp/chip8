use std::{
    fs,
    ops::{Index, IndexMut},
    path::{Path, PathBuf},
    thread::sleep,
    time::Duration,
};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use minifb::{Key, KeyRepeat};

/// A tiny CHIP-8 emulator.
#[derive(Parser, Debug)]
struct Args {
    /// Path to the program to execute.
    ///
    /// This typically ends in `.ch8` or `.rom`.
    program_path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut emulator = Emulator::new()?;
    emulator.load_program_file(&args.program_path)?;
    emulator.run()?;

    Ok(())
}

// As recommended by Gulrak, the CHIP-8 god
const CYCLES_PER_FRAME: usize = 11;

type Stack = Vec<usize>;

struct Emulator {
    memory: Memory,
    registers: Registers,
    call_stack: Stack,
    window: Window,
    program_counter: usize,
    delay_timer: u8,
    sound_timer: u8,
}

impl Emulator {
    pub fn new() -> Result<Emulator> {
        Ok(Emulator {
            memory: Memory::new(),
            registers: Registers::new(),
            call_stack: Stack::new(),
            window: Window::new()?,
            program_counter: MEMORY_PROGRAM_START,
            delay_timer: 0,
            sound_timer: 0,
        })
    }

    pub fn load_program_file(&mut self, path: &Path) -> Result<()> {
        let bytes =
            fs::read(path).context(format!("Failed to read program at {}", path.display()))?;

        self.memory.load_program_bytes(&bytes);

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        while !self.window.should_close() {
            for _ in 0..CYCLES_PER_FRAME {
                self.do_cycle()?;
            }
            self.window.update()?;
        }

        Ok(())
    }

    fn do_cycle(&mut self) -> Result<()> {
        if self.update_timers() {
            return Ok(());
        }
        self.do_instruction()?;
        Ok(())
    }

    fn update_timers(&mut self) -> bool {
        if self.delay_timer == 0 && self.sound_timer == 0 {
            return false;
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
        true
    }

    fn do_instruction(&mut self) -> Result<()> {
        let Instruction {
            opcode,
            op,
            x,
            y,
            n,
            nn,
            nnn,
        } = self.fetch_instruction();

        self.program_counter += 2;

        match op {
            0x0 if opcode == 0x00E0 => self.clear_screen(),
            0x0 if opcode == 0x00EE => self.return_from_subroutine()?,

            0x1 => self.jump(nnn),
            0x2 => self.call_subroutine(nnn),

            0x3 => self.skip_if_vx_equal_nn(x, nn),
            0x4 => self.skip_if_vx_not_equal_nn(x, nn),
            0x5 => self.skip_if_vx_equal_vy(x, y),

            0x6 => self.store_nn_in_vx(x, nn),
            0x7 => self.add_nn_to_vx(x, nn),

            0x8 if n == 0x0 => self.store_vy_in_vx(x, y),
            0x8 if n == 0x1 => self.set_vx_to_vx_or_vy(x, y),
            0x8 if n == 0x2 => self.set_vx_to_vx_and_vy(x, y),
            0x8 if n == 0x3 => self.set_vx_to_vx_xor_vy(x, y),
            0x8 if n == 0x4 => self.add_vy_to_vx(x, y),
            0x8 if n == 0x5 => self.subtract_vy_from_vx(x, y),
            0x8 if n == 0x6 => self.set_vx_to_vy_shr_1(x, y),
            0x8 if n == 0x7 => self.set_vx_to_vy_minus_vx(x, y),
            0x8 if n == 0xE => self.set_vx_to_vy_shl_1(x, y),

            0x9 if n == 0x0 => self.skip_if_vx_not_equal_vy(x, y),

            0xA => self.store_nnn_in_i(nnn),
            0xB => self.jump_to_nnn_plus_v0(nnn),
            0xC => self.set_vx_to_random_with_mask_nn(x, nn),
            0xD => self.draw_sprite(x, y, n),

            0xE if nn == 0x9E => self.skip_if_key_in_vx_down(x),
            0xE if nn == 0xA1 => self.skip_if_key_in_vx_not_down(x),

            0xF if nn == 0x07 => self.store_delay_timer_in_vx(x),
            0xF if nn == 0x0A => self.wait_for_key_and_store_in_vx(x),
            0xF if nn == 0x15 => self.set_delay_timer_to_vx(x),
            0xF if nn == 0x18 => self.set_sound_timer_to_vx(x),
            0xF if nn == 0x1E => self.add_vx_to_i(x),
            0xF if nn == 0x29 => self.set_i_to_addr_of_sprite_at_vx(x),
            0xF if nn == 0x33 => self.store_binary_coded_decimal_of_vx_at_i(x),
            0xF if nn == 0x55 => self.store_v0_to_vx_in_memory_at_i(x),
            0xF if nn == 0x65 => self.load_v0_to_vx_from_memory_at_i(x),

            _ => return Err(anyhow!("Unknown instruction: {opcode:X}")),
        }

        Ok(())
    }

    fn clear_screen(&mut self) {
        self.window.clear();
    }

    fn return_from_subroutine(&mut self) -> Result<()> {
        match self.call_stack.pop() {
            Some(addr) => self.program_counter = addr,
            None => return Err(anyhow!("RET outside subroutine")),
        }
        Ok(())
    }

    fn jump(&mut self, nnn: usize) {
        self.program_counter = nnn;
    }

    fn call_subroutine(&mut self, nnn: usize) {
        self.call_stack.push(self.program_counter);
        self.program_counter = nnn;
    }

    fn skip_if_vx_equal_nn(&mut self, x: usize, nn: u8) {
        if self.registers[x] == nn {
            self.program_counter += 2;
        }
    }

    fn skip_if_vx_not_equal_nn(&mut self, x: usize, nn: u8) {
        if self.registers[x] != nn {
            self.program_counter += 2;
        }
    }

    fn skip_if_vx_equal_vy(&mut self, x: usize, y: usize) {
        if self.registers[x] == self.registers[y] {
            self.program_counter += 2;
        }
    }

    fn store_nn_in_vx(&mut self, x: usize, nn: u8) {
        self.registers[x] = nn;
    }

    fn add_nn_to_vx(&mut self, x: usize, nn: u8) {
        self.registers[x] = self.registers[x].wrapping_add(nn);
    }

    fn store_vy_in_vx(&mut self, x: usize, y: usize) {
        self.registers[x] = self.registers[y];
    }

    fn set_vx_to_vx_or_vy(&mut self, x: usize, y: usize) {
        self.registers[x] |= self.registers[y];
    }

    fn set_vx_to_vx_and_vy(&mut self, x: usize, y: usize) {
        self.registers[x] &= self.registers[y];
    }

    fn set_vx_to_vx_xor_vy(&mut self, x: usize, y: usize) {
        self.registers[x] ^= self.registers[y];
    }

    fn add_vy_to_vx(&mut self, x: usize, y: usize) {
        let (result, overflow) = self.registers[x].overflowing_add(self.registers[y]);
        self.registers[x] = result;
        self.registers[0xF] = if overflow { 1 } else { 0 };
    }

    fn subtract_vy_from_vx(&mut self, x: usize, y: usize) {
        let (result, overflow) = self.registers[x].overflowing_sub(self.registers[y]);
        self.registers[x] = result;
        self.registers[0xF] = if !overflow { 1 } else { 0 };
    }

    fn set_vx_to_vy_shr_1(&mut self, x: usize, y: usize) {
        let vf = self.registers[y] & 1;
        self.registers[x] = self.registers[y] >> 1;
        self.registers[0xF] = vf;
    }

    fn set_vx_to_vy_minus_vx(&mut self, x: usize, y: usize) {
        let (result, overflow) = self.registers[y].overflowing_sub(self.registers[x]);
        self.registers[x] = result;
        self.registers[0xF] = if !overflow { 1 } else { 0 };
    }

    fn set_vx_to_vy_shl_1(&mut self, x: usize, y: usize) {
        let vf = (self.registers[y] & 0x80) >> 7;
        self.registers[x] = self.registers[y] << 1;
        self.registers[0xF] = vf;
    }

    fn skip_if_vx_not_equal_vy(&mut self, x: usize, y: usize) {
        if self.registers[x] != self.registers[y] {
            self.program_counter += 2;
        }
    }

    fn store_nnn_in_i(&mut self, nnn: usize) {
        self.registers.i = nnn;
    }

    fn jump_to_nnn_plus_v0(&mut self, nnn: usize) {
        self.program_counter = nnn + self.registers[0] as usize;
    }

    fn set_vx_to_random_with_mask_nn(&mut self, x: usize, nn: u8) {
        self.registers[x] = fastrand::u8(0..=u8::MAX) & nn;
    }

    fn draw_sprite(&mut self, x: usize, y: usize, n: u8) {
        let vx = self.registers[x] as usize;
        let vy = self.registers[y] as usize;
        let i = self.registers.i;

        for row in 0..n as usize {
            let y_pos = (vy + row) % DISPLAY_HEIGHT;
            let sprite_byte = self.memory[i + row];

            for col in 0..8 {
                let x_pos = (vx + col) % DISPLAY_WIDTH;
                let bit = (sprite_byte >> (7 - col)) & 1;

                if bit == 1 && self.window.xor_pixel(x_pos, y_pos) {
                    self.registers[0xF] = 1;
                }
            }
        }
    }

    fn skip_if_key_in_vx_down(&mut self, x: usize) {
        if self.window.is_key_down(self.registers[x]) {
            self.program_counter += 2;
        }
    }

    fn skip_if_key_in_vx_not_down(&mut self, x: usize) {
        if !self.window.is_key_down(self.registers[x]) {
            self.program_counter += 2;
        }
    }

    fn store_delay_timer_in_vx(&mut self, x: usize) {
        self.registers[x] = self.delay_timer;
    }

    fn wait_for_key_and_store_in_vx(&mut self, x: usize) {
        if let Some(key) = self.window.get_pressed_key() {
            self.registers[x] = key;
        } else {
            self.program_counter -= 2;
        }
    }

    fn set_delay_timer_to_vx(&mut self, x: usize) {
        self.delay_timer = self.registers[x];
    }

    fn set_sound_timer_to_vx(&mut self, x: usize) {
        self.sound_timer = self.registers[x];
    }

    fn add_vx_to_i(&mut self, x: usize) {
        self.registers.i += self.registers[x] as usize;
    }

    fn set_i_to_addr_of_sprite_at_vx(&mut self, x: usize) {
        self.registers.i = (self.registers[x] as usize) * 5;
    }

    fn store_binary_coded_decimal_of_vx_at_i(&mut self, x: usize) {
        let value = self.registers[x];
        let i = self.registers.i;
        self.memory[i] = value / 100;
        self.memory[i + 1] = (value % 100) / 10;
        self.memory[i + 2] = value % 10;
    }

    fn store_v0_to_vx_in_memory_at_i(&mut self, x: usize) {
        let i = self.registers.i;
        for offset in 0..=x {
            self.memory[i + offset] = self.registers[offset];
        }
    }

    fn load_v0_to_vx_from_memory_at_i(&mut self, x: usize) {
        let i = self.registers.i;
        for offset in 0..=x {
            self.registers[offset] = self.memory[i + offset];
        }
    }

    fn fetch_instruction(&self) -> Instruction {
        Instruction::from_opcode(u16::from_be_bytes([
            self.memory[self.program_counter],
            self.memory[self.program_counter + 1],
        ]))
    }
}

struct Instruction {
    pub opcode: u16,
    pub op: u8,
    pub x: usize,
    pub y: usize,
    pub n: u8,
    pub nn: u8,
    pub nnn: usize,
}

impl Instruction {
    pub fn from_opcode(opcode: u16) -> Instruction {
        Instruction {
            opcode,
            op: ((opcode & 0xF000) >> 12) as u8,
            x: ((opcode & 0x0F00) >> 8) as usize,
            y: ((opcode & 0x00F0) >> 4) as usize,
            n: (opcode & 0x000F) as u8,
            nn: (opcode & 0x00FF) as u8,
            nnn: (opcode & 0x0FFF) as usize,
        }
    }
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

struct Memory {
    buf: [u8; MEMORY_SIZE],
}

impl Memory {
    pub fn new() -> Memory {
        let mut mem = Memory {
            buf: [0; MEMORY_SIZE],
        };

        mem.buf[0..FONTSET_SIZE].copy_from_slice(&FONTSET);

        mem
    }

    pub fn load_program_bytes(&mut self, program_bytes: &[u8]) {
        self.buf[MEMORY_PROGRAM_START..MEMORY_PROGRAM_START + program_bytes.len()]
            .copy_from_slice(program_bytes);
    }
}

impl Index<usize> for Memory {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        &self.buf[index]
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        &mut self.buf[index]
    }
}

struct Registers {
    data: [u8; 16],
    i: usize,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            data: [0; 16],
            i: 0,
        }
    }
}

impl Index<usize> for Registers {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        &self.data[index]
    }
}

impl IndexMut<usize> for Registers {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        &mut self.data[index]
    }
}

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const WINDOW_WIDTH: usize = DISPLAY_WIDTH * SCALE;
const WINDOW_HEIGHT: usize = DISPLAY_HEIGHT * SCALE;
const SCALE: usize = 8;
const FPS: u64 = 60;
const MSPF: u64 = 1000 / FPS;

struct Window {
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
