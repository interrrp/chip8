use crate::instruction::Instruction;

pub(crate) struct Chip8<'a> {
    pub registers: Box<&'a [u8; 15]>,
    pub stack: Box<&'a [u16; 16]>,
    pub program: Vec<Instruction>,
    pub pc: u16,
}

impl<'a> Chip8<'a> {
    pub fn new() -> Chip8<'a> {
        Chip8 {
            registers: Box::new(&[0; 15]),
            stack: Box::new(&[0; 16]),
            program: Vec::new(),
            pc: 0,
        }
    }

    pub fn load_program(&mut self, program: Vec<Instruction>) {
        self.program = program;
    }

    pub fn run(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_contains_loaded_program() {
        let program = vec![Instruction::Call { addr: 0xabc }];

        let mut chip8 = Chip8::new();
        chip8.load_program(program.clone());
        assert_eq!(chip8.program, program);
    }
}
