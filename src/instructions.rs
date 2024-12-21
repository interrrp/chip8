use anyhow::{anyhow, Result};

/// An enumeration of all CHIP-8 instructions.
///
/// This is based on [Cowgod's CHIP-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM).
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum Instruction {
    /// Jump to location `addr`.
    Jp { addr: u16 },
    /// Jump to location nnn + V0.
    JpV0 { addr: u16 },

    /// Call subroutine at `addr`.
    Call { addr: u16 },
    /// Return from a subroutine.
    Ret,

    /// Skip next instruction if Vx = kk.
    SeVxByte { vx: u16, byte: u16 },
    /// Skip next instruction if Vx != kk.
    SneVxByte { vx: u16, byte: u16 },
    /// Skip next instruction if Vx = Vy.
    SeVxVy { vx: u16, vy: u16 },
    /// Skip next instruction if Vx != Vy.
    SneVxVy { vx: u16, vy: u16 },
    /// Skip next instruction if key with the value of Vx is pressed.
    Skp { vx: u16 },
    /// Skip next instruction if key with the value of Vx is not pressed.
    Sknp { vx: u16 },

    /// Set Vx = byte.
    LdVxByte { vx: u16, byte: u16 },
    /// Set Vx = Vy.
    LdVxVy { vx: u16, vy: u16 },
    /// Set I = addr.
    LdIAddr { addr: u16 },
    /// Wait for a key press, store the value of the key in Vx.
    LdK { vx: u16 },
    /// Set delay timer = Vx.
    LdDt { vx: u16 },
    /// Set sound timer = Vx.
    LdSt { vx: u16 },
    /// Set I = location of sprite for digit Vx.
    LdF { vx: u16 },
    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    LdB { vx: u16 },
    /// Store registers V0 through Vx in memory starting at location I.
    LdI { vx: u16 },
    /// Read registers V0 through Vx from memory starting at location I.
    LdIVx { vx: u16 },

    /// Set Vx = random byte AND kk.
    Rnd { vx: u16, byte: u16 },

    /// Set Vx = Vx + kk.
    AddVxByte { vx: u16, byte: u16 },
    /// Set Vx = Vx + Vy, set VF = carry.
    AddVxVy { vx: u16, vy: u16 },
    /// Set I = I + Vx.
    AddI { vx: u16 },
    /// Set Vx = Vx - Vy, set VF = NOT borrow.
    Sub { vx: u16, vy: u16 },
    /// Set Vx = Vy - Vx, set VF = NOT borrow.
    Subn { vx: u16, vy: u16 },

    /// Set Vx = Vx AND Vy.
    And { vx: u16, vy: u16 },
    /// Set Vx = Vx OR Vy.
    Or { vx: u16, vy: u16 },
    /// Set Vx = Vx XOR Vy.
    Xor { vx: u16, vy: u16 },

    /// Set Vx = Vx SHR 1.
    Shr { vx: u16 },
    /// Set Vx = Vx SHL 1.
    Shl { vx: u16 },

    /// Clear the display.
    Cls,
    /// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    Drw { vx: u16, vy: u16, nibble: u16 },
}

pub(crate) fn decode_instruction(op: u16) -> Result<Instruction> {
    let vx = (op & 0x0F00) >> 8;
    let vy = (op & 0x00F0) >> 4;
    let byte = op & 0x00FF;
    let addr = op & 0x0FFF;
    let e = op & 0x000F;

    Ok(match op & 0xF000 {
        0x0000 if op == 0x00E0 => Instruction::Cls,
        0x0000 if op == 0x00EE => Instruction::Ret,

        0x1000 => Instruction::Jp { addr },
        0x2000 => Instruction::Call { addr },
        0x3000 => Instruction::SeVxByte { vx, byte },
        0x4000 => Instruction::SneVxByte { vx, byte },
        0x5000 => Instruction::SeVxVy { vx, vy },
        0x6000 => Instruction::LdVxByte { vx, byte },
        0x7000 => Instruction::AddVxByte { vx, byte },

        0x8000 if e == 0 => Instruction::LdVxVy { vx, vy },
        0x8000 if e == 1 => Instruction::Or { vx, vy },
        0x8000 if e == 2 => Instruction::And { vx, vy },
        0x8000 if e == 3 => Instruction::Xor { vx, vy },
        0x8000 if e == 4 => Instruction::AddVxVy { vx, vy },
        0x8000 if e == 5 => Instruction::Sub { vx, vy },
        0x8000 if e == 6 => Instruction::Shr { vx },
        0x8000 if e == 7 => Instruction::Subn { vx, vy },
        0x8000 if e == 0xE => Instruction::Shl { vx },

        0x9000 => Instruction::SneVxVy { vx, vy },
        0xA000 => Instruction::LdIAddr { addr },
        0xB000 => Instruction::Jp { addr },
        0xC000 => Instruction::Rnd { vx, byte },
        0xD000 => Instruction::Drw { vx, vy, nibble: e },

        0xE000 if byte == 0x9E => Instruction::Skp { vx },
        0xE000 if byte == 0xA1 => Instruction::Sknp { vx },
        0xF000 if byte == 0x07 => Instruction::LdDt { vx },
        0xF000 if byte == 0x0A => Instruction::LdK { vx },
        0xF000 if byte == 0x15 => Instruction::LdDt { vx },
        0xF000 if byte == 0x18 => Instruction::LdSt { vx },
        0xF000 if byte == 0x1E => Instruction::AddI { vx },
        0xF000 if byte == 0x29 => Instruction::LdF { vx },
        0xF000 if byte == 0x33 => Instruction::LdB { vx },
        0xF000 if byte == 0x55 => Instruction::LdI { vx },
        0xF000 if byte == 0x65 => Instruction::LdIVx { vx },

        _ => return Err(anyhow!("Unknown instruction (opcode {:#x})", op)),
    })
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_arg() -> Result<()> {
        assert_eq!(decode_instruction(0x00E0)?, Instruction::Cls);
        assert_eq!(decode_instruction(0x00EE)?, Instruction::Ret);
        Ok(())
    }

    #[test]
    fn addr_args() -> Result<()> {
        assert_eq!(decode_instruction(0x1abc)?, Instruction::Jp { addr: 0xabc });
        assert_eq!(decode_instruction(0x2def)?, Instruction::Call { addr: 0xdef });
        Ok(())
    }

    #[test]
    fn byte_args() -> Result<()> {
        assert_eq!(
            decode_instruction(0x3abc)?,
            Instruction::SeVxByte { vx: 0xa, byte: 0xbc }
        );
        assert_eq!(
            decode_instruction(0x4def)?,
            Instruction::SneVxByte { vx: 0xd, byte: 0xef }
        );
        Ok(())
    }

    #[test]
    fn vx_vy_args() -> Result<()> {
        assert_eq!(decode_instruction(0x8ab0)?, Instruction::LdVxVy { vx: 0xa, vy: 0xb });
        assert_eq!(decode_instruction(0x8cd1)?, Instruction::Or { vx: 0xc, vy: 0xd });
        Ok(())
    }

    #[test]
    fn f_prefix() -> Result<()> {
        assert_eq!(decode_instruction(0xFA07)?, Instruction::LdDt { vx: 0xA });
        assert_eq!(decode_instruction(0xFB0A)?, Instruction::LdK { vx: 0xB });
        assert_eq!(decode_instruction(0xFC15)?, Instruction::LdDt { vx: 0xC });
        Ok(())
    }

    #[test]
    fn e_prefix() -> Result<()> {
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
