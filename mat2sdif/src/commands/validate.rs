//! Dry-run validation command.

use anyhow::{Context, Result, bail};
use colored::Colorize;

use sdif_rs::{MatFile, MatToSdifConfig, MatToSdifConverter};

use crate::cli::Args;
use crate::max_compat;
use crate::output;

/// Run the validate (dry-run) command.
pub fn run(args: &Args) -> Result<()> {
    output::print_info(
        &format!("{} (no files will be written)\n", "Dry run mode".yellow()),
        args.quiet,
    );

    // Load MAT file
    output::print_verbose(
        &format!("Opening MAT file: {}", args.input.display()),
        args.verbose,
    );

    let mat = MatFile::open(&args.input)
        .with_context(|| format!("Failed to open MAT file: {}", args.input.display()))?;

    if mat.is_empty() {
        bail!("No numeric variables found in MAT file");
    }

    println!("{}", "MAT File Analysis".bold().underline());
    println!();
    output::print_kv("File", &args.input.display().to_string(), 2);
    output::print_kv("Variables", &mat.len().to_string(), 2);

    // Build configuration
    let config = crate::commands::convert::build_config(args)?;

    // Create converter (validates variables)
    let converter = MatToSdifConverter::new(&mat, config)
        .context("Failed to set up conversion")?;

    println!();
    println!("{}", "Conversion Plan".bold().underline());
    println!();

    let num_frames = converter.num_frames();
    let (time_start, time_end) = converter.time_range();
    let cols_per_frame = converter.cols_per_frame();

    output::print_kv("Frames to write", &output::format_number(num_frames), 2);
    output::print_kv("Time range", &format!("{:.3}s to {:.3}s", time_start, time_end), 2);
    output::print_kv("Duration", &output::format_duration(time_end - time_start), 2);
    output::print_kv("Columns per frame", &cols_per_frame.to_string(), 2);

    println!();
    println!("{}", "SDIF Output".bold().underline());
    println!();

    if let Some(ref output) = args.output {
        output::print_kv("Output file", &output.display().to_string(), 2);
    } else {
        output::print_kv("Output file", "(not specified)", 2);
    }
    output::print_kv("Frame type", &args.frame_type, 2);
    output::print_kv("Matrix type", &args.matrix_type, 2);
    output::print_kv("Columns", &args.get_columns().join(", "), 2);
    output::print_kv("Max partials", &args.max_partials.to_string(), 2);

    // Max compatibility validation
    println!();
    println!("{}", "Compatibility Checks".bold().underline());
    println!();

    let warnings = max_compat::check_all(args, &converter);

    if warnings.is_empty() {
        println!("  {} All checks passed", "✓".green());
    } else {
        for warning in &warnings {
            println!("  {} {}", "⚠".yellow(), warning);
        }
    }

    // Estimate output size
    println!();
    println!("{}", "Estimates".bold().underline());
    println!();

    let estimated_bytes = estimate_output_size(num_frames, cols_per_frame, args);
    output::print_kv("Estimated output size", &output::format_size(estimated_bytes), 2);

    // Final verdict
    println!();
    if warnings.is_empty() {
        output::print_success("Validation passed - ready to convert", args.quiet);
        println!();
        println!(
            "Run without {} to perform the conversion.",
            "--dry-run".cyan()
        );
    } else {
        output::print_warning(&format!(
            "Validation completed with {} warning(s)",
            warnings.len()
        ));
        println!();
        println!(
            "Run without {} to convert anyway, or address the warnings first.",
            "--dry-run".cyan()
        );
    }

    Ok(())
}

/// Estimate output file size.
fn estimate_output_size(frames: usize, cols: usize, args: &Args) -> u64 {
    // SDIF overhead estimates:
    // - File header: ~100 bytes
    // - ASCII chunks (NVT, types): ~500 bytes
    // - Per frame: header (24 bytes) + matrix header (16 bytes) + data + padding

    let header_overhead: u64 = 600;
    let frame_overhead: u64 = 24 + 16 + 8; // frame header + matrix header + padding

    // Data size per frame (assuming f64)
    let rows_per_frame = if args.max_partials > 0 {
        args.max_partials.min(100) // Rough estimate
    } else {
        100
    };

    let data_per_frame = (rows_per_frame * cols * 8) as u64;

    header_overhead + (frames as u64) * (frame_overhead + data_per_frame)
}
