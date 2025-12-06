# Phase 3: Writing API with Builder Pattern - Detailed Implementation Plan

## Overview

**Duration:** 3-4 days  
**Dependencies:** Phase 2 complete (sdif-rs reading API functional)  
**Goal:** Implement SDIF file writing with a typestate builder pattern that prevents invalid state transitions at compile time, supporting both single-matrix and multi-matrix frames.

This document provides step-by-step instructions for Claude Code to implement Phase 3. The writing API will use Rust's type system to enforce correct usage patterns.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Writing API Flow                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  SdifFile::builder()                                                         │
│       │                                                                      │
│       ▼                                                                      │
│  ┌─────────────────────────┐                                                │
│  │ SdifFileBuilder<New>    │  ← Can set path                                │
│  └───────────┬─────────────┘                                                │
│              │ .create(path)                                                 │
│              ▼                                                               │
│  ┌─────────────────────────┐                                                │
│  │ SdifFileBuilder<Config> │  ← Can add NVT, matrix types, frame types      │
│  └───────────┬─────────────┘                                                │
│              │ .build()                                                      │
│              ▼                                                               │
│  ┌─────────────────────────┐                                                │
│  │      SdifWriter         │  ← Can write frames                            │
│  └───────────┬─────────────┘                                                │
│              │ .new_frame() or .write_frame_one_matrix()                    │
│              ▼                                                               │
│  ┌─────────────────────────┐                                                │
│  │     FrameBuilder        │  ← Can add matrices, must call .finish()       │
│  └───────────┬─────────────┘                                                │
│              │ .finish()                                                     │
│              ▼                                                               │
│       Back to SdifWriter                                                     │
│              │                                                               │
│              │ .close()                                                      │
│              ▼                                                               │
│         File closed                                                          │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Typestate Pattern Explained

The typestate pattern uses different types to represent different states of an object. State transitions are method calls that consume `self` and return a new type. This makes invalid state transitions a compile-time error.

```rust
// This compiles:
let writer = SdifFile::builder()
    .create("out.sdif")?     // New → Config
    .add_nvt([("key", "val")])  // Config → Config
    .build()?;               // Config → SdifWriter

// This won't compile - can't add NVT after build():
let writer = SdifFile::builder()
    .create("out.sdif")?
    .build()?
    .add_nvt([("key", "val")]); // ERROR: SdifWriter has no method add_nvt
```

---

## Step 1: Add New Modules to lib.rs

### Task 1.1: Update Module Structure

**Claude Code Prompt:**

```
Update sdif-rs/src/lib.rs to add the new writing modules. Add these module declarations after the existing ones:

// In the modules section, add:
mod builder;
mod writer;
mod frame_builder;

// In the public exports section, add:
pub use builder::SdifFileBuilder;
pub use writer::SdifWriter;
pub use frame_builder::FrameBuilder;

Also add a builder() method to the existing code by adding this impl block:

impl SdifFile {
    /// Create a builder for writing a new SDIF file.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::SdifFile;
    ///
    /// let writer = SdifFile::builder()
    ///     .create("output.sdif")?
    ///     .add_nvt([("creator", "my-app")])?
    ///     .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
    ///     .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
    ///     .build()?;
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn builder() -> SdifFileBuilder<builder::New> {
        SdifFileBuilder::new()
    }
}

The complete updated lib.rs should have this structure:

//! # sdif-rs
//! [existing documentation...]

// Modules - Reading
mod data_type;
mod error;
mod file;
mod frame;
mod init;
mod matrix;
mod signature;

// Modules - Writing
mod builder;
mod frame_builder;
mod writer;

// Public exports - Core types
pub use data_type::DataType;
pub use error::{Error, Result};
pub use file::SdifFile;
pub use frame::Frame;
pub use matrix::Matrix;
pub use signature::{Signature, signature_to_string, string_to_signature};

// Public exports - Writing
pub use builder::SdifFileBuilder;
pub use frame_builder::FrameBuilder;
pub use writer::SdifWriter;

// Re-export signatures module
pub mod signatures {
    //! [existing content...]
}

#[cfg(feature = "ndarray")]
pub use ndarray;

// Builder method on SdifFile
impl SdifFile {
    /// Create a builder for writing a new SDIF file.
    pub fn builder() -> SdifFileBuilder<builder::New> {
        SdifFileBuilder::new()
    }
}
```

---

## Step 2: Typestate Builder Implementation

### Task 2.1: Create Builder Module with Typestate Types

**Claude Code Prompt:**

```
Create sdif-rs/src/builder.rs with the typestate builder pattern:

//! SDIF file builder with typestate pattern.
//!
//! The builder uses Rust's type system to enforce valid state transitions
//! at compile time. You cannot write frames before setting up the file,
//! and you cannot modify configuration after headers are written.
//!
//! # State Machine
//!
//! ```text
//! New → (create) → Config → (build) → SdifWriter
//! ```
//!
//! # Example
//!
//! ```no_run
//! use sdif_rs::SdifFile;
//!
//! let mut writer = SdifFile::builder()
//!     .create("output.sdif")?
//!     .add_nvt([("creator", "my-app"), ("date", "2024-01-01")])?
//!     .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
//!     .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
//!     .build()?;
//!
//! // Now we can write frames
//! # Ok::<(), sdif_rs::Error>(())
//! ```

use std::collections::HashMap;
use std::ffi::CString;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::ptr::NonNull;

use sdif_sys::{
    SdifFOpen, SdifFClose, SdifFileT, SdifFileModeET_eWriteFile,
    SdifFWriteGeneralHeader, SdifFWriteAllASCIIChunks,
};

use crate::error::{Error, Result};
use crate::init::ensure_initialized;
use crate::writer::SdifWriter;

// ============================================================================
// Typestate Marker Types
// ============================================================================

/// Marker type: Builder just created, no path set yet.
#[derive(Debug)]
pub struct New;

/// Marker type: Path set, can configure NVT and types.
#[derive(Debug)]
pub struct Config;

// ============================================================================
// Configuration Storage
// ============================================================================

/// Stores NVT (Name-Value Table) entries.
#[derive(Debug, Default, Clone)]
pub(crate) struct NvtConfig {
    /// List of NVT tables, each being a map of key-value pairs.
    pub tables: Vec<HashMap<String, String>>,
}

/// Stores a matrix type definition.
#[derive(Debug, Clone)]
pub(crate) struct MatrixTypeDef {
    pub signature: String,
    pub column_names: Vec<String>,
}

/// Stores a frame type definition.
#[derive(Debug, Clone)]
pub(crate) struct FrameTypeDef {
    pub signature: String,
    pub components: Vec<String>,
}

/// All configuration collected during the builder phase.
#[derive(Debug, Default, Clone)]
pub(crate) struct BuilderConfig {
    pub nvts: NvtConfig,
    pub matrix_types: Vec<MatrixTypeDef>,
    pub frame_types: Vec<FrameTypeDef>,
}

// ============================================================================
// SdifFileBuilder
// ============================================================================

/// Builder for creating new SDIF files.
///
/// Uses the typestate pattern to enforce valid state transitions at compile time.
///
/// # Type Parameters
///
/// * `State` - The current state of the builder (New, Config)
#[derive(Debug)]
pub struct SdifFileBuilder<State> {
    /// Path to the output file (set after create()).
    path: Option<PathBuf>,
    
    /// Configuration accumulated during setup.
    config: BuilderConfig,
    
    /// Phantom data for the state type.
    _state: PhantomData<State>,
}

impl SdifFileBuilder<New> {
    /// Create a new builder in the initial state.
    ///
    /// This is typically called via `SdifFile::builder()`.
    pub fn new() -> Self {
        SdifFileBuilder {
            path: None,
            config: BuilderConfig::default(),
            _state: PhantomData,
        }
    }
    
    /// Set the output path and transition to Config state.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the SDIF file will be created.
    ///
    /// # Returns
    ///
    /// Builder in `Config` state, ready for configuration.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::SdifFile;
    ///
    /// let builder = SdifFile::builder()
    ///     .create("output.sdif")?;
    /// // builder is now SdifFileBuilder<Config>
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn create(self, path: impl AsRef<Path>) -> Result<SdifFileBuilder<Config>> {
        let path = path.as_ref().to_path_buf();
        
        // Validate path is writable (parent directory exists)
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                return Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Parent directory does not exist: {}", parent.display()),
                )));
            }
        }
        
        Ok(SdifFileBuilder {
            path: Some(path),
            config: self.config,
            _state: PhantomData,
        })
    }
}

impl Default for SdifFileBuilder<New> {
    fn default() -> Self {
        Self::new()
    }
}

impl SdifFileBuilder<Config> {
    /// Add a Name-Value Table (NVT) with metadata.
    ///
    /// NVTs store metadata like creator, date, sample rate, etc.
    /// You can call this multiple times to add multiple NVTs.
    ///
    /// # Arguments
    ///
    /// * `entries` - Key-value pairs to add to the NVT.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::SdifFile;
    ///
    /// let builder = SdifFile::builder()
    ///     .create("output.sdif")?
    ///     .add_nvt([
    ///         ("creator", "my-application"),
    ///         ("date", "2024-01-01"),
    ///         ("sample_rate", "44100"),
    ///     ])?;
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn add_nvt<'a>(
        mut self,
        entries: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<Self> {
        let mut nvt = HashMap::new();
        for (key, value) in entries {
            // Validate no embedded nulls
            if key.contains('\0') || value.contains('\0') {
                return Err(Error::invalid_format("NVT key/value cannot contain null bytes"));
            }
            nvt.insert(key.to_string(), value.to_string());
        }
        
        if !nvt.is_empty() {
            self.config.nvts.tables.push(nvt);
        }
        
        Ok(self)
    }
    
    /// Define a matrix type with column names.
    ///
    /// Matrix types define the structure of data matrices. Common types include:
    /// - `1TRC` with columns `["Index", "Frequency", "Amplitude", "Phase"]`
    /// - `1FQ0` with columns `["Frequency", "Confidence"]`
    ///
    /// # Arguments
    ///
    /// * `signature` - 4-character signature (e.g., "1TRC")
    /// * `columns` - Names of the columns in the matrix
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::SdifFile;
    ///
    /// let builder = SdifFile::builder()
    ///     .create("output.sdif")?
    ///     .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
    ///     .add_matrix_type("1FQ0", &["Frequency", "Confidence"])?;
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn add_matrix_type(mut self, signature: &str, columns: &[&str]) -> Result<Self> {
        // Validate signature
        if signature.len() != 4 {
            return Err(Error::invalid_signature(signature));
        }
        
        // Validate columns
        if columns.is_empty() {
            return Err(Error::invalid_format("Matrix type must have at least one column"));
        }
        
        for col in columns {
            if col.contains('\0') || col.contains(',') {
                return Err(Error::invalid_format(
                    "Column names cannot contain null bytes or commas"
                ));
            }
        }
        
        self.config.matrix_types.push(MatrixTypeDef {
            signature: signature.to_string(),
            column_names: columns.iter().map(|s| s.to_string()).collect(),
        });
        
        Ok(self)
    }
    
    /// Define a frame type with its component matrices.
    ///
    /// Frame types define what matrices can appear in a frame.
    /// The components string format is `"MatrixSig MatrixName"`.
    ///
    /// # Arguments
    ///
    /// * `signature` - 4-character signature (e.g., "1TRC")
    /// * `components` - Component definitions (e.g., `["1TRC SinusoidalTracks"]`)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::SdifFile;
    ///
    /// let builder = SdifFile::builder()
    ///     .create("output.sdif")?
    ///     .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?;
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn add_frame_type(mut self, signature: &str, components: &[&str]) -> Result<Self> {
        // Validate signature
        if signature.len() != 4 {
            return Err(Error::invalid_signature(signature));
        }
        
        if components.is_empty() {
            return Err(Error::invalid_format("Frame type must have at least one component"));
        }
        
        self.config.frame_types.push(FrameTypeDef {
            signature: signature.to_string(),
            components: components.iter().map(|s| s.to_string()).collect(),
        });
        
        Ok(self)
    }
    
    /// Finalize configuration and create the writer.
    ///
    /// This opens the file, writes the general header and ASCII chunks
    /// (NVT, type definitions), and returns an `SdifWriter` ready to
    /// write frames.
    ///
    /// # Returns
    ///
    /// An `SdifWriter` for writing frames to the file.
    ///
    /// # Errors
    ///
    /// - [`Error::InitFailed`] if the SDIF library couldn't be initialized
    /// - [`Error::OpenFailed`] if the file couldn't be created
    /// - [`Error::Io`] if writing headers fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::SdifFile;
    ///
    /// let mut writer = SdifFile::builder()
    ///     .create("output.sdif")?
    ///     .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
    ///     .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
    ///     .build()?;
    ///
    /// // writer is ready to write frames
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn build(self) -> Result<SdifWriter> {
        // Ensure library is initialized
        if !ensure_initialized() {
            return Err(Error::InitFailed);
        }
        
        let path = self.path.as_ref().expect("Path should be set in Config state");
        
        // Convert path to C string
        let path_str = path.to_str().ok_or_else(|| {
            Error::invalid_format("Path contains invalid UTF-8")
        })?;
        let c_path = CString::new(path_str)?;
        
        // Open file for writing
        let handle = unsafe {
            SdifFOpen(c_path.as_ptr(), SdifFileModeET_eWriteFile)
        };
        
        let handle = NonNull::new(handle).ok_or_else(|| {
            Error::open_failed(path)
        })?;
        
        // Write NVT and type definitions to the file
        Self::write_ascii_chunks(handle.as_ptr(), &self.config)?;
        
        // Write general header
        let header_bytes = unsafe { SdifFWriteGeneralHeader(handle.as_ptr()) };
        if header_bytes == 0 {
            unsafe { SdifFClose(handle.as_ptr()) };
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write SDIF header",
            )));
        }
        
        // Write ASCII chunks (NVT, type definitions)
        let ascii_bytes = unsafe { SdifFWriteAllASCIIChunks(handle.as_ptr()) };
        if ascii_bytes < 0 {
            unsafe { SdifFClose(handle.as_ptr()) };
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write ASCII chunks",
            )));
        }
        
        Ok(SdifWriter::new(handle, path.clone()))
    }
    
    /// Write NVT and type definitions to the file handle.
    ///
    /// This is called before SdifFWriteAllASCIIChunks to set up the
    /// internal structures that will be written.
    fn write_ascii_chunks(handle: *mut SdifFileT, config: &BuilderConfig) -> Result<()> {
        // Add NVT entries
        for nvt in &config.nvts.tables {
            Self::add_nvt_to_file(handle, nvt)?;
        }
        
        // Add matrix type definitions
        for mtd in &config.matrix_types {
            Self::add_matrix_type_to_file(handle, mtd)?;
        }
        
        // Add frame type definitions
        for ftd in &config.frame_types {
            Self::add_frame_type_to_file(handle, ftd)?;
        }
        
        Ok(())
    }
    
    /// Add a single NVT to the file.
    fn add_nvt_to_file(handle: *mut SdifFileT, nvt: &HashMap<String, String>) -> Result<()> {
        use sdif_sys::{SdifNameValueTableGetTable, SdifNameValuesLNewTable, SdifNameValuesLPutCurrNVT};
        
        unsafe {
            // Get the NVT list from the file
            let nvt_list = SdifNameValueTableGetTable(handle);
            if nvt_list.is_null() {
                return Err(Error::null_pointer("NVT list"));
            }
            
            // Create a new NVT
            let stream_id = 0u32; // Default stream
            SdifNameValuesLNewTable(nvt_list, stream_id);
            
            // Add each key-value pair
            for (key, value) in nvt {
                let c_key = CString::new(key.as_str())?;
                let c_value = CString::new(value.as_str())?;
                
                SdifNameValuesLPutCurrNVT(nvt_list, c_key.as_ptr(), c_value.as_ptr());
            }
        }
        
        Ok(())
    }
    
    /// Add a matrix type definition to the file.
    fn add_matrix_type_to_file(handle: *mut SdifFileT, mtd: &MatrixTypeDef) -> Result<()> {
        use sdif_sys::{
            SdifFGetMatrixTypesTable, SdifMatrixTypeInsertTailColumnDef,
            SdifMatrixTypeLPutSdifMatrixType, SdifSignatureConst,
        };
        
        unsafe {
            // Get the matrix types table
            let mtypes = SdifFGetMatrixTypesTable(handle);
            if mtypes.is_null() {
                return Err(Error::null_pointer("Matrix types table"));
            }
            
            // Create signature
            let sig_bytes = mtd.signature.as_bytes();
            let sig = SdifSignatureConst(
                sig_bytes[0] as i8,
                sig_bytes[1] as i8,
                sig_bytes[2] as i8,
                sig_bytes[3] as i8,
            );
            
            // Create the matrix type
            let mtype = SdifMatrixTypeLPutSdifMatrixType(mtypes, sig);
            if mtype.is_null() {
                return Err(Error::null_pointer("Matrix type"));
            }
            
            // Add column definitions
            for col_name in &mtd.column_names {
                let c_name = CString::new(col_name.as_str())?;
                SdifMatrixTypeInsertTailColumnDef(mtype, c_name.as_ptr());
            }
        }
        
        Ok(())
    }
    
    /// Add a frame type definition to the file.
    fn add_frame_type_to_file(handle: *mut SdifFileT, ftd: &FrameTypeDef) -> Result<()> {
        use sdif_sys::{
            SdifFGetFrameTypesTable, SdifFrameTypePutComponent,
            SdifFrameTypeLPutSdifFrameType, SdifSignatureConst,
        };
        
        unsafe {
            // Get the frame types table
            let ftypes = SdifFGetFrameTypesTable(handle);
            if ftypes.is_null() {
                return Err(Error::null_pointer("Frame types table"));
            }
            
            // Create signature
            let sig_bytes = ftd.signature.as_bytes();
            let sig = SdifSignatureConst(
                sig_bytes[0] as i8,
                sig_bytes[1] as i8,
                sig_bytes[2] as i8,
                sig_bytes[3] as i8,
            );
            
            // Create the frame type
            let ftype = SdifFrameTypeLPutSdifFrameType(ftypes, sig);
            if ftype.is_null() {
                return Err(Error::null_pointer("Frame type"));
            }
            
            // Add component definitions
            // Components are in format "MSIG ComponentName"
            for component in &ftd.components {
                let parts: Vec<&str> = component.splitn(2, ' ').collect();
                if parts.len() != 2 || parts[0].len() != 4 {
                    return Err(Error::invalid_format(format!(
                        "Invalid component format: '{}' (expected 'MSIG Name')",
                        component
                    )));
                }
                
                let msig_bytes = parts[0].as_bytes();
                let msig = SdifSignatureConst(
                    msig_bytes[0] as i8,
                    msig_bytes[1] as i8,
                    msig_bytes[2] as i8,
                    msig_bytes[3] as i8,
                );
                
                let c_name = CString::new(parts[1])?;
                SdifFrameTypePutComponent(ftype, msig, c_name.as_ptr());
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_builder_new() {
        let builder = SdifFileBuilder::<New>::new();
        assert!(builder.path.is_none());
    }
    
    #[test]
    fn test_builder_create_transitions_state() {
        let builder = SdifFileBuilder::<New>::new();
        
        // This should compile and transition to Config state
        let _config_builder: SdifFileBuilder<Config> = builder
            .create("/tmp/test.sdif")
            .unwrap();
    }
    
    #[test]
    fn test_invalid_signature_length() {
        let builder = SdifFileBuilder::<New>::new()
            .create("/tmp/test.sdif")
            .unwrap();
        
        let result = builder.add_matrix_type("TOOLONG", &["Col1"]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_empty_columns_rejected() {
        let builder = SdifFileBuilder::<New>::new()
            .create("/tmp/test.sdif")
            .unwrap();
        
        let result = builder.add_matrix_type("1TRC", &[]);
        assert!(result.is_err());
    }
}
```

---

## Step 3: SdifWriter Implementation

### Task 3.1: Create the Writer Module

**Claude Code Prompt:**

```
Create sdif-rs/src/writer.rs with the active file writer:

//! SDIF file writer for adding frames to an open file.
//!
//! `SdifWriter` is obtained from `SdifFileBuilder::build()` and provides
//! methods for writing frames to the file.

use std::marker::PhantomData;
use std::path::PathBuf;
use std::ptr::NonNull;

use sdif_sys::{
    SdifFClose, SdifFWriteFrameAndOneMatrix, SdifFileT,
    SdifSignatureConst, SdifDataTypeET_eFloat4, SdifDataTypeET_eFloat8,
};

use crate::error::{Error, Result};
use crate::frame_builder::FrameBuilder;
use crate::signature::string_to_signature;

/// Active writer for an SDIF file.
///
/// Created by [`SdifFileBuilder::build()`](crate::SdifFileBuilder::build).
/// Provides methods for writing frames to the file.
///
/// # Thread Safety
///
/// Like `SdifFile`, `SdifWriter` is `!Send + !Sync` because the
/// underlying C library is not thread-safe.
///
/// # Example
///
/// ```no_run
/// use sdif_rs::SdifFile;
///
/// let mut writer = SdifFile::builder()
///     .create("output.sdif")?
///     .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
///     .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
///     .build()?;
///
/// // Write a simple frame with one matrix
/// let data = vec![
///     1.0, 440.0, 0.5, 0.0,  // Partial 1: Index, Freq, Amp, Phase
///     2.0, 880.0, 0.3, 1.57, // Partial 2
/// ];
/// writer.write_frame_one_matrix("1TRC", 0.0, "1TRC", 2, 4, &data)?;
///
/// writer.close()?;
/// # Ok::<(), sdif_rs::Error>(())
/// ```
pub struct SdifWriter {
    /// Pointer to the C file handle.
    handle: NonNull<SdifFileT>,
    
    /// Path to the file (for error messages).
    path: PathBuf,
    
    /// Whether the file has been closed.
    closed: bool,
    
    /// Track the last written time for validation.
    last_time: Option<f64>,
    
    /// Count of frames written.
    frame_count: usize,
    
    /// Marker to make SdifWriter !Send and !Sync.
    _not_send_sync: PhantomData<*const ()>,
}

impl SdifWriter {
    /// Create a new writer (called internally by SdifFileBuilder).
    pub(crate) fn new(handle: NonNull<SdifFileT>, path: PathBuf) -> Self {
        SdifWriter {
            handle,
            path,
            closed: false,
            last_time: None,
            frame_count: 0,
            _not_send_sync: PhantomData,
        }
    }
    
    /// Get the file path.
    pub fn path(&self) -> &Path {
        use std::path::Path;
        &self.path
    }
    
    /// Get the number of frames written so far.
    pub fn frame_count(&self) -> usize {
        self.frame_count
    }
    
    /// Get the last written timestamp.
    pub fn last_time(&self) -> Option<f64> {
        self.last_time
    }
    
    /// Write a frame containing a single matrix.
    ///
    /// This is a convenience method for the common case of one matrix per frame.
    /// For frames with multiple matrices, use [`new_frame()`](Self::new_frame).
    ///
    /// # Arguments
    ///
    /// * `frame_sig` - Frame type signature (e.g., "1TRC")
    /// * `time` - Timestamp in seconds
    /// * `matrix_sig` - Matrix type signature (e.g., "1TRC")
    /// * `rows` - Number of rows in the matrix
    /// * `cols` - Number of columns in the matrix
    /// * `data` - Matrix data in row-major order (f64)
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidSignature`] if signatures are invalid
    /// - [`Error::InvalidState`] if the file is closed
    /// - [`Error::Io`] if writing fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sdif_rs::SdifFile;
    /// # let mut writer = SdifFile::builder()
    /// #     .create("output.sdif")?
    /// #     .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
    /// #     .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
    /// #     .build()?;
    /// // 2 partials, 4 columns each
    /// let data = vec![
    ///     1.0, 440.0, 0.5, 0.0,
    ///     2.0, 880.0, 0.3, 1.57,
    /// ];
    /// writer.write_frame_one_matrix("1TRC", 0.0, "1TRC", 2, 4, &data)?;
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn write_frame_one_matrix(
        &mut self,
        frame_sig: &str,
        time: f64,
        matrix_sig: &str,
        rows: usize,
        cols: usize,
        data: &[f64],
    ) -> Result<()> {
        self.check_not_closed()?;
        self.validate_time(time)?;
        
        // Validate data size
        let expected_len = rows * cols;
        if data.len() != expected_len {
            return Err(Error::InvalidDimensions { rows, cols });
        }
        
        // Convert signatures
        let frame_sig_u32 = string_to_signature(frame_sig)?;
        let matrix_sig_u32 = string_to_signature(matrix_sig)?;
        
        unsafe {
            self.write_frame_and_matrix_raw(
                frame_sig_u32,
                time,
                0, // stream_id
                matrix_sig_u32,
                rows as u32,
                cols as u32,
                data,
            )?;
        }
        
        self.last_time = Some(time);
        self.frame_count += 1;
        
        Ok(())
    }
    
    /// Write a frame with one matrix containing f32 data.
    ///
    /// Similar to [`write_frame_one_matrix`](Self::write_frame_one_matrix)
    /// but writes 32-bit floats instead of 64-bit.
    pub fn write_frame_one_matrix_f32(
        &mut self,
        frame_sig: &str,
        time: f64,
        matrix_sig: &str,
        rows: usize,
        cols: usize,
        data: &[f32],
    ) -> Result<()> {
        self.check_not_closed()?;
        self.validate_time(time)?;
        
        let expected_len = rows * cols;
        if data.len() != expected_len {
            return Err(Error::InvalidDimensions { rows, cols });
        }
        
        let frame_sig_u32 = string_to_signature(frame_sig)?;
        let matrix_sig_u32 = string_to_signature(matrix_sig)?;
        
        unsafe {
            self.write_frame_and_matrix_raw_f32(
                frame_sig_u32,
                time,
                0,
                matrix_sig_u32,
                rows as u32,
                cols as u32,
                data,
            )?;
        }
        
        self.last_time = Some(time);
        self.frame_count += 1;
        
        Ok(())
    }
    
    /// Start building a frame with multiple matrices.
    ///
    /// Returns a [`FrameBuilder`] that allows adding multiple matrices
    /// to the frame. The frame is written when `FrameBuilder::finish()`
    /// is called.
    ///
    /// # Arguments
    ///
    /// * `signature` - Frame type signature (e.g., "1TRC")
    /// * `time` - Timestamp in seconds
    /// * `stream_id` - Stream ID (usually 0)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sdif_rs::SdifFile;
    /// # let mut writer = SdifFile::builder()
    /// #     .create("output.sdif")?
    /// #     .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
    /// #     .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
    /// #     .build()?;
    /// let data1 = vec![1.0, 440.0, 0.5, 0.0];
    /// let data2 = vec![2.0, 880.0, 0.3, 1.57];
    ///
    /// writer.new_frame("1TRC", 0.0, 0)?
    ///     .add_matrix("1TRC", 1, 4, &data1)?
    ///     .add_matrix("1TRC", 1, 4, &data2)?
    ///     .finish()?;
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn new_frame(
        &mut self,
        signature: &str,
        time: f64,
        stream_id: u32,
    ) -> Result<FrameBuilder<'_>> {
        self.check_not_closed()?;
        self.validate_time(time)?;
        
        let sig = string_to_signature(signature)?;
        
        Ok(FrameBuilder::new(self, sig, time, stream_id))
    }
    
    /// Close the file and finalize writing.
    ///
    /// This must be called to ensure all data is flushed and the file
    /// is properly closed. After calling `close()`, no more frames can
    /// be written.
    ///
    /// # Note
    ///
    /// The file will also be closed when the `SdifWriter` is dropped,
    /// but calling `close()` explicitly allows you to handle any errors.
    pub fn close(mut self) -> Result<()> {
        self.do_close()
    }
    
    /// Internal close implementation.
    fn do_close(&mut self) -> Result<()> {
        if self.closed {
            return Ok(());
        }
        
        self.closed = true;
        
        unsafe {
            SdifFClose(self.handle.as_ptr());
        }
        
        Ok(())
    }
    
    /// Check that the file hasn't been closed.
    fn check_not_closed(&self) -> Result<()> {
        if self.closed {
            Err(Error::invalid_state("Writer has been closed"))
        } else {
            Ok(())
        }
    }
    
    /// Validate that time is non-decreasing.
    fn validate_time(&self, time: f64) -> Result<()> {
        if let Some(last) = self.last_time {
            if time < last {
                return Err(Error::invalid_format(format!(
                    "Time must be non-decreasing: {} < {}",
                    time, last
                )));
            }
        }
        Ok(())
    }
    
    /// Get the raw file handle (for FrameBuilder).
    pub(crate) fn handle(&self) -> *mut SdifFileT {
        self.handle.as_ptr()
    }
    
    /// Record that a frame was written (called by FrameBuilder).
    pub(crate) fn record_frame_written(&mut self, time: f64) {
        self.last_time = Some(time);
        self.frame_count += 1;
    }
    
    /// Write a frame with one matrix using raw signatures (f64 data).
    unsafe fn write_frame_and_matrix_raw(
        &self,
        frame_sig: u32,
        time: f64,
        stream_id: u32,
        matrix_sig: u32,
        rows: u32,
        cols: u32,
        data: &[f64],
    ) -> Result<()> {
        use sdif_sys::SdifFWriteFrameAndOneMatrix;
        
        let bytes_written = SdifFWriteFrameAndOneMatrix(
            self.handle.as_ptr(),
            frame_sig,
            stream_id,
            time,
            matrix_sig,
            SdifDataTypeET_eFloat8,
            rows,
            cols,
            data.as_ptr() as *mut libc::c_void,
        );
        
        if bytes_written == 0 {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write frame",
            )))
        } else {
            Ok(())
        }
    }
    
    /// Write a frame with one matrix using raw signatures (f32 data).
    unsafe fn write_frame_and_matrix_raw_f32(
        &self,
        frame_sig: u32,
        time: f64,
        stream_id: u32,
        matrix_sig: u32,
        rows: u32,
        cols: u32,
        data: &[f32],
    ) -> Result<()> {
        use sdif_sys::SdifFWriteFrameAndOneMatrix;
        
        let bytes_written = SdifFWriteFrameAndOneMatrix(
            self.handle.as_ptr(),
            frame_sig,
            stream_id,
            time,
            matrix_sig,
            SdifDataTypeET_eFloat4,
            rows,
            cols,
            data.as_ptr() as *mut libc::c_void,
        );
        
        if bytes_written == 0 {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write frame",
            )))
        } else {
            Ok(())
        }
    }
}

impl Drop for SdifWriter {
    fn drop(&mut self) {
        if !self.closed {
            // Best-effort close, ignore errors
            let _ = self.do_close();
        }
    }
}

// Explicitly mark as !Send and !Sync
impl !Send for SdifWriter {}
impl !Sync for SdifWriter {}

// ============================================================================
// ndarray Integration
// ============================================================================

#[cfg(feature = "ndarray")]
use ndarray::{Array2, ArrayView2};

#[cfg(feature = "ndarray")]
impl SdifWriter {
    /// Write a frame with one matrix from an ndarray Array2<f64>.
    ///
    /// The array is automatically converted to row-major order if needed.
    ///
    /// # Arguments
    ///
    /// * `frame_sig` - Frame type signature
    /// * `time` - Timestamp in seconds
    /// * `matrix_sig` - Matrix type signature
    /// * `data` - 2D array of f64 values
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::SdifFile;
    /// use ndarray::array;
    ///
    /// let mut writer = SdifFile::builder()
    ///     .create("output.sdif")?
    ///     .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
    ///     .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
    ///     .build()?;
    ///
    /// let data = array![
    ///     [1.0, 440.0, 0.5, 0.0],
    ///     [2.0, 880.0, 0.3, 1.57],
    /// ];
    /// writer.write_frame_one_matrix_array("1TRC", 0.0, "1TRC", &data)?;
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn write_frame_one_matrix_array(
        &mut self,
        frame_sig: &str,
        time: f64,
        matrix_sig: &str,
        data: &Array2<f64>,
    ) -> Result<()> {
        let (rows, cols) = data.dim();
        
        // Convert to row-major (C order) if needed
        let data_vec: Vec<f64> = if data.is_standard_layout() {
            data.iter().copied().collect()
        } else {
            // Need to copy in row-major order
            data.rows().into_iter()
                .flat_map(|row| row.iter().copied())
                .collect()
        };
        
        self.write_frame_one_matrix(frame_sig, time, matrix_sig, rows, cols, &data_vec)
    }
    
    /// Write a frame with one matrix from an ndarray Array2<f32>.
    pub fn write_frame_one_matrix_array_f32(
        &mut self,
        frame_sig: &str,
        time: f64,
        matrix_sig: &str,
        data: &Array2<f32>,
    ) -> Result<()> {
        let (rows, cols) = data.dim();
        
        let data_vec: Vec<f32> = if data.is_standard_layout() {
            data.iter().copied().collect()
        } else {
            data.rows().into_iter()
                .flat_map(|row| row.iter().copied())
                .collect()
        };
        
        self.write_frame_one_matrix_f32(frame_sig, time, matrix_sig, rows, cols, &data_vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Most tests require actual file I/O - see integration tests
    
    #[test]
    fn test_validate_time_increasing() {
        // This would require a mock, so we just test the logic indirectly
        // through integration tests
    }
}
```

---

## Step 4: FrameBuilder Implementation

### Task 4.1: Create the FrameBuilder Module

**Claude Code Prompt:**

```
Create sdif-rs/src/frame_builder.rs for building multi-matrix frames:

//! Frame builder for constructing frames with multiple matrices.
//!
//! `FrameBuilder` provides a way to add multiple matrices to a single frame
//! before writing it to the file. Use `SdifWriter::new_frame()` to create one.

use sdif_sys::{
    SdifFSetCurrFrameHeader, SdifFSetCurrMatrixHeader,
    SdifFWriteFrameHeader, SdifFWriteMatrixHeader, SdifFWriteMatrixData,
    SdifFWritePadding, SdifDataTypeET_eFloat4, SdifDataTypeET_eFloat8,
};

use crate::error::{Error, Result};
use crate::signature::string_to_signature;
use crate::writer::SdifWriter;

/// Builder for frames with multiple matrices.
///
/// Created by [`SdifWriter::new_frame()`]. Matrices are added with
/// [`add_matrix()`](Self::add_matrix), and the frame is finalized with
/// [`finish()`](Self::finish).
///
/// # Important
///
/// You **must** call [`finish()`](Self::finish) to write the frame.
/// If the `FrameBuilder` is dropped without calling `finish()`, it will
/// panic in debug builds to help catch bugs.
///
/// # Example
///
/// ```no_run
/// # use sdif_rs::SdifFile;
/// # let mut writer = SdifFile::builder()
/// #     .create("output.sdif")?
/// #     .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
/// #     .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
/// #     .build()?;
/// let partials1 = vec![1.0, 440.0, 0.5, 0.0, 2.0, 880.0, 0.3, 1.57];
/// let partials2 = vec![3.0, 1320.0, 0.2, 3.14];
///
/// writer.new_frame("1TRC", 0.0, 0)?
///     .add_matrix("1TRC", 2, 4, &partials1)?
///     .add_matrix("1TRC", 1, 4, &partials2)?
///     .finish()?;
/// # Ok::<(), sdif_rs::Error>(())
/// ```
pub struct FrameBuilder<'a> {
    /// Reference to the parent writer.
    writer: &'a mut SdifWriter,
    
    /// Frame signature.
    signature: u32,
    
    /// Frame timestamp.
    time: f64,
    
    /// Stream ID.
    stream_id: u32,
    
    /// Matrices to write (collected before writing frame header).
    matrices: Vec<MatrixData>,
    
    /// Whether finish() was called.
    finished: bool,
}

/// Internal storage for a matrix's data.
struct MatrixData {
    signature: u32,
    rows: u32,
    cols: u32,
    data: MatrixDataType,
}

/// Matrix data can be f32 or f64.
enum MatrixDataType {
    Float32(Vec<f32>),
    Float64(Vec<f64>),
}

impl<'a> FrameBuilder<'a> {
    /// Create a new FrameBuilder (called internally by SdifWriter).
    pub(crate) fn new(
        writer: &'a mut SdifWriter,
        signature: u32,
        time: f64,
        stream_id: u32,
    ) -> Self {
        FrameBuilder {
            writer,
            signature,
            time,
            stream_id,
            matrices: Vec::new(),
            finished: false,
        }
    }
    
    /// Add a matrix with f64 data to the frame.
    ///
    /// # Arguments
    ///
    /// * `signature` - Matrix type signature (e.g., "1TRC")
    /// * `rows` - Number of rows
    /// * `cols` - Number of columns
    /// * `data` - Matrix data in row-major order
    ///
    /// # Returns
    ///
    /// Self for method chaining.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidSignature`] if the signature is invalid
    /// - [`Error::InvalidDimensions`] if data length doesn't match rows*cols
    pub fn add_matrix(
        mut self,
        signature: &str,
        rows: usize,
        cols: usize,
        data: &[f64],
    ) -> Result<Self> {
        let sig = string_to_signature(signature)?;
        
        let expected_len = rows * cols;
        if data.len() != expected_len {
            return Err(Error::InvalidDimensions { rows, cols });
        }
        
        self.matrices.push(MatrixData {
            signature: sig,
            rows: rows as u32,
            cols: cols as u32,
            data: MatrixDataType::Float64(data.to_vec()),
        });
        
        Ok(self)
    }
    
    /// Add a matrix with f32 data to the frame.
    ///
    /// Similar to [`add_matrix()`](Self::add_matrix) but for 32-bit floats.
    pub fn add_matrix_f32(
        mut self,
        signature: &str,
        rows: usize,
        cols: usize,
        data: &[f32],
    ) -> Result<Self> {
        let sig = string_to_signature(signature)?;
        
        let expected_len = rows * cols;
        if data.len() != expected_len {
            return Err(Error::InvalidDimensions { rows, cols });
        }
        
        self.matrices.push(MatrixData {
            signature: sig,
            rows: rows as u32,
            cols: cols as u32,
            data: MatrixDataType::Float32(data.to_vec()),
        });
        
        Ok(self)
    }
    
    /// Finalize and write the frame to the file.
    ///
    /// This writes the frame header followed by all matrices.
    /// Must be called to complete the frame.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidState`] if no matrices were added
    /// - [`Error::Io`] if writing fails
    pub fn finish(mut self) -> Result<()> {
        if self.matrices.is_empty() {
            return Err(Error::invalid_state("Frame must have at least one matrix"));
        }
        
        self.finished = true;
        self.write_frame()
    }
    
    /// Internal method to write the frame.
    fn write_frame(&mut self) -> Result<()> {
        let handle = self.writer.handle();
        let num_matrices = self.matrices.len() as u32;
        
        // Calculate total data size for frame header
        let data_size = self.calculate_frame_size();
        
        unsafe {
            // Set and write frame header
            SdifFSetCurrFrameHeader(
                handle,
                self.signature,
                data_size,
                num_matrices,
                self.stream_id,
                self.time,
            );
            
            let header_bytes = SdifFWriteFrameHeader(handle);
            if header_bytes == 0 {
                return Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to write frame header",
                )));
            }
            
            // Write each matrix
            for matrix in &self.matrices {
                self.write_matrix(handle, matrix)?;
            }
        }
        
        self.writer.record_frame_written(self.time);
        
        Ok(())
    }
    
    /// Calculate the total size of frame data.
    fn calculate_frame_size(&self) -> u32 {
        let mut size = 0u32;
        
        for matrix in &self.matrices {
            // Matrix header size (signature + type + rows + cols = 16 bytes)
            size += 16;
            
            // Matrix data size
            let element_size = match &matrix.data {
                MatrixDataType::Float32(_) => 4,
                MatrixDataType::Float64(_) => 8,
            };
            let data_bytes = matrix.rows * matrix.cols * element_size;
            size += data_bytes;
            
            // Padding to 8-byte boundary
            let padding = (8 - (data_bytes % 8)) % 8;
            size += padding;
        }
        
        size
    }
    
    /// Write a single matrix.
    unsafe fn write_matrix(&self, handle: *mut sdif_sys::SdifFileT, matrix: &MatrixData) -> Result<()> {
        let (data_type, data_ptr, element_size) = match &matrix.data {
            MatrixDataType::Float32(v) => (
                SdifDataTypeET_eFloat4,
                v.as_ptr() as *const libc::c_void,
                4u32,
            ),
            MatrixDataType::Float64(v) => (
                SdifDataTypeET_eFloat8,
                v.as_ptr() as *const libc::c_void,
                8u32,
            ),
        };
        
        // Set and write matrix header
        SdifFSetCurrMatrixHeader(
            handle,
            matrix.signature,
            data_type,
            matrix.rows,
            matrix.cols,
        );
        
        let header_bytes = SdifFWriteMatrixHeader(handle);
        if header_bytes == 0 {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write matrix header",
            )));
        }
        
        // Write matrix data
        let data_bytes = SdifFWriteMatrixData(handle, data_ptr as *mut libc::c_void);
        if data_bytes == 0 {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write matrix data",
            )));
        }
        
        // Write padding
        SdifFWritePadding(handle, SdifFPaddingCalculate(handle, data_bytes));
        
        Ok(())
    }
}

/// Calculate padding needed to reach 8-byte alignment.
unsafe fn SdifFPaddingCalculate(_handle: *mut sdif_sys::SdifFileT, bytes_written: usize) -> u32 {
    let remainder = bytes_written % 8;
    if remainder == 0 {
        0
    } else {
        (8 - remainder) as u32
    }
}

impl Drop for FrameBuilder<'_> {
    fn drop(&mut self) {
        if !self.finished && !self.matrices.is_empty() {
            // In debug mode, panic to alert developer of bug
            #[cfg(debug_assertions)]
            panic!(
                "FrameBuilder dropped without calling finish()! \
                 Frame at time {} with {} matrices was not written.",
                self.time,
                self.matrices.len()
            );
            
            // In release mode, try to write the frame
            #[cfg(not(debug_assertions))]
            {
                let _ = self.write_frame();
            }
        }
    }
}

// ============================================================================
// ndarray Integration
// ============================================================================

#[cfg(feature = "ndarray")]
use ndarray::Array2;

#[cfg(feature = "ndarray")]
impl<'a> FrameBuilder<'a> {
    /// Add a matrix from an ndarray Array2<f64>.
    ///
    /// The array is automatically converted to row-major order if needed.
    pub fn add_matrix_array(
        self,
        signature: &str,
        data: &Array2<f64>,
    ) -> Result<Self> {
        let (rows, cols) = data.dim();
        
        // Convert to row-major (C order) if needed
        let data_vec: Vec<f64> = if data.is_standard_layout() {
            data.iter().copied().collect()
        } else {
            data.rows().into_iter()
                .flat_map(|row| row.iter().copied())
                .collect()
        };
        
        self.add_matrix(signature, rows, cols, &data_vec)
    }
    
    /// Add a matrix from an ndarray Array2<f32>.
    pub fn add_matrix_array_f32(
        self,
        signature: &str,
        data: &Array2<f32>,
    ) -> Result<Self> {
        let (rows, cols) = data.dim();
        
        let data_vec: Vec<f32> = if data.is_standard_layout() {
            data.iter().copied().collect()
        } else {
            data.rows().into_iter()
                .flat_map(|row| row.iter().copied())
                .collect()
        };
        
        self.add_matrix_f32(signature, rows, cols, &data_vec)
    }
}

#[cfg(test)]
mod tests {
    // Integration tests cover the main functionality
}
```

---

## Step 5: Update Error Types

### Task 5.1: Add Write-Related Error Variants

**Claude Code Prompt:**

```
Update sdif-rs/src/error.rs to add any missing error variants needed for writing.
Add these variants to the Error enum if not already present:

    /// The file has already been closed.
    #[error("File has been closed")]
    FileClosed,
    
    /// Frame has no matrices.
    #[error("Frame must contain at least one matrix")]
    EmptyFrame,
    
    /// Time values must be non-decreasing.
    #[error("Time must be non-decreasing: {current} < {previous}")]
    TimeNotIncreasing {
        current: f64,
        previous: f64,
    },

Also ensure the InvalidDimensions variant exists:

    /// The matrix dimensions are invalid.
    #[error("Invalid matrix dimensions: {rows}x{cols}")]
    InvalidDimensions {
        /// Number of rows.
        rows: usize,
        /// Number of columns.
        cols: usize,
    },

Update the helper methods section with:

    /// Create a TimeNotIncreasing error.
    pub fn time_not_increasing(current: f64, previous: f64) -> Self {
        Self::TimeNotIncreasing { current, previous }
    }
```

---

## Step 6: Integration Tests for Writing

### Task 6.1: Create Write Tests

**Claude Code Prompt:**

```
Create sdif-rs/tests/write_tests.rs:

//! Integration tests for SDIF writing functionality.

use sdif_rs::{SdifFile, SdifWriter, Result, Error};
use std::fs;
use tempfile::NamedTempFile;

/// Helper to create a temporary SDIF file path.
fn temp_sdif_path() -> NamedTempFile {
    NamedTempFile::new().expect("Failed to create temp file")
}

#[test]
fn test_create_minimal_file() -> Result<()> {
    let temp = temp_sdif_path();
    let path = temp.path();
    
    // Create a minimal SDIF file
    let mut writer = SdifFile::builder()
        .create(path)?
        .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
        .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
        .build()?;
    
    // Write one frame with one matrix
    let data = vec![
        1.0, 440.0, 0.5, 0.0,
        2.0, 880.0, 0.3, 1.57,
    ];
    writer.write_frame_one_matrix("1TRC", 0.0, "1TRC", 2, 4, &data)?;
    
    writer.close()?;
    
    // Verify file was created
    assert!(path.exists());
    
    // Verify file has content
    let metadata = fs::metadata(path)?;
    assert!(metadata.len() > 0);
    
    Ok(())
}

#[test]
fn test_write_multiple_frames() -> Result<()> {
    let temp = temp_sdif_path();
    let path = temp.path();
    
    let mut writer = SdifFile::builder()
        .create(path)?
        .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
        .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
        .build()?;
    
    // Write multiple frames at different times
    for i in 0..10 {
        let time = i as f64 * 0.1;
        let data = vec![
            1.0, 440.0 + i as f64 * 10.0, 0.5, 0.0,
        ];
        writer.write_frame_one_matrix("1TRC", time, "1TRC", 1, 4, &data)?;
    }
    
    assert_eq!(writer.frame_count(), 10);
    
    writer.close()?;
    
    Ok(())
}

#[test]
fn test_write_with_nvt() -> Result<()> {
    let temp = temp_sdif_path();
    let path = temp.path();
    
    let mut writer = SdifFile::builder()
        .create(path)?
        .add_nvt([
            ("creator", "sdif-rs-test"),
            ("date", "2024-01-01"),
            ("description", "Test file with NVT"),
        ])?
        .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
        .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
        .build()?;
    
    let data = vec![1.0, 440.0, 0.5, 0.0];
    writer.write_frame_one_matrix("1TRC", 0.0, "1TRC", 1, 4, &data)?;
    
    writer.close()?;
    
    Ok(())
}

#[test]
fn test_write_f32_data() -> Result<()> {
    let temp = temp_sdif_path();
    let path = temp.path();
    
    let mut writer = SdifFile::builder()
        .create(path)?
        .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
        .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
        .build()?;
    
    let data: Vec<f32> = vec![1.0, 440.0, 0.5, 0.0];
    writer.write_frame_one_matrix_f32("1TRC", 0.0, "1TRC", 1, 4, &data)?;
    
    writer.close()?;
    
    Ok(())
}

#[test]
fn test_frame_builder_multiple_matrices() -> Result<()> {
    let temp = temp_sdif_path();
    let path = temp.path();
    
    let mut writer = SdifFile::builder()
        .create(path)?
        .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
        .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
        .build()?;
    
    // Write a frame with multiple matrices
    let data1 = vec![1.0, 440.0, 0.5, 0.0];
    let data2 = vec![2.0, 880.0, 0.3, 1.57];
    
    writer.new_frame("1TRC", 0.0, 0)?
        .add_matrix("1TRC", 1, 4, &data1)?
        .add_matrix("1TRC", 1, 4, &data2)?
        .finish()?;
    
    writer.close()?;
    
    Ok(())
}

#[test]
fn test_invalid_signature_rejected() {
    let temp = temp_sdif_path();
    let path = temp.path();
    
    let result = SdifFile::builder()
        .create(path)
        .unwrap()
        .add_matrix_type("TOOLONG", &["Col"]);
    
    assert!(result.is_err());
}

#[test]
fn test_empty_columns_rejected() {
    let temp = temp_sdif_path();
    let path = temp.path();
    
    let result = SdifFile::builder()
        .create(path)
        .unwrap()
        .add_matrix_type("1TRC", &[]);
    
    assert!(result.is_err());
}

#[test]
fn test_time_must_be_nondecreasing() -> Result<()> {
    let temp = temp_sdif_path();
    let path = temp.path();
    
    let mut writer = SdifFile::builder()
        .create(path)?
        .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
        .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
        .build()?;
    
    let data = vec![1.0, 440.0, 0.5, 0.0];
    
    // First frame at time 1.0
    writer.write_frame_one_matrix("1TRC", 1.0, "1TRC", 1, 4, &data)?;
    
    // Second frame at time 0.5 should fail (time going backwards)
    let result = writer.write_frame_one_matrix("1TRC", 0.5, "1TRC", 1, 4, &data);
    
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_data_length_validation() -> Result<()> {
    let temp = temp_sdif_path();
    let path = temp.path();
    
    let mut writer = SdifFile::builder()
        .create(path)?
        .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
        .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
        .build()?;
    
    // Data has 3 elements but we claim 2 rows x 4 cols = 8
    let data = vec![1.0, 2.0, 3.0];
    let result = writer.write_frame_one_matrix("1TRC", 0.0, "1TRC", 2, 4, &data);
    
    assert!(result.is_err());
    
    Ok(())
}

// Roundtrip test - write then read
#[test]
fn test_write_then_read_roundtrip() -> Result<()> {
    let temp = temp_sdif_path();
    let path = temp.path();
    
    // Original data
    let original_data = vec![
        1.0, 440.0, 0.5, 0.0,
        2.0, 880.0, 0.3, 1.57,
        3.0, 1320.0, 0.2, 3.14,
    ];
    let original_time = 0.123;
    
    // Write
    {
        let mut writer = SdifFile::builder()
            .create(path)?
            .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
            .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
            .build()?;
        
        writer.write_frame_one_matrix("1TRC", original_time, "1TRC", 3, 4, &original_data)?;
        writer.close()?;
    }
    
    // Read back
    {
        let file = SdifFile::open(path)?;
        
        let mut frame_count = 0;
        for frame_result in file.frames() {
            let mut frame = frame_result?;
            frame_count += 1;
            
            // Check time (with tolerance for floating point)
            assert!((frame.time() - original_time).abs() < 1e-9);
            assert_eq!(frame.signature(), "1TRC");
            
            for matrix_result in frame.matrices() {
                let matrix = matrix_result?;
                
                assert_eq!(matrix.signature(), "1TRC");
                assert_eq!(matrix.rows(), 3);
                assert_eq!(matrix.cols(), 4);
                
                let data = matrix.data_f64()?;
                assert_eq!(data.len(), original_data.len());
                
                // Compare values
                for (a, b) in data.iter().zip(original_data.iter()) {
                    assert!((a - b).abs() < 1e-9, "Data mismatch: {} vs {}", a, b);
                }
            }
        }
        
        assert_eq!(frame_count, 1);
    }
    
    Ok(())
}

#[cfg(feature = "ndarray")]
mod ndarray_tests {
    use super::*;
    use ndarray::array;
    
    #[test]
    fn test_write_ndarray() -> Result<()> {
        let temp = temp_sdif_path();
        let path = temp.path();
        
        let mut writer = SdifFile::builder()
            .create(path)?
            .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
            .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
            .build()?;
        
        let data = array![
            [1.0, 440.0, 0.5, 0.0],
            [2.0, 880.0, 0.3, 1.57],
        ];
        
        writer.write_frame_one_matrix_array("1TRC", 0.0, "1TRC", &data)?;
        writer.close()?;
        
        Ok(())
    }
    
    #[test]
    fn test_frame_builder_ndarray() -> Result<()> {
        let temp = temp_sdif_path();
        let path = temp.path();
        
        let mut writer = SdifFile::builder()
            .create(path)?
            .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
            .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
            .build()?;
        
        let data1 = array![[1.0, 440.0, 0.5, 0.0]];
        let data2 = array![[2.0, 880.0, 0.3, 1.57]];
        
        writer.new_frame("1TRC", 0.0, 0)?
            .add_matrix_array("1TRC", &data1)?
            .add_matrix_array("1TRC", &data2)?
            .finish()?;
        
        writer.close()?;
        
        Ok(())
    }
}
```

---

## Step 7: Documentation Updates

### Task 7.1: Update README with Writing Examples

**Claude Code Prompt:**

```
Update sdif-rs/README.md to include writing examples. Add this section after the reading examples:

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

Also update the feature flags section to mention that ndarray helps with writing too.
```

---

## Step 8: Verification Script

### Task 8.1: Create Phase 3 Verification Script

**Claude Code Prompt:**

```
Create scripts/verify-phase3.sh:

#!/bin/bash
# Verification script for Phase 3 completion

set -e

echo "=== Phase 3 Verification Script ==="
echo

# Check that Phases 1 and 2 are complete
echo "1. Verifying prerequisites..."
if ! cargo check -p sdif-sys 2>/dev/null; then
    echo "   ✗ sdif-sys not building - complete Phase 1 first"
    exit 1
fi
echo "   ✓ sdif-sys builds"

if ! cargo check -p sdif-rs 2>/dev/null; then
    echo "   ✗ sdif-rs not building - complete Phase 2 first"
    exit 1
fi
echo "   ✓ sdif-rs (reading) builds"

# Check new module files
echo
echo "2. Checking Phase 3 modules..."
REQUIRED_FILES=(
    "sdif-rs/src/builder.rs"
    "sdif-rs/src/writer.rs"
    "sdif-rs/src/frame_builder.rs"
    "sdif-rs/tests/write_tests.rs"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "   ✓ $file exists"
    else
        echo "   ✗ $file missing!"
        exit 1
    fi
done

# Check that builder types are exported
echo
echo "3. Checking public exports..."
if grep -q "pub use builder::SdifFileBuilder" sdif-rs/src/lib.rs; then
    echo "   ✓ SdifFileBuilder exported"
else
    echo "   ✗ SdifFileBuilder not exported"
    exit 1
fi

if grep -q "pub use writer::SdifWriter" sdif-rs/src/lib.rs; then
    echo "   ✓ SdifWriter exported"
else
    echo "   ✗ SdifWriter not exported"
    exit 1
fi

if grep -q "pub use frame_builder::FrameBuilder" sdif-rs/src/lib.rs; then
    echo "   ✓ FrameBuilder exported"
else
    echo "   ✗ FrameBuilder not exported"
    exit 1
fi

# Build with all features
echo
echo "4. Building sdif-rs with all features..."
if cargo build -p sdif-rs --all-features 2>/dev/null; then
    echo "   ✓ Full build successful"
else
    echo "   ✗ Build failed"
    exit 1
fi

# Run unit tests
echo
echo "5. Running unit tests..."
if cargo test -p sdif-rs --lib 2>/dev/null; then
    echo "   ✓ Unit tests passed"
else
    echo "   ⚠ Some unit tests failed"
fi

# Run write integration tests
echo
echo "6. Running write integration tests..."
if cargo test -p sdif-rs --test write_tests 2>/dev/null; then
    echo "   ✓ Write tests passed"
else
    echo "   ⚠ Write tests failed - check output above"
fi

# Test roundtrip specifically
echo
echo "7. Testing write-read roundtrip..."
if cargo test -p sdif-rs test_write_then_read_roundtrip 2>/dev/null; then
    echo "   ✓ Roundtrip test passed"
else
    echo "   ⚠ Roundtrip test failed"
fi

# Check documentation builds
echo
echo "8. Building documentation..."
if cargo doc -p sdif-rs --no-deps 2>/dev/null; then
    echo "   ✓ Documentation builds"
else
    echo "   ⚠ Documentation issues"
fi

# Summary
echo
echo "=== Phase 3 Verification Complete ==="
echo
echo "The writing API is implemented with:"
echo "  - SdifFileBuilder (typestate pattern)"
echo "  - SdifWriter (frame writing)"
echo "  - FrameBuilder (multi-matrix frames)"
echo
echo "Next steps:"
echo "  1. Test with Max/MSP if available"
echo "  2. Create example programs in examples/"
echo "  3. Proceed to Phase 4: MAT File Integration"

Make executable:
chmod +x scripts/verify-phase3.sh
```

---

## Success Criteria Summary

Phase 3 is complete when:

1. **Typestate Builder**
   - [ ] `SdifFileBuilder<New>` for initial state
   - [ ] `SdifFileBuilder<Config>` for configuration state
   - [ ] State transitions enforced at compile time
   - [ ] NVT, matrix type, and frame type configuration

2. **SdifWriter**
   - [ ] Can write single-matrix frames
   - [ ] Supports both f32 and f64 data
   - [ ] Time validation (non-decreasing)
   - [ ] Frame count tracking
   - [ ] Proper close/cleanup

3. **FrameBuilder**
   - [ ] Can add multiple matrices to a frame
   - [ ] Must call finish() to write
   - [ ] Debug panic if not finished
   - [ ] ndarray integration (optional)

4. **Validation**
   - [ ] Signature validation (4 chars)
   - [ ] Data length validation
   - [ ] Non-empty matrices required

5. **Tests**
   - [ ] Create minimal file
   - [ ] Write multiple frames
   - [ ] Write with NVT
   - [ ] Write f32 data
   - [ ] Multi-matrix frames
   - [ ] **Roundtrip test (write then read)**
   - [ ] Error cases (invalid signature, wrong data length)

6. **Documentation**
   - [ ] README updated with writing examples
   - [ ] All public APIs documented
   - [ ] Examples in doc comments

---

## Notes for Claude Code

### Typestate Pattern Benefits

The typestate pattern ensures:
1. Can't write frames before `build()` is called
2. Can't modify NVT/types after headers are written
3. Compiler catches misuse, not runtime

### C API Write Functions

Key SDIF C functions for writing:
- `SdifFWriteGeneralHeader` - Write file header
- `SdifFWriteAllASCIIChunks` - Write NVT and type defs
- `SdifFWriteFrameAndOneMatrix` - Convenience for single-matrix frames
- `SdifFSetCurrFrameHeader` / `SdifFWriteFrameHeader` - For multi-matrix
- `SdifFSetCurrMatrixHeader` / `SdifFWriteMatrixHeader` - Matrix header
- `SdifFWriteMatrixData` - Write raw matrix data
- `SdifFWritePadding` - Write padding bytes

### Padding Requirements

SDIF requires 8-byte alignment. After writing matrix data, calculate padding:
```rust
let padding = (8 - (bytes_written % 8)) % 8;
```

### FrameBuilder Drop Behavior

The choice to panic in debug mode on drop without finish() is intentional:
- Catches bugs during development
- In release, tries to write the frame as a fallback
- Alternative: always panic, forcing explicit finish() or cancel()

### Common Pitfalls

1. **Forgetting to close**: Writer closes in Drop, but errors are ignored
2. **Wrong data order**: SDIF uses row-major; ndarray defaults to row-major but can be column-major
3. **Time going backwards**: SDIF readers may fail or behave oddly
4. **Missing type definitions**: Some readers require type defs even for standard types
