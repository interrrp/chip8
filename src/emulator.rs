use crate::{
    instructions::{decode_instruction, Instruction},
    memory::{Memory, MEMORY_UNRESTRICTED_START},
    registers::Registers,
    stack::Stack,
};
use anyhow::Result;

/// The emulator itself. You could also call it the CPU.
pub(crate) struct Emulator {
    registers: Registers,
    stack: Stack,
    memory: Memory,
    pc: usize,
}

impl Emulator {
    /// Return a new emulator with `program` pre-loaded into memory.
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

    /// Repeatedly fetch and execute all instructions in memory.
    pub fn run(&mut self) -> Result<()> {
        while self.pc < MEMORY_UNRESTRICTED_START + self.memory.program_len {
            let instruction = self.fetch_instruction()?;
            self.do_instruction(instruction)?;
        }
        Ok(())
    }

    fn do_instruction(&mut self, instruction: Instruction) -> Result<()> {
        match instruction {
            Instruction::Jp { addr } => self.pc = addr,
            Instruction::JpV0 { addr } => self.pc = addr + self.registers[0] as usize,

            // TODO: CALL
            // TODO: RET
            Instruction::SeVxByte { vx, byte } => {
                if self.registers[vx] == byte {
                    self.pc += 2;
                }
            }
            Instruction::SneVxByte { vx, byte } => {
                if self.registers[vx] != byte {
                    self.pc += 2;
                }
            }
            Instruction::SeVxVy { vx, vy } => {
                if self.registers[vx] == self.registers[vy] {
                    self.pc += 2;
                }
            }
            Instruction::SneVxVy { vx, vy } => {
                if self.registers[vx] != self.registers[vy] {
                    self.pc += 2;
                }
            }

            Instruction::LdVxByte { vx, byte } => self.registers[vx] = byte,
            Instruction::LdVxVy { vx, vy } => self.registers[vx] = self.registers[vy],

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
        assert_eq!(emulator.fetch_instruction()?, Instruction::Nop);

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

    #[test]
    fn ld() -> Result<()> {
        let mut emulator = Emulator::from_program(&[0x61, 0xab, 0x82, 0x10])?;
        emulator.run()?;
        assert_eq!(emulator.registers[1], 0xab);
        assert_eq!(emulator.registers[2], 0xab);
        Ok(())
    }

    #[test]
    fn skip_if() -> Result<()> {
        let mut emulator = Emulator::from_program(&[
            0x61, 0x02, // LD  Vx=1 byte=2
            0x62, 0x04, // LD  Vx=2 byte=4
            0x31, 0x02, // SE  Vx=1 byte=2
            0x63, 0x07, // LD  Vx=3 byte=7  Should not be executed
            0x64, 0x04, // SNE Vx=2 byte=4
            0x63, 0x06, // LD  Vx=3 byte=6  Should be executed
            0x51, 0x20, // SE  Vx=1 Vy=2
            0x64, 0x08, // LD  Vx=4 byte=8  Should be executed
            0x91, 0x20, // SNE Vx=1 Vy=2
            0x64, 0x09, // LD  Vx=4 byte=9  Should not be executed
        ])?;
        emulator.run()?;

        assert_eq!(emulator.registers[1], 2);
        assert_eq!(emulator.registers[2], 4);
        assert_eq!(emulator.registers[3], 6);
        assert_eq!(emulator.registers[4], 8);

        Ok(())
    }
}
