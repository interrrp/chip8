use crate::{
    instructions::{decode_instruction, Instruction},
    memory::{Memory, MEMORY_PROGRAM_START},
    registers::Registers,
    stack::Stack,
};
use anyhow::Result;

/// The emulator itself. You could also call it the CPU.
pub struct Emulator {
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
            pc: MEMORY_PROGRAM_START,
        };
        emulator.memory.load_program(program)?;
        Ok(emulator)
    }

    /// Repeatedly fetch and execute all instructions in memory.
    pub fn run(&mut self) -> Result<()> {
        while self.pc < MEMORY_PROGRAM_START + self.memory.program_len {
            let instruction = self.fetch_instruction()?;
            self.do_instruction(instruction);
        }
        Ok(())
    }

    fn do_instruction(&mut self, instruction: Instruction) {
        let r = &mut self.registers;

        match instruction {
            Instruction::Jp { addr } => self.pc = addr,
            Instruction::JpV0 { addr } => self.pc = addr + r[0] as usize,

            Instruction::SeVxByte { vx, byte } if r[vx] == byte => self.pc += 2,
            Instruction::SneVxByte { vx, byte } if r[vx] != byte => {
                self.pc += 2;
            }
            Instruction::SeVxVy { vx, vy } if r[vx] == r[vy] => self.pc += 2,
            Instruction::SneVxVy { vx, vy } if r[vx] != r[vy] => self.pc += 2,

            Instruction::LdVxByte { vx, byte } => r[vx] = byte,
            Instruction::LdVxVy { vx, vy } => r[vx] = r[vy],
            Instruction::LdIAddr { addr } => r.i = addr,

            Instruction::AddVxByte { vx, byte } => {
                r[vx] = r[vx].wrapping_add(byte);
            }
            Instruction::AddVxVy { vx, vy } => {
                let (result, carry) = r[vx].overflowing_add(r[vy]);
                r[vx] = result;
                r[0xf] = carry.into();
            }
            Instruction::AddI { vx } => r.i = r.i.wrapping_add(r[vx] as usize),
            Instruction::Sub { vx, vy } => {
                let (new_vx, vf) = r[vx].overflowing_sub(r[vy]);
                r[vx] = new_vx;
                r[0xf] = (!vf).into();
            }
            Instruction::Subn { vx, vy } => {
                let (new_vx, vf) = r[vy].overflowing_sub(r[vx]);
                r[vx] = new_vx;
                r[0xf] = (!vf).into();
            }

            Instruction::Drw { vx, vy, nibble } => {
                println!("DRW {vx} {vy} {nibble}");
            }

            _ => {}
        }
    }

    fn fetch_instruction(&mut self) -> Result<Instruction> {
        let opcode = u16::from_be_bytes([
            self.memory.at(self.pc)?,
            self.memory.at(self.pc + 1)?,
        ]);
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
        assert_eq!(emulator.memory.at(MEMORY_PROGRAM_START)?, 0x00);
        assert_eq!(emulator.memory.at(MEMORY_PROGRAM_START + 1)?, 0xE0);
        assert_eq!(emulator.memory.at(MEMORY_PROGRAM_START + 2)?, 0x00);
        assert_eq!(emulator.memory.at(MEMORY_PROGRAM_START + 3)?, 0xEE);
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
        assert_eq!(emulator.pc, MEMORY_PROGRAM_START);
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
            0x61, 0x02, // LD  V1 2   |
            0x62, 0x04, // LD  V2 4   |
            0x31, 0x02, // SE  V1 2   |
            0x63, 0x07, // LD  V3 7   |- Should not be executed
            0x64, 0x04, // SNE V2 4   |
            0x63, 0x06, // LD  V3 6   |- Should be executed
            0x51, 0x20, // SE  V1 V2  |
            0x64, 0x08, // LD  V4 8   |- Should be executed
            0x91, 0x20, // SNE V1 V2  |
            0x64, 0x09, // LD  V4 9   |- Should not be executed
        ])?;
        emulator.run()?;

        assert_eq!(emulator.registers[1], 2);
        assert_eq!(emulator.registers[2], 4);
        assert_eq!(emulator.registers[3], 6);
        assert_eq!(emulator.registers[4], 8);

        Ok(())
    }

    #[test]
    fn arithmetic() -> Result<()> {
        let mut emulator = Emulator::from_program(&[
            0x61, 0x02, // LD   V1 2   |  V1=2
            0x62, 0x04, // LD   V2 4   |  V1=2 V2=4
            0x72, 0x02, // ADD  V2 2   |  V1=2 V2=6
            0x81, 0x24, // ADD  V1 V2  |  V1=8 V2=6
            0x81, 0x25, // SUB  V1 V2  |  V1=2 V2=6
            0x81, 0x27, // SUBN V2 V1  |  V1=4 V2=6
        ])?;
        emulator.run()?;

        assert_eq!(emulator.registers[1], 4);
        assert_eq!(emulator.registers[2], 6);

        Ok(())
    }
}
