# Rust SDIF Library: Complete Technical Research and Implementation Plan

The IRCAM SDIF C library can be effectively wrapped in Rust using modern FFI patterns, with an idiomatic API inspired by Python's pysdif3. This report provides the complete technical foundation for building both a safe Rust wrapper and a .mat-to-SDIF conversion tool optimized for Max/MSP compatibility.

## Core architecture follows the two-crate pattern

The implementation should split into **sdif-sys** (raw FFI bindings) and **sdif-rs** (safe idiomatic wrapper). This separation, used by rusqlite and similar mature Rust libraries, isolates unsafe code and enables independent versioning. The high-level crate exposes a builder-pattern API where users construct `SdifFile`, `Frame`, and `Matrix` objects through chainable methods that internally handle all unsafe C calls.

The pysdif3 Python library provides excellent naming guidance: **snake_case** for all methods (`add_matrix_type`, `get_data`, `frame_read_header`), **PascalCase** for types (`SdifFile`, `Frame`, `Matrix`), and clear prefixes (`get_*` for accessors, `add_*` for mutations, `clone_*` for copying). The dual interface pattern—high-level iteration-based access alongside low-level C-mirroring functions—serves both common workflows and edge cases.

---

## SDIF C library presents a sequential streaming API

The IRCAM SDIF library operates through a **stateful file handle** (`SdifFileT`) that maintains current frame and matrix positions. Reading follows a strict sequence: `SdifFReadGeneralHeader` → `SdifFReadAllASCIIChunks` → frame loop (`SdifFReadFrameHeader` → matrix loop). This streaming design means the Rust wrapper must encode valid operation sequences, ideally using **typestate patterns** to prevent compile-time errors like reading matrix data before headers.

**Essential frame types for Max compatibility** are 1TRC (sinusoidal tracks), 1HRM (harmonic partials), and 1FQ0 (fundamental frequency). Each frame contains time-stamped matrices with columns: Index, Frequency, Amplitude, and optionally Phase. The **1TRC type is universally supported**; CNMAT externals in Max handle up to **1024 partials per frame** in modern versions. Critical requirement: partials must begin and end with **zero amplitude** to avoid click artifacts during synthesis playback.

| Frame Type | Purpose | Matrix Columns | Max Support |
|------------|---------|----------------|-------------|
| **1TRC** | Sinusoidal tracks | Index, Freq, Amp, Phase | ⭐ Primary |
| **1HRM** | Harmonic partials | Index, Freq, Amp, Phase | ⭐ AudioSculpt |
| **1FQ0** | Fundamental frequency | Freq, Confidence | ✓ Common |
| **1RES** | Resonances | Freq, Amp, Decay, Phase | ✓ Resonators |
| **1STF** | Short-time Fourier | Real, Imaginary | Optional |

Memory management is straightforward: `SdifGenInit()` initializes global type tables, `SdifFOpen()` allocates file handles, and `SdifFClose()` plus `SdifGenKill()` clean up. The Rust wrapper implements `Drop` on all handle types to guarantee cleanup. **Data types are exclusively float32 (`eFloat4`) or float64 (`eFloat8`)** with 8-byte alignment and big-endian storage.

---

## MAT file parsing leverages existing Rust infrastructure

The **`matfile` crate** (pure Rust, MIT license) handles Level 5 MAT files including v7 compression—exactly what's needed. It parses numeric arrays (double, single, complex) with ndarray integration but skips structures, cells, and sparse matrices. For audio analysis workflows, this subset is sufficient since typical MAT files contain time vectors, frequency vectors, spectrograms, and F0 trajectories as numeric arrays.

MAT-to-SDIF conversion maps:
- **MAT numeric arrays** → SDIF matrices (with column-major to row-major transpose)
- **Time column** → SDIF frame timestamps
- **Structure organization** → Multiple matrices per frame or separate frames

Level 5 MAT format uses tagged elements with miDOUBLE/miSINGLE data types and optional zlib compression (v7). The 128-byte header contains version and endianness indicators. Parsing is well-documented and the matfile crate abstracts most complexity.

---

## High-level Rust API proposal

```rust
// Reading SDIF files - Iterator-based high-level API
let file = SdifFile::open("input.sdif")?;
for frame in file.frames() {
    println!("Time: {}, Type: {}", frame.time(), frame.signature());
    for matrix in frame.matrices() {
        let data: ArrayView2<f64> = matrix.data()?;  // ndarray integration
    }
}

// Writing SDIF files - Builder pattern
let mut writer = SdifFile::builder()
    .create("output.sdif")?
    .add_nvt([("creator", "rust-sdif")])?
    .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
    .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?;

let data = array![[1.0, 440.0, 0.5, 0.0], [2.0, 880.0, 0.3, 0.0]];
writer.write_frame("1TRC", 0.0, 0)?
    .add_matrix("1TRC", &data)?
    .finish()?;

// Low-level API for fine control
let mut file = SdifFile::open_low_level("input.sdif")?;
while file.read_frame_header()? {
    let time = file.frame_time();
    for _ in 0..file.frame_num_matrices() {
        file.read_matrix_header()?;
        let data = file.read_matrix_data()?;
    }
}
```

**Key design decisions:**
- `SdifFile` wraps the C file handle with RAII cleanup
- `Frame` and `Matrix` types borrow from parent `SdifFile` (lifetime tracking via `PhantomData`)
- Writing uses consuming builder pattern for typestate safety
- All I/O returns `Result<T, SdifError>` converting C error codes
- `FrameBuilder` context manager pattern ensures proper frame finalization
- Optional ndarray feature for zero-copy array views

---

## Implementation phases and Claude Code prompts

### Phase 1: sdif-sys crate (raw FFI bindings)

**Objective:** Generate bindgen bindings from SDIF C headers, configure build.rs for compilation/linking.

```markdown
# Claude Code Prompt: sdif-sys FFI Bindings

Create a Rust `-sys` crate that wraps the IRCAM SDIF C library.

## Requirements:
1. **Cargo.toml** configuration:
   - `links = "sdif"` 
   - build-dependencies: `bindgen = "0.70"`, `cc = "1.0"`, `pkg-config = "0.3"`
   - Feature flags: `bundled` (compile from source), `static` (force static linking)

2. **build.rs** implementation:
   - Try pkg-config first: `pkg_config::probe("sdif")`
   - Fallback to bundled C source compilation with `cc::Build`
   - Generate bindings with bindgen from `sdif.h`:
     - Allowlist: `Sdif*`, `_Sdif*`, `eSdif*`, `eReadFile`, `eWriteFile`
     - Derive Debug, Default, Copy, Clone where possible
   - Set `SDIFTYPES` environment variable handling

3. **src/lib.rs**:
   - Include generated bindings
   - Re-export key types: `SdifFileT`, `SdifSignature`, `SdifDataTypeET`
   - Add constants: `eFloat4 = 0x0004`, `eFloat8 = 0x0008`

4. **Tests**:
   - Verify `SdifGenInit`/`SdifGenKill` don't crash
   - Verify signature conversion: `SdifSignatureConst` roundtrip

## Reference:
The SDIF C API uses these core functions:
- `SdifGenInit(char*)`, `SdifGenKill()`
- `SdifFOpen(char*, SdifFileModeET)`, `SdifFClose(SdifFileT*)`
- `SdifFReadGeneralHeader`, `SdifFReadAllASCIIChunks`
- `SdifFReadFrameHeader`, `SdifFReadMatrixHeader`, `SdifFReadOneRow`
- `SdifFWriteFrameAndOneMatrix`
- Accessor macros: `SdifFCurrTime`, `SdifFCurrNbMatrix`, `SdifFCurrNbRow`, `SdifFCurrNbCol`
```

### Phase 2: Core safe wrapper (sdif-rs reading)

**Objective:** Create safe Rust abstractions for reading SDIF files with iterator support.

```markdown
# Claude Code Prompt: sdif-rs Safe Reading API

Build the safe high-level API for reading SDIF files, wrapping sdif-sys.

## Requirements:

1. **Error handling** (`src/error.rs`):
   ```rust
   pub enum SdifError {
       IoError(std::io::Error),
       InitFailed,
       InvalidSignature(String),
       InvalidState(&'static str),
       NullPointer,
       CStringError(std::ffi::NulError),
   }
   ```
   - Implement `std::error::Error` and `Display`
   - Create `Result<T> = std::result::Result<T, SdifError>`

2. **Library initialization** (global state):
   ```rust
   static INIT: Once = Once::new();
   fn ensure_initialized() { INIT.call_once(|| unsafe { SdifGenInit(ptr::null()) }); }
   ```

3. **SdifFile struct** (`src/file.rs`):
   - Wrap `NonNull<SdifFileT>` with RAII Drop
   - `open(path: impl AsRef<Path>) -> Result<Self>`
   - `get_nvts() -> Vec<HashMap<String, String>>`
   - `frames() -> FrameIterator<'_>` (iterate all frames)
   - Implement `!Send + !Sync` (C library not thread-safe)

4. **Frame struct** (`src/frame.rs`):
   - Borrow from SdifFile: `Frame<'a> { file: &'a SdifFile, ... }`
   - Properties: `signature() -> &str`, `time() -> f64`, `stream_id() -> u32`, `num_matrices() -> usize`
   - `matrices() -> MatrixIterator<'_>`

5. **Matrix struct** (`src/matrix.rs`):
   - Properties: `signature() -> &str`, `rows() -> usize`, `cols() -> usize`, `dtype() -> DataType`
   - `data<T: SdifFloat>() -> Result<Array2<T>>` (with ndarray feature)
   - `data_vec() -> Result<Vec<f64>>` (always available)
   - `column_names() -> Option<Vec<String>>`

6. **Signature utilities**:
   ```rust
   pub fn str_to_signature(s: &str) -> Result<u32>;
   pub fn signature_to_str(sig: u32) -> String;
   ```

7. **Tests**:
   - Read a test SDIF file, iterate frames/matrices
   - Verify time values and matrix dimensions
   - Test signature conversion roundtrip

## Naming conventions (from pysdif3):
- Methods: snake_case (`get_nvts`, `frame_time`, `num_matrices`)
- Types: PascalCase (`SdifFile`, `Frame`, `Matrix`)
- Accessors: no get_ prefix for simple properties (`time()`, `signature()`)
```

### Phase 3: Writing API with builder pattern

**Objective:** Implement SDIF file writing with typestate builder pattern.

```markdown
# Claude Code Prompt: sdif-rs Writing API

Implement the writing API for creating SDIF files with builder pattern.

## Requirements:

1. **SdifFileBuilder** (typestate pattern):
   ```rust
   pub struct SdifFileBuilder<State> { ... }
   pub struct Unconfigured;
   pub struct HeaderWritten;
   
   impl SdifFileBuilder<Unconfigured> {
       pub fn create(path: impl AsRef<Path>) -> Result<Self>;
       pub fn add_nvt(self, entries: impl IntoIterator<Item = (&str, &str)>) -> Self;
       pub fn add_matrix_type(self, sig: &str, columns: &[&str]) -> Self;
       pub fn add_frame_type(self, sig: &str, components: &[&str]) -> Self;
       pub fn build(self) -> Result<SdifWriter>; // Writes headers
   }
   ```

2. **SdifWriter** (active writer):
   ```rust
   impl SdifWriter {
       pub fn new_frame(&mut self, sig: &str, time: f64, stream_id: u32) -> Result<FrameBuilder<'_>>;
       pub fn write_frame_one_matrix(&mut self, frame_sig: &str, time: f64, 
           matrix_sig: &str, data: &Array2<f64>) -> Result<()>;
       pub fn finish(self) -> Result<()>;
   }
   ```

3. **FrameBuilder** (frame context):
   ```rust
   impl<'a> FrameBuilder<'a> {
       pub fn add_matrix(self, sig: &str, data: &Array2<f64>) -> Result<Self>;
       pub fn finish(self) -> Result<()>;  // Must be called
   }
   impl Drop for FrameBuilder<'_> {
       fn drop(&mut self) { /* Panic if not finished? Or auto-finish? */ }
   }
   ```

4. **Data type handling**:
   - Auto-detect f32 vs f64 from array dtype
   - Support both `Array2<f32>` and `Array2<f64>`
   - Handle column-major (ndarray default) to row-major (SDIF) conversion

5. **Tests**:
   - Create minimal SDIF file with 1TRC frame
   - Read back and verify contents match
   - Test NVT metadata roundtrip

## Reference frame/matrix type definitions:
- Matrix "1TRC": columns "Index, Frequency, Amplitude, Phase"
- Frame "1TRC": contains "1TRC SinusoidalTracks"
- Matrix "1FQ0": columns "Frequency, Confidence, Score, RealAmplitude"
```

### Phase 4: MAT file reader integration

**Objective:** Add MAT file parsing using matfile crate, with SDIF-compatible data extraction.

```markdown
# Claude Code Prompt: MAT File Integration

Add MAT file reading capability for .mat to SDIF conversion.

## Requirements:

1. **Dependencies**:
   ```toml
   [dependencies]
   matfile = "0.5"
   ndarray = "0.15"
   ```

2. **MatData struct** (`src/mat.rs`):
   ```rust
   pub struct MatData {
       pub name: String,
       pub shape: Vec<usize>,
       pub data: Array2<f64>,  // Always convert to f64
       pub is_complex: bool,
   }
   
   impl MatData {
       pub fn from_file(path: impl AsRef<Path>) -> Result<Vec<Self>>;
       pub fn get(path: impl AsRef<Path>, name: &str) -> Result<Self>;
   }
   ```

3. **Time vector detection**:
   ```rust
   impl MatData {
       /// Heuristically detect if this is a time vector
       pub fn is_time_vector(&self) -> bool;
       
       /// Extract as time vector (1D)
       pub fn as_time_vector(&self) -> Option<Array1<f64>>;
   }
   ```

4. **SDIF conversion helpers**:
   ```rust
   pub struct MatToSdifConfig {
       pub time_variable: Option<String>,  // Name of time vector variable
       pub frame_type: String,             // Default "1TRC"
       pub matrix_type: String,            // Default "1TRC"  
       pub columns: Vec<String>,           // Column names
   }
   
   impl MatData {
       pub fn to_sdif_frames(&self, times: &Array1<f64>) -> Vec<SdifFrameData>;
   }
   ```

5. **Handle MAT format variations**:
   - Level 5 (v5/v6/v7) via matfile crate
   - Complex numbers: split real/imag or magnitude/phase
   - Transpose column-major to row-major

6. **Tests**:
   - Parse test MAT file with numeric arrays
   - Extract named variables
   - Verify shape and data integrity
```

### Phase 5: mat2sdif binary tool

**Objective:** Create command-line converter from MAT to SDIF format.

```markdown
# Claude Code Prompt: mat2sdif CLI Tool

Build the mat2sdif binary for converting MATLAB/Octave files to SDIF.

## Requirements:

1. **Dependencies**:
   ```toml
   [dependencies]
   clap = { version = "4.0", features = ["derive"] }
   sdif-rs = { path = "../sdif-rs" }
   ```

2. **CLI interface**:
   ```
   mat2sdif 0.1.0
   Convert MATLAB/Octave .mat files to SDIF format
   
   USAGE:
       mat2sdif [OPTIONS] <INPUT> <OUTPUT>
   
   ARGS:
       <INPUT>   Input .mat file
       <OUTPUT>  Output .sdif file
   
   OPTIONS:
       -t, --time-var <NAME>      Variable containing time vector
       -d, --data-var <NAME>      Variable containing data matrix (can repeat)
       -f, --frame-type <SIG>     SDIF frame type [default: 1TRC]
       -m, --matrix-type <SIG>    SDIF matrix type [default: 1TRC]
       -c, --columns <NAMES>      Column names, comma-separated
       --list                     List variables in MAT file and exit
       --max-partials <N>         Limit partials per frame for Max compatibility [default: 1024]
       -v, --verbose              Verbose output
   ```

3. **Core conversion logic**:
   ```rust
   fn convert(args: &Args) -> Result<()> {
       let mat_data = MatData::from_file(&args.input)?;
       
       // Auto-detect or use specified time variable
       let times = find_time_vector(&mat_data, args.time_var.as_deref())?;
       
       // Build SDIF file
       let mut writer = SdifFile::builder()
           .create(&args.output)?
           .add_nvt([("creator", "mat2sdif"), ("source", &args.input)])?
           .add_matrix_type(&args.matrix_type, &args.columns)?
           .add_frame_type(&args.frame_type, &[&format!("{} Data", args.matrix_type)])?
           .build()?;
       
       // Write frames
       for (i, time) in times.iter().enumerate() {
           let frame_data = extract_frame_data(&mat_data, i, args.max_partials)?;
           writer.write_frame_one_matrix(
               &args.frame_type, *time, &args.matrix_type, &frame_data
           )?;
       }
       
       writer.finish()?;
       Ok(())
   }
   ```

4. **Max compatibility features**:
   - Enforce partial limit (default 1024)
   - Ensure zero amplitude at track birth/death
   - Validate frame type is Max-compatible (1TRC, 1HRM, 1RES, 1FQ0)
   - Warn if partial count exceeds 256 (legacy compatibility)

5. **Variable listing mode**:
   ```
   $ mat2sdif --list analysis.mat
   Variables in analysis.mat:
     time      [1, 500]    float64   (likely time vector)
     partials  [500, 4]    float64
     f0        [500, 1]    float64
   ```

6. **Error handling**:
   - Clear error messages for unsupported MAT features
   - Validate SDIF output is well-formed
   - Support --dry-run for validation without writing

## Usage examples:
```bash
# Basic conversion
mat2sdif analysis.mat output.sdif -t time -d partials -c "Index,Freq,Amp,Phase"

# F0 data
mat2sdif pitch.mat f0.sdif -t time -d f0 -f 1FQ0 -m 1FQ0 -c "Frequency,Confidence"

# List contents
mat2sdif --list mystery.mat
```
```

### Phase 6: Documentation and testing

**Objective:** Complete documentation, examples, and integration tests.

```markdown
# Claude Code Prompt: Documentation and Testing

Complete the library documentation and test suite.

## Requirements:

1. **Crate-level documentation** (`src/lib.rs`):
   - Overview with feature summary
   - Quick start example (read + write)
   - Feature flags documentation
   - Link to SDIF format specification

2. **Rustdoc for all public items**:
   - Every public function/method needs `///` docs
   - Include `# Examples` with working doc-tests
   - Document `# Errors` for Result-returning functions
   - Document `# Panics` where applicable

3. **Examples directory**:
   - `examples/read_sdif.rs` - Basic file reading
   - `examples/write_sdif.rs` - Creating SDIF from scratch
   - `examples/copy_modify.rs` - Read, modify, write workflow
   - `examples/mat_convert.rs` - MAT to SDIF conversion

4. **Integration tests** (`tests/`):
   - `tests/roundtrip.rs` - Write then read, verify data
   - `tests/max_compatibility.rs` - Verify output works with Max
   - `tests/mat_conversion.rs` - End-to-end MAT conversion

5. **Test fixtures**:
   - Include small test SDIF files (1TRC, 1HRM, 1FQ0)
   - Include small test MAT files
   - Verify against known-good files from SPEAR/AudioSculpt

6. **README.md**:
   - Installation instructions
   - Quick usage examples
   - CLI tool documentation
   - Max/MSP compatibility notes
   - Contributing guidelines

7. **CI configuration** (`.github/workflows/ci.yml`):
   - Build on Linux, macOS, Windows
   - Run tests with and without bundled SDIF
   - Run clippy and rustfmt checks
```

---

## Critical implementation details

**Thread safety:** The SDIF C library maintains global state initialized by `SdifGenInit()`. Implement this as a `Once`-guarded singleton. Mark `SdifFile` as `!Send + !Sync` to prevent cross-thread usage.

**Padding and alignment:** SDIF requires 8-byte alignment for all data. The C library handles this internally via `SdifFPaddingCalculate` and `SdifFWritePadding`, but ensure the Rust wrapper accounts for this when calculating frame sizes.

**Signature handling:** 4-character signatures are stored as `u32`. Provide bidirectional conversion (`str_to_signature`, `signature_to_str`) using `SdifSignatureConst` internally.

**ndarray integration:** Use feature flag `ndarray` to optionally depend on ndarray. The `Array2<f64>` type maps directly to SDIF matrices. Handle column-major (Fortran order, ndarray default) vs row-major conversion.

**Max compatibility checklist:**
- Use 1TRC frame type for maximum compatibility
- Limit partials to ≤1024 per frame (256 for legacy)
- Include Index column starting from 1
- Ensure amplitude fades to zero at track birth/death
- Use ascending time stamps
- Store as float32 or float64 (not integers)

---

## Dependency summary

| Crate | Purpose | Version |
|-------|---------|---------|
| bindgen | Generate FFI bindings | 0.70 |
| cc | Compile C source | 1.0 |
| pkg-config | Find system libraries | 0.3 |
| libc | C type definitions | 0.2 |
| ndarray | N-dimensional arrays | 0.15 |
| matfile | MAT file parsing | 0.5 |
| clap | CLI argument parsing | 4.0 |
| thiserror | Error derive macros | 1.0 |

This research provides the complete foundation for implementing a production-quality Rust SDIF library. The phased approach allows incremental development and testing, with each phase building on validated infrastructure from previous phases.