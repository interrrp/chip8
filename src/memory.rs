use std::ops::{Index, IndexMut};

use anyhow::bail;
use log::warn;

/// The start of the program's memory.
///
/// From _CHIP-8 Technical Reference (Matthew Mikolay)_:
///
/// > CHIP-8 programs should be loaded into memory starting at address `0x200`.
pub const MEMORY_PROGRAM_START: usize = 0x200;
const MEMORY_SIZE: usize = 0xFFF;
const MEMORY_PROGRAM_SIZE: usize = MEMORY_SIZE - MEMORY_PROGRAM_START;

/// System memory.
///
/// From _CHIP-8 Technical Reference (Matthew Mikolay)_:
///
/// > CHIP-8 programs should be loaded into memory starting at address `0x200`. The memory addresses
/// > `0x000` to `0x1FF` are reserved for the CHIP-8 interpreter.
/// >
/// > In addition, the final 352 bytes of memory are reserved for “variables and display refresh,”
/// > and should not be used in CHIP-8 programs.
#[derive(Debug, Clone, Copy)]
pub struct Memory {
    memory: [u8; MEMORY_SIZE],

    /// The size of the program loaded by [`Memory::load_program`].
    ///
    /// If no program has been loaded, this is 0.
    pub program_len: usize,
}

impl Memory {
    /// Return an instance of system memory with everything set to zero.
    pub fn new() -> Memory {
        Memory {
            memory: [0; MEMORY_SIZE],
            program_len: 0,
        }
    }

    /// Load a program into memory.
    ///
    /// The loaded program starts at `0x200`.
    ///
    /// This returns an error if the program cannot be fit into memory (exceeds
    /// a length of 3583).
    pub fn load_program(&mut self, program: &[u8]) -> anyhow::Result<()> {
        if program.len() > MEMORY_PROGRAM_SIZE {
            bail!("Program cannot fit in memory (length {})", program.len());
        }

        self.program_len = program.len();

        self.memory[MEMORY_PROGRAM_START..MEMORY_PROGRAM_START + program.len()]
            .copy_from_slice(program);

        Ok(())
    }
}

impl Index<usize> for Memory {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        if index < MEMORY_PROGRAM_START {
            warn!("Accessed restricted memory at {index:#X}");
        }

        &self.memory[index]
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        if index < MEMORY_PROGRAM_START {
            warn!("Mutated restricted memory at {index:#X}");
        }

        &mut self.memory[index]
    }
}

#[cfg(test)]
mod tests {
    use log::Level;

    use super::*;

    /// Assert that [`Memory::new`] returns with everything set to zero.  
    #[test]
    fn new() {
        let memory = Memory::new();
        for i in 0..MEMORY_SIZE {
            assert_eq!(memory.memory[i], 0);
        }
    }

    /// Assert that getting a value by indexing [`Memory`] works correctly.
    #[test]
    fn get() {
        let mut memory = Memory::new();
        memory.memory[42] = 24;
        assert_eq!(memory[42], 24);
    }

    /// Assert that accessing a restricted area by indexing [`Memory`] raises
    /// a warning.
    #[test]
    fn get_restricted() {
        testing_logger::setup();

        let memory = Memory::new();
        let _ = memory[0x42];

        testing_logger::validate(|logs| {
            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].level, Level::Warn);
            assert_eq!(logs[0].body, "Accessed restricted memory at 0x42");
        });
    }

    /// Assert that setting a value by indexing [`Memory`] works correctly.
    #[test]
    fn set() {
        let mut memory = Memory::new();
        memory[42] = 24;
        assert_eq!(memory.memory[42], 24);
    }

    /// Assert that mutating a restricted area by indexing [`Memory`] raises
    /// a warning.
    #[test]
    fn set_restricted() {
        testing_logger::setup();

        let mut memory = Memory::new();
        memory[0x42] = 0x24;

        testing_logger::validate(|logs| {
            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].level, Level::Warn);
            assert_eq!(logs[0].body, "Mutated restricted memory at 0x42");
        });
    }

    /// Assert that loading a program into memory by [`Memory::load_program`]
    /// works correctly.
    #[test]
    fn load_program() {
        let mut memory = Memory::new();

        let program = [0x10, 0x42, 0x20, 0x24];
        memory.load_program(&program).unwrap();

        assert_eq!(
            &memory.memory[MEMORY_PROGRAM_START..MEMORY_PROGRAM_START + 4],
            &program
        );
    }

    /// Assert that loading a program that's too big to fit into memory by
    /// [`Memory::load_program`] returns an error.
    #[test]
    fn load_big_program_error() {
        let mut memory = Memory::new();
        assert!(memory.load_program(&[1; MEMORY_SIZE + 1]).is_err());
    }
}
