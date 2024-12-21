use anyhow::Result;
use clap::Parser;

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
    Ok(())
}
