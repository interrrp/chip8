use std::ops::Range;

use anyhow::{anyhow, Result};

pub const MEMORY_SIZE: usize = 0xFFF;
pub const MEMORY_PROGRAM_START: usize = 0x201;
pub const MEMORY_PROGRAM_SIZE: usize = MEMORY_SIZE - MEMORY_PROGRAM_START;

const FONTSET_SIZE: usize = 0x50;
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

/// The memory (RAM).
///
/// > The Chip-8 language is capable of accessing up to 4KB (4,096 bytes) of
/// > RAM, from location 0x000 (0) to 0xFFF (4095). The first 512 bytes, from
/// > 0x000 to 0x1FF, are where the original interpreter was located, and should
/// > not be used by programs.
/// >
/// > Most Chip-8 programs start at location 0x200 (512), but some begin at
/// > 0x600 (1536). Programs beginning at 0x600 are intended for the ETI 660
/// > computer.
/// >
/// > [_Cowgod's CHIP-8 Technical Reference, section
/// > 2.1_](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.1)
///
/// Though unspecified in the technical reference, the fontset starts at memory address `0`.
#[derive(Debug, Clone, Copy)]
pub struct Memory {
    memory: [u8; MEMORY_SIZE],

    /// The length of the program loaded in memory.
    ///
    /// If no program has been loaded, this is 0.
    pub program_len: usize,
}

impl Memory {
    /// Return an empty memory.
    ///
    /// This fills up all the data with zeroes (except for the fontset, starting from `0x0` to
    /// `0x50`), and sets `program_len` to 0.
    pub fn new() -> Memory {
        let mut memory = Memory {
            memory: [0; MEMORY_SIZE],
            program_len: 0,
        };

        memory.memory[0..FONTSET_SIZE].copy_from_slice(&FONTSET);

        memory
    }

    /// Try to get the value at `index`.
    pub fn at(&self, index: usize) -> u8 {
        self.memory[index]
    }

    /// Load a program into memory.
    ///
    /// The program will start at `MEMORY_PROGRAM_START`.
    pub fn load_program(&mut self, program: &[u8]) -> Result<()> {
        if program.len() > MEMORY_PROGRAM_SIZE {
            return Err(anyhow!(
                "Attempted to load program exceeding memory limit ({} bytes)",
                program.len()
            ));
        }

        let area = get_program_area(program);
        self.program_len = program.len();

        self.memory[area].copy_from_slice(program);

        Ok(())
    }
}

/// Return the area of a program in memory.
///
/// A program's area starts at the first unrestricted/"program" region.
fn get_program_area(program: &[u8]) -> Range<usize> {
    let start = MEMORY_PROGRAM_START;
    let end = MEMORY_PROGRAM_START + program.len();
    start..end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() -> Result<()> {
        let memory = Memory::new();

        // Ensure every value in the program region is zero
        for i in MEMORY_PROGRAM_START..MEMORY_SIZE {
            assert_eq!(memory.memory[i], 0);
        }

        // Ensure fontset is loaded correctly
        for i in 0..FONTSET_SIZE {
            assert_eq!(memory.memory[i], FONTSET[i]);
        }

        Ok(())
    }

    #[test]
    fn load_program() -> Result<()> {
        let mut memory = Memory::new();
        memory.load_program(&[0x00, 0xE0, 0x00, 0xEE])?;

        // Ensure the program was correctly loaded into memory
        assert_eq!(memory.at(MEMORY_PROGRAM_START), 0x00);
        assert_eq!(memory.at(MEMORY_PROGRAM_START + 1), 0xE0);
        assert_eq!(memory.at(MEMORY_PROGRAM_START + 2), 0x00);
        assert_eq!(memory.at(MEMORY_PROGRAM_START + 3), 0xEE);
        assert_eq!(memory.program_len, 4);

        Ok(())
    }

    #[test]
    fn program_too_big_error() {
        let mut memory = Memory::new();
        // Ensure loading a program that's too big returns an error
        assert!(memory.load_program(&[0; MEMORY_SIZE + 1]).is_err());
    }
}
