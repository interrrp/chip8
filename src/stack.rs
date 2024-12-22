use anyhow::{anyhow, Result};

const STACK_SIZE: usize = 16;

/// The CPU stack, used for storing return addresses during a subroutine.
///
/// > The stack is an array of 16 16-bit values, used to store the address that the interpreter should return to when
/// > finished with a subroutine. Chip-8 allows for up to 16 levels of nested subroutines.
/// >
/// > [_Cowgod's CHIP-8 Technical Reference, section 2.2_](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.2)
pub(crate) struct Stack {
    stack: [usize; STACK_SIZE],
    pointer: usize,
}

impl Stack {
    pub fn new() -> Stack {
        Stack {
            stack: [0; STACK_SIZE],
            pointer: 0,
        }
    }

    pub fn push(&mut self, value: usize) -> Result<()> {
        if self.pointer >= STACK_SIZE {
            return Err(anyhow!("Stack overflow (capacity {STACK_SIZE}, tried to push {value})"));
        }

        self.stack[self.pointer] = value;
        self.pointer += 1;

        Ok(())
    }

    pub fn pop(&mut self) -> Result<usize> {
        if self.pointer == 0 {
            return Err(anyhow!("Stack underflow"));
        }

        self.pointer -= 1;
        Ok(self.stack[self.pointer])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_stack_zeroes() {
        let stack = Stack::new();
        for i in 0..STACK_SIZE {
            assert_eq!(stack.stack[i], 0);
        }
    }

    #[test]
    fn push_pop() -> Result<()> {
        let mut stack = Stack::new();
        stack.push(42)?;
        stack.push(7)?;
        assert_eq!(stack.pop()?, 7);
        assert_eq!(stack.pop()?, 42);
        Ok(())
    }

    #[test]
    fn out_of_bounds_error() {
        let mut stack = Stack::new();
        for _ in 0..STACK_SIZE {
            stack.push(7).unwrap();
        }
        assert!(stack.push(7).is_err());

        let mut stack = Stack::new();
        assert!(stack.pop().is_err());
    }
}
