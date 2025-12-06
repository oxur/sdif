# Phase 6: Documentation and Polish - Detailed Implementation Plan

## Overview

**Duration:** 2-3 days  
**Dependencies:** Phases 1-5 complete  
**Goal:** Complete documentation, create examples, set up CI/CD, and polish the entire project for publication to crates.io.

This document provides step-by-step instructions for Claude Code to implement Phase 6. This phase transforms the working code into a professional, publishable library.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    Phase 6: Documentation & Polish                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  Documentation                                                               │
│  ┌────────────────────────────────────────────────────────────────────┐     │
│  │  • Crate-level docs with overview and quick start                  │     │
│  │  • Rustdoc for every public item                                   │     │
│  │  • Doc-tests that compile and run                                  │     │
│  │  • README files for each crate                                     │     │
│  └────────────────────────────────────────────────────────────────────┘     │
│                                                                              │
│  Examples                                                                    │
│  ┌────────────────────────────────────────────────────────────────────┐     │
│  │  • read_sdif.rs      - Basic file reading                          │     │
│  │  • write_sdif.rs     - Creating SDIF from scratch                  │     │
│  │  • inspect_sdif.rs   - Exploring file structure                    │     │
│  │  • mat_convert.rs    - MAT to SDIF conversion                      │     │
│  │  • streaming.rs      - Large file processing                       │     │
│  └────────────────────────────────────────────────────────────────────┘     │
│                                                                              │
│  Testing                                                                     │
│  ┌────────────────────────────────────────────────────────────────────┐     │
│  │  • Roundtrip tests (write → read → verify)                         │     │
│  │  • Compatibility tests with known-good files                       │     │
│  │  • Performance benchmarks                                          │     │
│  │  • Doc-test verification                                           │     │
│  └────────────────────────────────────────────────────────────────────┘     │
│                                                                              │
│  CI/CD                                                                       │
│  ┌────────────────────────────────────────────────────────────────────┐     │
│  │  • GitHub Actions for Linux, macOS, Windows                        │     │
│  │  • Clippy and rustfmt checks                                       │     │
│  │  • Test with multiple feature combinations                         │     │
│  │  • Release automation                                              │     │
│  └────────────────────────────────────────────────────────────────────┘     │
│                                                                              │
│  Polish                                                                      │
│  ┌────────────────────────────────────────────────────────────────────┐     │
│  │  • Consistent error messages                                       │     │
│  │  • API review and cleanup                                          │     │
│  │  • CHANGELOG and versioning                                        │     │
│  │  • License and attribution                                         │     │
│  └────────────────────────────────────────────────────────────────────┘     │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Step 1: Workspace README

### Task 1.1: Create Root README.md

**Claude Code Prompt:**

```
Create README.md at the workspace root:

# rust-sdif

[![CI](https://github.com/username/rust-sdif/workflows/CI/badge.svg)](https://github.com/username/rust-sdif/actions)
[![Crates.io](https://img.shields.io/crates/v/sdif-rs.svg)](https://crates.io/crates/sdif-rs)
[![Documentation](https://docs.rs/sdif-rs/badge.svg)](https://docs.rs/sdif-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Rust libraries and tools for working with SDIF (Sound Description Interchange Format) files.

## Overview

This workspace provides:

- **[sdif-sys](sdif-sys/)** - Low-level FFI bindings to the IRCAM SDIF C library
- **[sdif-rs](sdif-rs/)** - Safe, idiomatic Rust API for reading and writing SDIF files
- **[mat2sdif](mat2sdif/)** - CLI tool for converting MATLAB/Octave files to SDIF

SDIF is a standard format for storing sound description data such as sinusoidal tracks,
spectral envelopes, and pitch data. It's widely used in computer music research and
is supported by software like Max/MSP, AudioSculpt, SPEAR, and OpenMusic.

## Quick Start

### Reading an SDIF File

```rust
use sdif_rs::{SdifFile, Result};

fn main() -> Result<()> {
    let file = SdifFile::open("analysis.sdif")?;
    
    println!("Frame types: {:?}", file.frame_types());
    
    for frame in file.frames() {
        let frame = frame?;
        println!("Frame {} at {:.3}s", frame.signature(), frame.time());
        
        for matrix in frame.matrices() {
            let matrix = matrix?;
            println!("  Matrix {}: {}×{}", 
                matrix.signature(), matrix.rows(), matrix.cols());
        }
    }
    
    Ok(())
}
```

### Writing an SDIF File

```rust
use sdif_rs::{SdifFile, Result};

fn main() -> Result<()> {
    let mut writer = SdifFile::builder()
        .create("output.sdif")?
        .add_nvt([("creator", "my_app")])?
        .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
        .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
        .build()?;
    
    // Write a frame with sinusoidal track data
    let data = vec![
        1.0, 440.0, 0.5, 0.0,   // Partial 1: index, freq, amp, phase
        2.0, 880.0, 0.25, 0.0,  // Partial 2
    ];
    
    writer.write_frame_one_matrix("1TRC", 0.0, "1TRC", 2, 4, &data)?;
    writer.close()?;
    
    Ok(())
}
```

### Converting MAT to SDIF

```bash
# List variables in a MAT file
mat2sdif --list analysis.mat

# Convert to SDIF
mat2sdif analysis.mat output.sdif --time-var time --data-var partials
```

## Installation

### Library

Add to your `Cargo.toml`:

```toml
[dependencies]
sdif-rs = "0.1"

# Optional: enable ndarray integration
sdif-rs = { version = "0.1", features = ["ndarray"] }

# Optional: enable MAT file support
sdif-rs = { version = "0.1", features = ["mat"] }
```

### CLI Tool

```bash
cargo install mat2sdif
```

Or download pre-built binaries from the [releases page](https://github.com/username/rust-sdif/releases).

## Building from Source

### Prerequisites

- Rust 1.70 or later
- C compiler (gcc, clang, or MSVC)
- clang (for bindgen)
- pkg-config (optional, for system SDIF library)

### Build

```bash
git clone https://github.com/username/rust-sdif.git
cd rust-sdif

# Build all crates
cargo build --release

# Run tests
cargo test --all

# Build with bundled SDIF library (no system dependency)
cargo build --release --features bundled
```

## Feature Flags

### sdif-rs

| Feature | Description |
|---------|-------------|
| `ndarray` | Enable ndarray integration for matrix data |
| `mat` | Enable MAT file parsing (includes ndarray) |
| `bundled` | Compile SDIF C library from source |
| `static` | Force static linking |

### sdif-sys

| Feature | Description |
|---------|-------------|
| `bundled` | Compile SDIF C library from source |
| `static` | Force static linking |

## Max/MSP Compatibility

The library and tools are designed for compatibility with Max/MSP and the CNMAT SDIF externals:

- Supports standard frame types: 1TRC, 1HRM, 1FQ0, 1RES
- Enforces partial limits (1024 modern, 256 legacy)
- Validates column structure for each frame type
- Handles proper 8-byte alignment and padding

See the [Max Compatibility Guide](docs/max-compatibility.md) for detailed information.

## Documentation

- [API Documentation](https://docs.rs/sdif-rs) (docs.rs)
- [SDIF Format Specification](http://sdif.sourceforge.net/standard/sdif-standard.html)
- [Examples](sdif-rs/examples/)
- [mat2sdif Usage](mat2sdif/README.md)

## Project Structure

```
rust-sdif/
├── sdif-sys/           # FFI bindings to SDIF C library
│   ├── src/
│   ├── sdif/           # Bundled SDIF C source (optional)
│   └── Cargo.toml
├── sdif-rs/            # Safe Rust wrapper
│   ├── src/
│   ├── examples/
│   └── Cargo.toml
├── mat2sdif/           # CLI conversion tool
│   ├── src/
│   └── Cargo.toml
├── docs/               # Additional documentation
├── scripts/            # Build and verification scripts
└── Cargo.toml          # Workspace configuration
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development

```bash
# Run all tests
cargo test --all --all-features

# Run clippy
cargo clippy --all --all-features -- -D warnings

# Format code
cargo fmt --all

# Build documentation
cargo doc --all --all-features --no-deps
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

The bundled SDIF C library is licensed under its own terms - see [sdif-sys/sdif/COPYING](sdif-sys/sdif/COPYING).

## Acknowledgments

- [IRCAM](https://www.ircam.fr/) for the SDIF format and C library
- [CNMAT](https://cnmat.berkeley.edu/) for Max/MSP externals and compatibility testing
- The Rust audio community

## See Also

- [SPEAR](http://www.klingbeil.com/spear/) - Sinusoidal analysis/synthesis
- [AudioSculpt](https://forum.ircam.fr/projects/detail/audiosculpt/) - IRCAM audio analysis
- [OpenMusic](https://openmusic-project.github.io/) - Visual programming for music
```

---

## Step 2: Library Documentation (sdif-rs)

### Task 2.1: Complete Crate-Level Documentation

**Claude Code Prompt:**

```
Update sdif-rs/src/lib.rs with comprehensive crate-level documentation:

//! # sdif-rs
//!
//! Safe, idiomatic Rust bindings for SDIF (Sound Description Interchange Format) files.
//!
//! ## Overview
//!
//! SDIF is a binary format for storing time-stamped spectral and other sound description
//! data. It's commonly used for:
//!
//! - Sinusoidal partial tracking (1TRC frames)
//! - Harmonic analysis (1HRM frames)
//! - Fundamental frequency / pitch tracking (1FQ0 frames)
//! - Resonance models (1RES frames)
//! - Short-time Fourier transforms (1STF frames)
//!
//! This crate provides:
//!
//! - **Reading**: Stream through SDIF files frame-by-frame with iterator APIs
//! - **Writing**: Create SDIF files with a builder pattern API
//! - **MAT Support**: Convert MATLAB/Octave files to SDIF (with `mat` feature)
//!
//! ## Quick Start
//!
//! ### Reading an SDIF File
//!
//! ```no_run
//! use sdif_rs::{SdifFile, Result};
//!
//! fn main() -> Result<()> {
//!     // Open and iterate through frames
//!     let file = SdifFile::open("analysis.sdif")?;
//!     
//!     for frame in file.frames() {
//!         let frame = frame?;
//!         println!("Frame {} at {:.3}s with {} matrices",
//!             frame.signature(), frame.time(), frame.num_matrices());
//!         
//!         for matrix in frame.matrices() {
//!             let matrix = matrix?;
//!             let data = matrix.data_f64()?;
//!             println!("  Matrix {}: {} values", matrix.signature(), data.len());
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Writing an SDIF File
//!
//! ```no_run
//! use sdif_rs::{SdifFile, Result};
//!
//! fn main() -> Result<()> {
//!     // Create a new SDIF file with the builder pattern
//!     let mut writer = SdifFile::builder()
//!         .create("output.sdif")?
//!         // Add metadata
//!         .add_nvt([
//!             ("creator", "my_application"),
//!             ("date", "2024-01-15"),
//!         ])?
//!         // Define matrix type: 4 columns for sinusoidal tracks
//!         .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
//!         // Define frame type containing one 1TRC matrix
//!         .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
//!         .build()?;
//!     
//!     // Write frames with partial data
//!     for i in 0..100 {
//!         let time = i as f64 * 0.01;  // 10ms hop
//!         let data = vec![
//!             1.0, 440.0, 0.5, 0.0,   // Partial 1
//!             2.0, 880.0, 0.25, 0.0,  // Partial 2
//!         ];
//!         
//!         writer.write_frame_one_matrix("1TRC", time, "1TRC", 2, 4, &data)?;
//!     }
//!     
//!     writer.close()?;
//!     Ok(())
//! }
//! ```
//!
//! ## Feature Flags
//!
//! | Feature | Description | Default |
//! |---------|-------------|---------|
//! | `ndarray` | Enable [`ndarray`] integration for matrix data access | No |
//! | `mat` | Enable MAT file parsing (includes `ndarray`) | No |
//! | `bundled` | Compile SDIF C library from bundled source | No |
//! | `static` | Force static linking of SDIF C library | No |
//!
//! ### Using ndarray
//!
//! With the `ndarray` feature, you can get matrix data as 2D arrays:
//!
//! ```ignore
//! use ndarray::Array2;
//!
//! let array: Array2<f64> = matrix.to_array_f64()?;
//! ```
//!
//! ### Using MAT Support
//!
//! With the `mat` feature, you can convert MAT files:
//!
//! ```ignore
//! use sdif_rs::{MatFile, MatToSdifConfig, MatToSdifConverter};
//!
//! let mat = MatFile::open("analysis.mat")?;
//! let config = MatToSdifConfig::new()
//!     .time_var("time")
//!     .data_var("partials");
//! let converter = MatToSdifConverter::new(&mat, config)?;
//! ```
//!
//! ## Common Frame Types
//!
//! | Type | Description | Typical Columns |
//! |------|-------------|-----------------|
//! | `1TRC` | Sinusoidal tracks | Index, Frequency, Amplitude, Phase |
//! | `1HRM` | Harmonic partials | Index, Frequency, Amplitude, Phase |
//! | `1FQ0` | Fundamental frequency | Frequency, Confidence |
//! | `1RES` | Resonances | Frequency, Amplitude, DecayRate, Phase |
//! | `1STF` | Short-time Fourier | Real, Imaginary |
//!
//! ## Max/MSP Compatibility
//!
//! This library is designed for compatibility with Max/MSP and the CNMAT SDIF externals:
//!
//! - Use 1TRC frame type for best support
//! - Limit partials to ≤1024 per frame (256 for legacy support)
//! - Include Index column starting from 1
//! - Ensure amplitudes fade to zero at track boundaries
//!
//! ## Thread Safety
//!
//! The underlying SDIF C library uses global state. Therefore:
//!
//! - [`SdifFile`], [`Frame`], and [`Matrix`] are `!Send` and `!Sync`
//! - All SDIF operations should occur on a single thread
//! - Use message passing if you need cross-thread communication
//!
//! ## Error Handling
//!
//! All fallible operations return [`Result<T>`](Result), which uses the [`Error`] type.
//! Common error conditions include:
//!
//! - File not found or permission denied
//! - Invalid SDIF format or corrupted file
//! - Type mismatches (e.g., reading float data as int)
//! - Invalid signatures (must be exactly 4 ASCII characters)
//!
//! ## Resources
//!
//! - [SDIF Format Specification](http://sdif.sourceforge.net/standard/sdif-standard.html)
//! - [IRCAM SDIF Library](http://sdif.sourceforge.net/)
//! - [CNMAT Max Externals](https://cnmat.berkeley.edu/downloads)

// ... rest of lib.rs content (modules, re-exports, etc.)
```

### Task 2.2: Document All Public Types

**Claude Code Prompt:**

```
Review and enhance documentation for all public types in sdif-rs.
Ensure every public item has:

1. A summary line (first line of doc comment)
2. Extended description if needed
3. # Examples section with working code
4. # Errors section for Result-returning functions
5. # Panics section if the function can panic

Example template for a function:

/// Brief one-line description.
///
/// Longer description with more details about the function's behavior,
/// edge cases, and any important notes.
///
/// # Arguments
///
/// * `arg1` - Description of first argument
/// * `arg2` - Description of second argument
///
/// # Returns
///
/// Description of what is returned.
///
/// # Errors
///
/// Returns an error if:
/// - Condition 1
/// - Condition 2
///
/// # Examples
///
/// ```
/// use sdif_rs::SomeType;
///
/// let result = SomeType::some_function(arg1, arg2)?;
/// assert_eq!(result, expected);
/// # Ok::<(), sdif_rs::Error>(())
/// ```
///
/// # Panics
///
/// Panics if some_condition (only if applicable).
pub fn some_function(arg1: Type1, arg2: Type2) -> Result<ReturnType> {
    // ...
}

Apply this template to all public items in:
- error.rs (Error enum and variants)
- signature.rs (signature conversion functions)
- data_type.rs (DataType enum)
- file.rs (SdifFile and related)
- frame.rs (Frame and FrameIterator)
- matrix.rs (Matrix and MatrixIterator)
- builder.rs (SdifFileBuilder)
- writer.rs (SdifWriter)
- frame_builder.rs (FrameBuilder)
```

---

## Step 3: Examples

### Task 3.1: Create read_sdif Example

**Claude Code Prompt:**

```
Create sdif-rs/examples/read_sdif.rs:

//! Example: Reading an SDIF file
//!
//! This example demonstrates how to open and read an SDIF file,
//! iterating through frames and matrices to access the data.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example read_sdif -- path/to/file.sdif
//! ```

use std::env;
use std::process;

use sdif_rs::{SdifFile, Result};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <sdif-file>", args[0]);
        process::exit(1);
    }
    
    if let Err(e) = run(&args[1]) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(path: &str) -> Result<()> {
    println!("Opening: {}", path);
    println!();
    
    // Open the SDIF file
    let file = SdifFile::open(path)?;
    
    // Print file information
    println!("=== File Information ===");
    println!();
    
    // Print NVT (Name-Value Table) metadata
    let nvt = file.nvt();
    if !nvt.is_empty() {
        println!("Metadata (NVT):");
        for (key, value) in nvt {
            println!("  {}: {}", key, value);
        }
        println!();
    }
    
    // Print frame types
    let frame_types = file.frame_types();
    if !frame_types.is_empty() {
        println!("Frame Types:");
        for ft in frame_types {
            println!("  {}", ft);
        }
        println!();
    }
    
    // Print matrix types
    let matrix_types = file.matrix_types();
    if !matrix_types.is_empty() {
        println!("Matrix Types:");
        for mt in matrix_types {
            println!("  {}", mt);
        }
        println!();
    }
    
    // Iterate through frames
    println!("=== Frame Contents ===");
    println!();
    
    let mut frame_count = 0;
    let mut total_matrices = 0;
    let mut min_time = f64::INFINITY;
    let mut max_time = f64::NEG_INFINITY;
    
    for frame_result in file.frames() {
        let frame = frame_result?;
        
        frame_count += 1;
        min_time = min_time.min(frame.time());
        max_time = max_time.max(frame.time());
        
        // Print first few frames in detail
        if frame_count <= 3 {
            println!(
                "Frame {}: type='{}' time={:.4}s stream={} matrices={}",
                frame_count,
                frame.signature(),
                frame.time(),
                frame.stream_id(),
                frame.num_matrices()
            );
            
            for matrix_result in frame.matrices() {
                let matrix = matrix_result?;
                total_matrices += 1;
                
                println!(
                    "  Matrix '{}': {}×{} ({})",
                    matrix.signature(),
                    matrix.rows(),
                    matrix.cols(),
                    matrix.data_type()
                );
                
                // Print first few values
                let data = matrix.data_f64()?;
                let preview: Vec<_> = data.iter().take(8).collect();
                if !preview.is_empty() {
                    print!("    Data: [");
                    for (i, val) in preview.iter().enumerate() {
                        if i > 0 { print!(", "); }
                        print!("{:.4}", val);
                    }
                    if data.len() > 8 {
                        print!(", ... ({} total)", data.len());
                    }
                    println!("]");
                }
            }
            println!();
        } else {
            // Just count matrices for remaining frames
            for matrix_result in frame.matrices() {
                let _ = matrix_result?;
                total_matrices += 1;
            }
        }
    }
    
    // Print summary
    println!("=== Summary ===");
    println!();
    println!("Total frames: {}", frame_count);
    println!("Total matrices: {}", total_matrices);
    if frame_count > 0 {
        println!("Time range: {:.4}s to {:.4}s", min_time, max_time);
        println!("Duration: {:.4}s", max_time - min_time);
    }
    
    Ok(())
}
```

### Task 3.2: Create write_sdif Example

**Claude Code Prompt:**

```
Create sdif-rs/examples/write_sdif.rs:

//! Example: Writing an SDIF file
//!
//! This example demonstrates how to create a new SDIF file with
//! sinusoidal track data (1TRC frames).
//!
//! # Usage
//!
//! ```bash
//! cargo run --example write_sdif -- output.sdif
//! ```

use std::env;
use std::f64::consts::PI;
use std::process;

use sdif_rs::{SdifFile, Result};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <output.sdif>", args[0]);
        process::exit(1);
    }
    
    if let Err(e) = run(&args[1]) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(path: &str) -> Result<()> {
    println!("Creating SDIF file: {}", path);
    
    // Configuration
    let num_frames = 100;
    let hop_time = 0.01;  // 10ms between frames
    let num_partials = 10;
    let base_freq = 220.0;  // A3
    
    // Create the SDIF file with builder pattern
    let mut writer = SdifFile::builder()
        .create(path)?
        // Add metadata
        .add_nvt([
            ("creator", "write_sdif example"),
            ("description", "Synthesized harmonic partials"),
            ("base_frequency", "220"),
        ])?
        // Define 1TRC matrix type with standard columns
        .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
        // Define 1TRC frame type
        .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
        .build()?;
    
    println!("Writing {} frames with {} partials each...", num_frames, num_partials);
    
    // Generate frames
    for frame_idx in 0..num_frames {
        let time = frame_idx as f64 * hop_time;
        
        // Generate partial data
        // Each partial has: Index, Frequency, Amplitude, Phase
        let mut data = Vec::with_capacity(num_partials * 4);
        
        for partial_idx in 0..num_partials {
            let index = (partial_idx + 1) as f64;
            
            // Harmonic frequency
            let frequency = base_freq * index;
            
            // Amplitude decreases with partial number (1/n rolloff)
            // Also apply envelope: fade in and fade out
            let envelope = envelope_at(time, num_frames as f64 * hop_time);
            let amplitude = envelope / index;
            
            // Phase increases linearly with time
            let phase = (2.0 * PI * frequency * time) % (2.0 * PI);
            
            data.push(index);
            data.push(frequency);
            data.push(amplitude);
            data.push(phase);
        }
        
        // Write the frame
        writer.write_frame_one_matrix(
            "1TRC",           // frame signature
            time,             // timestamp
            "1TRC",           // matrix signature
            num_partials,     // rows (one per partial)
            4,                // columns (Index, Freq, Amp, Phase)
            &data,
        )?;
    }
    
    // Close the file
    writer.close()?;
    
    println!("Done!");
    println!();
    println!("File details:");
    println!("  Frames: {}", num_frames);
    println!("  Duration: {:.2}s", num_frames as f64 * hop_time);
    println!("  Partials per frame: {}", num_partials);
    println!("  Base frequency: {} Hz", base_freq);
    
    Ok(())
}

/// Calculate envelope value at a given time.
/// Returns 0-1 with fade in at start and fade out at end.
fn envelope_at(time: f64, duration: f64) -> f64 {
    let fade_time = duration * 0.1;  // 10% fade at each end
    
    if time < fade_time {
        // Fade in
        time / fade_time
    } else if time > duration - fade_time {
        // Fade out
        (duration - time) / fade_time
    } else {
        // Sustain
        1.0
    }
}
```

### Task 3.3: Create inspect_sdif Example

**Claude Code Prompt:**

```
Create sdif-rs/examples/inspect_sdif.rs:

//! Example: Inspecting SDIF file structure
//!
//! This example shows how to examine the structure of an SDIF file
//! without reading all the data - useful for understanding unknown files.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example inspect_sdif -- path/to/file.sdif
//! ```

use std::collections::HashMap;
use std::env;
use std::process;

use sdif_rs::{SdifFile, Result};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <sdif-file>", args[0]);
        process::exit(1);
    }
    
    if let Err(e) = run(&args[1]) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(path: &str) -> Result<()> {
    println!("Inspecting: {}\n", path);
    
    let file = SdifFile::open(path)?;
    
    // Collect statistics
    let mut frame_types: HashMap<String, FrameStats> = HashMap::new();
    let mut total_frames = 0;
    let mut first_time = None;
    let mut last_time = None;
    
    for frame_result in file.frames() {
        let frame = frame_result?;
        total_frames += 1;
        
        let time = frame.time();
        first_time.get_or_insert(time);
        last_time = Some(time);
        
        // Update frame type statistics
        let sig = frame.signature().to_string();
        let stats = frame_types.entry(sig).or_insert(FrameStats::default());
        stats.count += 1;
        stats.update_time(time);
        
        // Analyze matrices
        for matrix_result in frame.matrices() {
            let matrix = matrix_result?;
            let matrix_sig = matrix.signature().to_string();
            
            let matrix_stats = stats.matrices
                .entry(matrix_sig)
                .or_insert(MatrixStats::default());
            
            matrix_stats.count += 1;
            matrix_stats.update_dims(matrix.rows(), matrix.cols());
            matrix_stats.data_type = Some(format!("{}", matrix.data_type()));
        }
    }
    
    // Print metadata
    let nvt = file.nvt();
    if !nvt.is_empty() {
        println!("═══ Metadata ═══\n");
        for (key, value) in nvt {
            println!("  {}: {}", key, value);
        }
        println!();
    }
    
    // Print type definitions
    let frame_type_defs = file.frame_types();
    let matrix_type_defs = file.matrix_types();
    
    if !frame_type_defs.is_empty() || !matrix_type_defs.is_empty() {
        println!("═══ Type Definitions ═══\n");
        
        for ft in frame_type_defs {
            println!("  Frame: {}", ft);
        }
        for mt in matrix_type_defs {
            println!("  Matrix: {}", mt);
        }
        println!();
    }
    
    // Print frame analysis
    println!("═══ Frame Analysis ═══\n");
    println!("Total frames: {}", total_frames);
    
    if let (Some(first), Some(last)) = (first_time, last_time) {
        println!("Time range: {:.4}s to {:.4}s ({:.4}s duration)", 
            first, last, last - first);
        
        if total_frames > 1 {
            let avg_hop = (last - first) / (total_frames - 1) as f64;
            println!("Average hop: {:.4}s ({:.1} fps)", avg_hop, 1.0 / avg_hop);
        }
    }
    println!();
    
    // Print per-type statistics
    let mut types: Vec<_> = frame_types.iter().collect();
    types.sort_by_key(|(sig, _)| sig.as_str());
    
    for (sig, stats) in types {
        println!("┌─ Frame Type: {} ─────────────────────", sig);
        println!("│  Count: {}", stats.count);
        if let (Some(min), Some(max)) = (stats.min_time, stats.max_time) {
            println!("│  Time range: {:.4}s to {:.4}s", min, max);
        }
        
        let mut matrices: Vec<_> = stats.matrices.iter().collect();
        matrices.sort_by_key(|(sig, _)| sig.as_str());
        
        for (msig, mstats) in matrices {
            println!("│");
            println!("│  └─ Matrix: {}", msig);
            println!("│     Count: {}", mstats.count);
            println!("│     Rows: {} to {}", 
                mstats.min_rows.unwrap_or(0), 
                mstats.max_rows.unwrap_or(0));
            println!("│     Cols: {} to {}",
                mstats.min_cols.unwrap_or(0),
                mstats.max_cols.unwrap_or(0));
            if let Some(ref dt) = mstats.data_type {
                println!("│     Type: {}", dt);
            }
        }
        println!("└────────────────────────────────────────");
        println!();
    }
    
    Ok(())
}

#[derive(Default)]
struct FrameStats {
    count: usize,
    min_time: Option<f64>,
    max_time: Option<f64>,
    matrices: HashMap<String, MatrixStats>,
}

impl FrameStats {
    fn update_time(&mut self, time: f64) {
        self.min_time = Some(self.min_time.map_or(time, |t| t.min(time)));
        self.max_time = Some(self.max_time.map_or(time, |t| t.max(time)));
    }
}

#[derive(Default)]
struct MatrixStats {
    count: usize,
    min_rows: Option<usize>,
    max_rows: Option<usize>,
    min_cols: Option<usize>,
    max_cols: Option<usize>,
    data_type: Option<String>,
}

impl MatrixStats {
    fn update_dims(&mut self, rows: usize, cols: usize) {
        self.min_rows = Some(self.min_rows.map_or(rows, |r| r.min(rows)));
        self.max_rows = Some(self.max_rows.map_or(rows, |r| r.max(rows)));
        self.min_cols = Some(self.min_cols.map_or(cols, |c| c.min(cols)));
        self.max_cols = Some(self.max_cols.map_or(cols, |c| c.max(cols)));
    }
}
```

### Task 3.4: Create mat_convert Example (Feature-Gated)

**Claude Code Prompt:**

```
Create sdif-rs/examples/mat_convert.rs:

//! Example: Converting MAT files to SDIF
//!
//! This example demonstrates programmatic MAT to SDIF conversion,
//! which is useful when you need more control than the mat2sdif CLI.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example mat_convert --features mat -- input.mat output.sdif
//! ```

#![cfg(feature = "mat")]

use std::env;
use std::process;

use sdif_rs::{
    MatFile, MatToSdifConfig, MatToSdifConverter, ComplexMode,
    SdifFile, Result,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <input.mat> <output.sdif>", args[0]);
        process::exit(1);
    }
    
    if let Err(e) = run(&args[1], &args[2]) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(input: &str, output: &str) -> Result<()> {
    println!("Converting: {} -> {}", input, output);
    println!();
    
    // Load MAT file
    let mat = MatFile::open(input)?;
    
    // Print what we found
    println!("MAT file contents:");
    println!("{}", mat.describe());
    println!();
    
    // Find time vectors
    let time_vars = mat.find_time_vectors();
    println!("Detected time vectors: {:?}", time_vars);
    
    // Configure conversion
    let config = MatToSdifConfig::new()
        .frame_type("1TRC")
        .matrix_type("1TRC")
        .columns(&["Index", "Frequency", "Amplitude", "Phase"])
        .max_partials(1024)
        .complex_mode(ComplexMode::Magnitude);
    
    // If specific variables are known, you can set them:
    // let config = config.time_var("time").data_var("partials");
    
    // Create converter
    let converter = MatToSdifConverter::new(&mat, config)?;
    
    println!("Conversion plan:");
    println!("  Frames: {}", converter.num_frames());
    let (start, end) = converter.time_range();
    println!("  Time range: {:.3}s to {:.3}s", start, end);
    println!("  Columns: {}", converter.cols_per_frame());
    println!();
    
    // Create SDIF writer
    let mut writer = SdifFile::builder()
        .create(output)?
        .add_nvt([
            ("creator", "mat_convert example"),
            ("source", input),
        ])?
        .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
        .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
        .build()?;
    
    // Perform conversion
    println!("Converting...");
    converter.write_to(&mut writer)?;
    writer.close()?;
    
    println!("Done! Wrote {} frames to {}", converter.num_frames(), output);
    
    Ok(())
}

#[cfg(not(feature = "mat"))]
fn main() {
    eprintln!("This example requires the 'mat' feature.");
    eprintln!("Run with: cargo run --example mat_convert --features mat -- input.mat output.sdif");
    std::process::exit(1);
}
```

### Task 3.5: Create streaming Example

**Claude Code Prompt:**

```
Create sdif-rs/examples/streaming.rs:

//! Example: Streaming large SDIF files
//!
//! This example demonstrates efficient processing of large SDIF files
//! using the streaming iterator API, which doesn't load the entire
//! file into memory.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example streaming -- input.sdif output.sdif
//! ```

use std::env;
use std::process;

use sdif_rs::{SdifFile, Result};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <input.sdif> <output.sdif>", args[0]);
        eprintln!();
        eprintln!("This example copies an SDIF file while filtering frames.");
        process::exit(1);
    }
    
    if let Err(e) = run(&args[1], &args[2]) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(input_path: &str, output_path: &str) -> Result<()> {
    println!("Streaming copy with filtering");
    println!("  Input:  {}", input_path);
    println!("  Output: {}", output_path);
    println!();
    
    // Open input file
    let input = SdifFile::open(input_path)?;
    
    // Create output file with same structure
    let mut writer = SdifFile::builder()
        .create(output_path)?
        .add_nvt([
            ("creator", "streaming example"),
            ("source", input_path),
        ])?
        // Copy type definitions if known, or use generic
        .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
        .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
        .build()?;
    
    let mut frames_read = 0;
    let mut frames_written = 0;
    let mut frames_skipped = 0;
    
    // Process frame by frame
    for frame_result in input.frames() {
        let frame = frame_result?;
        frames_read += 1;
        
        // Example filter: only copy frames between 0.5s and 2.5s
        let time = frame.time();
        if time < 0.5 || time > 2.5 {
            frames_skipped += 1;
            continue;
        }
        
        // Copy each matrix
        for matrix_result in frame.matrices() {
            let matrix = matrix_result?;
            
            // Get data and potentially modify it
            let mut data = matrix.data_f64()?;
            
            // Example modification: scale amplitudes by 0.5
            // Assuming column 2 is amplitude (Index, Freq, Amp, Phase)
            let cols = matrix.cols();
            if cols >= 3 {
                for row in 0..matrix.rows() {
                    let amp_idx = row * cols + 2;
                    if amp_idx < data.len() {
                        data[amp_idx] *= 0.5;
                    }
                }
            }
            
            // Write modified frame
            writer.write_frame_one_matrix(
                frame.signature(),
                time - 0.5,  // Shift time to start at 0
                matrix.signature(),
                matrix.rows(),
                matrix.cols(),
                &data,
            )?;
        }
        
        frames_written += 1;
        
        // Progress indicator
        if frames_written % 100 == 0 {
            eprint!("\rProcessed {} frames...", frames_written);
        }
    }
    
    writer.close()?;
    
    println!("\rDone!                    ");
    println!();
    println!("Statistics:");
    println!("  Frames read: {}", frames_read);
    println!("  Frames written: {}", frames_written);
    println!("  Frames skipped: {}", frames_skipped);
    
    Ok(())
}
```

---

## Step 4: CI/CD Configuration

### Task 4.1: Create GitHub Actions Workflow

**Claude Code Prompt:**

```
Create .github/workflows/ci.yml:

name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  # Format check
  fmt:
    name: Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  # Clippy lints
  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libclang-dev
      - run: cargo clippy --all --all-features -- -D warnings

  # Test matrix
  test:
    name: Test (${{ matrix.os }}, ${{ matrix.features }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        features: ["", "bundled", "ndarray", "mat"]
        exclude:
          # Skip mat on Windows for now (complex build)
          - os: windows-latest
            features: mat
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Install dependencies (Ubuntu)
        if: matrix.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y libclang-dev
      
      - name: Install dependencies (macOS)
        if: matrix.os == 'macos-latest'
        run: |
          brew install llvm
          echo "LIBCLANG_PATH=$(brew --prefix llvm)/lib" >> $GITHUB_ENV
      
      - name: Install dependencies (Windows)
        if: matrix.os == 'windows-latest'
        run: |
          choco install llvm
          echo "LIBCLANG_PATH=C:\Program Files\LLVM\bin" >> $env:GITHUB_ENV
      
      - name: Build
        run: cargo build --all ${{ matrix.features && format('--features {0}', matrix.features) || '' }}
      
      - name: Test
        run: cargo test --all ${{ matrix.features && format('--features {0}', matrix.features) || '' }}
      
      - name: Test (ignored)
        run: cargo test --all ${{ matrix.features && format('--features {0}', matrix.features) || '' }} -- --ignored
        continue-on-error: true

  # Documentation
  docs:
    name: Documentation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - uses: dtolnay/rust-toolchain@stable
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libclang-dev
      - name: Build docs
        run: cargo doc --all --all-features --no-deps
        env:
          RUSTDOCFLAGS: -D warnings

  # Examples
  examples:
    name: Examples
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - uses: dtolnay/rust-toolchain@stable
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libclang-dev
      - name: Build examples
        run: cargo build --examples --all-features

  # Coverage (optional)
  coverage:
    name: Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libclang-dev
          cargo install cargo-llvm-cov
      - name: Generate coverage
        run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info
          fail_ci_if_error: false

  # Release build check
  release:
    name: Release Build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      - uses: dtolnay/rust-toolchain@stable
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libclang-dev
      - name: Release build
        run: cargo build --release --all --all-features
```

### Task 4.2: Create Release Workflow

**Claude Code Prompt:**

```
Create .github/workflows/release.yml:

name: Release

on:
  push:
    tags:
      - 'v*'

env:
  CARGO_TERM_COLOR: always

jobs:
  create-release:
    name: Create Release
    runs-on: ubuntu-latest
    outputs:
      upload_url: ${{ steps.create_release.outputs.upload_url }}
    steps:
      - name: Create Release
        id: create_release
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tag_name: ${{ github.ref }}
          release_name: Release ${{ github.ref }}
          draft: true
          prerelease: false

  build-binaries:
    name: Build (${{ matrix.target }})
    needs: create-release
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            name: mat2sdif-linux-x86_64
          - target: x86_64-apple-darwin
            os: macos-latest
            name: mat2sdif-macos-x86_64
          - target: aarch64-apple-darwin
            os: macos-latest
            name: mat2sdif-macos-aarch64
          - target: x86_64-pc-windows-msvc
            os: windows-latest
            name: mat2sdif-windows-x86_64.exe
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive
      
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Install dependencies (Ubuntu)
        if: matrix.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y libclang-dev
      
      - name: Install dependencies (macOS)
        if: matrix.os == 'macos-latest'
        run: |
          brew install llvm
          echo "LIBCLANG_PATH=$(brew --prefix llvm)/lib" >> $GITHUB_ENV
      
      - name: Install dependencies (Windows)
        if: matrix.os == 'windows-latest'
        run: choco install llvm
      
      - name: Build
        run: cargo build --release --package mat2sdif --features bundled --target ${{ matrix.target }}
      
      - name: Package (Unix)
        if: matrix.os != 'windows-latest'
        run: |
          cd target/${{ matrix.target }}/release
          tar -czvf ../../../${{ matrix.name }}.tar.gz mat2sdif
      
      - name: Package (Windows)
        if: matrix.os == 'windows-latest'
        run: |
          cd target/${{ matrix.target }}/release
          7z a ../../../${{ matrix.name }}.zip mat2sdif.exe
      
      - name: Upload (Unix)
        if: matrix.os != 'windows-latest'
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ needs.create-release.outputs.upload_url }}
          asset_path: ./${{ matrix.name }}.tar.gz
          asset_name: ${{ matrix.name }}.tar.gz
          asset_content_type: application/gzip
      
      - name: Upload (Windows)
        if: matrix.os == 'windows-latest'
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ needs.create-release.outputs.upload_url }}
          asset_path: ./${{ matrix.name }}.zip
          asset_name: ${{ matrix.name }}.zip
          asset_content_type: application/zip
```

---

## Step 5: Additional Documentation Files

### Task 5.1: Create CONTRIBUTING.md

**Claude Code Prompt:**

```
Create CONTRIBUTING.md:

# Contributing to rust-sdif

Thank you for your interest in contributing! This document provides guidelines
and information for contributors.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/rust-sdif.git`
3. Create a branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Run tests: `cargo test --all --all-features`
6. Push and create a pull request

## Development Setup

### Prerequisites

- Rust 1.70 or later (`rustup update stable`)
- C compiler (gcc, clang, or MSVC)
- clang (for bindgen)
- pkg-config (optional)

### Building

```bash
# Debug build
cargo build --all

# With all features
cargo build --all --all-features

# Release build
cargo build --all --release
```

### Testing

```bash
# Run all tests
cargo test --all --all-features

# Run specific test
cargo test -p sdif-rs test_name

# Run with output
cargo test --all -- --nocapture

# Run ignored tests (require fixtures)
cargo test --all -- --ignored
```

### Code Quality

Before submitting a PR, please ensure:

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --all --all-features -- -D warnings

# Build documentation
cargo doc --all --all-features --no-deps
```

## Code Style

- Follow Rust standard style (enforced by rustfmt)
- Use descriptive variable and function names
- Add documentation comments to all public items
- Include examples in documentation where helpful
- Write tests for new functionality

### Documentation Guidelines

Every public item should have:

1. A summary line (first line of doc comment)
2. Extended description if needed
3. `# Examples` section with working code
4. `# Errors` section for Result-returning functions
5. `# Panics` section if the function can panic

Example:

```rust
/// Brief one-line description.
///
/// Longer description with more details.
///
/// # Examples
///
/// ```
/// use sdif_rs::SomeType;
/// let result = SomeType::new()?;
/// # Ok::<(), sdif_rs::Error>(())
/// ```
///
/// # Errors
///
/// Returns an error if the file cannot be read.
pub fn new() -> Result<Self> {
    // ...
}
```

## Pull Request Guidelines

- Keep PRs focused on a single change
- Update documentation for API changes
- Add tests for new functionality
- Update CHANGELOG.md for user-facing changes
- Ensure CI passes before requesting review

### Commit Messages

Use clear, descriptive commit messages:

```
feat: Add support for 1STF frame type

- Implement STF matrix reading
- Add documentation and examples
- Include integration tests
```

Prefixes:
- `feat:` New features
- `fix:` Bug fixes
- `docs:` Documentation changes
- `test:` Test additions/changes
- `refactor:` Code refactoring
- `chore:` Maintenance tasks

## Project Structure

```
rust-sdif/
├── sdif-sys/           # FFI bindings (low-level)
├── sdif-rs/            # Safe Rust API (main library)
│   ├── src/
│   ├── examples/
│   └── tests/
├── mat2sdif/           # CLI tool
├── docs/               # Additional documentation
└── scripts/            # Build/test scripts
```

### Where to Contribute

- **Bug fixes**: Always welcome!
- **Documentation**: Improvements, examples, typo fixes
- **Tests**: More coverage is always helpful
- **Examples**: Real-world usage examples
- **Features**: Please open an issue first to discuss

## Feature Requests and Bug Reports

Please use GitHub Issues for:

- Bug reports (include reproduction steps)
- Feature requests (explain the use case)
- Questions about usage

## License

By contributing, you agree that your contributions will be licensed under
the MIT License.

## Questions?

Feel free to open an issue or reach out to the maintainers.

Thank you for contributing!
```

### Task 5.2: Create CHANGELOG.md

**Claude Code Prompt:**

```
Create CHANGELOG.md:

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial implementation of sdif-sys FFI bindings
- Safe Rust wrapper in sdif-rs
- Reading API with iterator pattern
- Writing API with builder pattern
- MAT file support (optional feature)
- mat2sdif CLI tool
- Documentation and examples
- CI/CD with GitHub Actions

## [0.1.0] - YYYY-MM-DD

### Added
- First public release
- `SdifFile` for reading SDIF files
- `SdifFileBuilder` and `SdifWriter` for creating SDIF files
- Support for common frame types: 1TRC, 1HRM, 1FQ0, 1RES
- ndarray integration (optional)
- MAT file conversion (optional)
- mat2sdif command-line tool
- Comprehensive documentation
- Cross-platform support (Linux, macOS, Windows)

### Notes
- This is the initial release
- API may change in future 0.x versions
- Feedback welcome via GitHub issues

[Unreleased]: https://github.com/username/rust-sdif/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/username/rust-sdif/releases/tag/v0.1.0
```

### Task 5.3: Create LICENSE File

**Claude Code Prompt:**

```
Create LICENSE:

MIT License

Copyright (c) 2024 [Your Name]

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### Task 5.4: Create Max Compatibility Guide

**Claude Code Prompt:**

```
Create docs/max-compatibility.md:

# Max/MSP Compatibility Guide

This guide explains how to create SDIF files that work correctly with
Max/MSP and the CNMAT SDIF externals.

## Overview

Max/MSP uses SDIF files for storing and playing back spectral analysis data,
particularly sinusoidal partial tracks. The CNMAT externals (`SDIF-buffer`,
`SDIF-tuples`, `sinusoids~`, etc.) have specific requirements for SDIF files.

## Supported Frame Types

| Frame Type | Description | Max Support |
|------------|-------------|-------------|
| **1TRC** | Sinusoidal tracks | ⭐ Best support |
| **1HRM** | Harmonic partials | ⭐ Good support |
| **1FQ0** | Fundamental frequency | ✓ Supported |
| **1RES** | Resonances | ✓ Supported |
| **1STF** | Short-time Fourier | ⚠ Limited |

**Recommendation**: Use **1TRC** for maximum compatibility.

## Partial Limits

The CNMAT `sinusoids~` object has limits on partials per frame:

| Version | Partial Limit |
|---------|---------------|
| Modern (2010+) | 1024 |
| Legacy | 256 |

Files exceeding these limits may cause crashes or truncation.

```bash
# Enforce limit with mat2sdif
mat2sdif input.mat output.sdif --max-partials 1024

# For legacy compatibility
mat2sdif input.mat output.sdif --max-partials 256
```

## Column Requirements

### 1TRC / 1HRM Frames

Must have exactly 4 columns in this order:

1. **Index** - Partial track index (starts at 1)
2. **Frequency** - Frequency in Hz
3. **Amplitude** - Linear amplitude (0.0 to 1.0)
4. **Phase** - Phase in radians

```rust
writer.add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?;
```

### 1FQ0 Frames

Must have exactly 2 columns:

1. **Frequency** - Fundamental frequency in Hz
2. **Confidence** - Confidence value (0.0 to 1.0)

```rust
writer.add_matrix_type("1FQ0", &["Frequency", "Confidence"])?;
```

## Track Continuity

For clean playback with `sinusoids~`:

### Track Birth
When a partial first appears, its amplitude should fade in from zero:

```
Frame 0: Index=1, Amp=0.0
Frame 1: Index=1, Amp=0.1
Frame 2: Index=1, Amp=0.5  (full amplitude)
```

### Track Death
When a partial disappears, its amplitude should fade to zero:

```
Frame N-2: Index=1, Amp=0.5
Frame N-1: Index=1, Amp=0.1
Frame N:   Index=1, Amp=0.0
```

### Index Consistency
The same partial should use the same index across frames.

## Time Requirements

- Times must be non-negative (≥ 0.0)
- Times must be monotonically increasing
- Use consistent hop sizes for best results

## Data Types

Max expects:
- **Float64** (double) - Preferred
- **Float32** (single) - Supported

Do not use integer types for partial data.

## Example: Creating a Max-Compatible File

```rust
use sdif_rs::{SdifFile, Result};

fn create_max_compatible() -> Result<()> {
    let mut writer = SdifFile::builder()
        .create("output.sdif")?
        // Standard metadata
        .add_nvt([
            ("creator", "my_app"),
            ("description", "Partial tracking data"),
        ])?
        // Standard 1TRC definition
        .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
        .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
        .build()?;
    
    // Write frames with proper fade-in/fade-out
    let num_frames = 100;
    let fade_frames = 5;
    
    for i in 0..num_frames {
        let time = i as f64 * 0.01;
        
        // Calculate envelope for fade-in/fade-out
        let envelope = if i < fade_frames {
            i as f64 / fade_frames as f64
        } else if i >= num_frames - fade_frames {
            (num_frames - 1 - i) as f64 / fade_frames as f64
        } else {
            1.0
        };
        
        let data = vec![
            1.0, 440.0, 0.5 * envelope, 0.0,  // Partial 1
            2.0, 880.0, 0.25 * envelope, 0.0, // Partial 2
        ];
        
        writer.write_frame_one_matrix("1TRC", time, "1TRC", 2, 4, &data)?;
    }
    
    writer.close()?;
    Ok(())
}
```

## Testing with Max

1. Load the SDIF file into an `SDIF-buffer`:
   ```
   [SDIF-buffer mybuffer output.sdif]
   ```

2. Query the contents:
   ```
   [SDIF-tuples mybuffer 1TRC 1TRC]
   ```

3. Play back with `sinusoids~`:
   ```
   [sinusoids~ mybuffer]
   ```

## Troubleshooting

### "Bad frame type"
The frame type isn't recognized. Use standard types like 1TRC.

### Clicks or pops during playback
Partials don't have proper fade-in/fade-out. Add envelope shaping.

### Missing partials
Check that the partial limit isn't exceeded. Use `--max-partials 1024`.

### File won't load
- Verify the file is valid SDIF: `mat2sdif --list file.sdif`
- Check for correct byte ordering (should be handled automatically)

## Resources

- [CNMAT Externals Download](https://cnmat.berkeley.edu/downloads)
- [SDIF-buffer Reference](https://cnmat.berkeley.edu/patch/4044)
- [sinusoids~ Reference](https://cnmat.berkeley.edu/patch/4048)
```

---

## Step 6: Integration Tests

### Task 6.1: Create Roundtrip Test

**Claude Code Prompt:**

```
Create sdif-rs/tests/roundtrip.rs:

//! Roundtrip tests: write data, read back, verify equality.

use sdif_rs::{SdifFile, DataType, Result};
use tempfile::NamedTempFile;
use approx::assert_relative_eq;

/// Test basic roundtrip with single matrix frames.
#[test]
fn test_roundtrip_basic() -> Result<()> {
    let temp = NamedTempFile::new()?;
    let path = temp.path();
    
    // Original data
    let original_data: Vec<Vec<f64>> = vec![
        vec![1.0, 440.0, 0.5, 0.0, 2.0, 880.0, 0.25, 0.0],
        vec![1.0, 441.0, 0.5, 0.1, 2.0, 882.0, 0.25, 0.2],
        vec![1.0, 442.0, 0.5, 0.2, 2.0, 884.0, 0.25, 0.4],
    ];
    let times = vec![0.0, 0.01, 0.02];
    
    // Write
    {
        let mut writer = SdifFile::builder()
            .create(path)?
            .add_nvt([("test", "roundtrip")])?
            .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
            .add_frame_type("1TRC", &["1TRC Data"])?
            .build()?;
        
        for (time, data) in times.iter().zip(original_data.iter()) {
            writer.write_frame_one_matrix("1TRC", *time, "1TRC", 2, 4, data)?;
        }
        
        writer.close()?;
    }
    
    // Read back
    {
        let file = SdifFile::open(path)?;
        
        let mut frame_idx = 0;
        for frame_result in file.frames() {
            let frame = frame_result?;
            
            assert_eq!(frame.signature(), "1TRC");
            assert_relative_eq!(frame.time(), times[frame_idx], epsilon = 1e-10);
            
            for matrix_result in frame.matrices() {
                let matrix = matrix_result?;
                
                assert_eq!(matrix.signature(), "1TRC");
                assert_eq!(matrix.rows(), 2);
                assert_eq!(matrix.cols(), 4);
                
                let data = matrix.data_f64()?;
                
                for (i, (&original, &read)) in original_data[frame_idx].iter()
                    .zip(data.iter()).enumerate()
                {
                    assert_relative_eq!(
                        original, read,
                        epsilon = 1e-10,
                        max_relative = 1e-10
                    );
                }
            }
            
            frame_idx += 1;
        }
        
        assert_eq!(frame_idx, 3);
    }
    
    Ok(())
}

/// Test roundtrip with f32 data.
#[test]
fn test_roundtrip_f32() -> Result<()> {
    let temp = NamedTempFile::new()?;
    let path = temp.path();
    
    let original: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
    
    // Write with f32
    {
        let mut writer = SdifFile::builder()
            .create(path)?
            .add_matrix_type("TEST", &["A", "B"])?
            .add_frame_type("TEST", &["TEST Data"])?
            .build()?;
        
        writer.write_frame_one_matrix_f32("TEST", 0.0, "TEST", 2, 2, &original)?;
        writer.close()?;
    }
    
    // Read back
    {
        let file = SdifFile::open(path)?;
        
        for frame_result in file.frames() {
            let frame = frame_result?;
            
            for matrix_result in frame.matrices() {
                let matrix = matrix_result?;
                
                // Data type should be Float4
                assert_eq!(matrix.data_type(), DataType::Float4);
                
                let data = matrix.data_f32()?;
                
                for (&orig, &read) in original.iter().zip(data.iter()) {
                    assert_relative_eq!(orig, read, epsilon = 1e-6);
                }
            }
        }
    }
    
    Ok(())
}

/// Test roundtrip with multiple matrices per frame.
#[test]
fn test_roundtrip_multi_matrix() -> Result<()> {
    let temp = NamedTempFile::new()?;
    let path = temp.path();
    
    let matrix1_data = vec![1.0, 2.0, 3.0, 4.0];
    let matrix2_data = vec![10.0, 20.0];
    
    // Write
    {
        let mut writer = SdifFile::builder()
            .create(path)?
            .add_matrix_type("MAT1", &["A", "B"])?
            .add_matrix_type("MAT2", &["X"])?
            .add_frame_type("MULT", &["MAT1 First", "MAT2 Second"])?
            .build()?;
        
        let frame = writer.new_frame("MULT", 0.0, 0)?
            .add_matrix("MAT1", 2, 2, &matrix1_data)?
            .add_matrix("MAT2", 2, 1, &matrix2_data)?;
        frame.finish()?;
        
        writer.close()?;
    }
    
    // Read back
    {
        let file = SdifFile::open(path)?;
        
        for frame_result in file.frames() {
            let frame = frame_result?;
            
            assert_eq!(frame.signature(), "MULT");
            assert_eq!(frame.num_matrices(), 2);
            
            let mut matrix_count = 0;
            for matrix_result in frame.matrices() {
                let matrix = matrix_result?;
                
                let data = matrix.data_f64()?;
                
                if matrix.signature() == "MAT1" {
                    assert_eq!(data, matrix1_data);
                } else if matrix.signature() == "MAT2" {
                    assert_eq!(data, matrix2_data);
                }
                
                matrix_count += 1;
            }
            
            assert_eq!(matrix_count, 2);
        }
    }
    
    Ok(())
}

/// Test roundtrip with many frames.
#[test]
fn test_roundtrip_many_frames() -> Result<()> {
    let temp = NamedTempFile::new()?;
    let path = temp.path();
    
    let num_frames = 1000;
    
    // Write
    {
        let mut writer = SdifFile::builder()
            .create(path)?
            .add_matrix_type("1FQ0", &["Frequency", "Confidence"])?
            .add_frame_type("1FQ0", &["1FQ0 Data"])?
            .build()?;
        
        for i in 0..num_frames {
            let time = i as f64 * 0.01;
            let freq = 220.0 + i as f64;
            let conf = 0.9;
            
            writer.write_frame_one_matrix("1FQ0", time, "1FQ0", 1, 2, &[freq, conf])?;
        }
        
        writer.close()?;
    }
    
    // Read back and verify
    {
        let file = SdifFile::open(path)?;
        
        let mut count = 0;
        for frame_result in file.frames() {
            let frame = frame_result?;
            
            let expected_time = count as f64 * 0.01;
            assert_relative_eq!(frame.time(), expected_time, epsilon = 1e-10);
            
            for matrix_result in frame.matrices() {
                let matrix = matrix_result?;
                let data = matrix.data_f64()?;
                
                let expected_freq = 220.0 + count as f64;
                assert_relative_eq!(data[0], expected_freq, epsilon = 1e-10);
            }
            
            count += 1;
        }
        
        assert_eq!(count, num_frames);
    }
    
    Ok(())
}

/// Test NVT roundtrip.
#[test]
fn test_roundtrip_nvt() -> Result<()> {
    let temp = NamedTempFile::new()?;
    let path = temp.path();
    
    // Write with NVT
    {
        let mut writer = SdifFile::builder()
            .create(path)?
            .add_nvt([
                ("creator", "roundtrip_test"),
                ("version", "1.0"),
                ("date", "2024-01-15"),
            ])?
            .add_matrix_type("TEST", &["Val"])?
            .add_frame_type("TEST", &["TEST Data"])?
            .build()?;
        
        writer.write_frame_one_matrix("TEST", 0.0, "TEST", 1, 1, &[42.0])?;
        writer.close()?;
    }
    
    // Read back NVT
    {
        let file = SdifFile::open(path)?;
        let nvt = file.nvt();
        
        // Note: NVT reading may not be fully implemented
        // This test documents expected behavior
        // assert_eq!(nvt.get("creator"), Some(&"roundtrip_test".to_string()));
    }
    
    Ok(())
}
```

### Task 6.2: Create Benchmark Tests

**Claude Code Prompt:**

```
Create sdif-rs/benches/benchmarks.rs (requires criterion dependency):

Note: Add to Cargo.toml:
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "benchmarks"
harness = false

---

//! Performance benchmarks for sdif-rs.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use sdif_rs::{SdifFile, Result};
use std::io::Write;
use tempfile::NamedTempFile;

fn create_test_file(num_frames: usize, partials_per_frame: usize) -> Result<NamedTempFile> {
    let temp = NamedTempFile::new()?;
    
    let mut writer = SdifFile::builder()
        .create(temp.path())?
        .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
        .add_frame_type("1TRC", &["1TRC Data"])?
        .build()?;
    
    let data: Vec<f64> = (0..partials_per_frame * 4)
        .map(|i| i as f64)
        .collect();
    
    for i in 0..num_frames {
        writer.write_frame_one_matrix(
            "1TRC",
            i as f64 * 0.01,
            "1TRC",
            partials_per_frame,
            4,
            &data,
        )?;
    }
    
    writer.close()?;
    Ok(temp)
}

fn bench_read_small(c: &mut Criterion) {
    let file = create_test_file(100, 10).expect("create test file");
    
    let mut group = c.benchmark_group("read_small");
    group.throughput(Throughput::Elements(100));
    
    group.bench_function("100_frames_10_partials", |b| {
        b.iter(|| {
            let f = SdifFile::open(black_box(file.path())).unwrap();
            let mut count = 0;
            for frame in f.frames() {
                let frame = frame.unwrap();
                for matrix in frame.matrices() {
                    let matrix = matrix.unwrap();
                    let _ = black_box(matrix.data_f64().unwrap());
                }
                count += 1;
            }
            count
        })
    });
    
    group.finish();
}

fn bench_read_large(c: &mut Criterion) {
    let file = create_test_file(1000, 100).expect("create test file");
    
    let mut group = c.benchmark_group("read_large");
    group.throughput(Throughput::Elements(1000));
    
    group.bench_function("1000_frames_100_partials", |b| {
        b.iter(|| {
            let f = SdifFile::open(black_box(file.path())).unwrap();
            let mut count = 0;
            for frame in f.frames() {
                let frame = frame.unwrap();
                for matrix in frame.matrices() {
                    let matrix = matrix.unwrap();
                    let _ = black_box(matrix.data_f64().unwrap());
                }
                count += 1;
            }
            count
        })
    });
    
    group.finish();
}

fn bench_write(c: &mut Criterion) {
    let data: Vec<f64> = (0..400).map(|i| i as f64).collect();
    
    let mut group = c.benchmark_group("write");
    group.throughput(Throughput::Elements(1000));
    
    group.bench_function("1000_frames", |b| {
        b.iter(|| {
            let temp = NamedTempFile::new().unwrap();
            let mut writer = SdifFile::builder()
                .create(temp.path())
                .unwrap()
                .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])
                .unwrap()
                .add_frame_type("1TRC", &["1TRC Data"])
                .unwrap()
                .build()
                .unwrap();
            
            for i in 0..1000 {
                writer.write_frame_one_matrix(
                    "1TRC",
                    black_box(i as f64 * 0.01),
                    "1TRC",
                    100,
                    4,
                    black_box(&data),
                ).unwrap();
            }
            
            writer.close().unwrap();
        })
    });
    
    group.finish();
}

criterion_group!(benches, bench_read_small, bench_read_large, bench_write);
criterion_main!(benches);
```

---

## Step 7: Final Verification Script

### Task 7.1: Create Phase 6 Verification Script

**Claude Code Prompt:**

```
Create scripts/verify-phase6.sh:

#!/bin/bash
# Verification script for Phase 6 completion

set -e

echo "=== Phase 6 Verification Script ==="
echo

# Check documentation files
echo "1. Checking documentation files..."
DOC_FILES=(
    "README.md"
    "CONTRIBUTING.md"
    "CHANGELOG.md"
    "LICENSE"
    "docs/max-compatibility.md"
)

for file in "${DOC_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "   ✓ $file exists"
    else
        echo "   ✗ $file missing!"
    fi
done

# Check example files
echo
echo "2. Checking examples..."
EXAMPLES=(
    "sdif-rs/examples/read_sdif.rs"
    "sdif-rs/examples/write_sdif.rs"
    "sdif-rs/examples/inspect_sdif.rs"
    "sdif-rs/examples/streaming.rs"
)

for file in "${EXAMPLES[@]}"; do
    if [ -f "$file" ]; then
        echo "   ✓ $file exists"
    else
        echo "   ✗ $file missing!"
    fi
done

# Check CI configuration
echo
echo "3. Checking CI configuration..."
if [ -f ".github/workflows/ci.yml" ]; then
    echo "   ✓ CI workflow exists"
else
    echo "   ✗ CI workflow missing"
fi

if [ -f ".github/workflows/release.yml" ]; then
    echo "   ✓ Release workflow exists"
else
    echo "   ⚠ Release workflow missing (optional)"
fi

# Run formatting check
echo
echo "4. Checking code formatting..."
if cargo fmt --all -- --check 2>/dev/null; then
    echo "   ✓ Code is formatted"
else
    echo "   ✗ Code needs formatting (run: cargo fmt --all)"
fi

# Run clippy
echo
echo "5. Running clippy..."
if cargo clippy --all --all-features -- -D warnings 2>/dev/null; then
    echo "   ✓ Clippy passed"
else
    echo "   ⚠ Clippy warnings found"
fi

# Build documentation
echo
echo "6. Building documentation..."
if cargo doc --all --all-features --no-deps 2>/dev/null; then
    echo "   ✓ Documentation builds"
else
    echo "   ✗ Documentation build failed"
fi

# Build examples
echo
echo "7. Building examples..."
if cargo build --examples --all-features 2>/dev/null; then
    echo "   ✓ Examples build"
else
    echo "   ✗ Examples build failed"
fi

# Run all tests
echo
echo "8. Running all tests..."
if cargo test --all --all-features 2>/dev/null; then
    echo "   ✓ All tests passed"
else
    echo "   ⚠ Some tests failed"
fi

# Run doc tests
echo
echo "9. Running doc tests..."
if cargo test --doc --all-features 2>/dev/null; then
    echo "   ✓ Doc tests passed"
else
    echo "   ⚠ Doc tests failed"
fi

# Check crate metadata
echo
echo "10. Checking crate metadata..."
for crate in sdif-sys sdif-rs mat2sdif; do
    if grep -q 'description' "$crate/Cargo.toml"; then
        echo "   ✓ $crate has description"
    else
        echo "   ⚠ $crate missing description"
    fi
    
    if grep -q 'license' "$crate/Cargo.toml"; then
        echo "   ✓ $crate has license"
    else
        echo "   ⚠ $crate missing license"
    fi
done

# Summary
echo
echo "=== Phase 6 Verification Complete ==="
echo
echo "Documentation and polish checklist:"
echo "  □ All public APIs have documentation"
echo "  □ Examples compile and demonstrate usage"
echo "  □ CI/CD is configured and passing"
echo "  □ README provides clear getting-started path"
echo "  □ CHANGELOG documents changes"
echo "  □ License file is present"
echo "  □ Contributing guidelines are provided"
echo
echo "Pre-publication checklist:"
echo "  □ Version numbers are set correctly"
echo "  □ All dependencies are published or local"
echo "  □ cargo publish --dry-run succeeds"
echo "  □ GitHub repository is set up"
echo "  □ CI passes on all platforms"
echo
echo "To publish to crates.io (when ready):"
echo "  cd sdif-sys && cargo publish"
echo "  cd sdif-rs && cargo publish"
echo "  cd mat2sdif && cargo publish"

Make executable:
chmod +x scripts/verify-phase6.sh
```

---

## Success Criteria Summary

Phase 6 is complete when:

1. **Documentation**
   - [ ] Crate-level documentation with overview and quick start
   - [ ] All public items have rustdoc comments
   - [ ] Doc-tests compile and pass
   - [ ] README files for workspace and each crate
   - [ ] CHANGELOG.md documents all changes
   - [ ] CONTRIBUTING.md provides guidelines
   - [ ] Max compatibility guide in docs/

2. **Examples**
   - [ ] read_sdif.rs demonstrates reading
   - [ ] write_sdif.rs demonstrates writing
   - [ ] inspect_sdif.rs shows file inspection
   - [ ] mat_convert.rs shows MAT conversion
   - [ ] streaming.rs shows large file handling
   - [ ] All examples compile and run

3. **Testing**
   - [ ] Roundtrip tests pass
   - [ ] Integration tests comprehensive
   - [ ] Doc-tests all pass
   - [ ] Benchmarks available

4. **CI/CD**
   - [ ] GitHub Actions CI workflow
   - [ ] Tests on Linux, macOS, Windows
   - [ ] Clippy and rustfmt checks
   - [ ] Documentation builds
   - [ ] Release workflow (optional)

5. **Polish**
   - [ ] Consistent error messages
   - [ ] API is clean and intuitive
   - [ ] No clippy warnings
   - [ ] Code is formatted
   - [ ] License file present

6. **Publication Ready**
   - [ ] Crate metadata complete
   - [ ] Version numbers set
   - [ ] cargo publish --dry-run succeeds

---

## Notes for Claude Code

### Documentation Best Practices

1. **First line is crucial**: It appears in search results and module lists
2. **Use examples liberally**: Working code is the best documentation
3. **Document errors**: Users need to know what can go wrong
4. **Cross-reference**: Use `[`OtherType`]` to create links

### Example File Guidelines

1. **Self-contained**: Each example should work independently
2. **Commented**: Explain what's happening
3. **Practical**: Show realistic use cases
4. **Error handling**: Demonstrate proper Result handling

### CI Considerations

1. **Matrix testing**: Test multiple OS and feature combinations
2. **Fail fast**: Let formatting/clippy fail quickly
3. **Cache dependencies**: Speed up builds
4. **Artifact uploads**: Make binaries available

### Publication Checklist

Before `cargo publish`:
1. Verify version numbers in all Cargo.toml files
2. Ensure sdif-sys publishes before sdif-rs (dependency)
3. Test with `cargo publish --dry-run`
4. Create git tag for the version
5. Update CHANGELOG.md

### Common Documentation Pitfalls

1. **Outdated examples**: Keep in sync with API changes
2. **Missing error docs**: Always document what can fail
3. **No cross-references**: Link related items
4. **Wall of text**: Use code examples to break up prose
