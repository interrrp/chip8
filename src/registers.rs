use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct Registers {
    data: [u8; 16],
    pub i: usize,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            data: [0; 16],
            i: 0,
        }
    }
}

impl Index<usize> for Registers {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        &self.data[index]
    }
}

impl IndexMut<usize> for Registers {
    fn index_mut(&mut self, index: usize) -> &mut u8 {
        &mut self.data[index]
    }
}
