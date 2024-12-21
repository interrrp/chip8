use crate::{
    instructions::{decode_instruction, Instruction},
    memory::{Memory, MEMORY_UNRESTRICTED_START},
    registers::Registers,
    stack::Stack,
};
use anyhow::Result;

pub(crate) struct Processor {
    registers: Registers,
    stack: Stack,
    memory: Memory,
    pc: usize,
}

impl Processor {
    pub fn from_program(program: &[u8]) -> Result<Processor> {
        let mut processor = Processor {
            registers: Registers::new(),
            stack: Stack::new(),
            memory: Memory::new(),
            pc: MEMORY_UNRESTRICTED_START,
        };
        processor.memory.load_program(program)?;
        Ok(processor)
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            let instruction = self.fetch_instruction()?;
            println!("{instruction:?}");
        }
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
        let processor = Processor::from_program(&[0x00, 0xE0, 0x00, 0xEE])?;
        assert_eq!(processor.memory.at(MEMORY_UNRESTRICTED_START)?, 0x00);
        assert_eq!(processor.memory.at(MEMORY_UNRESTRICTED_START + 1)?, 0xE0);
        assert_eq!(processor.memory.at(MEMORY_UNRESTRICTED_START + 2)?, 0x00);
        assert_eq!(processor.memory.at(MEMORY_UNRESTRICTED_START + 3)?, 0xEE);
        Ok(())
    }

    #[test]
    fn fetch_instruction() -> Result<()> {
        let mut processor = Processor::from_program(&[0x00, 0xE0, 0x00, 0xEE])?;

        assert_eq!(processor.fetch_instruction()?, Instruction::Cls);
        assert_eq!(processor.fetch_instruction()?, Instruction::Ret);
        assert!(processor.fetch_instruction().is_err());

        Ok(())
    }
}
