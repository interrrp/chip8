use anyhow::{anyhow, Result};
use raylib::{color::Color, prelude::RaylibDraw, RaylibHandle, RaylibThread};

use crate::{
    display::{Display, DISPLAY_SCALE},
    instructions::Instruction,
    memory::{Memory, MEMORY_PROGRAM_START},
    registers::Registers,
};

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
        let (mut rl, rl_thread) = raylib::init()
            .size(
                self.display.width as i32 * DISPLAY_SCALE,
                self.display.height as i32 * DISPLAY_SCALE,
            )
            .title("CHIP-8")
            .build();

        while self.pc < MEMORY_PROGRAM_START + self.memory.program_len {
            let instruction = self.fetch_instruction()?;
            self.do_instruction(instruction, &mut rl, &rl_thread)?;
        }
        Ok(())
    }

    /// Perform an instruction.
    ///
    /// An error is returned if `RET` was attempted outside a subroutine.
    fn do_instruction(
        &mut self,
        instruction: Instruction,
        rl: &mut RaylibHandle,
        rl_thread: &RaylibThread,
    ) -> Result<()> {
        let v = &mut self.registers.data;
        let i = &mut self.registers.i;

        let mut draw_handle = rl.begin_drawing(rl_thread);

        match instruction {
            Instruction::Jp { addr } => self.pc = addr - 2,
            Instruction::JpV0 { addr } => self.pc = addr + v[0] as usize - 2,

            Instruction::Call { addr } => {
                self.stack.push(self.pc);
                self.pc = addr - 2;
            }
            Instruction::Ret => match self.stack.pop() {
                Some(addr) => self.pc = addr,
                None => return Err(anyhow!("Attempted RET outside a subroutine")),
            },

            Instruction::SeVxByte { x, byte } if v[x] == byte => self.pc += 2,
            Instruction::SneVxByte { x, byte } if v[x] != byte => self.pc += 2,
            Instruction::SeVxVy { x, y } if v[x] == v[y] => self.pc += 2,
            Instruction::SneVxVy { x, y } if v[x] != v[y] => self.pc += 2,

            Instruction::LdVxByte { x, byte } => v[x] = byte,
            Instruction::LdVxVy { x, y } => v[x] = v[y],
            Instruction::LdIAddr { addr } => *i = addr,

            Instruction::AddVxByte { x, byte } => {
                v[x] = v[x].wrapping_add(byte);
            }
            Instruction::AddVxVy { x, y } => {
                let (result, carry) = v[x].overflowing_add(v[y]);
                v[x] = result;
                v[0xF] = carry.into();
            }
            Instruction::AddI { x } => *i = i.wrapping_add(v[x] as usize),
            Instruction::Sub { x, y } => {
                let (new_vx, vf) = v[x].overflowing_sub(v[y]);
                v[x] = new_vx;
                v[0xF] = (!vf).into();
            }
            Instruction::Subn { x, y } => {
                let (new_vx, vf) = v[y].overflowing_sub(v[x]);
                v[x] = new_vx;
                v[0xF] = (!vf).into();
            }

            Instruction::Or { x, y } => v[x] |= v[y],
            Instruction::And { x, y } => v[x] &= v[y],
            Instruction::Xor { x, y } => v[x] ^= v[y],
            Instruction::Shl { x } => v[x] <<= 1,
            Instruction::Shr { x } => v[x] >>= 1,

            Instruction::Cls => self.display.clear(),
            Instruction::Drw { x, y, nibble } => {
                // Reset collision flag
                v[0xF] = 0;

                // Get sprite coordinates from registers
                let x = v[x] as usize % self.display.width;
                let y = v[y] as usize % self.display.height;

                // Draw each row of the sprite
                for row in 0..nibble {
                    let sprite_byte = self.memory[self.registers.i + row as usize];

                    for bit in 0..8 {
                        if (sprite_byte & (0x80 >> bit)) != 0 {
                            let px = (x + bit) % self.display.width;
                            let py = (y + row as usize) % self.display.height;

                            if self.display.xor_pixel(px, py) {
                                v[0xF] = 1;
                            }
                        }
                    }
                }
            }

            _ => {}
        }

        self.pc += 2;

        #[cfg(not(test))]
        {
            use std::{thread::sleep, time::Duration};
            self.display.render(&mut draw_handle);
            sleep(Duration::from_millis(1));
        }

        Ok(())
    }

    /// Return the next instruction.
    ///
    /// This takes the next two bytes, combines them to get an opcode, then
    /// decodes the instruction.
    fn fetch_instruction(&mut self) -> Result<Instruction> {
        let opcode = u16::from_be_bytes([self.memory[self.pc], self.memory[self.pc + 1]]);
        Instruction::from_opcode(opcode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_program() -> Result<()> {
        let emulator = Emulator::from_program(&[0x00, 0xE0, 0x00, 0xEE])?;
        assert_eq!(emulator.memory[MEMORY_PROGRAM_START], 0x00);
        assert_eq!(emulator.memory[MEMORY_PROGRAM_START + 1], 0xE0);
        assert_eq!(emulator.memory[MEMORY_PROGRAM_START + 2], 0x00);
        assert_eq!(emulator.memory[MEMORY_PROGRAM_START + 3], 0xEE);
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

        assert_eq!(emulator.registers.data[1], 0x42);
        assert_eq!(emulator.registers.data[2], 0x24);

        Ok(())
    }

    #[test]
    fn ld() -> Result<()> {
        let mut emulator = Emulator::from_program(&[0x61, 0xAB, 0x82, 0x10])?;
        emulator.run()?;
        assert_eq!(emulator.registers.data[1], 0xAB);
        assert_eq!(emulator.registers.data[2], 0xAB);
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

        assert_eq!(emulator.registers.data[1], 2);
        assert_eq!(emulator.registers.data[2], 4);
        assert_eq!(emulator.registers.data[3], 6);
        assert_eq!(emulator.registers.data[4], 8);

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

        assert_eq!(emulator.registers.data[1], 4);
        assert_eq!(emulator.registers.data[2], 6);

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

        assert_eq!(emulator.registers.data[1], 0x42);
        assert_eq!(emulator.registers.data[2], 0x07);

        Ok(())
    }

    #[test]
    fn ret_outside_subroutine_error() -> Result<()> {
        let mut emulator = Emulator::from_program(&[0x00, 0xEE])?;
        assert!(emulator.run().is_err());
        Ok(())
    }
}
