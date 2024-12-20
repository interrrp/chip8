use crate::instruction::Instruction;

pub(crate) struct Chip8 {
    pub registers: [u8; 15],
    pub stack: [u16; 16],
    pub sp: usize,
    pub program: Vec<Instruction>,
    pub pc: usize,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            registers: [0; 15],
            stack: [0; 16],
            sp: 0,
            program: Vec::new(),
            pc: 0,
        }
    }

    pub fn load_program(&mut self, program: Vec<Instruction>) {
        self.program = program;
    }

    pub fn run(&mut self) {
        while self.pc < self.program.len() {
            self.do_instruction(self.program[self.pc]);
        }
    }

    fn do_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Jp { addr } => self.pc = addr as usize,
            _ => {}
        }

        // Only increment program counter if the instruction isn't a JP (jump)
        if !matches!(instruction, Instruction::Jp { .. }) {
            self.pc += 1;
        }
    }
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

    #[test]
    fn jp() {
        let mut chip8 = Chip8::new();
        chip8.load_program(vec![Instruction::Jp { addr: 0xa }]);

        assert_eq!(chip8.pc, 0);
        chip8.run();
        assert_eq!(chip8.pc, 0xa);
    }
}
