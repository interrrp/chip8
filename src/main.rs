#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use std::fs;

use anyhow::Result;
use clap::Parser;
use emulator::Emulator;

mod emulator;
mod instructions;
mod memory;
mod registers;
mod stack;

/// A CHIP-8 emulator.
#[derive(Parser)]
struct Args {
    program_path: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    Emulator::from_program(&fs::read(args.program_path)?)?.run()?;

    Ok(())
}
