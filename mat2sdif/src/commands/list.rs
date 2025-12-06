//! List variables command (--list mode).

use anyhow::{Context, Result};
use colored::Colorize;

use sdif_rs::MatFile;

use crate::cli::Args;
use crate::output;

/// Run the list command.
pub fn run(args: &Args) -> Result<()> {
    output::print_verbose(
        &format!("Opening MAT file: {}", args.input.display()),
        args.verbose,
    );

    let mat = MatFile::open(&args.input)
        .with_context(|| format!("Failed to open MAT file: {}", args.input.display()))?;

    if mat.is_empty() {
        output::print_warning("No numeric variables found in MAT file");
        println!("\nNote: mat2sdif only supports numeric arrays.");
        println!("Cell arrays, structs, and sparse matrices are not supported.");
        return Ok(());
    }

    // Print header
    println!("{}", format!("Variables in '{}':", args.input.display()).bold());
    println!();

    // Collect and sort variable names
    let mut vars: Vec<_> = mat.iter().collect();
    vars.sort_by_key(|(name, _)| name.to_lowercase());

    // Calculate column widths
    let max_name = vars.iter().map(|(n, _)| n.len()).max().unwrap_or(4).max(4);

    // Print header row
    println!(
        "  {:<width$}  {:>14}  {:>10}  {}",
        "Name", "Shape", "Type", "Notes",
        width = max_name
    );
    println!(
        "  {:-<width$}  {:->14}  {:->10}  -----",
        "", "", "",
        width = max_name
    );

    // Print each variable
    for (name, data) in vars {
        let shape = format!("{:?}", data.shape());
        let dtype = if data.is_complex() {
            format!("{} (complex)", data.dtype())
        } else {
            data.dtype().to_string()
        };

        let mut notes = Vec::new();

        if data.is_likely_time_vector() {
            notes.push("time vector?".green().to_string());
        }

        if data.is_1d() {
            notes.push("1D".dimmed().to_string());
        }

        if let Some(stats) = data.time_stats() {
            if stats.is_regular {
                notes.push(format!("hop={:.1}ms", stats.mean_hop * 1000.0).dimmed().to_string());
            }
        }

        println!(
            "  {:<width$}  {:>14}  {:>10}  {}",
            name,
            shape,
            dtype,
            notes.join(", "),
            width = max_name
        );
    }

    // Print summary
    println!();
    println!("{} numeric variables found", mat.len());

    // Print auto-detection hints
    let time_vars = mat.find_time_vectors();
    if !time_vars.is_empty() {
        println!();
        if time_vars.len() == 1 {
            println!(
                "{}: '{}' looks like a time vector",
                "hint".cyan(),
                time_vars[0]
            );
        } else {
            println!(
                "{}: Multiple possible time vectors found: {:?}",
                "hint".cyan(),
                time_vars
            );
            println!("      Use --time-var to specify which one to use.");
        }
    }

    // Find potential data variables
    let data_vars: Vec<_> = mat.iter()
        .filter(|(_, v)| v.is_2d() && !v.is_likely_time_vector())
        .map(|(n, _)| n)
        .collect();

    if !data_vars.is_empty() && data_vars.len() <= 3 {
        println!(
            "{}: Potential data variables: {:?}",
            "hint".cyan(),
            data_vars
        );
    }

    Ok(())
}
