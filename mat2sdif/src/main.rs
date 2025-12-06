//! mat2sdif - Convert MATLAB/Octave .mat files to SDIF format.
//!
//! This tool reads numeric arrays from MAT files and converts them to
//! SDIF format for use with Max/MSP, AudioSculpt, and other audio software.

mod cli;
mod commands;
mod max_compat;
mod output;

use anyhow::Result;
use clap::Parser;

use cli::Args;

fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Run the appropriate command
    if let Err(e) = run(args) {
        output::print_error(&e);
        std::process::exit(1);
    }
}

/// Main dispatch function.
fn run(args: Args) -> Result<()> {
    // Validate arguments
    args.validate().map_err(|e| anyhow::anyhow!("{}", e))?;

    // Dispatch to appropriate command
    if args.list {
        commands::list::run(&args)
    } else if args.dry_run {
        commands::validate::run(&args)
    } else {
        commands::convert::run(&args)
    }
}
