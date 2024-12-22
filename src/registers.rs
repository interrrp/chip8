use std::ops::{Index, IndexMut};

/// The CPU registers.
///
/// > Chip-8 has 16 general purpose 8-bit registers, usually referred to as Vx, where x is a hexadecimal digit (0
/// > through F). There is also a 16-bit register called I. This register is generally used to store memory addresses,
/// > so only the lowest (rightmost) 12 bits are usually used.
/// >
/// > [_Cowgod's CHIP-8 Technical Reference, section 2.2_](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.2)
pub(crate) struct Registers {
    registers: [u8; 16],
    pub i: usize,
}

impl Registers {
    /// Return a new `Registers` instance with every register's value as 0.
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
    fn get_set() {
        let mut registers = Registers::new();

        registers[1] = 2;
        registers[2] = 4;
        assert_eq!(registers[1], 2);
        assert_eq!(registers[2], 4);
    }
}
