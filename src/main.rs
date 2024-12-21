#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use std::fs;

use anyhow::Result;
use clap::Parser;
use processor::Processor;

mod instructions;
mod memory;
mod processor;
mod registers;
mod stack;

/// A CHIP-8 emulator.
#[derive(Parser)]
struct Args {
    program_path: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    Processor::from_program(&fs::read(args.program_path)?)?.run()?;

    Ok(())
}
