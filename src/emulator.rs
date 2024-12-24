use std::{thread::sleep, time::Duration};

use crate::{
    display::Display,
    instructions::{decode_instruction, Instruction},
    memory::{Memory, MEMORY_PROGRAM_START},
    registers::Registers,
};
use anyhow::{anyhow, Result};

/// Represents a complete CHIP-8 emulator, managing the core system state.
#[derive(Debug, Clone)]
pub struct Emulator {
    registers: Registers,
    stack: Vec<usize>,
    memory: Memory,
    display: Display,
    /// Program counter indicating the current instruction address.
    pc: usize,
}

impl Emulator {
    /// Return a new emulator with `program` pre-loaded into memory.
    ///
    /// If the program exceeds the memory capacity, this returns an error.
    pub fn from_program(program: &[u8]) -> Result<Emulator> {
        let mut emulator = Emulator {
            registers: Registers::new(),
            stack: Vec::new(),
            memory: Memory::new(),
            display: Display::new(),
            pc: MEMORY_PROGRAM_START,
        };
        emulator.memory.load_program(program)?;
        Ok(emulator)
    }

    /// Repeatedly fetch and execute all instructions in memory.
    ///
    /// If there is any error fetching or doing an instruction, execution will
    /// stop and the error will be returned.
    pub fn run(&mut self) -> Result<()> {
        while self.pc < MEMORY_PROGRAM_START + self.memory.program_len {
            let instruction = self.fetch_instruction()?;
            self.do_instruction(instruction)?;
        }
        Ok(())
    }

    /// Perform an instruction.
    ///
    /// An error is returned if `RET` was attempted outside a subroutine.
    fn do_instruction(&mut self, instruction: Instruction) -> Result<()> {
        let r = &mut self.registers;

        match instruction {
            Instruction::Jp { addr } => self.pc = addr - 1,
            Instruction::JpV0 { addr } => self.pc = addr + r[0] as usize - 2,

            Instruction::Call { addr } => {
                self.stack.push(addr - 5);
                self.pc = addr - 1;
            }
            Instruction::Ret => match self.stack.pop() {
                Some(addr) => self.pc = addr,
                None => return Err(anyhow!("Attempted RET outside a subroutine")),
            },

            Instruction::SeVxByte { vx, byte } if r[vx] == byte => self.pc += 2,
            Instruction::SneVxByte { vx, byte } if r[vx] != byte => self.pc += 2,
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

            Instruction::Cls => self.display.clear(),
            Instruction::Drw { vx, vy, nibble } => {
                // Reset collision flag
                self.registers[0xF] = 0;

                // Get sprite coordinates from registers
                let x = self.registers[vx] as usize % self.display.width;
                let y = self.registers[vy] as usize % self.display.height;

                // Draw each row of the sprite
                for row in 0..nibble {
                    let sprite_byte = self.memory.at(self.registers.i + row as usize + 1);

                    for bit in 0..8 {
                        if (sprite_byte & (0x80 >> bit)) != 0 {
                            let px = (x + bit) % self.display.width;
                            let py = (y + row as usize) % self.display.height;

                            // XOR pixel and set collision flag if pixel was previously set
                            if self.display.xor_pixel(px, py) {
                                self.registers[0xF] = 1;
                            }
                        }
                    }
                }
            }

            _ => {}
        }

        self.pc += 2;

        println!("\x1B[2J\x1B[1;1H");
        self.display.render();
        println!("pc: {}    {instruction:?}", self.pc);
        sleep(Duration::from_millis(10));

        Ok(())
    }

    /// Return the next instruction.
    ///
    /// This takes the next two bytes, combines them to get an opcode, then
    /// decodes the instruction.
    fn fetch_instruction(&mut self) -> Result<Instruction> {
        let opcode = u16::from_be_bytes([self.memory.at(self.pc), self.memory.at(self.pc + 1)]);
        decode_instruction(opcode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_program() -> Result<()> {
        let emulator = Emulator::from_program(&[0x00, 0xE0, 0x00, 0xEE])?;
        assert_eq!(emulator.memory.at(MEMORY_PROGRAM_START), 0x00);
        assert_eq!(emulator.memory.at(MEMORY_PROGRAM_START + 1), 0xE0);
        assert_eq!(emulator.memory.at(MEMORY_PROGRAM_START + 2), 0x00);
        assert_eq!(emulator.memory.at(MEMORY_PROGRAM_START + 3), 0xEE);
        Ok(())
    }

    #[test]
    fn fetch_instruction() -> Result<()> {
        let mut emulator = Emulator::from_program(&[0x00, 0xE0, 0x00, 0xEE])?;

        assert_eq!(emulator.fetch_instruction()?, Instruction::Cls);
        emulator.pc += 2;
        assert_eq!(emulator.fetch_instruction()?, Instruction::Ret);
        emulator.pc += 2;
        assert_eq!(emulator.fetch_instruction()?, Instruction::Nop);

        Ok(())
    }

    #[test]
    fn jump() -> Result<()> {
        let mut emulator = Emulator::from_program(&[
            0x61, 0x42, // LD V1    0x42  |- Load initial value
            0x12, 0x06, // JP 0x206       |- Jump over the bad instruction
            0x61, 0xFF, // LD V1    0xFF  |- This should be skipped
            0x62, 0x24, // LD V2    0x24  |- This is where we jump to
        ])?;
        emulator.run()?;

        assert_eq!(emulator.registers[1], 0x42);
        assert_eq!(emulator.registers[2], 0x24);

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

    #[test]
    fn subroutines() -> Result<()> {
        let mut emulator = Emulator::from_program(&[
            0x22, 0x06, // CALL 0x206    |- Call subroutine
            0x62, 0x07, // LD   V2 0x07  |- After return from subroutine
            0x13, 0x00, // JP   0x300    |- Jump to exit
            0x61, 0x42, // LD   V1 0x42  |- Subroutine
            0x00, 0xEE, // RET           |- Return from subroutine
        ])?;
        emulator.run()?;

        assert_eq!(emulator.registers[1], 0x42);
        assert_eq!(emulator.registers[2], 0x07);

        Ok(())
    }

    #[test]
    fn ret_outside_subroutine_error() -> Result<()> {
        let mut emulator = Emulator::from_program(&[0x00, 0xEE])?;
        assert!(emulator.run().is_err());
        Ok(())
    }
}
