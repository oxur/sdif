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
