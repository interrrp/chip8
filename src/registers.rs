/// The CPU registers.
///
/// From _CHIP-8 Technical Reference (Matthew Mikolay)_:
///
/// > The CHIP-8 interpreter defines sixteen general purpose data registers, one for each
/// > hexadecimal digit: V0 to VF. Each data register is eight bits in length, capable of storing
/// > unsigned integer values between 0x00 and 0xFF inclusive.
///
/// > The 16-bit address register `I` is used with operations related to reading and writing memory.
/// > Though it is sixteen bits wide, it can only be loaded with a 12-bit memory address due to the
/// > range of memory accessible to CHIP-8 instructions.
#[derive(Debug, Clone, Copy)]
pub struct Registers {
    /// General-purpose data registers.
    ///
    /// From _CHIP-8 Technical Reference (Matthew Mikolay)_:
    ///
    /// > The data registers are the primary means of data manipulation provided by the CHIP-8
    /// > language. Using various CHIP-8 instructions, registers can be loaded with values, added,
    /// > subtracted, and more. While any register can be used for data manipulation, it should be
    /// > noted that the VF register is often modified by certain instructions to act as a flag.
    pub data: [u8; 16],

    /// The special 16-bit `I` register, used for addresses.
    ///
    /// From _CHIP-8 Technical Reference (Matthew Mikolay)_:
    ///
    /// > The 16-bit address register `I` is used with operations related to reading and writing
    /// > memory. Though it is sixteen bits wide, it can only be loaded with a 12-bit memory address
    /// > due to the range of memory accessible to CHIP-8 instructions.
    pub i: usize,
}

impl Registers {
    /// Return a `Registers` instance with every register set to 0.
    pub fn new() -> Registers {
        Registers {
            data: [0; 16],
            i: 0,
        }
    }
}
