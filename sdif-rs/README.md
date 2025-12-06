# sdif-rs

Safe, idiomatic Rust wrapper for reading and writing SDIF (Sound Description Interchange Format) files.

## Overview

SDIF is a standard format for storing and exchanging sound descriptions, commonly used for:

- Sinusoidal/additive synthesis data (1TRC frames)
- Spectral analysis results
- Pitch tracking (1FQ0 frames)
- Harmonic analysis (1HRM frames)

This crate provides a safe Rust API on top of the IRCAM SDIF C library.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sdif-rs = "0.1"
```

### Features

- `ndarray` - Enable ndarray integration for matrix data
- `bundled` - Compile SDIF C library from source
- `static` - Force static linking

## Quick Start

### Reading SDIF Files

```rust
use sdif_rs::{SdifFile, Result};

fn main() -> Result<()> {
    let file = SdifFile::open("analysis.sdif")?;

    // Read metadata
    if let Some(creator) = file.nvt_get("creator") {
        println!("Created by: {}", creator);
    }

    // Iterate over frames
    for frame in file.frames() {
        let frame = frame?;
        println!("Frame {} at {:.3}s", frame.signature(), frame.time());

        // Iterate over matrices in each frame
        for matrix in frame.matrices() {
            let matrix = matrix?;
            println!("  Matrix {}: {}x{}",
                matrix.signature(),
                matrix.rows(),
                matrix.cols()
            );

            // Get matrix data
            let data = matrix.data_f64()?;
            println!("  First value: {:.4}", data[0]);
        }
    }

    Ok(())
}
```

### With ndarray

Enable the `ndarray` feature for 2D array support:

```toml
[dependencies]
sdif-rs = { version = "0.1", features = ["ndarray"] }
```

```rust
use sdif_rs::SdifFile;
use ndarray::Array2;

let file = SdifFile::open("analysis.sdif")?;

for frame in file.frames() {
    for matrix in frame?.matrices() {
        let matrix = matrix?;
        let array: Array2<f64> = matrix.to_array_f64()?;

        // Use ndarray operations
        let mean = array.mean().unwrap_or(0.0);
        println!("Mean value: {:.4}", mean);
    }
}
```

### Writing SDIF Files

```rust
use sdif_rs::{SdifFile, Result};

fn main() -> Result<()> {
    // Create a new SDIF file with the builder pattern
    let mut writer = SdifFile::builder()
        .create("output.sdif")?
        // Add metadata
        .add_nvt([
            ("creator", "my-application"),
            ("date", "2024-01-01"),
        ])?
        // Define matrix type with column names
        .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
        // Define frame type with its components
        .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
        .build()?;

    // Write frames with data
    for i in 0..100 {
        let time = i as f64 * 0.01; // 10ms hop

        // Create partial data: 3 partials, 4 columns each
        let data = vec![
            1.0, 440.0 * (1.0 + 0.001 * i as f64), 0.5, 0.0,
            2.0, 880.0 * (1.0 + 0.001 * i as f64), 0.3, 1.57,
            3.0, 1320.0 * (1.0 + 0.001 * i as f64), 0.2, 3.14,
        ];

        writer.write_frame_one_matrix("1TRC", time, "1TRC", 3, 4, &data)?;
    }

    // Don't forget to close!
    writer.close()?;

    println!("Wrote {} frames", 100);
    Ok(())
}
```

### Multi-Matrix Frames

For frames containing multiple matrices, use the `FrameBuilder`:

```rust
use sdif_rs::SdifFile;

let mut writer = SdifFile::builder()
    .create("multi.sdif")?
    .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
    .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
    .build()?;

// Build a frame with multiple matrices
writer.new_frame("1TRC", 0.0, 0)?
    .add_matrix("1TRC", 2, 4, &[1.0, 440.0, 0.5, 0.0, 2.0, 880.0, 0.3, 1.57])?
    .add_matrix("1TRC", 1, 4, &[3.0, 1320.0, 0.2, 3.14])?
    .finish()?;  // Must call finish()!

writer.close()?;
```

## Supported Frame Types

| Signature | Name | Description |
|-----------|------|-------------|
| 1TRC | Sinusoidal Tracks | Time-varying partials for additive synthesis |
| 1HRM | Harmonic Partials | Harmonic partial data |
| 1FQ0 | Fundamental Frequency | Pitch tracking data |
| 1RES | Resonances | Resonance/formant data |
| 1STF | Short-Time Fourier | STFT magnitude/phase |

## Thread Safety

The underlying SDIF C library uses global state and is not thread-safe.
`SdifFile` is marked as `!Send + !Sync` to prevent cross-thread usage.
Perform all SDIF operations on a single thread.

## Error Handling

All fallible operations return `Result<T, sdif_rs::Error>`. Error types include:

- `Error::OpenFailed` - File couldn't be opened
- `Error::InvalidFormat` - Not a valid SDIF file
- `Error::ReadError` - Error reading data
- `Error::InvalidSignature` - Invalid 4-character signature

## Performance

- Streaming iteration avoids loading entire files into memory
- Zero-copy data access where possible with ndarray
- Efficient row-by-row reading matches SDIF's sequential access pattern

## See Also

- [sdif-sys](../sdif-sys) - Raw FFI bindings
- [mat2sdif](../mat2sdif) - MAT to SDIF converter
- [SDIF Specification](http://sdif.sourceforge.net/standard/sdif-standard.html)

## License

MIT License
