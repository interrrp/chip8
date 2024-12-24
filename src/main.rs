#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use std::fs;

use anyhow::Result;
use clap::Parser;

use crate::emulator::Emulator;

mod display;
mod emulator;
mod instructions;
mod memory;
mod registers;

/// A CHIP-8 emulator.
#[derive(Parser, Debug, Clone)]
struct Args {
    /// Path to the program.
    ///
    /// This typically ends in `.ch8`.
    program_path: String,
}

fn main() -> Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let args = Args::parse();

    let program = fs::read(args.program_path)?;
    let mut emulator = Emulator::from_program(&program)?;
    emulator.run()?;

    Ok(())
}
