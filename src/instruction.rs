/// An enumeration of all CHIP-8 instructions.
///
/// This is based on [Cowgod's CHIP-8 Technical
/// Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM).
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    /// Clear the display.
    Cls,

    /// Return from a subroutine.
    ///
    /// The interpreter sets the program counter to the address at the top of
    /// the stack, then subtracts 1 from the stack pointer.
    Ret,

    /// Jump to location `addr`.
    ///
    /// The interpreter sets the program counter to `addr`.
    Jp { addr: u16 },

    /// Call subroutine at `addr`.
    ///
    /// The interpreter increments the stack pointer, then puts the current PC
    /// on the top of the stack. The PC is then set to `addr`.
    Call { addr: u16 },

    /// Skip next instruction if Vx = kk.
    SeVxByte { vx: u16, byte: u16 },

    /// Skip next instruction if Vx != kk.
    SneVxByte { vx: u16, byte: u16 },

    /// Skip next instruction if Vx = Vy.
    SeVxVy { vx: u16, vy: u16 },

    /// Set Vx = kk.
    LdVxByte { vx: u16, byte: u16 },

    /// Set Vx = Vx + kk.
    AddVxByte { vx: u16, byte: u16 },

    /// Set Vx = Vy.
    LdVxVy { vx: u16, vy: u16 },

    /// Set Vx = Vx OR Vy.
    OrVxVy { vx: u16, vy: u16 },

    /// Set Vx = Vx AND Vy.
    AndVxVy { vx: u16, vy: u16 },

    /// Set Vx = Vx XOR Vy.
    XorVxVy { vx: u16, vy: u16 },

    /// Set Vx = Vx + Vy, set VF = carry.
    AddVxVy { vx: u16, vy: u16 },

    /// Set Vx = Vx - Vy, set VF = NOT borrow.
    SubVxVy { vx: u16, vy: u16 },

    /// Set Vx = Vx SHR 1.
    ShrVx { vx: u16 },

    /// Set Vx = Vy - Vx, set VF = NOT borrow.
    SubnVxVy { vx: u16, vy: u16 },

    /// Set Vx = Vx SHL 1.
    ShlVx { vx: u16 },

    /// Skip next instruction if Vx != Vy.
    SneVxVy { vx: u16, vy: u16 },

    /// Set I = addr.
    LdIAddr { addr: u16 },

    /// Jump to location nnn + V0.
    JpV0Addr { addr: u16 },

    /// Set Vx = random byte AND kk.
    Rnd { vx: u16, byte: u16 },

    /// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    Drw { vx: u16, vy: u16, nibble: u16 },

    /// Skip next instruction if key with the value of Vx is pressed.
    SkpVx { vx: u16 },

    /// Skip next instruction if key with the value of Vx is not pressed.
    SknpVx { vx: u16 },

    /// Set Vx = delay timer value.
    LdVxDt { vx: u16 },

    /// Wait for a key press, store the value of the key in Vx.
    LdVxK { vx: u16 },

    /// Set delay timer = Vx.
    LdDtVx { vx: u16 },

    /// Set sound timer = Vx.
    LdStVx { vx: u16 },

    /// Set I = I + Vx.
    AddIVx { vx: u16 },

    /// Set I = location of sprite for digit Vx.
    LdFVx { vx: u16 },

    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    LdBVx { vx: u16 },

    /// Store registers V0 through Vx in memory starting at location I.
    LdIVx { vx: u16 },

    /// Read registers V0 through Vx from memory starting at location I.
    LdVxI { vx: u16 },
}

pub(crate) fn parse_instructions(opcodes: &[u16]) -> Vec<Instruction> {
    let mut instructions = Vec::new();

    for op in opcodes {
        let vx = (op & 0x0F00) >> 8;
        let vy = (op & 0x00F0) >> 4;
        let byte = op & 0x00FF;
        let addr = op & 0x0FFF;
        let e = op & 0x000F;

        let instruction = match op & 0xF000 {
            0x0000 if *op == 0x00E0 => Instruction::Cls,
            0x0000 if *op == 0x00EE => Instruction::Ret,

            0x1000 => Instruction::Jp { addr },
            0x2000 => Instruction::Call { addr },
            0x3000 => Instruction::SeVxByte { vx, byte },
            0x4000 => Instruction::SneVxByte { vx, byte },
            0x5000 => Instruction::SeVxVy { vx, vy },
            0x6000 => Instruction::LdVxByte { vx, byte },
            0x7000 => Instruction::AddVxByte { vx, byte },

            0x8000 if e == 0 => Instruction::LdVxVy { vx, vy },
            0x8000 if e == 1 => Instruction::OrVxVy { vx, vy },
            0x8000 if e == 2 => Instruction::AndVxVy { vx, vy },
            0x8000 if e == 3 => Instruction::XorVxVy { vx, vy },
            0x8000 if e == 4 => Instruction::AddVxVy { vx, vy },
            0x8000 if e == 5 => Instruction::SubVxVy { vx, vy },
            0x8000 if e == 6 => Instruction::ShrVx { vx },
            0x8000 if e == 7 => Instruction::SubnVxVy { vx, vy },
            0x8000 if e == 0xE => Instruction::ShlVx { vx },

            0x9000 => Instruction::SneVxVy { vx, vy },
            0xA000 => Instruction::LdIAddr { addr },
            0xB000 => Instruction::Jp { addr },
            0xC000 => Instruction::Rnd { vx, byte },
            0xD000 => Instruction::Drw { vx, vy, nibble: e },

            0xE000 if byte == 0x9E => Instruction::SkpVx { vx },
            0xE000 if byte == 0xA1 => Instruction::SknpVx { vx },
            0xF000 if byte == 0x07 => Instruction::LdVxDt { vx },
            0xF000 if byte == 0x0A => Instruction::LdVxK { vx },
            0xF000 if byte == 0x15 => Instruction::LdDtVx { vx },
            0xF000 if byte == 0x18 => Instruction::LdStVx { vx },
            0xF000 if byte == 0x1E => Instruction::AddIVx { vx },
            0xF000 if byte == 0x29 => Instruction::LdFVx { vx },
            0xF000 if byte == 0x33 => Instruction::LdBVx { vx },
            0xF000 if byte == 0x55 => Instruction::LdIVx { vx },
            0xF000 if byte == 0x65 => Instruction::LdVxI { vx },

            _ => continue,
        };
        instructions.push(instruction);
    }

    instructions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_instructions_with_no_args() {
        assert_eq!(
            parse_instructions(&[0x00E0, 0x00EE]),
            vec![Instruction::Cls, Instruction::Ret],
        );
    }

    #[test]
    fn load_instructions_with_addr_arg() {
        assert_eq!(
            parse_instructions(&[0x1abc, 0x2def]),
            vec![Instruction::Jp { addr: 0xabc }, Instruction::Call { addr: 0xdef }],
        );
    }

    #[test]
    fn load_instructions_with_vx_byte_args() {
        assert_eq!(
            parse_instructions(&[0x3abc, 0x4def]),
            vec![
                Instruction::SeVxByte { vx: 0xa, byte: 0xbc },
                Instruction::SneVxByte { vx: 0xd, byte: 0xef },
            ],
        );
    }

    #[test]
    fn load_instructions_with_vx_vy_args() {
        assert_eq!(
            parse_instructions(&[0x8ab0, 0x8cd1]),
            vec![
                Instruction::LdVxVy { vx: 0xa, vy: 0xb },
                Instruction::OrVxVy { vx: 0xc, vy: 0xd },
            ],
        );
    }

    #[test]
    fn load_instructions_with_f_prefix() {
        assert_eq!(
            parse_instructions(&[0xFA07, 0xFB0A, 0xFC15]),
            vec![
                Instruction::LdVxDt { vx: 0xA },
                Instruction::LdVxK { vx: 0xB },
                Instruction::LdDtVx { vx: 0xC },
            ],
        );
    }

    #[test]
    fn load_instructions_with_e_prefix() {
        assert_eq!(
            parse_instructions(&[0xEA9E, 0xEBA1]),
            vec![Instruction::SkpVx { vx: 0xA }, Instruction::SknpVx { vx: 0xB },],
        );
    }

    #[test]
    fn load_draw_instruction() {
        assert_eq!(
            parse_instructions(&[0xD123]),
            vec![Instruction::Drw {
                vx: 0x1,
                vy: 0x2,
                nibble: 0x3
            }],
        );
    }
}
