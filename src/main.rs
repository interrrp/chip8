use std::fs;

use anyhow::{Context, Result};
use clap::Parser;
use instruction::parse_instructions_from_opcodes;

mod instruction;

/// A CHIP-8 emulator.
#[derive(Parser)]
struct Args {
    program_path: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Read the opcodes from the program file.
    //
    // Since the program is initially read as a series of `u8`s, we chunk and turn them into a series of `u16`s for
    // `parse_instructions_from_opcodes`.
    let bytes = fs::read(&args.program_path).context(format!("Failed to read program: {}", &args.program_path))?;
    let opcodes: Vec<u16> = bytes
        .chunks(2)
        .filter(|chunk| chunk.len() == 2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect();
    let instructions = parse_instructions_from_opcodes(&opcodes);

    println!("{:?}", instructions);

    Ok(())
}
