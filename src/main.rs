use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::emulator::Emulator;

mod emulator;
mod memory;
mod registers;
mod window;

/// A tiny CHIP-8 emulator.
#[derive(Parser, Debug)]
struct Args {
    /// Path to the program to execute.
    ///
    /// This typically ends in `.ch8` or `.rom`.
    program_path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut emulator = Emulator::new()?;
    emulator.load_program_file(&args.program_path)?;
    emulator.run()?;

    Ok(())
}
