use anyhow::{anyhow, Result};

const STACK_SIZE: usize = 16;

/// The CPU stack, used for storing return addresses during a subroutine.
///
/// > The stack is an array of 16 16-bit values, used to store the address that
/// > the interpreter should return to when finished with a subroutine. Chip-8
/// > allows for up to 16 levels of nested subroutines.
/// >
/// > [_Cowgod's CHIP-8 Technical Reference, section
/// > 2.2_](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.2)
#[derive(Debug, Clone, Copy)]
pub struct Stack {
    stack: [usize; STACK_SIZE],
    pointer: usize,
}

impl Stack {
    /// Return an empty call stack.
    pub fn new() -> Stack {
        Stack {
            stack: [0; STACK_SIZE],
            pointer: 0,
        }
    }

    /// Push an address onto the call stack.
    ///
    /// This is called when entering a subroutine (`CALL`).
    pub fn push(&mut self, address: usize) -> Result<()> {
        if self.pointer >= STACK_SIZE {
            return Err(anyhow!("Stack overflow (tried to push {address:#x})"));
        }

        self.stack[self.pointer] = address;
        self.pointer += 1;

        Ok(())
    }

    /// Pop an address off the stack, and return the popped address.
    ///
    /// This is called when exiting a subroutine (`RET`), and the program
    /// counter is set to the popped address.
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
    fn new() {
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
    fn overflow_underflow_error() {
        let mut stack = Stack::new();
        for _ in 0..STACK_SIZE {
            stack.push(7).unwrap();
        }
        assert!(stack.push(7).is_err());

        let mut stack = Stack::new();
        assert!(stack.pop().is_err());
    }
}
