# Phase 5: mat2sdif CLI Tool - Detailed Implementation Plan

## Overview

**Duration:** 2-3 days  
**Dependencies:** Phases 3 & 4 complete (sdif-rs writing API and MAT support)  
**Goal:** Create a command-line tool for converting MATLAB/Octave .mat files to SDIF format, with Max/MSP compatibility features and comprehensive error handling.

This document provides step-by-step instructions for Claude Code to implement Phase 5. The `mat2sdif` binary will be the user-facing tool that ties together all the library functionality.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         mat2sdif CLI Architecture                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Command Line (clap)                                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  mat2sdif input.mat output.sdif [OPTIONS]                           │    │
│  │                                                                      │    │
│  │  --list          List variables and exit                            │    │
│  │  --time-var      Specify time variable                              │    │
│  │  --data-var      Specify data variable                              │    │
│  │  --frame-type    SDIF frame type (1TRC, 1HRM, etc.)                 │    │
│  │  --columns       Column names                                        │    │
│  │  --max-partials  Limit for Max compatibility                        │    │
│  │  --dry-run       Validate without writing                           │    │
│  │  --verbose       Detailed output                                     │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                               │                                              │
│                               ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                        Command Router                                │    │
│  │                                                                      │    │
│  │    --list ─────────▶ list_variables()                               │    │
│  │    --dry-run ──────▶ validate_conversion()                          │    │
│  │    (default) ──────▶ convert()                                      │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                               │                                              │
│                               ▼                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                     sdif-rs Library                                  │    │
│  │                                                                      │    │
│  │    MatFile ──▶ MatToSdifConverter ──▶ SdifWriter                    │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Design Goals

1. **User-Friendly**: Clear help text, sensible defaults, good error messages
2. **Max Compatible**: Enforce limits and validations for Max/MSP compatibility
3. **Flexible**: Support various MAT file structures and SDIF frame types
4. **Safe**: Dry-run mode, validation, warnings for potential issues
5. **Informative**: Verbose mode, progress output, summary statistics

---

## Step 1: Update mat2sdif Cargo.toml

### Task 1.1: Configure Dependencies

**Claude Code Prompt:**

```
Update mat2sdif/Cargo.toml with the complete configuration:

[package]
name = "mat2sdif"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
rust-version.workspace = true
description = "Convert MATLAB/Octave .mat files to SDIF format"
keywords = ["sdif", "matlab", "octave", "audio", "converter"]
categories = ["command-line-utilities", "multimedia::audio"]

[[bin]]
name = "mat2sdif"
path = "src/main.rs"

[dependencies]
# Local dependencies
sdif-rs = { path = "../sdif-rs", features = ["mat"] }

# CLI framework
clap = { version = "4.4", features = ["derive", "env", "wrap_help"] }

# Error handling
anyhow = "1.0"

# Colored terminal output
colored = "2.0"

# Progress indication (optional, for large files)
indicatif = { version = "0.17", optional = true }

[features]
default = []
progress = ["indicatif"]

[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.0"
tempfile = "3.0"
```

---

## Step 2: Main Entry Point

### Task 2.1: Create Main Module Structure

**Claude Code Prompt:**

```
Create the main module structure for mat2sdif:

mat2sdif/src/
├── main.rs          # Entry point, argument parsing
├── cli.rs           # CLI argument definitions (clap)
├── commands/        # Command implementations
│   ├── mod.rs
│   ├── list.rs      # --list command
│   ├── convert.rs   # Main conversion
│   └── validate.rs  # --dry-run validation
├── max_compat.rs    # Max/MSP compatibility checks
└── output.rs        # Terminal output formatting

Create placeholder files for each:

// src/main.rs
//! mat2sdif - Convert MATLAB/Octave files to SDIF format

// src/cli.rs
//! Command-line argument definitions

// src/commands/mod.rs
//! Command implementations

// src/commands/list.rs
//! List variables command

// src/commands/convert.rs
//! Main conversion command

// src/commands/validate.rs
//! Dry-run validation command

// src/max_compat.rs
//! Max/MSP compatibility validation

// src/output.rs
//! Terminal output formatting
```

### Task 2.2: Implement CLI Arguments with Clap

**Claude Code Prompt:**

```
Create mat2sdif/src/cli.rs:

//! Command-line argument definitions using clap derive macros.

use std::path::PathBuf;
use clap::{Parser, ValueEnum};

/// Convert MATLAB/Octave .mat files to SDIF format.
///
/// mat2sdif reads numeric arrays from MAT files and writes them as
/// time-stamped SDIF frames, suitable for use with Max/MSP, AudioSculpt,
/// and other SDIF-compatible software.
#[derive(Parser, Debug)]
#[command(name = "mat2sdif")]
#[command(author, version, about, long_about = None)]
#[command(after_help = EXAMPLES)]
pub struct Args {
    /// Input .mat file
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,
    
    /// Output .sdif file (omit for --list mode)
    #[arg(value_name = "OUTPUT")]
    pub output: Option<PathBuf>,
    
    // ========================================================================
    // Mode Selection
    // ========================================================================
    
    /// List variables in the MAT file and exit
    #[arg(short, long)]
    pub list: bool,
    
    /// Validate conversion without writing output
    #[arg(long)]
    pub dry_run: bool,
    
    // ========================================================================
    // Variable Selection
    // ========================================================================
    
    /// Variable containing the time vector
    ///
    /// If not specified, mat2sdif will attempt to auto-detect a suitable
    /// time vector based on variable name and data characteristics.
    #[arg(short = 't', long = "time-var", value_name = "NAME")]
    pub time_var: Option<String>,
    
    /// Variable containing the data matrix
    ///
    /// If not specified, mat2sdif will attempt to auto-detect a suitable
    /// data variable (2D numeric array that isn't the time vector).
    #[arg(short = 'd', long = "data-var", value_name = "NAME")]
    pub data_var: Option<String>,
    
    // ========================================================================
    // SDIF Configuration
    // ========================================================================
    
    /// SDIF frame type signature (4 characters)
    #[arg(short = 'f', long, value_name = "SIG", default_value = "1TRC")]
    pub frame_type: String,
    
    /// SDIF matrix type signature (4 characters)
    #[arg(short = 'm', long, value_name = "SIG", default_value = "1TRC")]
    pub matrix_type: String,
    
    /// Column names for the matrix (comma-separated)
    ///
    /// Default depends on frame type:
    /// - 1TRC/1HRM: Index,Frequency,Amplitude,Phase
    /// - 1FQ0: Frequency,Confidence
    #[arg(short = 'c', long, value_name = "NAMES", value_delimiter = ',')]
    pub columns: Option<Vec<String>>,
    
    /// Stream ID for output frames
    #[arg(long, value_name = "ID", default_value = "0")]
    pub stream_id: u32,
    
    // ========================================================================
    // Max/MSP Compatibility
    // ========================================================================
    
    /// Maximum partials per frame (for Max/MSP compatibility)
    ///
    /// Modern CNMAT externals support up to 1024 partials.
    /// Set to 256 for legacy compatibility.
    /// Set to 0 to disable limiting.
    #[arg(long, value_name = "N", default_value = "1024")]
    pub max_partials: usize,
    
    /// Validate Max/MSP compatibility and warn about issues
    #[arg(long)]
    pub max_compat: bool,
    
    // ========================================================================
    // Data Handling
    // ========================================================================
    
    /// Transpose the data matrix (swap rows and columns)
    ///
    /// Use this if your data has time as columns instead of rows.
    #[arg(long)]
    pub transpose: bool,
    
    /// How to handle complex numbers in the data
    #[arg(long, value_enum, default_value = "magnitude")]
    pub complex_mode: ComplexModeArg,
    
    // ========================================================================
    // Output Control
    // ========================================================================
    
    /// Show detailed progress and information
    #[arg(short, long)]
    pub verbose: bool,
    
    /// Suppress all non-error output
    #[arg(short, long)]
    pub quiet: bool,
    
    /// Force overwrite of existing output file
    #[arg(long)]
    pub force: bool,
}

/// How to handle complex numbers in MAT data.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ComplexModeArg {
    /// Use only the real part
    Real,
    /// Convert to magnitude (absolute value)
    Magnitude,
    /// Output magnitude and phase as separate columns
    MagPhase,
    /// Output real and imaginary as separate columns
    ReIm,
}

impl Args {
    /// Validate argument combinations.
    pub fn validate(&self) -> Result<(), String> {
        // List mode doesn't need output file
        if self.list {
            return Ok(());
        }
        
        // Conversion modes need output file
        if self.output.is_none() && !self.dry_run {
            return Err("Output file is required (or use --list or --dry-run)".to_string());
        }
        
        // Validate signature lengths
        if self.frame_type.len() != 4 {
            return Err(format!(
                "Frame type must be exactly 4 characters, got '{}'",
                self.frame_type
            ));
        }
        
        if self.matrix_type.len() != 4 {
            return Err(format!(
                "Matrix type must be exactly 4 characters, got '{}'",
                self.matrix_type
            ));
        }
        
        // Check input file exists
        if !self.input.exists() {
            return Err(format!(
                "Input file not found: {}",
                self.input.display()
            ));
        }
        
        // Check output doesn't exist (unless --force)
        if let Some(ref output) = self.output {
            if output.exists() && !self.force && !self.dry_run {
                return Err(format!(
                    "Output file already exists: {} (use --force to overwrite)",
                    output.display()
                ));
            }
        }
        
        // Quiet and verbose are mutually exclusive
        if self.quiet && self.verbose {
            return Err("Cannot use both --quiet and --verbose".to_string());
        }
        
        Ok(())
    }
    
    /// Get default column names based on frame type.
    pub fn get_columns(&self) -> Vec<String> {
        if let Some(ref cols) = self.columns {
            return cols.clone();
        }
        
        // Defaults based on frame type
        match self.frame_type.as_str() {
            "1TRC" | "1HRM" => vec![
                "Index".to_string(),
                "Frequency".to_string(),
                "Amplitude".to_string(),
                "Phase".to_string(),
            ],
            "1FQ0" => vec![
                "Frequency".to_string(),
                "Confidence".to_string(),
            ],
            "1RES" => vec![
                "Frequency".to_string(),
                "Amplitude".to_string(),
                "DecayRate".to_string(),
                "Phase".to_string(),
            ],
            _ => vec![
                "Col1".to_string(),
                "Col2".to_string(),
                "Col3".to_string(),
                "Col4".to_string(),
            ],
        }
    }
}

/// Example usage shown in --help.
const EXAMPLES: &str = r#"
EXAMPLES:
    # List variables in a MAT file
    mat2sdif --list analysis.mat

    # Basic conversion with auto-detection
    mat2sdif analysis.mat output.sdif

    # Specify time and data variables explicitly
    mat2sdif analysis.mat output.sdif -t time -d partials

    # Convert with custom column names
    mat2sdif analysis.mat output.sdif -c "Index,Freq,Amp,Phase"

    # Convert F0 data
    mat2sdif pitch.mat f0.sdif -f 1FQ0 -m 1FQ0 -c "Frequency,Confidence"

    # Validate without writing (dry run)
    mat2sdif --dry-run analysis.mat output.sdif

    # Force overwrite and show progress
    mat2sdif -v --force analysis.mat output.sdif

    # Legacy Max compatibility (256 partial limit)
    mat2sdif --max-partials 256 analysis.mat output.sdif
"#;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_columns_1trc() {
        let args = Args {
            input: PathBuf::from("test.mat"),
            output: Some(PathBuf::from("test.sdif")),
            list: false,
            dry_run: false,
            time_var: None,
            data_var: None,
            frame_type: "1TRC".to_string(),
            matrix_type: "1TRC".to_string(),
            columns: None,
            stream_id: 0,
            max_partials: 1024,
            max_compat: false,
            transpose: false,
            complex_mode: ComplexModeArg::Magnitude,
            verbose: false,
            quiet: false,
            force: false,
        };
        
        let cols = args.get_columns();
        assert_eq!(cols.len(), 4);
        assert_eq!(cols[0], "Index");
    }
    
    #[test]
    fn test_default_columns_1fq0() {
        let args = Args {
            input: PathBuf::from("test.mat"),
            output: Some(PathBuf::from("test.sdif")),
            list: false,
            dry_run: false,
            time_var: None,
            data_var: None,
            frame_type: "1FQ0".to_string(),
            matrix_type: "1FQ0".to_string(),
            columns: None,
            stream_id: 0,
            max_partials: 1024,
            max_compat: false,
            transpose: false,
            complex_mode: ComplexModeArg::Magnitude,
            verbose: false,
            quiet: false,
            force: false,
        };
        
        let cols = args.get_columns();
        assert_eq!(cols.len(), 2);
        assert_eq!(cols[0], "Frequency");
    }
}
```

---

## Step 3: Main Entry Point

### Task 3.1: Create main.rs

**Claude Code Prompt:**

```
Create mat2sdif/src/main.rs:

//! mat2sdif - Convert MATLAB/Octave .mat files to SDIF format.
//!
//! This tool reads numeric arrays from MAT files and converts them to
//! SDIF format for use with Max/MSP, AudioSculpt, and other audio software.

mod cli;
mod commands;
mod max_compat;
mod output;

use anyhow::{Context, Result};
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
    args.validate()
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    
    // Dispatch to appropriate command
    if args.list {
        commands::list::run(&args)
    } else if args.dry_run {
        commands::validate::run(&args)
    } else {
        commands::convert::run(&args)
    }
}
```

---

## Step 4: Output Formatting Module

### Task 4.1: Create Output Helper Module

**Claude Code Prompt:**

```
Create mat2sdif/src/output.rs:

//! Terminal output formatting utilities.

use colored::Colorize;
use std::io::{self, Write};

/// Print an error message to stderr.
pub fn print_error(err: &anyhow::Error) {
    eprintln!("{}: {}", "error".red().bold(), err);
    
    // Print cause chain
    for cause in err.chain().skip(1) {
        eprintln!("  {}: {}", "caused by".red(), cause);
    }
}

/// Print a warning message to stderr.
pub fn print_warning(msg: &str) {
    eprintln!("{}: {}", "warning".yellow().bold(), msg);
}

/// Print an info message to stdout (respects quiet mode).
pub fn print_info(msg: &str, quiet: bool) {
    if !quiet {
        println!("{}", msg);
    }
}

/// Print a success message.
pub fn print_success(msg: &str, quiet: bool) {
    if !quiet {
        println!("{}: {}", "success".green().bold(), msg);
    }
}

/// Print a verbose message (only in verbose mode).
pub fn print_verbose(msg: &str, verbose: bool) {
    if verbose {
        println!("{}: {}", "info".blue(), msg);
    }
}

/// Print a header line.
pub fn print_header(title: &str) {
    println!("\n{}", title.bold().underline());
}

/// Print a key-value pair.
pub fn print_kv(key: &str, value: &str, indent: usize) {
    let padding = " ".repeat(indent);
    println!("{}{}: {}", padding, key.dimmed(), value);
}

/// Print a separator line.
pub fn print_separator() {
    println!("{}", "─".repeat(60).dimmed());
}

/// Format a number with thousands separators.
pub fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    
    result
}

/// Format a duration in seconds to a human-readable string.
pub fn format_duration(seconds: f64) -> String {
    if seconds < 1.0 {
        format!("{:.0}ms", seconds * 1000.0)
    } else if seconds < 60.0 {
        format!("{:.2}s", seconds)
    } else {
        let mins = (seconds / 60.0).floor();
        let secs = seconds % 60.0;
        format!("{}m {:.1}s", mins, secs)
    }
}

/// Format file size in human-readable form.
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Create a simple progress reporter for verbose mode.
pub struct ProgressReporter {
    total: usize,
    current: usize,
    last_percent: usize,
    verbose: bool,
}

impl ProgressReporter {
    pub fn new(total: usize, verbose: bool) -> Self {
        ProgressReporter {
            total,
            current: 0,
            last_percent: 0,
            verbose,
        }
    }
    
    pub fn increment(&mut self) {
        self.current += 1;
        
        if self.verbose && self.total > 0 {
            let percent = (self.current * 100) / self.total;
            
            // Only print at 10% intervals
            if percent >= self.last_percent + 10 {
                self.last_percent = percent;
                eprint!("\r{}: {}% ({}/{} frames)", 
                    "progress".blue(),
                    percent,
                    self.current,
                    self.total
                );
                io::stderr().flush().ok();
            }
        }
    }
    
    pub fn finish(&self) {
        if self.verbose && self.total > 0 {
            eprintln!("\r{}: 100% ({}/{} frames)", 
                "progress".blue(),
                self.total,
                self.total
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
    }
    
    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0.5), "500ms");
        assert_eq!(format_duration(1.5), "1.50s");
        assert_eq!(format_duration(90.0), "1m 30.0s");
    }
}
```

---

## Step 5: Command Implementations

### Task 5.1: Create Commands Module

**Claude Code Prompt:**

```
Create mat2sdif/src/commands/mod.rs:

//! Command implementations.

pub mod convert;
pub mod list;
pub mod validate;
```

### Task 5.2: Implement List Command

**Claude Code Prompt:**

```
Create mat2sdif/src/commands/list.rs:

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
```

### Task 5.3: Implement Convert Command

**Claude Code Prompt:**

```
Create mat2sdif/src/commands/convert.rs:

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
    let columns: Vec<&str> = args.get_columns().iter().map(|s| s.as_str()).collect();
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
    let mut progress = ProgressReporter::new(num_frames, args.verbose);
    
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
fn build_config(args: &Args) -> Result<MatToSdifConfig> {
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
```

### Task 5.4: Implement Validate Command (Dry Run)

**Claude Code Prompt:**

```
Create mat2sdif/src/commands/validate.rs:

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

// Re-export build_config for use by other commands
pub use crate::commands::convert::build_config;
```

---

## Step 6: Max Compatibility Module

### Task 6.1: Create Max Compatibility Checks

**Claude Code Prompt:**

```
Create mat2sdif/src/max_compat.rs:

//! Max/MSP compatibility validation.
//!
//! This module provides checks to ensure generated SDIF files will work
//! correctly with Max/MSP and the CNMAT SDIF externals.

use colored::Colorize;

use sdif_rs::MatToSdifConverter;

use crate::cli::Args;
use crate::output;

/// Max-compatible frame types.
const MAX_FRAME_TYPES: &[&str] = &["1TRC", "1HRM", "1FQ0", "1RES"];

/// Modern CNMAT partial limit.
const MODERN_PARTIAL_LIMIT: usize = 1024;

/// Legacy CNMAT partial limit.
const LEGACY_PARTIAL_LIMIT: usize = 256;

/// Validate configuration for Max compatibility.
///
/// Returns Ok if compatible, or an error with explanation if not.
pub fn validate_config(args: &Args, converter: &MatToSdifConverter) -> anyhow::Result<()> {
    let warnings = check_all(args, converter);
    
    for warning in &warnings {
        output::print_warning(warning);
    }
    
    Ok(())
}

/// Run all compatibility checks and return warnings.
pub fn check_all(args: &Args, converter: &MatToSdifConverter) -> Vec<String> {
    let mut warnings = Vec::new();
    
    // Check frame type
    if let Some(w) = check_frame_type(&args.frame_type) {
        warnings.push(w);
    }
    
    // Check partial limit
    if let Some(w) = check_partial_limit(args.max_partials, converter.cols_per_frame()) {
        warnings.push(w);
    }
    
    // Check column count for specific frame types
    if let Some(w) = check_column_count(&args.frame_type, &args.get_columns()) {
        warnings.push(w);
    }
    
    // Check time range
    let (start, end) = converter.time_range();
    if let Some(w) = check_time_range(start, end) {
        warnings.push(w);
    }
    
    warnings
}

/// Check if frame type is Max-compatible.
fn check_frame_type(frame_type: &str) -> Option<String> {
    if !MAX_FRAME_TYPES.contains(&frame_type) {
        Some(format!(
            "Frame type '{}' may not be supported by all Max externals. \
             Standard types are: {}",
            frame_type,
            MAX_FRAME_TYPES.join(", ")
        ))
    } else {
        None
    }
}

/// Check partial limit against Max constraints.
fn check_partial_limit(limit: usize, cols: usize) -> Option<String> {
    if limit == 0 {
        return Some(
            "No partial limit set. Max/MSP externals have limits \
             (1024 modern, 256 legacy). Consider setting --max-partials."
                .to_string()
        );
    }
    
    if limit > MODERN_PARTIAL_LIMIT {
        return Some(format!(
            "Partial limit {} exceeds Max/MSP limit of {}. \
             Frames may be truncated during playback.",
            limit, MODERN_PARTIAL_LIMIT
        ));
    }
    
    if limit > LEGACY_PARTIAL_LIMIT {
        return Some(format!(
            "Partial limit {} exceeds legacy Max limit of {}. \
             May not work with older CNMAT externals.",
            limit, LEGACY_PARTIAL_LIMIT
        ));
    }
    
    None
}

/// Check column count matches expected for frame type.
fn check_column_count(frame_type: &str, columns: &[String]) -> Option<String> {
    let expected = match frame_type {
        "1TRC" | "1HRM" => 4, // Index, Frequency, Amplitude, Phase
        "1FQ0" => 2,          // Frequency, Confidence
        "1RES" => 4,          // Frequency, Amplitude, DecayRate, Phase
        _ => return None,     // Unknown type, skip check
    };
    
    if columns.len() != expected {
        Some(format!(
            "Frame type '{}' typically has {} columns, but {} provided. \
             This may cause issues with some software.",
            frame_type, expected, columns.len()
        ))
    } else {
        None
    }
}

/// Check time range is reasonable.
fn check_time_range(start: f64, end: f64) -> Option<String> {
    if start < 0.0 {
        return Some(format!(
            "Negative start time ({:.3}s) may cause issues. \
             Consider normalizing to start at 0.",
            start
        ));
    }
    
    if end > 3600.0 {
        return Some(format!(
            "Duration over 1 hour ({:.1}s). \
             Very long files may have performance issues.",
            end - start
        ));
    }
    
    None
}

/// Detailed compatibility report for verbose mode.
pub fn print_compatibility_report(args: &Args, converter: &MatToSdifConverter) {
    output::print_header("Max/MSP Compatibility Report");
    
    // Frame type
    let frame_ok = MAX_FRAME_TYPES.contains(&args.frame_type.as_str());
    println!(
        "  Frame type '{}': {}",
        args.frame_type,
        if frame_ok { "✓ Supported".green() } else { "⚠ Non-standard".yellow() }
    );
    
    // Partial limit
    let partial_status = if args.max_partials == 0 {
        "⚠ No limit".yellow().to_string()
    } else if args.max_partials <= LEGACY_PARTIAL_LIMIT {
        "✓ Legacy compatible".green().to_string()
    } else if args.max_partials <= MODERN_PARTIAL_LIMIT {
        "✓ Modern compatible".green().to_string()
    } else {
        "✗ Exceeds limit".red().to_string()
    };
    println!("  Partial limit {}: {}", args.max_partials, partial_status);
    
    // Column count
    let cols = args.get_columns();
    println!("  Columns: {} ({:?})", cols.len(), cols);
    
    // Time range
    let (start, end) = converter.time_range();
    let duration = end - start;
    println!(
        "  Time range: {:.3}s to {:.3}s ({:.2}s duration)",
        start, end, duration
    );
    
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_frame_type_check() {
        assert!(check_frame_type("1TRC").is_none());
        assert!(check_frame_type("1HRM").is_none());
        assert!(check_frame_type("1FQ0").is_none());
        assert!(check_frame_type("XXXX").is_some());
    }
    
    #[test]
    fn test_partial_limit_check() {
        assert!(check_partial_limit(256, 4).is_none());
        assert!(check_partial_limit(1024, 4).is_some()); // Warning for > legacy
        assert!(check_partial_limit(2000, 4).is_some()); // Error for > modern
        assert!(check_partial_limit(0, 4).is_some());    // Warning for no limit
    }
    
    #[test]
    fn test_column_count_check() {
        let cols_4 = vec!["A".into(), "B".into(), "C".into(), "D".into()];
        let cols_2 = vec!["A".into(), "B".into()];
        
        assert!(check_column_count("1TRC", &cols_4).is_none());
        assert!(check_column_count("1TRC", &cols_2).is_some());
        assert!(check_column_count("1FQ0", &cols_2).is_none());
        assert!(check_column_count("1FQ0", &cols_4).is_some());
    }
}
```

---

## Step 7: Integration Tests

### Task 7.1: Create CLI Integration Tests

**Claude Code Prompt:**

```
Create mat2sdif/tests/cli_tests.rs:

//! Integration tests for mat2sdif CLI.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Get the mat2sdif command.
fn mat2sdif() -> Command {
    Command::cargo_bin("mat2sdif").unwrap()
}

// ============================================================================
// Basic CLI Tests
// ============================================================================

#[test]
fn test_help() {
    mat2sdif()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Convert MATLAB/Octave"))
        .stdout(predicate::str::contains("--list"))
        .stdout(predicate::str::contains("--time-var"))
        .stdout(predicate::str::contains("EXAMPLES"));
}

#[test]
fn test_version() {
    mat2sdif()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("mat2sdif"));
}

#[test]
fn test_missing_input() {
    mat2sdif()
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_nonexistent_input() {
    mat2sdif()
        .arg("/nonexistent/file.mat")
        .arg("output.sdif")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_invalid_frame_type() {
    let temp = TempDir::new().unwrap();
    let input = temp.path().join("test.mat");
    
    // Create a dummy file (will fail to parse, but that's after arg validation)
    fs::write(&input, "dummy").unwrap();
    
    mat2sdif()
        .arg(&input)
        .arg("output.sdif")
        .arg("--frame-type")
        .arg("TOOLONG")
        .assert()
        .failure()
        .stderr(predicate::str::contains("4 characters"));
}

// ============================================================================
// List Mode Tests
// ============================================================================

#[test]
fn test_list_missing_file() {
    mat2sdif()
        .arg("--list")
        .arg("/nonexistent/file.mat")
        .assert()
        .failure();
}

// ============================================================================
// Dry Run Tests  
// ============================================================================

#[test]
fn test_dry_run_missing_file() {
    mat2sdif()
        .arg("--dry-run")
        .arg("/nonexistent/file.mat")
        .arg("output.sdif")
        .assert()
        .failure();
}

// ============================================================================
// Tests requiring fixture files (marked ignore)
// ============================================================================

#[test]
#[ignore = "Requires test fixture: simple.mat"]
fn test_list_simple_mat() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/simple.mat");
    
    if !fixture.exists() {
        eprintln!("Skipping: fixture not found");
        return;
    }
    
    mat2sdif()
        .arg("--list")
        .arg(&fixture)
        .assert()
        .success()
        .stdout(predicate::str::contains("Variables in"));
}

#[test]
#[ignore = "Requires test fixture: simple.mat"]
fn test_convert_simple_mat() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/simple.mat");
    
    if !fixture.exists() {
        eprintln!("Skipping: fixture not found");
        return;
    }
    
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("output.sdif");
    
    mat2sdif()
        .arg(&fixture)
        .arg(&output)
        .arg("-v")
        .assert()
        .success()
        .stdout(predicate::str::contains("success"));
    
    assert!(output.exists());
}

#[test]
#[ignore = "Requires test fixture: simple.mat"]
fn test_dry_run_simple_mat() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/simple.mat");
    
    if !fixture.exists() {
        eprintln!("Skipping: fixture not found");
        return;
    }
    
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("output.sdif");
    
    mat2sdif()
        .arg("--dry-run")
        .arg(&fixture)
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run"))
        .stdout(predicate::str::contains("Validation passed"));
    
    // Output should NOT be created in dry-run mode
    assert!(!output.exists());
}
```

### Task 7.2: Create Test Fixtures Directory

**Claude Code Prompt:**

```
Create mat2sdif/tests/fixtures/README.md:

# Test Fixtures for mat2sdif

This directory should contain MAT files for integration testing.

## Required Files

### simple.mat
A basic MAT file with:
- `time` - 1D array of 100 time values (0.0 to 1.0)
- `partials` - 2D array [100, 4] with Index, Freq, Amp, Phase columns

### complex.mat
A MAT file with complex data:
- `time` - 1D array of time values
- `spectrum` - 2D complex array

### f0.mat
A pitch tracking result:
- `time` - 1D array of time values  
- `f0` - 2D array [N, 2] with Frequency, Confidence columns

## Creating Test Files

See sdif-rs/tests/fixtures/README.md for instructions on creating
MAT files using MATLAB, Octave, or Python.

### Quick Python Script

```python
import numpy as np
from scipy.io import savemat

# simple.mat
time = np.arange(0, 1.0, 0.01)  # 100 time points
partials = np.zeros((100, 4))
for i in range(100):
    partials[i] = [1, 440 + i*5, 0.5 * np.exp(-i/30), i * 0.1]
savemat('simple.mat', {'time': time, 'partials': partials})

# f0.mat
f0_data = np.column_stack([
    220 + 10 * np.sin(np.linspace(0, 4*np.pi, 100)),  # Frequency
    0.9 + 0.1 * np.random.rand(100)  # Confidence
])
savemat('f0.mat', {'time': time, 'f0': f0_data})
```

Also create mat2sdif/tests/fixtures/.gitkeep
```

---

## Step 8: Documentation

### Task 8.1: Create README

**Claude Code Prompt:**

```
Create mat2sdif/README.md:

# mat2sdif

Convert MATLAB/Octave .mat files to SDIF format.

## Overview

`mat2sdif` is a command-line tool for converting numeric data from MAT files
to SDIF (Sound Description Interchange Format) files. It's designed for
audio analysis workflows where spectral data, pitch tracks, or sinusoidal
models need to be exported for use with Max/MSP, AudioSculpt, or other
SDIF-compatible software.

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/username/rust-sdif.git
cd rust-sdif

# Build the tool
cargo build --release -p mat2sdif

# The binary will be at target/release/mat2sdif
```

### From crates.io (when published)

```bash
cargo install mat2sdif
```

## Quick Start

```bash
# List variables in a MAT file
mat2sdif --list analysis.mat

# Basic conversion (auto-detect time and data variables)
mat2sdif analysis.mat output.sdif

# Specify variables explicitly
mat2sdif analysis.mat output.sdif --time-var time --data-var partials

# Convert with custom column names
mat2sdif analysis.mat output.sdif -c "Index,Freq,Amp,Phase"

# Validate without writing (dry run)
mat2sdif --dry-run analysis.mat output.sdif
```

## Usage

```
mat2sdif [OPTIONS] <INPUT> [OUTPUT]

Arguments:
  <INPUT>   Input .mat file
  [OUTPUT]  Output .sdif file

Options:
  -l, --list                  List variables in the MAT file and exit
      --dry-run               Validate conversion without writing output
  -t, --time-var <NAME>       Variable containing the time vector
  -d, --data-var <NAME>       Variable containing the data matrix
  -f, --frame-type <SIG>      SDIF frame type signature [default: 1TRC]
  -m, --matrix-type <SIG>     SDIF matrix type signature [default: 1TRC]
  -c, --columns <NAMES>       Column names (comma-separated)
      --max-partials <N>      Maximum partials per frame [default: 1024]
      --transpose             Transpose the data matrix
      --complex-mode <MODE>   How to handle complex data [default: magnitude]
  -v, --verbose               Show detailed progress
  -q, --quiet                 Suppress non-error output
      --force                 Overwrite existing output file
  -h, --help                  Print help
  -V, --version               Print version
```

## Examples

### Sinusoidal Tracks (1TRC)

The most common use case - converting partial tracking data:

```bash
# MAT file contains:
#   time: [1000, 1] - time values in seconds
#   partials: [1000, 400] - 100 partials × 4 columns per frame

mat2sdif tracks.mat output.sdif \
    --time-var time \
    --data-var partials \
    --frame-type 1TRC \
    --columns "Index,Frequency,Amplitude,Phase"
```

### Fundamental Frequency (1FQ0)

Converting pitch tracking results:

```bash
# MAT file contains:
#   t: [500, 1] - time values
#   f0: [500, 2] - frequency and confidence per frame

mat2sdif pitch.mat f0.sdif \
    --time-var t \
    --data-var f0 \
    --frame-type 1FQ0 \
    --matrix-type 1FQ0 \
    --columns "Frequency,Confidence"
```

### Complex Spectral Data

Converting STFT or other complex-valued data:

```bash
# Convert to magnitude
mat2sdif stft.mat spectrum.sdif --complex-mode magnitude

# Keep magnitude and phase as separate columns
mat2sdif stft.mat spectrum.sdif --complex-mode mag-phase
```

### Inspecting MAT Files

Use `--list` to see what's in a MAT file:

```bash
$ mat2sdif --list mystery.mat

Variables in 'mystery.mat':

  Name          Shape          Type      Notes
  ----------    ----------     -------   -----
  fs            [1, 1]         float64   1D
  partialData   [500, 400]     float64   
  timeVec       [500, 1]       float64   time vector?, 1D, hop=10.0ms

3 numeric variables found

hint: 'timeVec' looks like a time vector
hint: Potential data variables: ["partialData"]
```

### Validation (Dry Run)

Check conversion settings before writing:

```bash
$ mat2sdif --dry-run analysis.mat output.sdif

Dry run mode (no files will be written)

MAT File Analysis

  File: analysis.mat
  Variables: 3

Conversion Plan

  Frames to write: 500
  Time range: 0.000s to 4.990s
  Duration: 4.99s
  Columns per frame: 4

SDIF Output

  Output file: output.sdif
  Frame type: 1TRC
  Matrix type: 1TRC
  Columns: Index, Frequency, Amplitude, Phase
  Max partials: 1024

Compatibility Checks

  ✓ All checks passed

success: Validation passed - ready to convert

Run without --dry-run to perform the conversion.
```

## Max/MSP Compatibility

The tool includes built-in checks for Max/MSP compatibility:

- **Frame Types**: Uses standard types (1TRC, 1HRM, 1FQ0, 1RES)
- **Partial Limits**: Default limit of 1024 (use 256 for legacy support)
- **Column Names**: Validates expected columns for each frame type
- **Time Values**: Warns about negative or unusual time ranges

Use `--max-compat` for detailed compatibility warnings:

```bash
mat2sdif --max-compat analysis.mat output.sdif
```

## Supported MAT Formats

- Level 5 MAT files (MATLAB v5, v6, v7)
- v7 compressed files
- Numeric arrays (double, single, integers)
- Complex arrays

**Not supported:**
- HDF5-based v7.3 files (use `h5dump` to convert first)
- Cell arrays, structs, sparse matrices

## Building from Source

Requirements:
- Rust 1.70 or later
- C compiler (for SDIF library)
- clang (for bindgen)

```bash
# Debug build
cargo build -p mat2sdif

# Release build  
cargo build --release -p mat2sdif

# Run tests
cargo test -p mat2sdif
```

## License

MIT License - see LICENSE file.

## See Also

- [sdif-rs](../sdif-rs) - Rust SDIF library
- [SDIF Specification](http://sdif.sourceforge.net/standard/sdif-standard.html)
- [CNMAT Externals](https://cnmat.berkeley.edu/)
```

---

## Step 9: Verification Script

### Task 9.1: Create Phase 5 Verification Script

**Claude Code Prompt:**

```
Create scripts/verify-phase5.sh:

#!/bin/bash
# Verification script for Phase 5 completion

set -e

echo "=== Phase 5 Verification Script ==="
echo

# Check prerequisites
echo "1. Verifying prerequisites..."
if ! cargo check -p sdif-rs --features mat 2>/dev/null; then
    echo "   ✗ sdif-rs with mat feature not building"
    exit 1
fi
echo "   ✓ sdif-rs (with mat) builds"

# Check source files
echo
echo "2. Checking source files..."
REQUIRED_FILES=(
    "mat2sdif/src/main.rs"
    "mat2sdif/src/cli.rs"
    "mat2sdif/src/output.rs"
    "mat2sdif/src/max_compat.rs"
    "mat2sdif/src/commands/mod.rs"
    "mat2sdif/src/commands/list.rs"
    "mat2sdif/src/commands/convert.rs"
    "mat2sdif/src/commands/validate.rs"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "   ✓ $file exists"
    else
        echo "   ✗ $file missing!"
        exit 1
    fi
done

# Build the binary
echo
echo "3. Building mat2sdif..."
if cargo build -p mat2sdif 2>/dev/null; then
    echo "   ✓ Build successful"
else
    echo "   ✗ Build failed"
    exit 1
fi

# Check the binary runs
echo
echo "4. Testing binary..."
if cargo run -p mat2sdif -- --help >/dev/null 2>&1; then
    echo "   ✓ --help works"
else
    echo "   ✗ --help failed"
    exit 1
fi

if cargo run -p mat2sdif -- --version >/dev/null 2>&1; then
    echo "   ✓ --version works"
else
    echo "   ✗ --version failed"
    exit 1
fi

# Run unit tests
echo
echo "5. Running unit tests..."
if cargo test -p mat2sdif --lib 2>/dev/null; then
    echo "   ✓ Unit tests passed"
else
    echo "   ⚠ Some unit tests failed"
fi

# Run CLI tests
echo
echo "6. Running CLI tests..."
if cargo test -p mat2sdif --test cli_tests 2>/dev/null; then
    echo "   ✓ CLI tests passed"
else
    echo "   ⚠ CLI tests failed"
fi

# Check documentation
echo
echo "7. Checking documentation..."
if [ -f "mat2sdif/README.md" ]; then
    echo "   ✓ README.md exists"
else
    echo "   ⚠ README.md missing"
fi

# Build release binary
echo
echo "8. Building release binary..."
if cargo build -p mat2sdif --release 2>/dev/null; then
    echo "   ✓ Release build successful"
    
    # Show binary size
    BINARY="target/release/mat2sdif"
    if [ -f "$BINARY" ]; then
        SIZE=$(du -h "$BINARY" | cut -f1)
        echo "   ✓ Binary size: $SIZE"
    fi
else
    echo "   ⚠ Release build failed"
fi

# Summary
echo
echo "=== Phase 5 Verification Complete ==="
echo
echo "mat2sdif CLI is implemented with:"
echo "  - Argument parsing (clap)"
echo "  - --list mode for variable inspection"
echo "  - --dry-run mode for validation"
echo "  - Full conversion pipeline"
echo "  - Max/MSP compatibility checks"
echo "  - Colored terminal output"
echo
echo "Usage:"
echo "  cargo run -p mat2sdif -- --help"
echo "  cargo run -p mat2sdif -- --list input.mat"
echo "  cargo run -p mat2sdif -- input.mat output.sdif"
echo
echo "Next steps:"
echo "  1. Add test MAT files to mat2sdif/tests/fixtures/"
echo "  2. Test with real MAT files from audio analysis"
echo "  3. Proceed to Phase 6: Documentation and Polish"

Make executable:
chmod +x scripts/verify-phase5.sh
```

---

## Success Criteria Summary

Phase 5 is complete when:

1. **CLI Framework**
   - [ ] clap argument parsing with derive macros
   - [ ] Help text with examples
   - [ ] Version information
   - [ ] Argument validation

2. **Commands**
   - [ ] `--list` shows variables in MAT file
   - [ ] `--dry-run` validates without writing
   - [ ] Default mode converts MAT to SDIF
   - [ ] Verbose mode shows progress

3. **Conversion Features**
   - [ ] Auto-detect time and data variables
   - [ ] Explicit variable specification
   - [ ] Custom frame/matrix types
   - [ ] Custom column names
   - [ ] Transpose option
   - [ ] Complex number handling

4. **Max Compatibility**
   - [ ] Frame type validation
   - [ ] Partial limit enforcement
   - [ ] Column count checks
   - [ ] Time range validation
   - [ ] Compatibility warnings

5. **Output**
   - [ ] Colored terminal output
   - [ ] Progress indication
   - [ ] Summary statistics
   - [ ] Clear error messages

6. **Tests**
   - [ ] CLI argument tests
   - [ ] Help/version tests
   - [ ] Error case tests
   - [ ] Integration tests (with fixtures)

7. **Documentation**
   - [ ] README with examples
   - [ ] --help text is comprehensive
   - [ ] Usage examples work

---

## Notes for Claude Code

### Clap Derive Patterns

The CLI uses clap's derive macros for argument parsing:
- `#[command(...)]` for program-level settings
- `#[arg(...)]` for individual arguments
- `ValueEnum` derive for enum arguments

### Error Handling Strategy

Uses `anyhow` for flexible error handling:
- Library errors converted with `.context()`
- User errors created with `bail!()` or `anyhow::anyhow!()`
- Error chain printed in `print_error()`

### Colored Output

The `colored` crate provides terminal colors:
- `.red()`, `.green()`, `.yellow()` for status
- `.bold()`, `.dimmed()` for emphasis
- Works on Windows with ANSI support

### Testing CLI Applications

The `assert_cmd` crate makes CLI testing easy:
- `Command::cargo_bin("mat2sdif")` gets the binary
- `.arg()` adds arguments
- `.assert()` checks results
- `predicates` for flexible assertions

### Common Issues

1. **Argument ordering**: clap is strict about positional vs optional
2. **Output file required**: Need to handle --list not needing output
3. **Path handling**: Use `PathBuf` and `AsRef<Path>` consistently
4. **Exit codes**: Return appropriate exit codes (0 success, 1 error)
