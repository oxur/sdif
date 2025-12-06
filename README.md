# Rust SDIF Library

A Rust implementation for reading and writing SDIF (Sound Description Interchange Format) files,
with tools for converting MATLAB/Octave analysis data.

## Crates

This workspace contains three crates:

| Crate | Description | Status |
|-------|-------------|--------|
| [sdif-sys](./sdif-sys) | Raw FFI bindings to IRCAM SDIF C library | 🚧 In Progress |
| [sdif-rs](./sdif-rs) | Safe, idiomatic Rust wrapper | 📋 Planned |
| [mat2sdif](./mat2sdif) | CLI tool for MAT to SDIF conversion | 📋 Planned |

## Quick Start

```rust
use sdif_rs::SdifFile;

// Reading SDIF files
let file = SdifFile::open("input.sdif")?;
for frame in file.frames() {
    println!("Time: {}, Type: {}", frame.time(), frame.signature());
    for matrix in frame.matrices() {
        let data = matrix.data()?;
        // Process matrix data...
    }
}

// Writing SDIF files
let mut writer = SdifFile::builder()
    .create("output.sdif")?
    .add_nvt([("creator", "rust-sdif")])?
    .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?;

writer.write_frame("1TRC", 0.0, 0)?
    .add_matrix("1TRC", &data)?
    .finish()?;
```

## SDIF Format

SDIF (Sound Description Interchange Format) is a standard format for storing and exchanging
sound descriptions, particularly suited for:

- Sinusoidal/additive synthesis data
- Spectral analysis results
- Time-varying audio parameters
- Multi-track frequency/amplitude data

### Supported Frame Types

| Type | Description | Use Case |
|------|-------------|----------|
| 1TRC | Sinusoidal Tracks | Additive synthesis |
| 1HRM | Harmonic Partials | Harmonic analysis |
| 1FQ0 | Fundamental Frequency | Pitch tracking |
| 1RES | Resonances | Modal synthesis |

## Max/MSP Compatibility

The library is designed to produce SDIF files compatible with Max/MSP and the CNMAT externals:

- Uses 1TRC frame type for maximum compatibility
- Supports up to 1024 partials per frame
- Ensures proper amplitude fade-in/fade-out
- Float32/Float64 data storage

## Building

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Build with bundled SDIF source
cargo build --features sdif-sys/bundled
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

### Bundled SDIF C Library

The optional bundled SDIF C library (`sdif-sys/sdif/`) is developed by IRCAM and licensed under **LGPL-2.0**. See `sdif-sys/sdif/COPYING` for the full license text.

When using the `bundled` feature (which statically links the SDIF library), the LGPL-2.0 requirements apply to the SDIF library portion:

- You must provide a way for users to relink with a modified SDIF library
- This is satisfied by providing your source code, or object files sufficient for relinking
- You must include the LGPL-2.0 license text and attribution

When using a system-installed SDIF library (dynamic linking), your application is considered a "work that uses the library" and is not subject to LGPL copyleft requirements.

**SDIF Library Source:** https://sourceforge.net/projects/sdif/

## References

- [SDIF Specification](http://sdif.sourceforge.net/standard/sdif-standard.html)
- [IRCAM SDIF Library](https://github.com/IRCAM-WAM/SDIF)
- [CNMAT Max Externals](https://cnmat.berkeley.edu/)
