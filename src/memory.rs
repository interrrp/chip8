use anyhow::{anyhow, Result};

pub(crate) const MEMORY_UNRESTRICTED_START: usize = 0x201;
pub(crate) const MEMORY_UNRESTRICTED_END: usize = 0xFFF;
pub(crate) const MEMORY_UNRESTRICTED_SIZE: usize = MEMORY_UNRESTRICTED_END - MEMORY_UNRESTRICTED_START;

/// The memory (RAM).
///
/// > The Chip-8 language is capable of accessing up to 4KB (4,096 bytes) of RAM, from location 0x000 (0) to 0xFFF
/// > (4095). The first 512 bytes, from 0x000 to 0x1FF, are where the original interpreter was located, and should not
/// > be used by programs.
/// >
/// > Most Chip-8 programs start at location 0x200 (512), but some begin at 0x600 (1536). Programs beginning at 0x600
/// > are intended for the ETI 660 computer.
/// >
/// > [_Cowgod's CHIP-8 Technical Reference, section 2.1_](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.1)
pub(crate) struct Memory {
    memory: [u8; MEMORY_UNRESTRICTED_END],
    pub program_len: usize,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            memory: [0; MEMORY_UNRESTRICTED_END],
            program_len: 0,
        }
    }

    pub fn at(&self, index: usize) -> Result<u8> {
        if index < MEMORY_UNRESTRICTED_START {
            return Err(anyhow!("Attempted access to restricted area: {:#x}", index));
        }
        Ok(self.memory[index])
    }

    pub fn load_program(&mut self, program: &[u8]) -> Result<()> {
        if program.len() > MEMORY_UNRESTRICTED_SIZE {
            return Err(anyhow!(
                "Attempted to load program exceeding memory limit ({} bytes)",
                program.len()
            ));
        }

        self.program_len = program.len();
        self.memory[MEMORY_UNRESTRICTED_START..MEMORY_UNRESTRICTED_START + program.len()].copy_from_slice(program);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_memory_zeroes() -> Result<()> {
        let memory = Memory::new();
        for i in MEMORY_UNRESTRICTED_START..MEMORY_UNRESTRICTED_END {
            assert_eq!(memory.at(i)?, 0);
        }
        Ok(())
    }

    #[test]
    fn access_restricted_error() {
        let memory = Memory::new();
        for i in 0..MEMORY_UNRESTRICTED_START {
            assert!(memory.at(i).is_err());
        }
    }

    #[test]
    fn load_program() -> Result<()> {
        let mut memory = Memory::new();
        memory.load_program(&[0x00, 0xE0, 0x00, 0xEE])?;

        assert_eq!(memory.at(MEMORY_UNRESTRICTED_START)?, 0x00);
        assert_eq!(memory.at(MEMORY_UNRESTRICTED_START + 1)?, 0xE0);
        assert_eq!(memory.at(MEMORY_UNRESTRICTED_START + 2)?, 0x00);
        assert_eq!(memory.at(MEMORY_UNRESTRICTED_START + 3)?, 0xEE);
        assert_eq!(memory.program_len, 4);

        Ok(())
    }
}
