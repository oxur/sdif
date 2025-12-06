# Phase 1: FFI Foundation (sdif-sys) - Detailed Implementation Plan

## Overview

**Duration:** 1-2 days  
**Dependencies:** None  
**Goal:** Create the `sdif-sys` crate with bindgen-generated FFI bindings to the IRCAM SDIF C library, establishing the foundation for the safe Rust wrapper.

This document provides step-by-step instructions for Claude Code to implement Phase 1. The workspace will be a Cargo workspace containing three crates: `sdif-sys`, `sdif-rs`, and `mat2sdif`.

---

## Step 0: Obtain SDIF C Library Source (REQUIRED FIRST)

Before implementing any code, the SDIF C library source must be downloaded and placed in the correct location. **This step is mandatory for the `bundled` feature to work and for testing.**

### Task 0.1: Download SDIF Source

**Claude Code Prompt:**

```
Download the SDIF C library source from SourceForge and set it up for bundled compilation.

The SDIF library is hosted at: https://sourceforge.net/projects/sdif/
Latest version: SDIF-3.11.7 (or SDIF-3.11.4)
License: LGPL v2.0

Execute these commands:

# Navigate to sdif-sys directory (create if needed)
mkdir -p sdif-sys
cd sdif-sys

# Download the source archive
curl -L "https://sourceforge.net/projects/sdif/files/sdif/SDIF-3.11.4/SDIF-3.11.4-src.zip/download" -o SDIF-src.zip

# Extract
unzip SDIF-src.zip

# Rename to 'sdif' for consistency with build.rs expectations
mv SDIF-3.11.4-src sdif

# Clean up archive
rm SDIF-src.zip

# Verify the structure
ls sdif/include/sdif.h      # Should exist - main header
ls sdif/sdif/*.c | head     # Should show C source files
```

### Expected Structure After Download

```
sdif-sys/
└── sdif/                      # SDIF C library source
    ├── COPYING                # LGPL v2.0 license
    ├── README
    ├── configure
    ├── include/
    │   └── sdif.h             # Main header (bindgen input)
    ├── sdif/
    │   ├── SdifFile.c
    │   ├── SdifFrame.c
    │   ├── SdifMatrix.c
    │   ├── SdifGenInit.c
    │   └── ... (many .c files)
    └── ...
```

### Verification

```bash
# Confirm header exists
test -f sdif-sys/sdif/include/sdif.h && echo "✓ Header found" || echo "✗ Header missing"

# Count source files
echo "Source files: $(ls sdif-sys/sdif/sdif/*.c 2>/dev/null | wc -l)"
```

### Alternative: System Installation

If you prefer to use a system-installed SDIF library (detected via pkg-config), you can skip bundling:

```bash
# On Ubuntu/Debian (if packaged)
sudo apt-get install libsdif-dev

# Or build from source system-wide
cd /tmp
curl -L "https://sourceforge.net/projects/sdif/files/sdif/SDIF-3.11.4/SDIF-3.11.4-src.zip/download" -o sdif.zip
unzip sdif.zip
cd SDIF-3.11.4-src
./configure --prefix=/usr/local
make
sudo make install

# Verify
pkg-config --libs --cflags sdif
```

With a system installation, the build.rs will find it via pkg-config and skip bundled compilation.

---

## Step 1: Workspace and Directory Structure Setup

### Task 1.1: Create the Workspace Root

**Claude Code Prompt:**

```
Create a new Rust workspace for the SDIF library project. The workspace will contain three crates:
1. sdif-sys - Raw FFI bindings (this phase)
2. sdif-rs - Safe Rust wrapper (future phase)
3. mat2sdif - CLI conversion tool (future phase)

Create the following directory structure:

rust-sdif/
├── Cargo.toml           # Workspace manifest
├── README.md            # Project overview
├── LICENSE              # MIT license
├── .gitignore           # Rust-specific gitignore
├── sdif-sys/            # FFI bindings crate
│   ├── Cargo.toml
│   ├── build.rs
│   ├── src/
│   │   └── lib.rs
│   ├── sdif/            # Bundled SDIF C source (to be added)
│   └── wrapper.h        # Bindgen input header
├── sdif-rs/             # Safe wrapper (placeholder for now)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
└── mat2sdif/            # CLI tool (placeholder for now)
    ├── Cargo.toml
    └── src/
        └── main.rs

For the workspace Cargo.toml, use:

[workspace]
resolver = "2"
members = [
    "sdif-sys",
    "sdif-rs",
    "mat2sdif",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
repository = "https://github.com/USERNAME/rust-sdif"
rust-version = "1.70"

Create placeholder Cargo.toml files for sdif-rs and mat2sdif with minimal content:
- sdif-rs should depend on sdif-sys = { path = "../sdif-sys" }
- mat2sdif should depend on sdif-rs = { path = "../sdif-rs" }

Create placeholder lib.rs/main.rs files with just a comment indicating they are placeholders for future phases.
```

### Task 1.2: Create .gitignore

**Claude Code Prompt:**

```
Create a .gitignore file at the workspace root with standard Rust ignores:

/target/
**/*.rs.bak
Cargo.lock
*.swp
*.swo
.DS_Store
*.orig

# IDE
.idea/
.vscode/
*.iml

# Build artifacts
*.o
*.a
*.so
*.dylib
*.dll

# Generated bindings (optional - some projects track these)
# sdif-sys/src/bindings.rs
```

---

## Step 2: sdif-sys Crate Configuration

### Task 2.1: Create sdif-sys/Cargo.toml

**Claude Code Prompt:**

```
Create the Cargo.toml for the sdif-sys crate with the following content:

[package]
name = "sdif-sys"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
rust-version.workspace = true
description = "Raw FFI bindings to the IRCAM SDIF (Sound Description Interchange Format) C library"
keywords = ["sdif", "audio", "ffi", "ircam", "sound"]
categories = ["external-ffi-bindings", "multimedia::audio"]
links = "sdif"
build = "build.rs"

[features]
default = []
# Compile SDIF from bundled source instead of using system library
bundled = []
# Force static linking (implies bundled on most systems)
static = ["bundled"]

[dependencies]
libc = "0.2"

[build-dependencies]
bindgen = "0.70"
cc = "1.0"
pkg-config = "0.3"

[dev-dependencies]
# None needed for initial phase

Important notes:
- The `links = "sdif"` directive ensures only one version of the SDIF library is linked
- The build dependencies are needed for generating bindings and compiling C code
- Features allow flexibility in how the C library is obtained
```

### Task 2.2: Create wrapper.h Header

**Claude Code Prompt:**

```
Create sdif-sys/wrapper.h - the header file that bindgen will use to generate Rust bindings.

This file includes the main SDIF headers and defines any necessary configuration:

#ifndef WRAPPER_H
#define WRAPPER_H

/* 
 * SDIF Library Wrapper Header for Bindgen
 * 
 * This header includes the necessary SDIF headers for generating
 * Rust FFI bindings via bindgen.
 */

/* SDIF main header - this typically includes all necessary sub-headers */
#include <sdif.h>

#endif /* WRAPPER_H */

Note: The actual SDIF headers will be located either in the system include path
(when using pkg-config) or in the bundled sdif/ directory. The build.rs will
configure the include paths appropriately.
```

---

## Step 3: build.rs Implementation

### Task 3.1: Create Comprehensive build.rs

**Claude Code Prompt:**

```
Create sdif-sys/build.rs with the following complete implementation. This is the most critical file in Phase 1:

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");
    
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // Determine linking strategy
    let use_bundled = env::var("CARGO_FEATURE_BUNDLED").is_ok() 
        || env::var("CARGO_FEATURE_STATIC").is_ok();
    
    let (include_path, lib_path) = if use_bundled {
        build_bundled(&out_dir)
    } else {
        match try_pkg_config() {
            Some(paths) => paths,
            None => {
                println!("cargo:warning=pkg-config failed to find SDIF library, falling back to bundled");
                build_bundled(&out_dir)
            }
        }
    };
    
    // Generate bindings
    generate_bindings(&include_path, &out_dir);
    
    // Output linking directives
    if let Some(lib_path) = lib_path {
        println!("cargo:rustc-link-search=native={}", lib_path.display());
    }
    
    if use_bundled || env::var("CARGO_FEATURE_STATIC").is_ok() {
        println!("cargo:rustc-link-lib=static=sdif");
    } else {
        println!("cargo:rustc-link-lib=sdif");
    }
}

/// Try to find SDIF using pkg-config
fn try_pkg_config() -> Option<(PathBuf, Option<PathBuf>)> {
    // Try pkg-config first
    match pkg_config::Config::new()
        .atleast_version("3.0")
        .probe("sdif")
    {
        Ok(lib) => {
            let include_path = lib.include_paths
                .first()
                .cloned()
                .unwrap_or_else(|| PathBuf::from("/usr/include"));
            
            let lib_path = lib.link_paths.first().cloned();
            
            println!("cargo:info=Found SDIF via pkg-config");
            Some((include_path, lib_path))
        }
        Err(e) => {
            println!("cargo:warning=pkg-config error: {}", e);
            None
        }
    }
}

/// Build SDIF from bundled source
fn build_bundled(out_dir: &PathBuf) -> (PathBuf, Option<PathBuf>) {
    println!("cargo:info=Building SDIF from bundled source");
    
    let sdif_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("sdif");
    
    // Check if bundled source exists
    if !sdif_dir.exists() {
        panic!(
            "Bundled SDIF source not found at {:?}. \n\
             Either install the SDIF library system-wide, \n\
             or download the SDIF source and place it in the sdif/ directory. \n\
             See README.md for instructions.",
            sdif_dir
        );
    }
    
    // Collect C source files
    // Note: Adjust these paths based on actual SDIF source structure
    let src_dir = sdif_dir.join("src");
    let include_dir = sdif_dir.join("include");
    
    let c_files: Vec<_> = std::fs::read_dir(&src_dir)
        .expect("Failed to read sdif/src directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().map(|e| e == "c").unwrap_or(false) {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    
    if c_files.is_empty() {
        panic!("No C source files found in {:?}", src_dir);
    }
    
    // Build the static library
    let mut build = cc::Build::new();
    build
        .files(&c_files)
        .include(&include_dir)
        .warnings(false)  // SDIF code may have warnings we can't fix
        .opt_level(2);
    
    // Platform-specific settings
    if cfg!(target_os = "windows") {
        build.define("WIN32", None);
    }
    
    // Set SDIFTYPES path if needed
    if let Ok(types_path) = env::var("SDIFTYPES") {
        build.define("SDIFTYPES_FILE", Some(types_path.as_str()));
    }
    
    build.compile("sdif");
    
    // Mark source files for rebuild tracking
    for file in &c_files {
        println!("cargo:rerun-if-changed={}", file.display());
    }
    
    (include_dir, Some(out_dir.clone()))
}

/// Generate Rust bindings using bindgen
fn generate_bindings(include_path: &PathBuf, out_dir: &PathBuf) {
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", include_path.display()))
        
        // Allowlist SDIF types and functions
        .allowlist_function("Sdif.*")
        .allowlist_function("_Sdif.*")
        .allowlist_type("Sdif.*")
        .allowlist_type("_Sdif.*")
        .allowlist_type("eSdif.*")
        .allowlist_var("Sdif.*")
        .allowlist_var("eSdif.*")
        
        // File mode enums
        .allowlist_type("SdifFileModeET")
        .allowlist_var("eReadFile")
        .allowlist_var("eWriteFile")
        .allowlist_var("ePredefinedTypes")
        .allowlist_var("eModeMask")
        
        // Data type enums
        .allowlist_type("SdifDataTypeET")
        .allowlist_var("eFloat4")
        .allowlist_var("eFloat8")
        .allowlist_var("eInt1")
        .allowlist_var("eInt2")
        .allowlist_var("eInt4")
        .allowlist_var("eUInt1")
        .allowlist_var("eUInt2")
        .allowlist_var("eUInt4")
        .allowlist_var("eText")
        
        // Derive useful traits where possible
        .derive_debug(true)
        .derive_default(true)
        .derive_copy(true)
        .derive_eq(true)
        .derive_hash(true)
        .derive_partialeq(true)
        
        // Other options
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate_comments(true)
        .layout_tests(true)
        
        .generate()
        .expect("Failed to generate bindings");
    
    let bindings_path = out_dir.join("bindings.rs");
    bindings
        .write_to_file(&bindings_path)
        .expect("Failed to write bindings");
    
    println!("cargo:info=Generated bindings at {:?}", bindings_path);
}

Key features of this build.rs:
1. Tries pkg-config first for system library
2. Falls back to bundled source compilation
3. Generates bindings with comprehensive allowlists
4. Handles platform-specific settings
5. Proper rerun-if-changed directives
6. Clear error messages when things fail
```

---

## Step 4: Library Source Implementation

### Task 4.1: Create src/lib.rs

**Claude Code Prompt:**

```
Create sdif-sys/src/lib.rs with the following content:

//! # sdif-sys
//! 
//! Raw FFI bindings to the IRCAM SDIF (Sound Description Interchange Format) library.
//! 
//! This crate provides low-level, unsafe bindings to the SDIF C library. For a safe,
//! idiomatic Rust API, use the `sdif-rs` crate instead.
//! 
//! ## Usage
//! 
//! These bindings are primarily intended for use by the `sdif-rs` crate. Direct usage
//! requires careful attention to:
//! 
//! - Calling `SdifGenInit` before any other SDIF functions
//! - Calling `SdifGenKill` during cleanup
//! - Managing `SdifFileT` pointer lifetimes
//! - Following the correct sequence of read/write operations
//! 
//! ## Example
//! 
//! ```no_run
//! use sdif_sys::*;
//! use std::ptr;
//! use std::ffi::CString;
//! 
//! unsafe {
//!     // Initialize the library (required before any operations)
//!     SdifGenInit(ptr::null());
//!     
//!     // Open a file for reading
//!     let path = CString::new("test.sdif").unwrap();
//!     let file = SdifFOpen(path.as_ptr(), SdifFileModeET_eReadFile);
//!     
//!     if !file.is_null() {
//!         // Read general header
//!         let bytes_read = SdifFReadGeneralHeader(file);
//!         
//!         // Read ASCII chunks (NVT, type definitions, etc.)
//!         let ascii_bytes = SdifFReadAllASCIIChunks(file);
//!         
//!         // Close the file
//!         SdifFClose(file);
//!     }
//!     
//!     // Cleanup
//!     SdifGenKill();
//! }
//! ```
//! 
//! ## Feature Flags
//! 
//! - `bundled`: Compile SDIF from bundled source instead of linking to system library
//! - `static`: Force static linking (implies `bundled` on most systems)

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::all)]

// Include the generated bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// ============================================================================
// Additional Constants and Type Aliases
// ============================================================================

/// SDIF signature for 4-character type identifiers.
/// 
/// Signatures are 4-byte codes like "1TRC", "1HRM", etc.
pub type SdifSignature = u32;

/// Convert a 4-character string to an SDIF signature.
/// 
/// # Safety
/// 
/// The C function is called internally. The input must be exactly 4 ASCII characters.
/// 
/// # Panics
/// 
/// Panics if the string is not exactly 4 bytes.
pub fn signature_from_str(s: &str) -> SdifSignature {
    assert_eq!(s.len(), 4, "SDIF signatures must be exactly 4 characters");
    let bytes = s.as_bytes();
    ((bytes[0] as u32) << 24)
        | ((bytes[1] as u32) << 16)
        | ((bytes[2] as u32) << 8)
        | (bytes[3] as u32)
}

/// Convert an SDIF signature to a 4-character string.
pub fn signature_to_string(sig: SdifSignature) -> String {
    let bytes = [
        ((sig >> 24) & 0xFF) as u8,
        ((sig >> 16) & 0xFF) as u8,
        ((sig >> 8) & 0xFF) as u8,
        (sig & 0xFF) as u8,
    ];
    String::from_utf8_lossy(&bytes).into_owned()
}

// ============================================================================
// Common Frame Type Signatures
// ============================================================================

/// 1TRC - Sinusoidal Tracks (most common for additive synthesis)
pub const SIG_1TRC: SdifSignature = signature_from_str_const(b"1TRC");

/// 1HRM - Harmonic Partials
pub const SIG_1HRM: SdifSignature = signature_from_str_const(b"1HRM");

/// 1FQ0 - Fundamental Frequency
pub const SIG_1FQ0: SdifSignature = signature_from_str_const(b"1FQ0");

/// 1RES - Resonances
pub const SIG_1RES: SdifSignature = signature_from_str_const(b"1RES");

/// 1STF - Short-Time Fourier
pub const SIG_1STF: SdifSignature = signature_from_str_const(b"1STF");

/// Convert a 4-byte array to signature at compile time
const fn signature_from_str_const(s: &[u8; 4]) -> SdifSignature {
    ((s[0] as u32) << 24)
        | ((s[1] as u32) << 16)
        | ((s[2] as u32) << 8)
        | (s[3] as u32)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;
    
    #[test]
    fn test_signature_conversion() {
        assert_eq!(signature_from_str("1TRC"), SIG_1TRC);
        assert_eq!(signature_to_string(SIG_1TRC), "1TRC");
        
        assert_eq!(signature_from_str("1HRM"), SIG_1HRM);
        assert_eq!(signature_to_string(SIG_1HRM), "1HRM");
        
        // Roundtrip test
        let sig = signature_from_str("TEST");
        assert_eq!(signature_to_string(sig), "TEST");
    }
    
    #[test]
    #[should_panic(expected = "SDIF signatures must be exactly 4 characters")]
    fn test_signature_wrong_length() {
        signature_from_str("TOO_LONG");
    }
    
    #[test]
    fn test_init_and_kill() {
        // This test verifies that the C library can be initialized and cleaned up
        // without crashing. It's a basic smoke test.
        unsafe {
            SdifGenInit(ptr::null());
            SdifGenKill();
        }
    }
    
    #[test]
    fn test_double_init_is_safe() {
        // SDIF library should handle multiple init calls gracefully
        unsafe {
            SdifGenInit(ptr::null());
            SdifGenInit(ptr::null());
            SdifGenKill();
            SdifGenKill();
        }
    }
    
    #[test]
    fn test_file_mode_constants() {
        // Verify the file mode constants exist and have expected values
        // These are typically defined as 1 and 2 in the SDIF library
        assert!(SdifFileModeET_eReadFile != SdifFileModeET_eWriteFile);
    }
    
    #[test]
    fn test_data_type_sizes() {
        // Verify data type constants match expected sizes
        // eFloat4 = 4 bytes (f32), eFloat8 = 8 bytes (f64)
        // Note: The actual values may vary by SDIF version; adjust as needed
        unsafe {
            let size_f4 = SdifSizeofDataType(SdifDataTypeET_eFloat4);
            let size_f8 = SdifSizeofDataType(SdifDataTypeET_eFloat8);
            
            assert_eq!(size_f4, 4, "eFloat4 should be 4 bytes");
            assert_eq!(size_f8, 8, "eFloat8 should be 8 bytes");
        }
    }
}
```

Note: The exact enum variant names (like `SdifFileModeET_eReadFile`) will depend on what bindgen generates. After initial compilation, you may need to adjust these names based on the actual generated bindings.
```

---

## Step 5: Bundled SDIF Source Setup

### Task 5.1: Create Instructions for Obtaining SDIF Source

**Claude Code Prompt:**

```
Create sdif-sys/sdif/README.md with instructions for obtaining the SDIF C source:

# Bundled SDIF Source

This directory should contain the IRCAM SDIF C library source code.

## Obtaining the Source

### Option 1: Download from IRCAM

1. Visit the IRCAM Forges: https://github.com/Music-Data-Computing/SDIF
2. Download the source archive
3. Extract it into this directory

### Option 2: Clone from GitHub

```bash
cd sdif-sys
git clone https://github.com/Ircam-Partiels/sdif.git sdif
```

## Expected Structure

After setup, this directory should contain:

```
sdif/
├── include/
│   ├── sdif.h
│   ├── SdifFile.h
│   ├── SdifFrame.h
│   ├── SdifMatrix.h
│   └── ... (other headers)
├── src/
│   ├── SdifFile.c
│   ├── SdifFrame.c
│   ├── SdifMatrix.c
│   └── ... (other source files)
└── README.md (this file)
```

## Alternative: System Installation

Instead of bundling, you can install SDIF system-wide:

### macOS (Homebrew)
```bash
brew install sdif
```

### Linux (Debian/Ubuntu)
```bash
sudo apt-get install libsdif-dev
```

### From Source
```bash
cd sdif
./configure --prefix=/usr/local
make
sudo make install
```

When using a system installation, build without the `bundled` feature:
```bash
cargo build --no-default-features
```

Also create sdif-sys/sdif/.gitkeep as an empty file to ensure the directory exists:

# This file ensures the sdif/ directory is tracked by git
# The actual SDIF source should be placed here for bundled builds
```

---

## Step 6: Documentation

### Task 6.1: Create sdif-sys README

**Claude Code Prompt:**

```
Create sdif-sys/README.md:

# sdif-sys

Raw FFI bindings to the IRCAM SDIF (Sound Description Interchange Format) C library.

## Overview

This crate provides low-level, unsafe bindings to the SDIF library. For most use cases,
you should use the `sdif-rs` crate which provides a safe, idiomatic Rust API.

## Building

### Using System Library (Recommended)

If you have SDIF installed system-wide:

```bash
cargo build
```

The build script will use `pkg-config` to find the library.

### Using Bundled Source

To compile SDIF from source:

1. Download the SDIF source (see `sdif/README.md`)
2. Build with the bundled feature:

```bash
cargo build --features bundled
```

### Static Linking

For static linking:

```bash
cargo build --features static
```

## Requirements

- Rust 1.70 or later
- clang (for bindgen)
- pkg-config (for system library detection)
- SDIF C library (system or bundled)

## Safety

All functions in this crate are unsafe. The SDIF library:
- Requires initialization before use (`SdifGenInit`)
- Uses global state (not thread-safe)
- Requires specific call sequences for reading/writing

## License

MIT License - see LICENSE file.

## See Also

- [sdif-rs](../sdif-rs) - Safe Rust wrapper
- [SDIF Specification](http://sdif.sourceforge.net/standard/sdif-standard.html)
- [IRCAM SDIF](https://github.com/Ircam-Partiels/sdif)
```

### Task 6.2: Update Workspace README

**Claude Code Prompt:**

```
Create/update the workspace root README.md:

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

MIT License

## References

- [SDIF Specification](http://sdif.sourceforge.net/standard/sdif-standard.html)
- [IRCAM SDIF Library](https://github.com/Ircam-Partiels/sdif)
- [CNMAT Max Externals](https://cnmat.berkeley.edu/)
```

---

## Step 7: Testing and Validation

### Task 7.1: Create Integration Test Structure

**Claude Code Prompt:**

```
Create sdif-sys/tests/integration.rs for integration testing:

//! Integration tests for sdif-sys
//! 
//! These tests verify that the FFI bindings work correctly with the SDIF library.

use sdif_sys::*;
use std::ffi::CString;
use std::ptr;

/// Test fixture that handles SDIF initialization/cleanup
struct SdifTestContext;

impl SdifTestContext {
    fn new() -> Self {
        unsafe {
            SdifGenInit(ptr::null());
        }
        SdifTestContext
    }
}

impl Drop for SdifTestContext {
    fn drop(&mut self) {
        unsafe {
            SdifGenKill();
        }
    }
}

#[test]
fn test_library_initialization() {
    let _ctx = SdifTestContext::new();
    // If we get here without crashing, initialization succeeded
}

#[test]
fn test_signature_constant_macro() {
    // Test the SdifSignatureConst function/macro if available
    let _ctx = SdifTestContext::new();
    
    unsafe {
        // Note: Adjust based on actual generated binding name
        let sig = SdifSignatureConst(
            '1' as i8,
            'T' as i8,
            'R' as i8,
            'C' as i8,
        );
        
        // Verify roundtrip
        let str_sig = signature_to_string(sig);
        assert_eq!(str_sig, "1TRC");
    }
}

#[test]
fn test_open_nonexistent_file() {
    let _ctx = SdifTestContext::new();
    
    unsafe {
        let path = CString::new("/nonexistent/path/to/file.sdif").unwrap();
        let file = SdifFOpen(path.as_ptr(), SdifFileModeET_eReadFile);
        
        // Should return null for nonexistent file
        assert!(file.is_null(), "Opening nonexistent file should return null");
    }
}

#[test]
fn test_predefined_types_loaded() {
    let _ctx = SdifTestContext::new();
    
    // After initialization, predefined types should be available
    // This tests that the type tables were loaded correctly
    unsafe {
        // Try to look up a predefined type
        let sig = SdifSignatureConst(
            '1' as i8,
            'T' as i8,
            'R' as i8,
            'C' as i8,
        );
        
        // The signature should be valid (non-zero)
        assert_ne!(sig, 0, "1TRC signature should be non-zero");
    }
}

// Additional tests to add once we have test SDIF files:
// - test_read_simple_file
// - test_read_frames_and_matrices
// - test_nvt_parsing
```

### Task 7.2: Add Test Fixtures Directory

**Claude Code Prompt:**

```
Create sdif-sys/tests/fixtures/README.md:

# Test Fixtures

This directory contains SDIF test files for integration testing.

## Required Files

For full test coverage, add the following files:

1. `simple.sdif` - A minimal SDIF file with:
   - One 1TRC frame
   - One matrix with a few rows
   - Basic NVT entries

2. `multiframe.sdif` - An SDIF file with:
   - Multiple frames at different times
   - Multiple matrices per frame

3. `all_types.sdif` - An SDIF file demonstrating:
   - 1TRC, 1HRM, 1FQ0 frame types
   - Float32 and Float64 data
   - Complex NVT data

## Creating Test Files

Test files can be created using:
- SPEAR (spectral analysis application)
- AudioSculpt
- pysdif3 (Python SDIF library)
- The sdif-rs write API (once implemented)

## Example: Creating with pysdif3

```python
import pysdif3

with pysdif3.SdifFile('simple.sdif', 'w') as f:
    f.add_NVT({'creator': 'test', 'date': '2024-01-01'})
    f.add_frame_type('1TRC', '1TRC SinusoidalTracks')
    f.add_matrix_type('1TRC', 'Index, Frequency, Amplitude, Phase')
    
    f.new_frame('1TRC', 0.0, 1)
    f.add_matrix('1TRC', [[1, 440.0, 0.5, 0.0], [2, 880.0, 0.3, 0.0]])
```

Also create an empty .gitkeep file:
sdif-sys/tests/fixtures/.gitkeep
```

---

## Step 8: CI Configuration

### Task 8.1: Create GitHub Actions Workflow

**Claude Code Prompt:**

```
Create .github/workflows/ci.yml:

name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  # Check formatting
  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  # Run clippy lints
  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - name: Install SDIF dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libsdif-dev || true
      - run: cargo clippy --all-targets --all-features -- -D warnings
        continue-on-error: true  # May fail without SDIF library

  # Test on multiple platforms
  test:
    name: Test ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        include:
          - os: ubuntu-latest
            features: bundled
          - os: macos-latest
            features: bundled
          - os: windows-latest
            features: bundled
    
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: true
      
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Install dependencies (Ubuntu)
        if: matrix.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y llvm-dev libclang-dev clang
      
      - name: Install dependencies (macOS)
        if: matrix.os == 'macos-latest'
        run: |
          brew install llvm
      
      - name: Install dependencies (Windows)
        if: matrix.os == 'windows-latest'
        run: |
          choco install llvm
      
      - name: Setup SDIF source
        run: |
          # Clone SDIF source for bundled build
          git clone https://github.com/Ircam-Partiels/sdif.git sdif-sys/sdif || true
      
      - name: Build
        run: cargo build --features ${{ matrix.features }}
      
      - name: Run tests
        run: cargo test --features ${{ matrix.features }}

  # Documentation build
  docs:
    name: Documentation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Check documentation
        run: cargo doc --no-deps
        continue-on-error: true  # May fail without SDIF library
```

---

## Step 9: Final Checklist and Verification

### Task 9.1: Verification Script

**Claude Code Prompt:**

```
Create a verification script at scripts/verify-phase1.sh:

#!/bin/bash
# Verification script for Phase 1 completion

set -e

echo "=== Phase 1 Verification Script ==="
echo

# Check directory structure
echo "1. Checking directory structure..."
REQUIRED_DIRS=(
    "sdif-sys/src"
    "sdif-sys/sdif"
    "sdif-sys/tests/fixtures"
    "sdif-rs/src"
    "mat2sdif/src"
)

for dir in "${REQUIRED_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        echo "   ✓ $dir exists"
    else
        echo "   ✗ $dir missing!"
        exit 1
    fi
done

# Check required files
echo
echo "2. Checking required files..."
REQUIRED_FILES=(
    "Cargo.toml"
    "sdif-sys/Cargo.toml"
    "sdif-sys/build.rs"
    "sdif-sys/src/lib.rs"
    "sdif-sys/wrapper.h"
    "sdif-rs/Cargo.toml"
    "sdif-rs/src/lib.rs"
    "mat2sdif/Cargo.toml"
    "mat2sdif/src/main.rs"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "   ✓ $file exists"
    else
        echo "   ✗ $file missing!"
        exit 1
    fi
done

# Check if SDIF source is available
echo
echo "3. Checking SDIF source..."
if [ -d "sdif-sys/sdif/src" ] && [ -d "sdif-sys/sdif/include" ]; then
    echo "   ✓ Bundled SDIF source found"
elif pkg-config --exists sdif 2>/dev/null; then
    echo "   ✓ System SDIF library found"
else
    echo "   ⚠ No SDIF source found - bundled build will fail"
    echo "     Run: git clone https://github.com/Ircam-Partiels/sdif.git sdif-sys/sdif"
fi

# Try to build
echo
echo "4. Attempting build..."
if cargo check -p sdif-sys 2>/dev/null; then
    echo "   ✓ sdif-sys compiles successfully"
else
    echo "   ⚠ Build check failed (may need SDIF source)"
fi

# Run tests if build succeeded
echo
echo "5. Running tests..."
if cargo test -p sdif-sys 2>/dev/null; then
    echo "   ✓ All tests passed"
else
    echo "   ⚠ Tests failed or skipped"
fi

echo
echo "=== Phase 1 Verification Complete ==="

Make it executable:
chmod +x scripts/verify-phase1.sh
```

---

## Success Criteria Summary

Phase 1 is complete when:

1. **Directory Structure**
   - [ ] Workspace with three crates (sdif-sys, sdif-rs, mat2sdif)
   - [ ] Proper Cargo.toml hierarchy
   - [ ] All placeholder files in place

2. **sdif-sys Crate**
   - [ ] Cargo.toml with correct metadata and dependencies
   - [ ] build.rs that handles pkg-config and bundled builds
   - [ ] Bindgen configuration generates complete bindings
   - [ ] lib.rs includes bindings and adds helper utilities

3. **Functionality**
   - [ ] `SdifGenInit`/`SdifGenKill` work without crashes
   - [ ] Signature conversion utilities work correctly
   - [ ] File mode and data type constants are accessible
   - [ ] Basic tests pass

4. **Documentation**
   - [ ] README files for workspace and sdif-sys
   - [ ] Instructions for obtaining SDIF source
   - [ ] Code documentation with examples

5. **CI/CD**
   - [ ] GitHub Actions workflow for multi-platform builds
   - [ ] Verification script for local testing

---

## Notes for Claude Code

### Build Issues to Watch For

1. **Bindgen Clang Issues**: Ensure LLVM/Clang is available. The build.rs should provide helpful error messages if clang is missing.

2. **Enum Naming**: Bindgen may generate enum variants with prefixes like `SdifFileModeET_eReadFile`. Inspect the generated bindings in `target/*/build/sdif-sys-*/out/bindings.rs` to verify exact names.

3. **Missing Headers**: If pkg-config succeeds but headers aren't found, the include path may need adjustment.

4. **Windows Builds**: Windows may require additional configuration for clang and library paths.

### Debugging Tips

1. **View Generated Bindings**:
   ```bash
   find target -name "bindings.rs" -path "*/sdif-sys-*/*" | head -1 | xargs cat
   ```

2. **Force Rebuild**:
   ```bash
   cargo clean -p sdif-sys
   cargo build -p sdif-sys -vv
   ```

3. **Check pkg-config**:
   ```bash
   pkg-config --cflags --libs sdif
   ```
