use anyhow::{anyhow, Result};

/// An enumeration of all CHIP-8 instructions.
///
/// This is based on [Cowgod's CHIP-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM).
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
    SeVxByte { vx: usize, byte: u8 },
    /// Skip next instruction if Vx != kk.
    SneVxByte { vx: usize, byte: u8 },
    /// Skip next instruction if Vx = Vy.
    SeVxVy { vx: usize, vy: usize },
    /// Skip next instruction if Vx != Vy.
    SneVxVy { vx: usize, vy: usize },
    /// Skip next instruction if key with the value of Vx is pressed.
    Skp { vx: usize },
    /// Skip next instruction if key with the value of Vx is not pressed.
    Sknp { vx: usize },

    /// Set Vx = byte.
    LdVxByte { vx: usize, byte: u8 },
    /// Set Vx = Vy.
    LdVxVy { vx: usize, vy: usize },
    /// Set I = addr.
    LdIAddr { addr: usize },
    /// Wait for a key press, store the value of the key in Vx.
    LdK { vx: usize },
    /// Set delay timer = Vx.
    LdDt { vx: usize },
    /// Set sound timer = Vx.
    LdSt { vx: usize },
    /// Set I = location of sprite for digit Vx.
    LdF { vx: usize },
    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    LdB { vx: usize },
    /// Store registers V0 through Vx in memory starting at location I.
    LdI { vx: usize },
    /// Read registers V0 through Vx from memory starting at location I.
    LdIVx { vx: usize },
    /// Set Vx = random byte AND kk.
    Rnd { vx: usize, byte: u8 },

    /// Set Vx = Vx + kk.
    AddVxByte { vx: usize, byte: u8 },
    /// Set Vx = Vx + Vy, set VF = carry.
    AddVxVy { vx: usize, vy: usize },
    /// Set I = I + Vx.
    AddI { vx: usize },
    /// Set Vx = Vx - Vy, set VF = NOT borrow.
    Sub { vx: usize, vy: usize },
    /// Set Vx = Vy - Vx, set VF = NOT borrow.
    Subn { vx: usize, vy: usize },

    /// Set Vx = Vx AND Vy.
    And { vx: usize, vy: usize },
    /// Set Vx = Vx OR Vy.
    Or { vx: usize, vy: usize },
    /// Set Vx = Vx XOR Vy.
    Xor { vx: usize, vy: usize },
    /// Set Vx = Vx SHR 1.
    Shr { vx: usize },
    /// Set Vx = Vx SHL 1.
    Shl { vx: usize },

    /// Clear the display.
    Cls,
    /// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    Drw { vx: usize, vy: usize, nibble: u8 },
}

/// Decode the instruction from an opcode.
pub fn decode_instruction(op: u16) -> Result<Instruction> {
    let vx = ((op & 0x0F00) >> 8) as usize;
    let vy = ((op & 0x00F0) >> 4) as usize;
    let byte = (op & 0x00FF) as u8;
    let addr = (op & 0x0FFF) as usize;
    let e = (op & 0x000F) as u8;

    Ok(match op & 0xF000 {
        0x0000 if op == 0x0000 => Instruction::Nop,
        0x1000 => Instruction::Jp { addr },
        0xB000 => Instruction::JpV0 { addr },

        0x2000 => Instruction::Call { addr },
        0x0000 if op == 0x00EE => Instruction::Ret,

        0x3000 => Instruction::SeVxByte { vx, byte },
        0x4000 => Instruction::SneVxByte { vx, byte },
        0x5000 => Instruction::SeVxVy { vx, vy },
        0x9000 => Instruction::SneVxVy { vx, vy },
        0xE000 if byte == 0x9E => Instruction::Skp { vx },
        0xE000 if byte == 0xA1 => Instruction::Sknp { vx },

        0x6000 => Instruction::LdVxByte { vx, byte },
        0x8000 if e == 0 => Instruction::LdVxVy { vx, vy },
        0xA000 => Instruction::LdIAddr { addr },
        0xF000 if byte == 0x07 => Instruction::LdDt { vx },
        0xF000 if byte == 0x0A => Instruction::LdK { vx },
        0xF000 if byte == 0x15 => Instruction::LdDt { vx },
        0xF000 if byte == 0x18 => Instruction::LdSt { vx },
        0xF000 if byte == 0x1E => Instruction::AddI { vx },
        0xF000 if byte == 0x29 => Instruction::LdF { vx },
        0xF000 if byte == 0x33 => Instruction::LdB { vx },
        0xF000 if byte == 0x55 => Instruction::LdI { vx },
        0xF000 if byte == 0x65 => Instruction::LdIVx { vx },
        0xC000 => Instruction::Rnd { vx, byte },

        0x8000 if e == 1 => Instruction::Or { vx, vy },
        0x8000 if e == 2 => Instruction::And { vx, vy },
        0x8000 if e == 3 => Instruction::Xor { vx, vy },
        0x8000 if e == 6 => Instruction::Shr { vx },
        0x8000 if e == 0xE => Instruction::Shl { vx },

        0x7000 => Instruction::AddVxByte { vx, byte },
        0x8000 if e == 4 => Instruction::AddVxVy { vx, vy },
        0x8000 if e == 5 => Instruction::Sub { vx, vy },
        0x8000 if e == 7 => Instruction::Subn { vx, vy },

        0xD000 => Instruction::Drw { vx, vy, nibble: e },
        0x0000 if op == 0x00E0 => Instruction::Cls,

        _ => return Err(anyhow!("Unknown instruction (opcode {:#x})", op)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_no_args() -> Result<()> {
        assert_eq!(decode_instruction(0x00E0)?, Instruction::Cls);
        assert_eq!(decode_instruction(0x00EE)?, Instruction::Ret);
        Ok(())
    }

    #[test]
    fn with_addr_arg() -> Result<()> {
        assert_eq!(
            decode_instruction(0x1abc)?,
            Instruction::Jp { addr: 0xabc }
        );
        assert_eq!(
            decode_instruction(0x2def)?,
            Instruction::Call { addr: 0xdef }
        );
        Ok(())
    }

    #[test]
    fn with_byte_arg() -> Result<()> {
        assert_eq!(
            decode_instruction(0x3abc)?,
            Instruction::SeVxByte {
                vx: 0xa,
                byte: 0xbc
            }
        );
        assert_eq!(
            decode_instruction(0x4def)?,
            Instruction::SneVxByte {
                vx: 0xd,
                byte: 0xef
            }
        );
        Ok(())
    }

    #[test]
    fn with_vx_vy() -> Result<()> {
        assert_eq!(
            decode_instruction(0x8ab0)?,
            Instruction::LdVxVy { vx: 0xa, vy: 0xb }
        );
        assert_eq!(
            decode_instruction(0x8cd1)?,
            Instruction::Or { vx: 0xc, vy: 0xd }
        );
        Ok(())
    }

    #[test]
    fn with_f_prefix() -> Result<()> {
        assert_eq!(decode_instruction(0xFA07)?, Instruction::LdDt { vx: 0xA });
        assert_eq!(decode_instruction(0xFB0A)?, Instruction::LdK { vx: 0xB });
        assert_eq!(decode_instruction(0xFC15)?, Instruction::LdDt { vx: 0xC });
        Ok(())
    }

    #[test]
    fn with_e_prefix() -> Result<()> {
        assert_eq!(decode_instruction(0xEA9E)?, Instruction::Skp { vx: 0xA });
        assert_eq!(decode_instruction(0xEBA1)?, Instruction::Sknp { vx: 0xB });
        Ok(())
    }

    #[test]
    fn draw() -> Result<()> {
        assert_eq!(
            decode_instruction(0xD123)?,
            Instruction::Drw {
                vx: 0x1,
                vy: 0x2,
                nibble: 0x3
            }
        );
        Ok(())
    }
}
