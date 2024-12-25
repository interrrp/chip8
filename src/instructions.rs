use anyhow::{anyhow, Result};

/// An enumeration of all CHIP-8 instructions.
///
/// See [CHIP-8 Instruction Set (Matthew
/// Mikolay)](https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Instruction-Set).
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Instruction {
    /// Do nothing.
    Nop,
    /// Jump to location `addr`.
    Jp { addr: usize },
    /// Jump to location nnn + V0.
    JpV0 { addr: usize },

    /// Call subroutine at `addr`.
    Call { addr: usize },
    /// Return from a subroutine.
    Ret,

    /// Skip next instruction if Vx = kk.
    SeVxByte { x: usize, byte: u8 },
    /// Skip next instruction if Vx != kk.
    SneVxByte { x: usize, byte: u8 },
    /// Skip next instruction if Vx = Vy.
    SeVxVy { x: usize, y: usize },
    /// Skip next instruction if Vx != Vy.
    SneVxVy { x: usize, y: usize },
    /// Skip next instruction if key with the value of Vx is pressed.
    Skp { x: usize },
    /// Skip next instruction if key with the value of Vx is not pressed.
    Sknp { x: usize },

    /// Set Vx = byte.
    LdVxByte { x: usize, byte: u8 },
    /// Set Vx = Vy.
    LdVxVy { x: usize, y: usize },
    /// Set I = addr.
    LdIAddr { addr: usize },
    /// Wait for a key press, store the value of the key in Vx.
    LdK { x: usize },
    /// Set delay timer = Vx.
    LdDt { x: usize },
    /// Set sound timer = Vx.
    LdSt { x: usize },
    /// Set I = location of sprite for digit Vx.
    LdF { x: usize },
    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    LdB { x: usize },
    /// Store registers V0 through Vx in memory starting at location I.
    LdI { x: usize },
    /// Read registers V0 through Vx from memory starting at location I.
    LdIVx { x: usize },
    /// Set Vx = random byte AND kk.
    Rnd { x: usize, byte: u8 },

    /// Set Vx = Vx + kk.
    AddVxByte { x: usize, byte: u8 },
    /// Set Vx = Vx + Vy, set VF = carry.
    AddVxVy { x: usize, y: usize },
    /// Set I = I + Vx.
    AddI { x: usize },
    /// Set Vx = Vx - Vy, set VF = NOT borrow.
    Sub { x: usize, y: usize },
    /// Set Vx = Vy - Vx, set VF = NOT borrow.
    Subn { x: usize, y: usize },

    /// Set Vx = Vx AND Vy.
    And { x: usize, y: usize },
    /// Set Vx = Vx OR Vy.
    Or { x: usize, y: usize },
    /// Set Vx = Vx XOR Vy.
    Xor { x: usize, y: usize },
    /// Set Vx = Vx SHR 1.
    Shr { x: usize },
    /// Set Vx = Vx SHL 1.
    Shl { x: usize },

    /// Clear the display.
    Cls,
    /// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF
    /// = collision.
    Drw { x: usize, y: usize, nibble: u8 },
}

impl Instruction {
    /// Decode the instruction from an opcode.
    ///
    /// If the opcode is unrecognized, this will return an error.
    pub fn from_opcode(op: u16) -> Result<Instruction> {
        let x = ((op & 0x0F00) >> 8) as usize;
        let y = ((op & 0x00F0) >> 4) as usize;
        let byte = (op & 0x00FF) as u8;
        let addr = (op & 0x0FFF) as usize;
        let e = (op & 0x000F) as u8;

        Ok(match op & 0xF000 {
            0x0000 if op == 0x0000 => Instruction::Nop,
            0x1000 => Instruction::Jp { addr },
            0xB000 => Instruction::JpV0 { addr },

            0x2000 => Instruction::Call { addr },
            0x0000 if op == 0x00EE => Instruction::Ret,

            0x3000 => Instruction::SeVxByte { x, byte },
            0x4000 => Instruction::SneVxByte { x, byte },
            0x5000 => Instruction::SeVxVy { x, y },
            0x9000 => Instruction::SneVxVy { x, y },
            0xE000 if byte == 0x9E => Instruction::Skp { x },
            0xE000 if byte == 0xA1 => Instruction::Sknp { x },

            0x6000 => Instruction::LdVxByte { x, byte },
            0x8000 if e == 0 => Instruction::LdVxVy { x, y },
            0xA000 => Instruction::LdIAddr { addr },
            0xF000 if byte == 0x07 => Instruction::LdDt { x },
            0xF000 if byte == 0x0A => Instruction::LdK { x },
            0xF000 if byte == 0x15 => Instruction::LdDt { x },
            0xF000 if byte == 0x18 => Instruction::LdSt { x },
            0xF000 if byte == 0x1E => Instruction::AddI { x },
            0xF000 if byte == 0x29 => Instruction::LdF { x },
            0xF000 if byte == 0x33 => Instruction::LdB { x },
            0xF000 if byte == 0x55 => Instruction::LdI { x },
            0xF000 if byte == 0x65 => Instruction::LdIVx { x },
            0xC000 => Instruction::Rnd { x, byte },

            0x8000 if e == 1 => Instruction::Or { x, y },
            0x8000 if e == 2 => Instruction::And { x, y },
            0x8000 if e == 3 => Instruction::Xor { x, y },
            0x8000 if e == 6 => Instruction::Shr { x },
            0x8000 if e == 0xE => Instruction::Shl { x },

            0x7000 => Instruction::AddVxByte { x, byte },
            0x8000 if e == 4 => Instruction::AddVxVy { x, y },
            0x8000 if e == 5 => Instruction::Sub { x, y },
            0x8000 if e == 7 => Instruction::Subn { x, y },

            0xD000 => Instruction::Drw { x, y, nibble: e },
            0x0000 if op == 0x00E0 => Instruction::Cls,

            _ => return Err(anyhow!("Unknown instruction (opcode {:#X})", op)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is(opcode: u16, instruction: Instruction) -> bool {
        Instruction::from_opcode(opcode).unwrap() == instruction
    }

    #[test]
    fn decode() {
        assert!(is(0x0000, Instruction::Nop));
        assert!(is(0x1234, Instruction::Jp { addr: 0x234 }));
        assert!(is(0xB234, Instruction::JpV0 { addr: 0x234 }));
        assert!(is(0x2345, Instruction::Call { addr: 0x345 }));
        assert!(is(0x00EE, Instruction::Ret));

        assert!(is(0x3122, Instruction::SeVxByte { x: 1, byte: 0x22 }));
        assert!(is(0x4122, Instruction::SneVxByte { x: 1, byte: 0x22 }));
        assert!(is(0x5120, Instruction::SeVxVy { x: 1, y: 2 }));
        assert!(is(0x9120, Instruction::SneVxVy { x: 1, y: 2 }));
        assert!(is(0xE19E, Instruction::Skp { x: 1 }));
        assert!(is(0xE1A1, Instruction::Sknp { x: 1 }));

        assert!(is(0x6122, Instruction::LdVxByte { x: 1, byte: 0x22 }));
        assert!(is(0x8120, Instruction::LdVxVy { x: 1, y: 2 }));
        assert!(is(0xA123, Instruction::LdIAddr { addr: 0x123 }));
        assert!(is(0xF107, Instruction::LdDt { x: 1 }));
        assert!(is(0xF10A, Instruction::LdK { x: 1 }));
        assert!(is(0xF115, Instruction::LdDt { x: 1 }));
        assert!(is(0xF118, Instruction::LdSt { x: 1 }));
        assert!(is(0xF11E, Instruction::AddI { x: 1 }));
        assert!(is(0xF129, Instruction::LdF { x: 1 }));
        assert!(is(0xF133, Instruction::LdB { x: 1 }));
        assert!(is(0xF155, Instruction::LdI { x: 1 }));
        assert!(is(0xF165, Instruction::LdIVx { x: 1 }));
        assert!(is(0xC122, Instruction::Rnd { x: 1, byte: 0x22 }));

        assert!(is(0x8121, Instruction::Or { x: 1, y: 2 }));
        assert!(is(0x8122, Instruction::And { x: 1, y: 2 }));
        assert!(is(0x8123, Instruction::Xor { x: 1, y: 2 }));
        assert!(is(0x8126, Instruction::Shr { x: 1 }));
        assert!(is(0x812E, Instruction::Shl { x: 1 }));

        assert!(is(0x7122, Instruction::AddVxByte { x: 1, byte: 0x22 }));
        assert!(is(0x8124, Instruction::AddVxVy { x: 1, y: 2 }));
        assert!(is(0x8125, Instruction::Sub { x: 1, y: 2 }));
        assert!(is(0x8127, Instruction::Subn { x: 1, y: 2 }));

        assert!(is(
            0xD123,
            Instruction::Drw {
                x: 1,
                y: 2,
                nibble: 3
            }
        ));
        assert!(is(0x00E0, Instruction::Cls));
    }
}
