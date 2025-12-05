# Phase 2: Safe Reading API (sdif-rs) - Detailed Implementation Plan

## Overview

**Duration:** 3-4 days  
**Dependencies:** Phase 1 complete (sdif-sys crate functional)  
**Goal:** Create the safe, idiomatic Rust wrapper for reading SDIF files with iterator support, RAII memory management, and comprehensive error handling.

This document provides step-by-step instructions for Claude Code to implement Phase 2. The `sdif-rs` crate will wrap the unsafe `sdif-sys` bindings in a safe, ergonomic API.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         sdif-rs                                  │
├─────────────────────────────────────────────────────────────────┤
│  Public API                                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │  SdifFile   │  │   Frame     │  │   Matrix    │              │
│  │  (reader)   │──│  (iterator) │──│  (data)     │              │
│  └─────────────┘  └─────────────┘  └─────────────┘              │
│         │                                                        │
│  ┌──────┴──────────────────────────────────────────────┐        │
│  │  Internal: init.rs, signature.rs, error.rs          │        │
│  └─────────────────────────────────────────────────────┘        │
├─────────────────────────────────────────────────────────────────┤
│                       sdif-sys (FFI)                             │
└─────────────────────────────────────────────────────────────────┘
```

### Key Design Principles

1. **RAII Everywhere**: All C resources wrapped with `Drop` implementations
2. **Lifetime Safety**: `Frame` and `Matrix` borrow from `SdifFile` to prevent use-after-free
3. **Iterator-Based**: Natural Rust iteration over frames and matrices
4. **Error Propagation**: All fallible operations return `Result<T, SdifError>`
5. **Thread Safety Markers**: `!Send + !Sync` to reflect C library limitations
6. **Optional ndarray**: Feature-gated integration for scientific computing workflows

---

## Step 1: Update sdif-rs Cargo.toml

### Task 1.1: Configure Dependencies and Features

**Claude Code Prompt:**

```
Update sdif-rs/Cargo.toml with the complete configuration for Phase 2:

[package]
name = "sdif-rs"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
rust-version.workspace = true
description = "Safe, idiomatic Rust wrapper for SDIF (Sound Description Interchange Format) files"
keywords = ["sdif", "audio", "ircam", "sound", "spectral"]
categories = ["multimedia::audio", "parser-implementations"]

[features]
default = []
# Enable ndarray integration for matrix data access
ndarray = ["dep:ndarray"]
# Pass through to sdif-sys
bundled = ["sdif-sys/bundled"]
static = ["sdif-sys/static"]

[dependencies]
sdif-sys = { path = "../sdif-sys" }
thiserror = "1.0"
libc = "0.2"

# Optional dependencies
ndarray = { version = "0.15", optional = true }

[dev-dependencies]
tempfile = "3.0"
approx = "0.5"  # For floating-point comparisons in tests

Key points:
- thiserror for ergonomic error type derivation
- ndarray is optional, gated behind a feature flag
- tempfile for creating test files
- approx for floating-point assertions
```

---

## Step 2: Module Structure

### Task 2.1: Create Module Layout

**Claude Code Prompt:**

```
Set up the module structure for sdif-rs. Create the following files:

sdif-rs/src/
├── lib.rs          # Crate root, public exports
├── error.rs        # Error types
├── init.rs         # Global initialization
├── signature.rs    # Signature conversion utilities
├── file.rs         # SdifFile reader
├── frame.rs        # Frame type and iterator
├── matrix.rs       # Matrix type and data access
└── data_type.rs    # DataType enum and conversions

Create each file with a module-level doc comment placeholder:

// src/error.rs
//! Error types for SDIF operations.

// src/init.rs
//! Global SDIF library initialization.

// src/signature.rs  
//! SDIF signature (4-character codes) utilities.

// src/file.rs
//! SDIF file reading operations.

// src/frame.rs
//! SDIF frame representation and iteration.

// src/matrix.rs
//! SDIF matrix representation and data access.

// src/data_type.rs
//! SDIF data type enumeration.
```

### Task 2.2: Create lib.rs with Public Exports

**Claude Code Prompt:**

```
Create sdif-rs/src/lib.rs:

//! # sdif-rs
//!
//! Safe, idiomatic Rust bindings for reading and writing SDIF
//! (Sound Description Interchange Format) files.
//!
//! SDIF is a standard format for storing and exchanging sound descriptions,
//! particularly suited for spectral analysis data, sinusoidal models, and
//! time-varying audio parameters.
//!
//! ## Quick Start
//!
//! ### Reading SDIF Files
//!
//! ```no_run
//! use sdif_rs::{SdifFile, Result};
//!
//! fn main() -> Result<()> {
//!     let file = SdifFile::open("analysis.sdif")?;
//!     
//!     // Print NVT metadata
//!     for nvt in file.nvts() {
//!         for (key, value) in nvt {
//!             println!("{}: {}", key, value);
//!         }
//!     }
//!     
//!     // Iterate over frames
//!     for frame in file.frames() {
//!         let frame = frame?;
//!         println!("Frame {} at time {:.3}s", frame.signature(), frame.time());
//!         
//!         for matrix in frame.matrices() {
//!             let matrix = matrix?;
//!             println!("  Matrix {}: {}x{}", 
//!                 matrix.signature(), 
//!                 matrix.rows(), 
//!                 matrix.cols()
//!             );
//!             
//!             // Get matrix data as Vec<f64>
//!             let data = matrix.data_f64()?;
//!             println!("  First value: {}", data.first().unwrap_or(&0.0));
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### With ndarray (optional feature)
//!
//! ```no_run
//! # #[cfg(feature = "ndarray")]
//! use sdif_rs::{SdifFile, Result};
//! # #[cfg(feature = "ndarray")]
//! use ndarray::Array2;
//!
//! # #[cfg(feature = "ndarray")]
//! fn example() -> Result<()> {
//!     let file = SdifFile::open("analysis.sdif")?;
//!     
//!     for frame in file.frames() {
//!         for matrix in frame?.matrices() {
//!             let matrix = matrix?;
//!             // Get data as 2D array
//!             let array: Array2<f64> = matrix.to_array_f64()?;
//!             println!("Shape: {:?}", array.shape());
//!         }
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Supported Frame Types
//!
//! | Signature | Description | Common Use |
//! |-----------|-------------|------------|
//! | 1TRC | Sinusoidal Tracks | Additive synthesis |
//! | 1HRM | Harmonic Partials | Harmonic analysis |
//! | 1FQ0 | Fundamental Frequency | Pitch tracking |
//! | 1RES | Resonances | Modal synthesis |
//!
//! ## Feature Flags
//!
//! - `ndarray`: Enable `ndarray` integration for matrix data access
//! - `bundled`: Compile SDIF C library from bundled source
//! - `static`: Force static linking of SDIF C library
//!
//! ## Thread Safety
//!
//! The underlying SDIF C library uses global state and is not thread-safe.
//! `SdifFile` is marked as `!Send + !Sync` to prevent cross-thread usage.
//! All SDIF operations should occur on a single thread.

// Modules
mod data_type;
mod error;
mod file;
mod frame;
mod init;
mod matrix;
mod signature;

// Public exports
pub use data_type::DataType;
pub use error::{Error, Result};
pub use file::SdifFile;
pub use frame::Frame;
pub use matrix::Matrix;
pub use signature::{Signature, signature_to_string, string_to_signature};

// Re-export common signatures for convenience
pub mod signatures {
    //! Common SDIF frame/matrix type signatures.
    
    use super::Signature;
    
    /// 1TRC - Sinusoidal Tracks (most widely supported)
    pub const TRC: Signature = super::signature::sig_const(b"1TRC");
    
    /// 1HRM - Harmonic Partials
    pub const HRM: Signature = super::signature::sig_const(b"1HRM");
    
    /// 1FQ0 - Fundamental Frequency
    pub const FQ0: Signature = super::signature::sig_const(b"1FQ0");
    
    /// 1RES - Resonances
    pub const RES: Signature = super::signature::sig_const(b"1RES");
    
    /// 1STF - Short-Time Fourier Transform
    pub const STF: Signature = super::signature::sig_const(b"1STF");
}

// Conditional re-exports
#[cfg(feature = "ndarray")]
pub use ndarray;
```

---

## Step 3: Error Handling

### Task 3.1: Implement Error Types

**Claude Code Prompt:**

```
Create sdif-rs/src/error.rs with comprehensive error handling:

//! Error types for SDIF operations.
//!
//! This module provides the [`Error`] enum covering all possible failure modes
//! when working with SDIF files, along with a convenient [`Result`] type alias.

use std::ffi::NulError;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for SDIF operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during SDIF operations.
#[derive(Error, Debug)]
pub enum Error {
    /// I/O error from the underlying file system.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    
    /// Failed to initialize the SDIF library.
    #[error("Failed to initialize SDIF library")]
    InitFailed,
    
    /// The SDIF file could not be opened.
    #[error("Failed to open SDIF file: {path}")]
    OpenFailed {
        /// Path to the file that could not be opened.
        path: PathBuf,
    },
    
    /// The file is not a valid SDIF file or has a corrupted header.
    #[error("Invalid SDIF file format: {reason}")]
    InvalidFormat {
        /// Description of the format error.
        reason: String,
    },
    
    /// Invalid 4-character signature string.
    #[error("Invalid signature: '{value}' (must be exactly 4 ASCII characters)")]
    InvalidSignature {
        /// The invalid signature value.
        value: String,
    },
    
    /// Operation performed in wrong state (e.g., reading matrix before frame header).
    #[error("Invalid state: {message}")]
    InvalidState {
        /// Description of the state error.
        message: &'static str,
    },
    
    /// A required pointer was null when it shouldn't have been.
    #[error("Null pointer encountered: {context}")]
    NullPointer {
        /// Context where the null pointer was found.
        context: &'static str,
    },
    
    /// Error converting a Rust string to C string (embedded null byte).
    #[error("Invalid C string: {0}")]
    CString(#[from] NulError),
    
    /// Requested data type doesn't match the matrix's actual type.
    #[error("Data type mismatch: expected {expected}, found {found}")]
    DataTypeMismatch {
        /// Expected data type.
        expected: String,
        /// Actual data type in the matrix.
        found: String,
    },
    
    /// Failed to read data from the file.
    #[error("Read error: {message}")]
    ReadError {
        /// Description of the read error.
        message: String,
    },
    
    /// End of file reached unexpectedly.
    #[error("Unexpected end of file")]
    UnexpectedEof,
    
    /// The matrix dimensions are invalid.
    #[error("Invalid matrix dimensions: {rows}x{cols}")]
    InvalidDimensions {
        /// Number of rows.
        rows: usize,
        /// Number of columns.
        cols: usize,
    },
}

impl Error {
    /// Create an OpenFailed error for the given path.
    pub fn open_failed(path: impl Into<PathBuf>) -> Self {
        Self::OpenFailed { path: path.into() }
    }
    
    /// Create an InvalidFormat error with the given reason.
    pub fn invalid_format(reason: impl Into<String>) -> Self {
        Self::InvalidFormat { reason: reason.into() }
    }
    
    /// Create an InvalidSignature error.
    pub fn invalid_signature(value: impl Into<String>) -> Self {
        Self::InvalidSignature { value: value.into() }
    }
    
    /// Create an InvalidState error.
    pub const fn invalid_state(message: &'static str) -> Self {
        Self::InvalidState { message }
    }
    
    /// Create a NullPointer error.
    pub const fn null_pointer(context: &'static str) -> Self {
        Self::NullPointer { context }
    }
    
    /// Create a DataTypeMismatch error.
    pub fn type_mismatch(expected: impl Into<String>, found: impl Into<String>) -> Self {
        Self::DataTypeMismatch {
            expected: expected.into(),
            found: found.into(),
        }
    }
    
    /// Create a ReadError.
    pub fn read_error(message: impl Into<String>) -> Self {
        Self::ReadError { message: message.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let err = Error::invalid_signature("TOOLONG");
        assert!(err.to_string().contains("TOOLONG"));
        
        let err = Error::invalid_state("cannot read matrix without frame");
        assert!(err.to_string().contains("cannot read matrix"));
    }
    
    #[test]
    fn test_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }
}
```

---

## Step 4: Global Initialization

### Task 4.1: Implement Thread-Safe Initialization

**Claude Code Prompt:**

```
Create sdif-rs/src/init.rs with global initialization management:

//! Global SDIF library initialization.
//!
//! The SDIF C library requires initialization before any operations can be
//! performed. This module ensures the library is initialized exactly once,
//! in a thread-safe manner.
//!
//! Users don't need to call these functions directly - initialization is
//! handled automatically when opening an SDIF file.

use std::ptr;
use std::sync::Once;

use sdif_sys::SdifGenInit;

/// Static guard for one-time initialization.
static INIT: Once = Once::new();

/// Flag to track if initialization succeeded.
/// 
/// We use a simple atomic bool pattern here. In practice, SdifGenInit
/// always succeeds, but we track it for safety.
static mut INIT_SUCCEEDED: bool = false;

/// Ensures the SDIF library is initialized.
///
/// This function is safe to call multiple times from any thread - the
/// initialization will only happen once. Subsequent calls are no-ops.
///
/// # Returns
///
/// `true` if the library is (now) initialized, `false` if initialization failed.
///
/// # Example
///
/// ```
/// use sdif_rs::init::ensure_initialized;
///
/// // Called automatically by SdifFile::open, but can be called manually
/// assert!(ensure_initialized());
/// ```
pub fn ensure_initialized() -> bool {
    INIT.call_once(|| {
        // SAFETY: SdifGenInit is called exactly once, protected by Once.
        // Passing null uses the default types file path.
        unsafe {
            SdifGenInit(ptr::null());
            INIT_SUCCEEDED = true;
        }
    });
    
    // SAFETY: INIT_SUCCEEDED is only written inside call_once,
    // which guarantees it completes before any read.
    unsafe { INIT_SUCCEEDED }
}

/// Check if the library has been initialized.
///
/// Returns `true` if `ensure_initialized()` has been called successfully.
pub fn is_initialized() -> bool {
    if INIT.is_completed() {
        // SAFETY: If Once is completed, INIT_SUCCEEDED has its final value.
        unsafe { INIT_SUCCEEDED }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_initialization() {
        // First call should initialize
        assert!(ensure_initialized());
        
        // Subsequent calls should be no-ops but still return true
        assert!(ensure_initialized());
        assert!(ensure_initialized());
        
        // Should report as initialized
        assert!(is_initialized());
    }
}
```

---

## Step 5: Signature Utilities

### Task 5.1: Implement Signature Type and Conversions

**Claude Code Prompt:**

```
Create sdif-rs/src/signature.rs with signature utilities:

//! SDIF signature (4-character code) utilities.
//!
//! SDIF uses 4-character ASCII codes to identify frame and matrix types.
//! These are stored as 32-bit unsigned integers for efficiency.
//!
//! Common signatures include:
//! - `1TRC` - Sinusoidal tracks
//! - `1HRM` - Harmonic partials  
//! - `1FQ0` - Fundamental frequency
//!
//! # Example
//!
//! ```
//! use sdif_rs::{string_to_signature, signature_to_string};
//!
//! let sig = string_to_signature("1TRC").unwrap();
//! assert_eq!(signature_to_string(sig), "1TRC");
//! ```

use crate::error::{Error, Result};

/// A 4-character SDIF signature stored as a 32-bit integer.
pub type Signature = u32;

/// Convert a 4-character string to an SDIF signature.
///
/// # Arguments
///
/// * `s` - A string that must be exactly 4 ASCII characters.
///
/// # Returns
///
/// The signature as a `u32`, or an error if the string is invalid.
///
/// # Errors
///
/// Returns [`Error::InvalidSignature`] if:
/// - The string is not exactly 4 bytes
/// - The string contains non-ASCII characters
///
/// # Example
///
/// ```
/// use sdif_rs::string_to_signature;
///
/// let sig = string_to_signature("1TRC").unwrap();
/// assert_eq!(sig, 0x31545243); // '1' 'T' 'R' 'C' in big-endian
/// ```
pub fn string_to_signature(s: &str) -> Result<Signature> {
    let bytes = s.as_bytes();
    
    if bytes.len() != 4 {
        return Err(Error::invalid_signature(s));
    }
    
    // Verify all ASCII
    if !bytes.iter().all(|b| b.is_ascii()) {
        return Err(Error::invalid_signature(s));
    }
    
    Ok(sig_const_from_slice(bytes))
}

/// Convert an SDIF signature to its 4-character string representation.
///
/// # Arguments
///
/// * `sig` - The signature as a `u32`.
///
/// # Returns
///
/// A 4-character string. Non-printable bytes are replaced with '?'.
///
/// # Example
///
/// ```
/// use sdif_rs::signature_to_string;
///
/// let s = signature_to_string(0x31545243);
/// assert_eq!(s, "1TRC");
/// ```
pub fn signature_to_string(sig: Signature) -> String {
    let bytes = [
        ((sig >> 24) & 0xFF) as u8,
        ((sig >> 16) & 0xFF) as u8,
        ((sig >> 8) & 0xFF) as u8,
        (sig & 0xFF) as u8,
    ];
    
    // Replace non-printable with '?'
    let clean: Vec<u8> = bytes
        .iter()
        .map(|&b| if b.is_ascii_graphic() || b == b' ' { b } else { b'?' })
        .collect();
    
    String::from_utf8_lossy(&clean).into_owned()
}

/// Create a signature from a 4-byte array at compile time.
///
/// This is used internally to define signature constants.
#[doc(hidden)]
pub const fn sig_const(s: &[u8; 4]) -> Signature {
    ((s[0] as u32) << 24)
        | ((s[1] as u32) << 16)
        | ((s[2] as u32) << 8)
        | (s[3] as u32)
}

/// Create a signature from a byte slice (runtime version).
fn sig_const_from_slice(s: &[u8]) -> Signature {
    debug_assert_eq!(s.len(), 4);
    ((s[0] as u32) << 24)
        | ((s[1] as u32) << 16)
        | ((s[2] as u32) << 8)
        | (s[3] as u32)
}

/// Check if a signature matches a known type.
pub fn is_known_signature(sig: Signature) -> bool {
    matches!(
        sig,
        crate::signatures::TRC
            | crate::signatures::HRM
            | crate::signatures::FQ0
            | crate::signatures::RES
            | crate::signatures::STF
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_string_to_signature() {
        let sig = string_to_signature("1TRC").unwrap();
        assert_eq!(sig, 0x31545243);
        
        let sig = string_to_signature("1HRM").unwrap();
        assert_eq!(sig, 0x3148524D);
    }
    
    #[test]
    fn test_signature_to_string() {
        assert_eq!(signature_to_string(0x31545243), "1TRC");
        assert_eq!(signature_to_string(0x3148524D), "1HRM");
    }
    
    #[test]
    fn test_roundtrip() {
        let original = "TEST";
        let sig = string_to_signature(original).unwrap();
        let recovered = signature_to_string(sig);
        assert_eq!(original, recovered);
    }
    
    #[test]
    fn test_invalid_signatures() {
        // Too short
        assert!(string_to_signature("ABC").is_err());
        
        // Too long
        assert!(string_to_signature("ABCDE").is_err());
        
        // Empty
        assert!(string_to_signature("").is_err());
    }
    
    #[test]
    fn test_const_signature() {
        assert_eq!(sig_const(b"1TRC"), 0x31545243);
    }
    
    #[test]
    fn test_known_signatures() {
        assert!(is_known_signature(crate::signatures::TRC));
        assert!(is_known_signature(crate::signatures::HRM));
        assert!(!is_known_signature(0x00000000));
    }
}
```

---

## Step 6: Data Type Enumeration

### Task 6.1: Implement DataType Enum

**Claude Code Prompt:**

```
Create sdif-rs/src/data_type.rs:

//! SDIF data type enumeration.
//!
//! SDIF matrices can contain data in several numeric formats.
//! The most common are `Float4` (f32) and `Float8` (f64).

use std::fmt;

/// SDIF matrix data types.
///
/// SDIF supports various numeric data types for matrix storage.
/// In practice, most audio analysis data uses `Float4` or `Float8`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum DataType {
    /// 32-bit floating point (f32)
    Float4 = 0x0004,
    
    /// 64-bit floating point (f64)
    Float8 = 0x0008,
    
    /// 8-bit signed integer (i8)
    Int1 = 0x0101,
    
    /// 16-bit signed integer (i16)
    Int2 = 0x0102,
    
    /// 32-bit signed integer (i32)
    Int4 = 0x0104,
    
    /// 8-bit unsigned integer (u8)
    UInt1 = 0x0201,
    
    /// 16-bit unsigned integer (u16)
    UInt2 = 0x0202,
    
    /// 32-bit unsigned integer (u32)
    UInt4 = 0x0204,
    
    /// UTF-8 text data
    Text = 0x0301,
    
    /// Unknown or unsupported type
    Unknown = 0x0000,
}

impl DataType {
    /// Create a DataType from its raw C enum value.
    ///
    /// # Arguments
    ///
    /// * `value` - The raw value from the C library.
    ///
    /// # Returns
    ///
    /// The corresponding `DataType`, or `Unknown` if not recognized.
    pub fn from_raw(value: u32) -> Self {
        match value {
            0x0004 => DataType::Float4,
            0x0008 => DataType::Float8,
            0x0101 => DataType::Int1,
            0x0102 => DataType::Int2,
            0x0104 => DataType::Int4,
            0x0201 => DataType::UInt1,
            0x0202 => DataType::UInt2,
            0x0204 => DataType::UInt4,
            0x0301 => DataType::Text,
            _ => DataType::Unknown,
        }
    }
    
    /// Get the size in bytes of a single element of this type.
    ///
    /// # Returns
    ///
    /// The byte size, or 0 for `Text` and `Unknown`.
    pub const fn size_bytes(&self) -> usize {
        match self {
            DataType::Float4 => 4,
            DataType::Float8 => 8,
            DataType::Int1 | DataType::UInt1 => 1,
            DataType::Int2 | DataType::UInt2 => 2,
            DataType::Int4 | DataType::UInt4 => 4,
            DataType::Text | DataType::Unknown => 0,
        }
    }
    
    /// Check if this type is a floating-point type.
    pub const fn is_float(&self) -> bool {
        matches!(self, DataType::Float4 | DataType::Float8)
    }
    
    /// Check if this type is an integer type.
    pub const fn is_integer(&self) -> bool {
        matches!(
            self,
            DataType::Int1
                | DataType::Int2
                | DataType::Int4
                | DataType::UInt1
                | DataType::UInt2
                | DataType::UInt4
        )
    }
    
    /// Check if this type is a signed integer type.
    pub const fn is_signed(&self) -> bool {
        matches!(self, DataType::Int1 | DataType::Int2 | DataType::Int4)
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Float4 => write!(f, "float32"),
            DataType::Float8 => write!(f, "float64"),
            DataType::Int1 => write!(f, "int8"),
            DataType::Int2 => write!(f, "int16"),
            DataType::Int4 => write!(f, "int32"),
            DataType::UInt1 => write!(f, "uint8"),
            DataType::UInt2 => write!(f, "uint16"),
            DataType::UInt4 => write!(f, "uint32"),
            DataType::Text => write!(f, "text"),
            DataType::Unknown => write!(f, "unknown"),
        }
    }
}

impl Default for DataType {
    fn default() -> Self {
        DataType::Float8 // Most common for audio data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_from_raw() {
        assert_eq!(DataType::from_raw(0x0004), DataType::Float4);
        assert_eq!(DataType::from_raw(0x0008), DataType::Float8);
        assert_eq!(DataType::from_raw(0xFFFF), DataType::Unknown);
    }
    
    #[test]
    fn test_size_bytes() {
        assert_eq!(DataType::Float4.size_bytes(), 4);
        assert_eq!(DataType::Float8.size_bytes(), 8);
        assert_eq!(DataType::Int2.size_bytes(), 2);
    }
    
    #[test]
    fn test_type_checks() {
        assert!(DataType::Float4.is_float());
        assert!(DataType::Float8.is_float());
        assert!(!DataType::Int4.is_float());
        
        assert!(DataType::Int4.is_integer());
        assert!(DataType::Int4.is_signed());
        assert!(!DataType::UInt4.is_signed());
    }
    
    #[test]
    fn test_display() {
        assert_eq!(format!("{}", DataType::Float4), "float32");
        assert_eq!(format!("{}", DataType::Float8), "float64");
    }
}
```

---

## Step 7: SdifFile Implementation

### Task 7.1: Implement Core SdifFile Reader

**Claude Code Prompt:**

```
Create sdif-rs/src/file.rs with the main SdifFile reader:

//! SDIF file reading operations.
//!
//! This module provides [`SdifFile`], the main entry point for reading SDIF files.
//!
//! # Example
//!
//! ```no_run
//! use sdif_rs::SdifFile;
//!
//! let file = SdifFile::open("analysis.sdif")?;
//! println!("Opened SDIF file with {} NVT entries", file.nvts().len());
//! # Ok::<(), sdif_rs::Error>(())
//! ```

use std::cell::Cell;
use std::collections::HashMap;
use std::ffi::CString;
use std::marker::PhantomData;
use std::path::Path;
use std::ptr::NonNull;

use sdif_sys::{
    SdifFClose, SdifFOpen, SdifFReadAllASCIIChunks, SdifFReadGeneralHeader,
    SdifFileT, SdifFileModeET_eReadFile,
};

use crate::error::{Error, Result};
use crate::frame::{Frame, FrameIterator};
use crate::init::ensure_initialized;

/// An SDIF file opened for reading.
///
/// `SdifFile` wraps the C library's file handle with RAII semantics.
/// The file is automatically closed when the `SdifFile` is dropped.
///
/// # Thread Safety
///
/// `SdifFile` is `!Send` and `!Sync` because the underlying C library
/// uses global state and is not thread-safe. All operations on an
/// `SdifFile` must occur on the same thread.
///
/// # Example
///
/// ```no_run
/// use sdif_rs::SdifFile;
///
/// let file = SdifFile::open("input.sdif")?;
///
/// // Iterate over all frames
/// for frame_result in file.frames() {
///     let frame = frame_result?;
///     println!("Frame at time {:.3}s", frame.time());
/// }
/// # Ok::<(), sdif_rs::Error>(())
/// ```
pub struct SdifFile {
    /// Pointer to the C file handle. Never null after construction.
    handle: NonNull<SdifFileT>,
    
    /// Cached NVT (Name-Value Table) entries read from the file.
    nvts: Vec<HashMap<String, String>>,
    
    /// Track whether we're currently iterating frames.
    /// Prevents multiple simultaneous iterators.
    iterating: Cell<bool>,
    
    /// Marker to make SdifFile !Send and !Sync.
    /// The C library uses global state and isn't thread-safe.
    _not_send_sync: PhantomData<*const ()>,
}

impl SdifFile {
    /// Open an SDIF file for reading.
    ///
    /// This reads the general header and all ASCII chunks (NVT, type definitions).
    /// After opening, use [`frames()`](Self::frames) to iterate over data frames.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the SDIF file.
    ///
    /// # Returns
    ///
    /// An `SdifFile` ready for reading, or an error if the file couldn't be opened.
    ///
    /// # Errors
    ///
    /// - [`Error::InitFailed`] if the SDIF library couldn't be initialized
    /// - [`Error::OpenFailed`] if the file doesn't exist or isn't readable
    /// - [`Error::InvalidFormat`] if the file isn't a valid SDIF file
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::SdifFile;
    ///
    /// let file = SdifFile::open("analysis.sdif")?;
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        
        // Ensure library is initialized
        if !ensure_initialized() {
            return Err(Error::InitFailed);
        }
        
        // Convert path to C string
        let path_str = path.to_str().ok_or_else(|| {
            Error::invalid_format("Path contains invalid UTF-8")
        })?;
        let c_path = CString::new(path_str)?;
        
        // Open the file
        let handle = unsafe {
            SdifFOpen(c_path.as_ptr(), SdifFileModeET_eReadFile)
        };
        
        let handle = NonNull::new(handle).ok_or_else(|| {
            Error::open_failed(path)
        })?;
        
        // Read general header
        let header_bytes = unsafe { SdifFReadGeneralHeader(handle.as_ptr()) };
        if header_bytes == 0 {
            // Clean up and return error
            unsafe { SdifFClose(handle.as_ptr()) };
            return Err(Error::invalid_format("Failed to read SDIF header"));
        }
        
        // Read ASCII chunks (NVT, type definitions)
        let ascii_bytes = unsafe { SdifFReadAllASCIIChunks(handle.as_ptr()) };
        if ascii_bytes < 0 {
            unsafe { SdifFClose(handle.as_ptr()) };
            return Err(Error::invalid_format("Failed to read ASCII chunks"));
        }
        
        // Parse NVTs
        let nvts = Self::read_nvts(handle.as_ptr());
        
        Ok(SdifFile {
            handle,
            nvts,
            iterating: Cell::new(false),
            _not_send_sync: PhantomData,
        })
    }
    
    /// Get the Name-Value Tables (NVT) from the file.
    ///
    /// NVTs contain metadata about the file, such as creator, date,
    /// sample rate, and other application-specific information.
    ///
    /// # Returns
    ///
    /// A slice of hash maps, where each map represents one NVT.
    /// Most files have a single NVT.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::SdifFile;
    ///
    /// let file = SdifFile::open("input.sdif")?;
    /// for nvt in file.nvts() {
    ///     if let Some(creator) = nvt.get("creator") {
    ///         println!("Created by: {}", creator);
    ///     }
    /// }
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn nvts(&self) -> &[HashMap<String, String>] {
        &self.nvts
    }
    
    /// Get a specific value from the first NVT.
    ///
    /// This is a convenience method for accessing common metadata.
    ///
    /// # Arguments
    ///
    /// * `key` - The NVT key to look up.
    ///
    /// # Returns
    ///
    /// The value if found, or `None`.
    pub fn nvt_get(&self, key: &str) -> Option<&str> {
        self.nvts.first()?.get(key).map(|s| s.as_str())
    }
    
    /// Create an iterator over all frames in the file.
    ///
    /// Frames are read sequentially from the current file position.
    /// Each frame contains one or more matrices of data.
    ///
    /// # Returns
    ///
    /// A [`FrameIterator`] that yields [`Frame`] objects.
    ///
    /// # Panics
    ///
    /// Panics if called while another frame iterator is active.
    /// Only one iterator can be active at a time because the
    /// underlying C library maintains file position state.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::SdifFile;
    ///
    /// let file = SdifFile::open("input.sdif")?;
    /// for frame_result in file.frames() {
    ///     let frame = frame_result?;
    ///     println!("Time: {:.3}, Sig: {}", frame.time(), frame.signature());
    /// }
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn frames(&self) -> FrameIterator<'_> {
        if self.iterating.get() {
            panic!("Cannot create multiple frame iterators simultaneously");
        }
        self.iterating.set(true);
        FrameIterator::new(self)
    }
    
    /// Get the raw C file handle.
    ///
    /// # Safety
    ///
    /// This is for internal use. The caller must not close the handle
    /// or use it after the `SdifFile` is dropped.
    pub(crate) fn handle(&self) -> *mut SdifFileT {
        self.handle.as_ptr()
    }
    
    /// Mark that frame iteration has ended.
    pub(crate) fn end_iteration(&self) {
        self.iterating.set(false);
    }
    
    /// Read NVT entries from the file.
    fn read_nvts(handle: *mut SdifFileT) -> Vec<HashMap<String, String>> {
        // TODO: Implement NVT reading using SDIF C API
        // For now, return empty vec - will implement with proper C API calls
        // The C API provides SdifFGetAllNVT, SdifNameValueTableGetNbData, etc.
        
        // Placeholder - actual implementation requires walking the NVT structures
        Vec::new()
    }
}

impl Drop for SdifFile {
    fn drop(&mut self) {
        // SAFETY: We own the handle and it's valid (NonNull).
        // After this, the handle is invalid, but we're being dropped anyway.
        unsafe {
            SdifFClose(self.handle.as_ptr());
        }
    }
}

// Explicitly mark as !Send and !Sync
// The PhantomData<*const ()> already does this, but let's be explicit
impl !Send for SdifFile {}
impl !Sync for SdifFile {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_open_nonexistent() {
        let result = SdifFile::open("/nonexistent/path/to/file.sdif");
        assert!(result.is_err());
        
        match result.unwrap_err() {
            Error::OpenFailed { path } => {
                assert!(path.to_str().unwrap().contains("nonexistent"));
            }
            other => panic!("Expected OpenFailed, got {:?}", other),
        }
    }
    
    // Additional tests require test fixtures - see integration tests
}
```

---

## Step 8: Frame Implementation

### Task 8.1: Implement Frame Type and Iterator

**Claude Code Prompt:**

```
Create sdif-rs/src/frame.rs:

//! SDIF frame representation and iteration.
//!
//! A frame is a time-stamped container for one or more matrices.
//! Frames are the primary unit of data organization in SDIF files.

use std::marker::PhantomData;

use sdif_sys::{
    SdifFCurrFrameSignature, SdifFCurrNbMatrix, SdifFCurrTime,
    SdifFGetSignature, SdifFReadFrameHeader, SdifFSkipFrameData,
    SdifFileT,
};

use crate::error::{Error, Result};
use crate::file::SdifFile;
use crate::matrix::MatrixIterator;
use crate::signature::{signature_to_string, Signature};

/// A single frame from an SDIF file.
///
/// A frame represents a snapshot of data at a specific point in time.
/// It contains one or more matrices, all sharing the same timestamp.
///
/// `Frame` borrows from its parent [`SdifFile`], ensuring the file
/// remains open while the frame is in use.
///
/// # Example
///
/// ```no_run
/// use sdif_rs::SdifFile;
///
/// let file = SdifFile::open("input.sdif")?;
/// for frame in file.frames() {
///     let frame = frame?;
///     println!("Frame '{}' at {:.3}s with {} matrices",
///         frame.signature(),
///         frame.time(),
///         frame.num_matrices()
///     );
/// }
/// # Ok::<(), sdif_rs::Error>(())
/// ```
pub struct Frame<'a> {
    /// Reference to the parent file.
    file: &'a SdifFile,
    
    /// Frame timestamp in seconds.
    time: f64,
    
    /// Frame type signature.
    signature: Signature,
    
    /// Stream ID for this frame.
    stream_id: u32,
    
    /// Number of matrices in this frame.
    num_matrices: u32,
    
    /// Current matrix index during iteration.
    current_matrix: u32,
    
    /// Whether we've finished reading this frame's data.
    finished: bool,
    
    /// Lifetime marker.
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Frame<'a> {
    /// Create a new Frame from the current file state.
    ///
    /// This should only be called after SdifFReadFrameHeader succeeds.
    pub(crate) fn from_current(file: &'a SdifFile) -> Self {
        let handle = file.handle();
        
        let time = unsafe { SdifFCurrTime(handle) };
        let signature = unsafe { SdifFCurrFrameSignature(handle) };
        let stream_id = unsafe { SdifFGetSignature(handle) }; // Stream ID is stored here
        let num_matrices = unsafe { SdifFCurrNbMatrix(handle) };
        
        Frame {
            file,
            time,
            signature,
            stream_id,
            num_matrices,
            current_matrix: 0,
            finished: false,
            _phantom: PhantomData,
        }
    }
    
    /// Get the frame timestamp in seconds.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sdif_rs::SdifFile;
    /// # let file = SdifFile::open("input.sdif")?;
    /// # let frame = file.frames().next().unwrap()?;
    /// if frame.time() >= 1.0 {
    ///     println!("Frame is at or after 1 second");
    /// }
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn time(&self) -> f64 {
        self.time
    }
    
    /// Get the frame type signature as a string (e.g., "1TRC").
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sdif_rs::SdifFile;
    /// # let file = SdifFile::open("input.sdif")?;
    /// # let frame = file.frames().next().unwrap()?;
    /// if frame.signature() == "1TRC" {
    ///     println!("This is a sinusoidal tracks frame");
    /// }
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn signature(&self) -> String {
        signature_to_string(self.signature)
    }
    
    /// Get the frame type signature as a raw u32.
    pub fn signature_raw(&self) -> Signature {
        self.signature
    }
    
    /// Get the stream ID for this frame.
    ///
    /// Stream IDs allow multiple parallel streams in one SDIF file.
    /// Most files use stream ID 0.
    pub fn stream_id(&self) -> u32 {
        self.stream_id
    }
    
    /// Get the number of matrices in this frame.
    ///
    /// Most frames contain a single matrix, but some frame types
    /// (like 1TRC) can contain multiple matrices per frame.
    pub fn num_matrices(&self) -> usize {
        self.num_matrices as usize
    }
    
    /// Create an iterator over the matrices in this frame.
    ///
    /// Matrices are read sequentially. Each matrix can only be
    /// read once per frame iteration.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sdif_rs::SdifFile;
    /// # let file = SdifFile::open("input.sdif")?;
    /// for frame in file.frames() {
    ///     let frame = frame?;
    ///     for matrix in frame.matrices() {
    ///         let matrix = matrix?;
    ///         println!("  Matrix '{}': {}x{}",
    ///             matrix.signature(),
    ///             matrix.rows(),
    ///             matrix.cols()
    ///         );
    ///     }
    /// }
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn matrices(&mut self) -> MatrixIterator<'_> {
        MatrixIterator::new(self)
    }
    
    /// Get the file handle for matrix reading.
    pub(crate) fn handle(&self) -> *mut SdifFileT {
        self.file.handle()
    }
    
    /// Get the current matrix index.
    pub(crate) fn current_matrix_index(&self) -> u32 {
        self.current_matrix
    }
    
    /// Increment the matrix counter.
    pub(crate) fn advance_matrix(&mut self) {
        self.current_matrix += 1;
    }
    
    /// Check if there are more matrices to read.
    pub(crate) fn has_more_matrices(&self) -> bool {
        self.current_matrix < self.num_matrices
    }
    
    /// Mark this frame as finished (all matrices read or skipped).
    pub(crate) fn mark_finished(&mut self) {
        self.finished = true;
    }
    
    /// Skip remaining matrices in this frame.
    ///
    /// Called when the frame is dropped without reading all matrices.
    fn skip_remaining(&mut self) {
        if !self.finished && self.current_matrix < self.num_matrices {
            // Skip remaining frame data
            unsafe {
                SdifFSkipFrameData(self.file.handle());
            }
        }
        self.finished = true;
    }
}

impl Drop for Frame<'_> {
    fn drop(&mut self) {
        self.skip_remaining();
    }
}

/// Iterator over frames in an SDIF file.
///
/// Created by [`SdifFile::frames()`].
pub struct FrameIterator<'a> {
    file: &'a SdifFile,
    finished: bool,
}

impl<'a> FrameIterator<'a> {
    pub(crate) fn new(file: &'a SdifFile) -> Self {
        FrameIterator {
            file,
            finished: false,
        }
    }
}

impl<'a> Iterator for FrameIterator<'a> {
    type Item = Result<Frame<'a>>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        
        let handle = self.file.handle();
        
        // Try to read the next frame header
        let bytes_read = unsafe { SdifFReadFrameHeader(handle) };
        
        if bytes_read == 0 {
            // End of file or error
            self.finished = true;
            return None;
        }
        
        if bytes_read < 0 {
            // Read error
            self.finished = true;
            return Some(Err(Error::read_error("Failed to read frame header")));
        }
        
        // Successfully read a frame header
        Some(Ok(Frame::from_current(self.file)))
    }
}

impl Drop for FrameIterator<'_> {
    fn drop(&mut self) {
        self.file.end_iteration();
    }
}

#[cfg(test)]
mod tests {
    // Tests require test fixtures - see integration tests
}
```

---

## Step 9: Matrix Implementation

### Task 9.1: Implement Matrix Type with Data Access

**Claude Code Prompt:**

```
Create sdif-rs/src/matrix.rs:

//! SDIF matrix representation and data access.
//!
//! Matrices are the fundamental data containers in SDIF files.
//! Each matrix has a signature, dimensions (rows x columns), and
//! typed numeric data.

use std::marker::PhantomData;

use sdif_sys::{
    SdifFCurrDataType, SdifFCurrMatrixSignature, SdifFCurrNbCol,
    SdifFCurrNbRow, SdifFReadMatrixData, SdifFReadMatrixHeader,
    SdifFCurrOneRowData, SdifFReadOneRow, SdifFSkipMatrixData,
    SdifFileT, SdifDataTypeET_eFloat4, SdifDataTypeET_eFloat8,
};

use crate::data_type::DataType;
use crate::error::{Error, Result};
use crate::frame::Frame;
use crate::signature::{signature_to_string, Signature};

#[cfg(feature = "ndarray")]
use ndarray::{Array2, ShapeBuilder};

/// A matrix of data from an SDIF frame.
///
/// Matrices contain 2D arrays of numeric data. Common columns include
/// Index, Frequency, Amplitude, and Phase for sinusoidal data.
///
/// # Data Access
///
/// Data can be accessed in several ways:
///
/// - [`data_f64()`](Self::data_f64) - Get all data as `Vec<f64>` (row-major)
/// - [`data_f32()`](Self::data_f32) - Get all data as `Vec<f32>` (row-major)
/// - [`to_array_f64()`](Self::to_array_f64) - Get as `ndarray::Array2<f64>` (requires `ndarray` feature)
///
/// # Example
///
/// ```no_run
/// # use sdif_rs::SdifFile;
/// let file = SdifFile::open("input.sdif")?;
/// for frame in file.frames() {
///     for matrix in frame?.matrices() {
///         let matrix = matrix?;
///         println!("Matrix '{}': {}x{} ({})",
///             matrix.signature(),
///             matrix.rows(),
///             matrix.cols(),
///             matrix.data_type()
///         );
///         
///         // Get data as f64
///         let data = matrix.data_f64()?;
///         println!("Total elements: {}", data.len());
///     }
/// }
/// # Ok::<(), sdif_rs::Error>(())
/// ```
pub struct Matrix<'a> {
    /// Reference to parent frame (for file handle access).
    frame: &'a Frame<'a>,
    
    /// Matrix type signature.
    signature: Signature,
    
    /// Number of rows.
    rows: u32,
    
    /// Number of columns.
    cols: u32,
    
    /// Data type of matrix elements.
    data_type: DataType,
    
    /// Whether data has been read.
    data_read: bool,
    
    /// Lifetime marker.
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Matrix<'a> {
    /// Create a new Matrix from the current file state.
    ///
    /// This should only be called after SdifFReadMatrixHeader succeeds.
    pub(crate) fn from_current(frame: &'a Frame<'a>) -> Self {
        let handle = frame.handle();
        
        let signature = unsafe { SdifFCurrMatrixSignature(handle) };
        let rows = unsafe { SdifFCurrNbRow(handle) };
        let cols = unsafe { SdifFCurrNbCol(handle) };
        let raw_dtype = unsafe { SdifFCurrDataType(handle) };
        let data_type = DataType::from_raw(raw_dtype as u32);
        
        Matrix {
            frame,
            signature,
            rows,
            cols,
            data_type,
            data_read: false,
            _phantom: PhantomData,
        }
    }
    
    /// Get the matrix type signature as a string (e.g., "1TRC").
    pub fn signature(&self) -> String {
        signature_to_string(self.signature)
    }
    
    /// Get the matrix type signature as a raw u32.
    pub fn signature_raw(&self) -> Signature {
        self.signature
    }
    
    /// Get the number of rows in the matrix.
    pub fn rows(&self) -> usize {
        self.rows as usize
    }
    
    /// Get the number of columns in the matrix.
    pub fn cols(&self) -> usize {
        self.cols as usize
    }
    
    /// Get the total number of elements in the matrix.
    pub fn len(&self) -> usize {
        self.rows() * self.cols()
    }
    
    /// Check if the matrix is empty (no elements).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Get the data type of matrix elements.
    pub fn data_type(&self) -> DataType {
        self.data_type
    }
    
    /// Get the matrix dimensions as a tuple (rows, cols).
    pub fn shape(&self) -> (usize, usize) {
        (self.rows(), self.cols())
    }
    
    /// Read matrix data as f64 values in row-major order.
    ///
    /// This reads all matrix data and converts to f64 if necessary.
    /// The data is returned in row-major order (C order).
    ///
    /// # Returns
    ///
    /// A vector of f64 values with length `rows * cols`.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidState`] if data was already read
    /// - [`Error::ReadError`] if data couldn't be read
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sdif_rs::SdifFile;
    /// # let file = SdifFile::open("input.sdif")?;
    /// # let mut frame = file.frames().next().unwrap()?;
    /// # let matrix = frame.matrices().next().unwrap()?;
    /// let data = matrix.data_f64()?;
    /// 
    /// // Access element at row 2, col 3
    /// let cols = matrix.cols();
    /// let value = data[2 * cols + 3];
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn data_f64(mut self) -> Result<Vec<f64>> {
        if self.data_read {
            return Err(Error::invalid_state("Matrix data already read"));
        }
        self.data_read = true;
        
        let handle = self.frame.handle();
        let total_elements = self.len();
        let mut data = Vec::with_capacity(total_elements);
        
        // Read row by row
        for _row in 0..self.rows {
            let bytes_read = unsafe { SdifFReadOneRow(handle) };
            if bytes_read <= 0 {
                return Err(Error::read_error("Failed to read matrix row"));
            }
            
            // Get pointer to row data
            let row_data = unsafe { SdifFCurrOneRowData(handle) };
            if row_data.is_null() {
                return Err(Error::null_pointer("Row data pointer"));
            }
            
            // Copy data based on type
            match self.data_type {
                DataType::Float8 => {
                    let ptr = row_data as *const f64;
                    for col in 0..self.cols as usize {
                        data.push(unsafe { *ptr.add(col) });
                    }
                }
                DataType::Float4 => {
                    let ptr = row_data as *const f32;
                    for col in 0..self.cols as usize {
                        data.push(unsafe { *ptr.add(col) } as f64);
                    }
                }
                _ => {
                    return Err(Error::type_mismatch("float", self.data_type.to_string()));
                }
            }
        }
        
        Ok(data)
    }
    
    /// Read matrix data as f32 values in row-major order.
    ///
    /// Similar to [`data_f64()`](Self::data_f64) but returns f32 values.
    /// If the source data is f64, it will be truncated to f32.
    pub fn data_f32(mut self) -> Result<Vec<f32>> {
        if self.data_read {
            return Err(Error::invalid_state("Matrix data already read"));
        }
        self.data_read = true;
        
        let handle = self.frame.handle();
        let total_elements = self.len();
        let mut data = Vec::with_capacity(total_elements);
        
        for _row in 0..self.rows {
            let bytes_read = unsafe { SdifFReadOneRow(handle) };
            if bytes_read <= 0 {
                return Err(Error::read_error("Failed to read matrix row"));
            }
            
            let row_data = unsafe { SdifFCurrOneRowData(handle) };
            if row_data.is_null() {
                return Err(Error::null_pointer("Row data pointer"));
            }
            
            match self.data_type {
                DataType::Float4 => {
                    let ptr = row_data as *const f32;
                    for col in 0..self.cols as usize {
                        data.push(unsafe { *ptr.add(col) });
                    }
                }
                DataType::Float8 => {
                    let ptr = row_data as *const f64;
                    for col in 0..self.cols as usize {
                        data.push(unsafe { *ptr.add(col) } as f32);
                    }
                }
                _ => {
                    return Err(Error::type_mismatch("float", self.data_type.to_string()));
                }
            }
        }
        
        Ok(data)
    }
    
    /// Read matrix data as an ndarray Array2<f64>.
    ///
    /// Requires the `ndarray` feature.
    ///
    /// # Returns
    ///
    /// A 2D array with shape (rows, cols) in row-major (C) order.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(feature = "ndarray")]
    /// # fn example() -> sdif_rs::Result<()> {
    /// use sdif_rs::SdifFile;
    /// use ndarray::Array2;
    ///
    /// let file = SdifFile::open("input.sdif")?;
    /// let mut frame = file.frames().next().unwrap()?;
    /// let matrix = frame.matrices().next().unwrap()?;
    ///
    /// let array: Array2<f64> = matrix.to_array_f64()?;
    /// println!("Shape: {:?}", array.shape());
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "ndarray")]
    pub fn to_array_f64(self) -> Result<Array2<f64>> {
        let shape = self.shape();
        let data = self.data_f64()?;
        
        // Create array in row-major (C) order
        Array2::from_shape_vec(shape.strides((shape.1, 1)), data)
            .map_err(|e| Error::invalid_format(format!("Array shape error: {}", e)))
    }
    
    /// Read matrix data as an ndarray Array2<f32>.
    ///
    /// Requires the `ndarray` feature.
    #[cfg(feature = "ndarray")]
    pub fn to_array_f32(self) -> Result<Array2<f32>> {
        let shape = self.shape();
        let data = self.data_f32()?;
        
        Array2::from_shape_vec(shape.strides((shape.1, 1)), data)
            .map_err(|e| Error::invalid_format(format!("Array shape error: {}", e)))
    }
    
    /// Skip this matrix's data without reading it.
    ///
    /// Useful when you want to skip matrices you're not interested in.
    pub fn skip(mut self) -> Result<()> {
        if self.data_read {
            return Ok(());
        }
        self.data_read = true;
        
        let result = unsafe { SdifFSkipMatrixData(self.frame.handle()) };
        if result < 0 {
            Err(Error::read_error("Failed to skip matrix data"))
        } else {
            Ok(())
        }
    }
}

impl Drop for Matrix<'_> {
    fn drop(&mut self) {
        // If data wasn't read, skip it to maintain file position
        if !self.data_read {
            unsafe {
                SdifFSkipMatrixData(self.frame.handle());
            }
            self.data_read = true;
        }
    }
}

/// Iterator over matrices in a frame.
///
/// Created by [`Frame::matrices()`].
pub struct MatrixIterator<'a> {
    frame: &'a mut Frame<'a>,
}

impl<'a> MatrixIterator<'a> {
    pub(crate) fn new(frame: &'a mut Frame<'a>) -> Self {
        MatrixIterator { frame }
    }
}

impl<'a> Iterator for MatrixIterator<'a> {
    type Item = Result<Matrix<'a>>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if !self.frame.has_more_matrices() {
            self.frame.mark_finished();
            return None;
        }
        
        let handle = self.frame.handle();
        
        // Read matrix header
        let bytes_read = unsafe { SdifFReadMatrixHeader(handle) };
        
        if bytes_read <= 0 {
            return Some(Err(Error::read_error("Failed to read matrix header")));
        }
        
        self.frame.advance_matrix();
        
        // SAFETY: We need to create a Matrix with the same lifetime as the frame.
        // This is safe because the iterator borrows frame mutably, preventing
        // other access, and Matrix will be dropped before the next iteration.
        let matrix = unsafe {
            std::mem::transmute::<Matrix<'_>, Matrix<'a>>(Matrix::from_current(self.frame))
        };
        
        Some(Ok(matrix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_type_size() {
        assert_eq!(DataType::Float4.size_bytes(), 4);
        assert_eq!(DataType::Float8.size_bytes(), 8);
    }
}
```

---

## Step 10: Integration Tests

### Task 10.1: Create Integration Test Suite

**Claude Code Prompt:**

```
Create sdif-rs/tests/integration.rs:

//! Integration tests for sdif-rs
//!
//! These tests verify the complete reading workflow.

use sdif_rs::{SdifFile, Error, Result};
use std::path::PathBuf;

/// Get path to test fixtures directory
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Get path to a specific test fixture
fn fixture(name: &str) -> PathBuf {
    fixtures_dir().join(name)
}

#[test]
fn test_open_nonexistent_file() {
    let result = SdifFile::open("/nonexistent/path.sdif");
    assert!(result.is_err());
    
    match result {
        Err(Error::OpenFailed { path }) => {
            assert!(path.to_string_lossy().contains("nonexistent"));
        }
        Err(e) => panic!("Expected OpenFailed, got: {:?}", e),
        Ok(_) => panic!("Expected error for nonexistent file"),
    }
}

#[test]
fn test_signatures_module() {
    use sdif_rs::signatures;
    use sdif_rs::signature_to_string;
    
    assert_eq!(signature_to_string(signatures::TRC), "1TRC");
    assert_eq!(signature_to_string(signatures::HRM), "1HRM");
    assert_eq!(signature_to_string(signatures::FQ0), "1FQ0");
}

#[test]
fn test_signature_roundtrip() {
    use sdif_rs::{string_to_signature, signature_to_string};
    
    let original = "TEST";
    let sig = string_to_signature(original).unwrap();
    let recovered = signature_to_string(sig);
    assert_eq!(original, recovered);
}

#[test]
fn test_invalid_signature() {
    use sdif_rs::string_to_signature;
    
    // Too short
    assert!(string_to_signature("ABC").is_err());
    
    // Too long
    assert!(string_to_signature("ABCDE").is_err());
}

#[test]
fn test_data_type_properties() {
    use sdif_rs::DataType;
    
    assert!(DataType::Float4.is_float());
    assert!(DataType::Float8.is_float());
    assert_eq!(DataType::Float4.size_bytes(), 4);
    assert_eq!(DataType::Float8.size_bytes(), 8);
    
    assert!(DataType::Int4.is_integer());
    assert!(DataType::Int4.is_signed());
    assert!(!DataType::UInt4.is_signed());
}

// Tests that require actual SDIF files
// These will be skipped if fixtures don't exist

#[test]
#[ignore = "Requires test fixture: simple.sdif"]
fn test_read_simple_file() {
    let path = fixture("simple.sdif");
    if !path.exists() {
        eprintln!("Skipping test: {} not found", path.display());
        return;
    }
    
    let file = SdifFile::open(&path).expect("Failed to open test file");
    
    let mut frame_count = 0;
    for frame_result in file.frames() {
        let frame = frame_result.expect("Failed to read frame");
        frame_count += 1;
        
        assert!(frame.time() >= 0.0, "Frame time should be non-negative");
        assert!(!frame.signature().is_empty(), "Frame should have a signature");
    }
    
    assert!(frame_count > 0, "File should have at least one frame");
}

#[test]
#[ignore = "Requires test fixture: simple.sdif"]
fn test_read_matrices() {
    let path = fixture("simple.sdif");
    if !path.exists() {
        return;
    }
    
    let file = SdifFile::open(&path).expect("Failed to open test file");
    
    for frame_result in file.frames() {
        let mut frame = frame_result.expect("Failed to read frame");
        
        for matrix_result in frame.matrices() {
            let matrix = matrix_result.expect("Failed to read matrix");
            
            assert!(matrix.rows() > 0, "Matrix should have rows");
            assert!(matrix.cols() > 0, "Matrix should have columns");
            
            // Read data
            let data = matrix.data_f64().expect("Failed to read matrix data");
            assert_eq!(data.len(), matrix.rows() * matrix.cols());
        }
    }
}

#[cfg(feature = "ndarray")]
#[test]
#[ignore = "Requires test fixture: simple.sdif"]
fn test_ndarray_integration() {
    let path = fixture("simple.sdif");
    if !path.exists() {
        return;
    }
    
    let file = SdifFile::open(&path).expect("Failed to open test file");
    
    for frame_result in file.frames() {
        let mut frame = frame_result.expect("Failed to read frame");
        
        for matrix_result in frame.matrices() {
            let matrix = matrix_result.expect("Failed to read matrix");
            let expected_shape = matrix.shape();
            
            let array = matrix.to_array_f64().expect("Failed to convert to ndarray");
            
            assert_eq!(array.shape(), &[expected_shape.0, expected_shape.1]);
        }
    }
}
```

### Task 10.2: Create Test Fixtures README

**Claude Code Prompt:**

```
Create sdif-rs/tests/fixtures/README.md:

# Test Fixtures for sdif-rs

This directory should contain SDIF test files for integration testing.

## Required Files

1. **simple.sdif** - Minimal SDIF file with:
   - At least one 1TRC frame
   - At least one matrix with a few rows
   - Basic NVT entries (creator, date)

2. **multiframe.sdif** - SDIF file with:
   - Multiple frames at different times
   - Multiple matrices per frame

3. **all_types.sdif** - SDIF file demonstrating:
   - Various frame types (1TRC, 1HRM, 1FQ0)
   - Both Float32 and Float64 data

## Creating Test Files

### Using Python (pysdif3)

```python
import pysdif3
import numpy as np

# Create simple.sdif
with pysdif3.SdifFile('simple.sdif', 'w') as f:
    # Add metadata
    f.add_NVT({'creator': 'sdif-rs-test', 'date': '2024-01-01'})
    
    # Define types
    f.add_frame_type('1TRC', '1TRC SinusoidalTracks')
    f.add_matrix_type('1TRC', 'Index, Frequency, Amplitude, Phase')
    
    # Write a frame
    data = np.array([
        [1, 440.0, 0.5, 0.0],
        [2, 880.0, 0.3, 1.57],
        [3, 1320.0, 0.2, 3.14],
    ])
    
    f.new_frame('1TRC', 0.0, 1)
    f.add_matrix('1TRC', data)
    
    f.new_frame('1TRC', 0.1, 1)
    f.add_matrix('1TRC', data * 0.9)

print("Created simple.sdif")
```

### Using SPEAR

1. Open an audio file in SPEAR
2. Perform analysis
3. Export as SDIF

## Running Tests with Fixtures

Once fixtures are in place:

```bash
# Run all tests including those requiring fixtures
cargo test -- --include-ignored

# Run only fixture-dependent tests  
cargo test --test integration -- --include-ignored
```

Also create an empty .gitkeep:
sdif-rs/tests/fixtures/.gitkeep
```

---

## Step 11: Documentation

### Task 11.1: Update sdif-rs README

**Claude Code Prompt:**

```
Create sdif-rs/README.md:

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
```

---

## Step 12: Verification

### Task 12.1: Create Verification Script

**Claude Code Prompt:**

```
Create scripts/verify-phase2.sh:

#!/bin/bash
# Verification script for Phase 2 completion

set -e

echo "=== Phase 2 Verification Script ==="
echo

# Check that Phase 1 is complete
echo "1. Verifying Phase 1 prerequisites..."
if ! cargo check -p sdif-sys 2>/dev/null; then
    echo "   ✗ sdif-sys not building - complete Phase 1 first"
    exit 1
fi
echo "   ✓ sdif-sys builds successfully"

# Check module structure
echo
echo "2. Checking module structure..."
REQUIRED_FILES=(
    "sdif-rs/src/lib.rs"
    "sdif-rs/src/error.rs"
    "sdif-rs/src/init.rs"
    "sdif-rs/src/signature.rs"
    "sdif-rs/src/data_type.rs"
    "sdif-rs/src/file.rs"
    "sdif-rs/src/frame.rs"
    "sdif-rs/src/matrix.rs"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "   ✓ $file exists"
    else
        echo "   ✗ $file missing!"
        exit 1
    fi
done

# Check Cargo.toml has required dependencies
echo
echo "3. Checking dependencies..."
if grep -q "thiserror" sdif-rs/Cargo.toml; then
    echo "   ✓ thiserror dependency present"
else
    echo "   ✗ thiserror dependency missing"
    exit 1
fi

if grep -q 'sdif-sys.*path' sdif-rs/Cargo.toml; then
    echo "   ✓ sdif-sys path dependency present"
else
    echo "   ✗ sdif-sys path dependency missing"
    exit 1
fi

# Try to build sdif-rs
echo
echo "4. Building sdif-rs..."
if cargo build -p sdif-rs 2>/dev/null; then
    echo "   ✓ sdif-rs builds successfully"
else
    echo "   ✗ sdif-rs build failed"
    exit 1
fi

# Try to build with ndarray feature
echo
echo "5. Building with ndarray feature..."
if cargo build -p sdif-rs --features ndarray 2>/dev/null; then
    echo "   ✓ ndarray feature builds successfully"
else
    echo "   ⚠ ndarray feature build failed (optional)"
fi

# Run tests
echo
echo "6. Running tests..."
if cargo test -p sdif-rs 2>/dev/null; then
    echo "   ✓ All unit tests passed"
else
    echo "   ⚠ Some tests failed"
fi

# Check documentation builds
echo
echo "7. Building documentation..."
if cargo doc -p sdif-rs --no-deps 2>/dev/null; then
    echo "   ✓ Documentation builds successfully"
else
    echo "   ⚠ Documentation build had issues"
fi

# Summary
echo
echo "=== Phase 2 Verification Complete ==="
echo
echo "Next steps:"
echo "  1. Add test fixture files to sdif-rs/tests/fixtures/"
echo "  2. Run integration tests: cargo test -p sdif-rs -- --include-ignored"
echo "  3. Proceed to Phase 3: Writing API"

Make executable:
chmod +x scripts/verify-phase2.sh
```

---

## Success Criteria Summary

Phase 2 is complete when:

1. **Module Structure**
   - [ ] All source files created (lib.rs, error.rs, init.rs, signature.rs, data_type.rs, file.rs, frame.rs, matrix.rs)
   - [ ] Proper module declarations in lib.rs
   - [ ] Public API exports configured

2. **Error Handling**
   - [ ] Comprehensive Error enum with thiserror
   - [ ] Result type alias
   - [ ] Meaningful error messages

3. **Core Types**
   - [ ] SdifFile with RAII Drop
   - [ ] Frame with lifetime borrowing
   - [ ] Matrix with data access methods
   - [ ] DataType enum
   - [ ] Signature utilities

4. **Iterators**
   - [ ] FrameIterator over file frames
   - [ ] MatrixIterator over frame matrices
   - [ ] Proper cleanup on drop

5. **Data Access**
   - [ ] data_f64() returns Vec<f64>
   - [ ] data_f32() returns Vec<f32>
   - [ ] ndarray integration (feature-gated)

6. **Thread Safety**
   - [ ] !Send + !Sync markers on SdifFile
   - [ ] Once-guarded initialization

7. **Tests**
   - [ ] Unit tests for all modules
   - [ ] Integration test structure
   - [ ] Test fixtures directory

8. **Documentation**
   - [ ] Crate-level rustdoc
   - [ ] All public items documented
   - [ ] README with examples

---

## Notes for Claude Code

### Lifetime Complexity

The trickiest part of Phase 2 is the lifetime relationships:

```
SdifFile (owns C handle)
    └── Frame<'a> (borrows from SdifFile)
            └── Matrix<'a> (borrows from Frame)
```

The Frame must ensure all its matrices are processed before the next frame is read. The MatrixIterator handles this by taking `&mut Frame`.

### C API Quirks

1. **Sequential Access**: The C library maintains internal state. You must read frame header → matrix header → matrix data in order.

2. **Skip Functions**: If you don't read a matrix's data, you must call `SdifFSkipMatrixData` to advance the file position.

3. **Global State**: `SdifGenInit` must be called once before any operations. We handle this with `std::sync::Once`.

### Testing Without Fixtures

Many tests can run without actual SDIF files by testing:
- Signature conversion
- Error type construction
- DataType properties
- Open failure for nonexistent files

Mark fixture-dependent tests with `#[ignore]` and run them separately when fixtures are available.

### Potential Issues

1. **Bindgen Names**: The exact names of C types/functions may vary. Check the generated bindings in `target/*/build/sdif-sys-*/out/bindings.rs`.

2. **Row Data Pointer**: The `SdifFCurrOneRowData` function returns a pointer that's only valid until the next row read. Don't store it.

3. **NVT Reading**: The NVT reading code is marked TODO - it requires walking C linked list structures. Can be implemented later or left returning empty vec for now.
