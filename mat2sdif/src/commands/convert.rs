//! Main conversion command.

use std::time::Instant;

use anyhow::{Context, Result, bail};
use colored::Colorize;

use sdif_rs::{MatFile, MatToSdifConfig, MatToSdifConverter, ComplexMode, SdifFile};

use crate::cli::{Args, ComplexModeArg};
use crate::max_compat;
use crate::output::{self, ProgressReporter};

/// Run the convert command.
pub fn run(args: &Args) -> Result<()> {
    let start_time = Instant::now();

    // Get output path (validated in Args::validate)
    let output_path = args.output.as_ref().unwrap();

    output::print_verbose(
        &format!("Opening MAT file: {}", args.input.display()),
        args.verbose,
    );

    // Load MAT file
    let mat = MatFile::open(&args.input)
        .with_context(|| format!("Failed to open MAT file: {}", args.input.display()))?;

    if mat.is_empty() {
        bail!("No numeric variables found in MAT file");
    }

    output::print_verbose(
        &format!("Found {} variables", mat.len()),
        args.verbose,
    );

    // Build configuration
    let config = build_config(args)?;

    // Create converter
    let converter = MatToSdifConverter::new(&mat, config)
        .context("Failed to set up conversion")?;

    let num_frames = converter.num_frames();
    let (time_start, time_end) = converter.time_range();

    output::print_verbose(
        &format!("Converting {} frames ({:.3}s to {:.3}s)",
            num_frames, time_start, time_end),
        args.verbose,
    );

    // Max compatibility checks
    if args.max_compat {
        max_compat::validate_config(args, &converter)?;
    }

    // Create SDIF writer
    let columns_strings = args.get_columns();
    let columns: Vec<&str> = columns_strings.iter().map(|s| s.as_str()).collect();
    let component = format!("{} Data", args.matrix_type);

    let mut writer = SdifFile::builder()
        .create(output_path)
        .with_context(|| format!("Failed to create output file: {}", output_path.display()))?
        .add_nvt([
            ("creator", "mat2sdif"),
            ("source", args.input.to_str().unwrap_or("unknown")),
        ])?
        .add_matrix_type(&args.matrix_type, &columns)?
        .add_frame_type(&args.frame_type, &[&component])?
        .build()
        .context("Failed to initialize SDIF file")?;

    // Progress reporter
    let progress = ProgressReporter::new(num_frames, args.verbose);

    // Write frames
    converter.write_to(&mut writer)
        .context("Failed to write frames")?;

    progress.finish();

    // Close file
    writer.close()
        .context("Failed to close output file")?;

    // Print summary
    let elapsed = start_time.elapsed();

    if !args.quiet {
        print_summary(args, num_frames, time_end - time_start, elapsed);
    }

    Ok(())
}

/// Build MatToSdifConfig from command line arguments.
pub(crate) fn build_config(args: &Args) -> Result<MatToSdifConfig> {
    let mut config = MatToSdifConfig::new()
        .frame_type(&args.frame_type)
        .matrix_type(&args.matrix_type)
        .columns(&args.get_columns().iter().map(|s| s.as_str()).collect::<Vec<_>>())
        .stream_id(args.stream_id)
        .transpose(args.transpose);

    // Set max partials (0 = no limit)
    if args.max_partials > 0 {
        config = config.max_partials(args.max_partials);
    } else {
        config = config.no_partial_limit();
    }

    // Set time variable if specified
    if let Some(ref tv) = args.time_var {
        config = config.time_var(tv);
    }

    // Set data variable if specified
    if let Some(ref dv) = args.data_var {
        config = config.data_var(dv);
    }

    // Set complex mode
    config = config.complex_mode(match args.complex_mode {
        ComplexModeArg::Real => ComplexMode::RealOnly,
        ComplexModeArg::Magnitude => ComplexMode::Magnitude,
        ComplexModeArg::MagPhase => ComplexMode::MagnitudePhase,
        ComplexModeArg::ReIm => ComplexMode::RealImag,
    });

    Ok(config)
}

/// Print conversion summary.
fn print_summary(args: &Args, frames: usize, duration: f64, elapsed: std::time::Duration) {
    println!();
    output::print_success(
        &format!("Converted {} to {}",
            args.input.display(),
            args.output.as_ref().unwrap().display()
        ),
        false,
    );

    println!();
    output::print_kv("Frames written", &output::format_number(frames), 2);
    output::print_kv("Audio duration", &output::format_duration(duration), 2);
    output::print_kv("Frame type", &args.frame_type, 2);
    output::print_kv("Processing time", &format!("{:.2?}", elapsed), 2);

    // Performance stat
    if elapsed.as_secs_f64() > 0.001 {
        let fps = frames as f64 / elapsed.as_secs_f64();
        output::print_kv("Speed", &format!("{:.0} frames/sec", fps), 2);
    }
}
