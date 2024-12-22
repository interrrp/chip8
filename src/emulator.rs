use crate::{
    instructions::{decode_instruction, Instruction},
    memory::{Memory, MEMORY_UNRESTRICTED_START},
    registers::Registers,
    stack::Stack,
};
use anyhow::Result;

pub(crate) struct Emulator {
    registers: Registers,
    stack: Stack,
    memory: Memory,
    pc: usize,
}

impl Emulator {
    pub fn from_program(program: &[u8]) -> Result<Emulator> {
        let mut emulator = Emulator {
            registers: Registers::new(),
            stack: Stack::new(),
            memory: Memory::new(),
            pc: MEMORY_UNRESTRICTED_START,
        };
        emulator.memory.load_program(program)?;
        Ok(emulator)
    }

    pub fn run(&mut self) -> Result<()> {
        while self.pc < MEMORY_UNRESTRICTED_START + self.memory.program_len {
            let instruction = self.fetch_instruction()?;
            self.do_instruction(instruction)?;
        }
        Ok(())
    }

    fn do_instruction(&mut self, instruction: Instruction) -> Result<()> {
        match instruction {
            Instruction::Jp { addr } => self.pc = addr as usize,
            _ => {}
        }

        Ok(())
    }

    fn fetch_instruction(&mut self) -> Result<Instruction> {
        let opcode = u16::from_be_bytes([self.memory.at(self.pc)?, self.memory.at(self.pc + 1)?]);
        self.pc += 2;
        decode_instruction(opcode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_program() -> Result<()> {
        let emulator = Emulator::from_program(&[0x00, 0xE0, 0x00, 0xEE])?;
        assert_eq!(emulator.memory.at(MEMORY_UNRESTRICTED_START)?, 0x00);
        assert_eq!(emulator.memory.at(MEMORY_UNRESTRICTED_START + 1)?, 0xE0);
        assert_eq!(emulator.memory.at(MEMORY_UNRESTRICTED_START + 2)?, 0x00);
        assert_eq!(emulator.memory.at(MEMORY_UNRESTRICTED_START + 3)?, 0xEE);
        Ok(())
    }

    #[test]
    fn fetch_instruction() -> Result<()> {
        let mut emulator = Emulator::from_program(&[0x00, 0xE0, 0x00, 0xEE])?;

        assert_eq!(emulator.fetch_instruction()?, Instruction::Cls);
        assert_eq!(emulator.fetch_instruction()?, Instruction::Ret);
        assert!(emulator.fetch_instruction().is_err());

        Ok(())
    }

    #[test]
    fn jp() -> Result<()> {
        let mut emulator = Emulator::from_program(&[0x12, 0x26])?;
        assert_eq!(emulator.pc, MEMORY_UNRESTRICTED_START);
        emulator.run()?;
        assert_eq!(emulator.pc, 0x226);
        Ok(())
    }
}
