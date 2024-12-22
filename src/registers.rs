use std::ops::{Index, IndexMut};

const NUM_REGISTERS: usize = 16;

/// The CPU registers.
///
/// > Chip-8 has 16 general purpose 8-bit registers, usually referred to as Vx, where x is a hexadecimal digit (0
/// > through F). There is also a 16-bit register called I. This register is generally used to store memory addresses,
/// > so only the lowest (rightmost) 12 bits are usually used.
/// >
/// > [_Cowgod's CHIP-8 Technical Reference, section 2.2_](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.2)
pub struct Registers {
    registers: [u8; NUM_REGISTERS],

    /// The special 16-bit `I` register.
    ///
    /// This is generally used to store memory addresses.
    pub i: usize,
}

impl Registers {
    /// Return a `Registers` instance with every register set to 0.
    pub fn new() -> Registers {
        Registers {
            registers: [0; 16],
            i: 0,
        }
    }
}

impl Index<usize> for Registers {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        &self.registers[index]
    }
}

impl IndexMut<usize> for Registers {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        &mut self.registers[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let registers = Registers::new();

        // Ensure that every register has a value of 0
        for i in 0..NUM_REGISTERS {
            assert_eq!(registers.registers[i], 0);
        }

        // ...including I
        assert_eq!(registers.i, 0);
    }

    #[test]
    fn get_set() {
        let mut registers = Registers::new();

        // Ensure that the Index and IndexMut implementations work as intended
        registers[1] = 2;
        registers[2] = 4;
        assert_eq!(registers[1], 2);
        assert_eq!(registers[2], 4);
    }
}
