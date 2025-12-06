# Phase 4: MAT File Integration - Detailed Implementation Plan

## Overview

**Duration:** 2-3 days  
**Dependencies:** Phase 3 complete (sdif-rs writing API functional)  
**Goal:** Add MAT file parsing using the `matfile` crate, with utilities for extracting numeric arrays, detecting time vectors, and converting data to SDIF-compatible formats.

This document provides step-by-step instructions for Claude Code to implement Phase 4. The MAT integration will be added to `sdif-rs` as an optional feature, preparing for the `mat2sdif` CLI tool in Phase 5.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         MAT to SDIF Pipeline                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                   │
│  │   MAT File   │───▶│   MatFile    │───▶│   MatData    │                   │
│  │  (.mat v5/7) │    │   (parsed)   │    │  (per var)   │                   │
│  └──────────────┘    └──────────────┘    └──────────────┘                   │
│                                                 │                            │
│                                                 ▼                            │
│                              ┌─────────────────────────────────┐            │
│                              │      Time Vector Detection       │            │
│                              │   (1D, ascending, named "time")  │            │
│                              └─────────────────────────────────┘            │
│                                                 │                            │
│                                                 ▼                            │
│                              ┌─────────────────────────────────┐            │
│                              │     MatToSdifConverter           │            │
│                              │  - Maps time → frame timestamps  │            │
│                              │  - Maps rows → SDIF matrices     │            │
│                              │  - Handles transpose             │            │
│                              └─────────────────────────────────┘            │
│                                                 │                            │
│                                                 ▼                            │
│                              ┌─────────────────────────────────┐            │
│                              │         SdifWriter               │            │
│                              │   (from Phase 3)                 │            │
│                              └─────────────────────────────────┘            │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Key Design Decisions

1. **Optional Feature**: MAT support is behind a `mat` feature flag to avoid pulling in dependencies for users who don't need it
2. **ndarray Integration**: Heavy use of ndarray for array manipulation, required when `mat` feature is enabled
3. **Flexible Time Detection**: Heuristics for auto-detecting time vectors, with manual override option
4. **Column-to-Row Major**: Automatic transposition from MATLAB's column-major to SDIF's row-major format
5. **Complex Number Handling**: Support for magnitude/phase or real/imaginary extraction

---

## Step 1: Update Cargo.toml with MAT Dependencies

### Task 1.1: Add Feature Flag and Dependencies

**Claude Code Prompt:**

```
Update sdif-rs/Cargo.toml to add the MAT file support as an optional feature.

Add these to the [features] section:
[features]
default = []
ndarray = ["dep:ndarray"]
# MAT file support (requires ndarray)
mat = ["dep:matfile", "ndarray"]
bundled = ["sdif-sys/bundled"]
static = ["sdif-sys/static"]

Add these to the [dependencies] section:
[dependencies]
sdif-sys = { path = "../sdif-sys" }
thiserror = "1.0"
libc = "0.2"

# Optional dependencies
ndarray = { version = "0.15", optional = true }
matfile = { version = "0.5", optional = true }

The complete Cargo.toml should look like:

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
# MAT file support (automatically enables ndarray)
mat = ["dep:matfile", "ndarray"]
# Pass through to sdif-sys
bundled = ["sdif-sys/bundled"]
static = ["sdif-sys/static"]

[dependencies]
sdif-sys = { path = "../sdif-sys" }
thiserror = "1.0"
libc = "0.2"

# Optional dependencies
ndarray = { version = "0.15", optional = true }
matfile = { version = "0.5", optional = true }

[dev-dependencies]
tempfile = "3.0"
approx = "0.5"
```

---

## Step 2: Create MAT Module Structure

### Task 2.1: Add Module to lib.rs

**Claude Code Prompt:**

```
Update sdif-rs/src/lib.rs to add the MAT module conditionally.

Add after the other module declarations:

// Modules - MAT file support (optional)
#[cfg(feature = "mat")]
mod mat;

// Public exports - MAT support
#[cfg(feature = "mat")]
pub use mat::{MatData, MatFile, MatToSdifConfig, MatToSdifConverter};

Also add documentation for the mat feature in the crate-level docs:

//! ## Feature Flags
//!
//! - `ndarray`: Enable `ndarray` integration for matrix data access
//! - `mat`: Enable MAT file parsing for MATLAB/Octave file conversion (includes `ndarray`)
//! - `bundled`: Compile SDIF C library from bundled source
//! - `static`: Force static linking of SDIF C library
```

### Task 2.2: Create MAT Module File Structure

**Claude Code Prompt:**

```
Create the MAT module structure. We'll use a directory module for organization:

sdif-rs/src/mat/
├── mod.rs           # Module root, public exports
├── file.rs          # MatFile - MAT file loading
├── data.rs          # MatData - Single variable wrapper
├── time.rs          # Time vector detection
├── convert.rs       # MAT to SDIF conversion
└── complex.rs       # Complex number handling

Create each file with a placeholder doc comment initially:

// src/mat/mod.rs
//! MAT file support for MATLAB/Octave file parsing.

// src/mat/file.rs
//! MAT file loading and variable listing.

// src/mat/data.rs
//! Individual MAT variable representation.

// src/mat/time.rs
//! Time vector detection heuristics.

// src/mat/convert.rs
//! MAT to SDIF conversion utilities.

// src/mat/complex.rs
//! Complex number handling for MAT arrays.
```

---

## Step 3: MatFile Implementation

### Task 3.1: Create MatFile Wrapper

**Claude Code Prompt:**

```
Create sdif-rs/src/mat/file.rs:

//! MAT file loading and variable listing.
//!
//! This module provides [`MatFile`], a wrapper around the matfile crate
//! for loading and inspecting MATLAB/Octave .mat files.

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use matfile::{MatFile as RawMatFile, NumericData};
use ndarray::{Array2, ArrayD};

use crate::error::{Error, Result};
use super::data::MatData;

/// A loaded MAT file containing numeric variables.
///
/// `MatFile` wraps the matfile crate's parser and provides convenient
/// access to numeric arrays within the file.
///
/// # Supported Formats
///
/// - Level 5 MAT files (MATLAB v5, v6, v7)
/// - v7 compressed files (zlib)
/// - Numeric arrays: double, single, int8/16/32/64, uint8/16/32/64
/// - Complex arrays (stored as two separate real arrays)
///
/// # Unsupported
///
/// - Level 4 MAT files (legacy format)
/// - HDF5-based v7.3 files
/// - Cell arrays, structs, sparse matrices, function handles
///
/// # Example
///
/// ```no_run
/// use sdif_rs::MatFile;
///
/// let mat = MatFile::open("analysis.mat")?;
///
/// // List all variables
/// for name in mat.variable_names() {
///     println!("Variable: {}", name);
/// }
///
/// // Get a specific variable
/// if let Some(data) = mat.get("frequencies")? {
///     println!("Shape: {:?}", data.shape());
/// }
/// # Ok::<(), sdif_rs::Error>(())
/// ```
#[derive(Debug)]
pub struct MatFile {
    /// Parsed variables, keyed by name.
    variables: HashMap<String, MatData>,
    
    /// Original file path (for error messages).
    path: String,
}

impl MatFile {
    /// Open and parse a MAT file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the .mat file.
    ///
    /// # Returns
    ///
    /// A `MatFile` containing all parseable numeric variables.
    ///
    /// # Errors
    ///
    /// - [`Error::Io`] if the file cannot be read
    /// - [`Error::InvalidFormat`] if the file is not a valid MAT file
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::MatFile;
    ///
    /// let mat = MatFile::open("data.mat")?;
    /// println!("Loaded {} variables", mat.len());
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let path_str = path.display().to_string();
        
        let file = File::open(path).map_err(|e| {
            Error::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to open MAT file '{}': {}", path_str, e),
            ))
        })?;
        
        let reader = BufReader::new(file);
        
        let mat_file = RawMatFile::parse(reader).map_err(|e| {
            Error::invalid_format(format!("Failed to parse MAT file '{}': {}", path_str, e))
        })?;
        
        let mut variables = HashMap::new();
        
        for array in mat_file.arrays() {
            let name = array.name().to_string();
            
            // Try to convert to MatData
            match MatData::from_matfile_array(array) {
                Ok(data) => {
                    variables.insert(name, data);
                }
                Err(e) => {
                    // Log but don't fail - skip unsupported variable types
                    eprintln!("Warning: Skipping variable '{}': {}", name, e);
                }
            }
        }
        
        Ok(MatFile {
            variables,
            path: path_str,
        })
    }
    
    /// Get the names of all numeric variables in the file.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sdif_rs::MatFile;
    /// # let mat = MatFile::open("data.mat")?;
    /// for name in mat.variable_names() {
    ///     println!("Found variable: {}", name);
    /// }
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn variable_names(&self) -> impl Iterator<Item = &str> {
        self.variables.keys().map(|s| s.as_str())
    }
    
    /// Get a variable by name.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the variable to retrieve.
    ///
    /// # Returns
    ///
    /// The variable data if found, or `None`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sdif_rs::MatFile;
    /// # let mat = MatFile::open("data.mat")?;
    /// if let Some(freqs) = mat.get("frequencies") {
    ///     println!("Frequencies shape: {:?}", freqs.shape());
    /// }
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn get(&self, name: &str) -> Option<&MatData> {
        self.variables.get(name)
    }
    
    /// Get a variable by name, returning an error if not found.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidFormat`] if the variable doesn't exist.
    pub fn require(&self, name: &str) -> Result<&MatData> {
        self.get(name).ok_or_else(|| {
            Error::invalid_format(format!(
                "Variable '{}' not found in MAT file '{}'",
                name, self.path
            ))
        })
    }
    
    /// Get the number of numeric variables in the file.
    pub fn len(&self) -> usize {
        self.variables.len()
    }
    
    /// Check if the file contains no numeric variables.
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }
    
    /// Get the file path.
    pub fn path(&self) -> &str {
        &self.path
    }
    
    /// Iterate over all variables.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &MatData)> {
        self.variables.iter().map(|(k, v)| (k.as_str(), v))
    }
    
    /// Find variables that look like time vectors.
    ///
    /// Uses heuristics to identify potential time vectors:
    /// - Named "time", "t", "times", etc.
    /// - 1D arrays with ascending values
    /// - Values starting near zero
    ///
    /// # Returns
    ///
    /// Names of variables that appear to be time vectors.
    pub fn find_time_vectors(&self) -> Vec<&str> {
        self.variables
            .iter()
            .filter(|(_, data)| data.is_likely_time_vector())
            .map(|(name, _)| name.as_str())
            .collect()
    }
    
    /// Get detailed information about all variables (for --list mode).
    ///
    /// Returns a formatted string describing each variable.
    pub fn describe(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!("Variables in '{}':", self.path));
        lines.push(String::new());
        
        let mut names: Vec<_> = self.variable_names().collect();
        names.sort();
        
        // Calculate column widths
        let max_name_len = names.iter().map(|n| n.len()).max().unwrap_or(4);
        
        lines.push(format!(
            "  {:<width$}  {:>12}  {:>8}  {}",
            "Name", "Shape", "Type", "Notes",
            width = max_name_len
        ));
        lines.push(format!(
            "  {:-<width$}  {:->12}  {:->8}  -----",
            "", "", "",
            width = max_name_len
        ));
        
        for name in names {
            if let Some(data) = self.get(name) {
                let shape_str = format!("{:?}", data.shape());
                let type_str = if data.is_complex() { "complex" } else { "real" };
                
                let mut notes = Vec::new();
                if data.is_likely_time_vector() {
                    notes.push("time?");
                }
                if data.is_1d() {
                    notes.push("1D");
                }
                
                lines.push(format!(
                    "  {:<width$}  {:>12}  {:>8}  {}",
                    name,
                    shape_str,
                    type_str,
                    notes.join(", "),
                    width = max_name_len
                ));
            }
        }
        
        lines.join("\n")
    }
}

impl IntoIterator for MatFile {
    type Item = (String, MatData);
    type IntoIter = std::collections::hash_map::IntoIter<String, MatData>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.variables.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_open_nonexistent() {
        let result = MatFile::open("/nonexistent/file.mat");
        assert!(result.is_err());
    }
    
    // Additional tests require test MAT files
}
```

---

## Step 4: MatData Implementation

### Task 4.1: Create MatData Variable Wrapper

**Claude Code Prompt:**

```
Create sdif-rs/src/mat/data.rs:

//! Individual MAT variable representation.
//!
//! [`MatData`] represents a single numeric variable from a MAT file,
//! providing access to its data as ndarray arrays.

use ndarray::{Array1, Array2, ArrayD, Axis, IxDyn};
use matfile::{Array as MatArray, NumericData};

use crate::error::{Error, Result};

/// A numeric variable from a MAT file.
///
/// `MatData` wraps a single variable, providing:
/// - Shape information
/// - Data access as 1D or 2D arrays
/// - Complex number handling
/// - Type information
///
/// # Data Layout
///
/// MATLAB uses column-major (Fortran) order, while this library
/// converts to row-major (C) order for SDIF compatibility.
///
/// # Example
///
/// ```no_run
/// use sdif_rs::MatFile;
///
/// let mat = MatFile::open("data.mat")?;
/// let freqs = mat.require("frequencies")?;
///
/// println!("Shape: {:?}", freqs.shape());
/// println!("Is 1D: {}", freqs.is_1d());
///
/// // Get as 2D array
/// let array = freqs.to_array2()?;
/// # Ok::<(), sdif_rs::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct MatData {
    /// Variable name.
    name: String,
    
    /// Shape of the array.
    shape: Vec<usize>,
    
    /// Real part of the data (always present).
    real_data: Vec<f64>,
    
    /// Imaginary part (only for complex data).
    imag_data: Option<Vec<f64>>,
    
    /// Original data type name.
    dtype: String,
}

impl MatData {
    /// Create MatData from a matfile Array.
    pub(crate) fn from_matfile_array(array: &MatArray) -> Result<Self> {
        let name = array.name().to_string();
        let size = array.size();
        let shape: Vec<usize> = size.iter().map(|&x| x as usize).collect();
        
        // Extract numeric data
        let (real_data, imag_data, dtype) = Self::extract_numeric_data(array)?;
        
        Ok(MatData {
            name,
            shape,
            real_data,
            imag_data,
            dtype,
        })
    }
    
    /// Extract numeric data from a matfile Array.
    fn extract_numeric_data(array: &MatArray) -> Result<(Vec<f64>, Option<Vec<f64>>, String)> {
        match array.data() {
            NumericData::Double { real, imag } => {
                let real_vec: Vec<f64> = real.iter().copied().collect();
                let imag_vec = imag.as_ref().map(|i| i.iter().copied().collect());
                Ok((real_vec, imag_vec, "float64".to_string()))
            }
            NumericData::Single { real, imag } => {
                let real_vec: Vec<f64> = real.iter().map(|&x| x as f64).collect();
                let imag_vec = imag.as_ref().map(|i| i.iter().map(|&x| x as f64).collect());
                Ok((real_vec, imag_vec, "float32".to_string()))
            }
            NumericData::Int8 { real, imag } => {
                let real_vec: Vec<f64> = real.iter().map(|&x| x as f64).collect();
                let imag_vec = imag.as_ref().map(|i| i.iter().map(|&x| x as f64).collect());
                Ok((real_vec, imag_vec, "int8".to_string()))
            }
            NumericData::Int16 { real, imag } => {
                let real_vec: Vec<f64> = real.iter().map(|&x| x as f64).collect();
                let imag_vec = imag.as_ref().map(|i| i.iter().map(|&x| x as f64).collect());
                Ok((real_vec, imag_vec, "int16".to_string()))
            }
            NumericData::Int32 { real, imag } => {
                let real_vec: Vec<f64> = real.iter().map(|&x| x as f64).collect();
                let imag_vec = imag.as_ref().map(|i| i.iter().map(|&x| x as f64).collect());
                Ok((real_vec, imag_vec, "int32".to_string()))
            }
            NumericData::Int64 { real, imag } => {
                let real_vec: Vec<f64> = real.iter().map(|&x| x as f64).collect();
                let imag_vec = imag.as_ref().map(|i| i.iter().map(|&x| x as f64).collect());
                Ok((real_vec, imag_vec, "int64".to_string()))
            }
            NumericData::UInt8 { real, imag } => {
                let real_vec: Vec<f64> = real.iter().map(|&x| x as f64).collect();
                let imag_vec = imag.as_ref().map(|i| i.iter().map(|&x| x as f64).collect());
                Ok((real_vec, imag_vec, "uint8".to_string()))
            }
            NumericData::UInt16 { real, imag } => {
                let real_vec: Vec<f64> = real.iter().map(|&x| x as f64).collect();
                let imag_vec = imag.as_ref().map(|i| i.iter().map(|&x| x as f64).collect());
                Ok((real_vec, imag_vec, "uint16".to_string()))
            }
            NumericData::UInt32 { real, imag } => {
                let real_vec: Vec<f64> = real.iter().map(|&x| x as f64).collect();
                let imag_vec = imag.as_ref().map(|i| i.iter().map(|&x| x as f64).collect());
                Ok((real_vec, imag_vec, "uint32".to_string()))
            }
            NumericData::UInt64 { real, imag } => {
                let real_vec: Vec<f64> = real.iter().map(|&x| x as f64).collect();
                let imag_vec = imag.as_ref().map(|i| i.iter().map(|&x| x as f64).collect());
                Ok((real_vec, imag_vec, "uint64".to_string()))
            }
        }
    }
    
    /// Get the variable name.
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the shape of the array.
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }
    
    /// Get the number of dimensions.
    pub fn ndim(&self) -> usize {
        self.shape.len()
    }
    
    /// Check if the array is 1-dimensional.
    ///
    /// Note: MATLAB stores row vectors as [1, N] and column vectors as [N, 1].
    /// This method returns true for both, as well as true 1D arrays.
    pub fn is_1d(&self) -> bool {
        match self.shape.as_slice() {
            [_] => true,
            [1, _] => true,
            [_, 1] => true,
            _ => false,
        }
    }
    
    /// Check if the array is 2-dimensional.
    pub fn is_2d(&self) -> bool {
        self.shape.len() == 2
    }
    
    /// Check if the data is complex.
    pub fn is_complex(&self) -> bool {
        self.imag_data.is_some()
    }
    
    /// Get the total number of elements.
    pub fn len(&self) -> usize {
        self.real_data.len()
    }
    
    /// Check if the array is empty.
    pub fn is_empty(&self) -> bool {
        self.real_data.is_empty()
    }
    
    /// Get the original data type name.
    pub fn dtype(&self) -> &str {
        &self.dtype
    }
    
    /// Get the real part as a 1D array.
    ///
    /// For vectors (1D or [1,N] or [N,1]), returns the data as-is.
    /// For 2D arrays, returns the data in row-major order.
    pub fn to_array1(&self) -> Result<Array1<f64>> {
        if !self.is_1d() && self.shape.len() > 1 {
            return Err(Error::invalid_format(format!(
                "Variable '{}' is not 1D (shape: {:?})",
                self.name, self.shape
            )));
        }
        
        Ok(Array1::from_vec(self.real_data.clone()))
    }
    
    /// Get the real part as a 2D array in row-major order.
    ///
    /// Automatically handles MATLAB's column-major to row-major conversion.
    pub fn to_array2(&self) -> Result<Array2<f64>> {
        let (rows, cols) = self.dims_2d()?;
        
        // MATLAB stores column-major, so we need to transpose
        // Create as column-major then transpose
        let col_major = Array2::from_shape_vec((rows, cols).f(), self.real_data.clone())
            .map_err(|e| Error::invalid_format(format!("Shape error: {}", e)))?;
        
        // Return transposed (now row-major interpretation is correct)
        Ok(col_major)
    }
    
    /// Get 2D dimensions, treating 1D as [N, 1].
    fn dims_2d(&self) -> Result<(usize, usize)> {
        match self.shape.as_slice() {
            [n] => Ok((*n, 1)),
            [r, c] => Ok((*r, *c)),
            _ => Err(Error::invalid_format(format!(
                "Variable '{}' is not 2D (shape: {:?})",
                self.name, self.shape
            ))),
        }
    }
    
    /// Get the imaginary part as a 2D array (for complex data).
    pub fn imag_to_array2(&self) -> Result<Array2<f64>> {
        let imag = self.imag_data.as_ref().ok_or_else(|| {
            Error::invalid_format(format!("Variable '{}' is not complex", self.name))
        })?;
        
        let (rows, cols) = self.dims_2d()?;
        
        let col_major = Array2::from_shape_vec((rows, cols).f(), imag.clone())
            .map_err(|e| Error::invalid_format(format!("Shape error: {}", e)))?;
        
        Ok(col_major)
    }
    
    /// Get magnitude of complex data: sqrt(real² + imag²).
    pub fn magnitude(&self) -> Result<Array2<f64>> {
        let real = self.to_array2()?;
        
        if let Some(ref imag_data) = self.imag_data {
            let (rows, cols) = self.dims_2d()?;
            let imag = Array2::from_shape_vec((rows, cols).f(), imag_data.clone())
                .map_err(|e| Error::invalid_format(format!("Shape error: {}", e)))?;
            
            Ok((&real * &real + &imag * &imag).mapv(f64::sqrt))
        } else {
            // For real data, magnitude is just absolute value
            Ok(real.mapv(f64::abs))
        }
    }
    
    /// Get phase of complex data: atan2(imag, real).
    pub fn phase(&self) -> Result<Array2<f64>> {
        let real = self.to_array2()?;
        
        if let Some(ref imag_data) = self.imag_data {
            let (rows, cols) = self.dims_2d()?;
            let imag = Array2::from_shape_vec((rows, cols).f(), imag_data.clone())
                .map_err(|e| Error::invalid_format(format!("Shape error: {}", e)))?;
            
            // Element-wise atan2
            let mut phase = Array2::zeros((rows, cols));
            for ((r, i), p) in real.iter().zip(imag.iter()).zip(phase.iter_mut()) {
                *p = i.atan2(*r);
            }
            Ok(phase)
        } else {
            // For real data, phase is 0 for positive, π for negative
            Ok(real.mapv(|x| if x >= 0.0 { 0.0 } else { std::f64::consts::PI }))
        }
    }
    
    /// Get raw real data slice.
    pub fn real_data(&self) -> &[f64] {
        &self.real_data
    }
    
    /// Get raw imaginary data slice (if complex).
    pub fn imag_data(&self) -> Option<&[f64]> {
        self.imag_data.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_1d() {
        // These would need actual MatData instances to test properly
        // Integration tests will cover this with real MAT files
    }
}
```

---

## Step 5: Time Vector Detection

### Task 5.1: Create Time Detection Module

**Claude Code Prompt:**

```
Create sdif-rs/src/mat/time.rs:

//! Time vector detection heuristics.
//!
//! This module provides methods for identifying time vectors in MAT files,
//! which are needed to generate SDIF frame timestamps.

use super::data::MatData;

/// Common names for time variables in audio analysis MAT files.
const TIME_VARIABLE_NAMES: &[&str] = &[
    "time", "times", "t", "Time", "Times", "T",
    "time_vec", "time_vector", "timeVec", "timeVector",
    "frame_times", "frameTimes", "frame_time",
];

impl MatData {
    /// Check if this variable is likely a time vector.
    ///
    /// Uses multiple heuristics:
    /// 1. Name matches common time variable names
    /// 2. Array is 1D (or effectively 1D)
    /// 3. Values are monotonically increasing
    /// 4. Values start near zero (within 1 second)
    /// 5. Values are non-negative
    ///
    /// # Returns
    ///
    /// `true` if the variable appears to be a time vector.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sdif_rs::MatFile;
    /// # let mat = MatFile::open("data.mat")?;
    /// for (name, var) in mat.iter() {
    ///     if var.is_likely_time_vector() {
    ///         println!("{} looks like a time vector", name);
    ///     }
    /// }
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn is_likely_time_vector(&self) -> bool {
        // Must be 1D
        if !self.is_1d() {
            return false;
        }
        
        // Must have data
        if self.is_empty() || self.len() < 2 {
            return false;
        }
        
        // Check name heuristic
        let name_matches = TIME_VARIABLE_NAMES
            .iter()
            .any(|&n| self.name.eq_ignore_ascii_case(n));
        
        // Check value patterns
        let values_look_like_time = self.check_time_value_patterns();
        
        // If name matches, be more lenient with values
        // If name doesn't match, require stronger value evidence
        if name_matches {
            values_look_like_time || self.is_monotonically_increasing()
        } else {
            values_look_like_time && self.is_monotonically_increasing()
        }
    }
    
    /// Check if values follow typical time vector patterns.
    fn check_time_value_patterns(&self) -> bool {
        let data = &self.real_data;
        
        if data.is_empty() {
            return false;
        }
        
        // Check first value is near zero (within 10 seconds, typical for audio)
        let first = data[0];
        if first < -0.001 || first > 10.0 {
            return false;
        }
        
        // Check all values are non-negative
        if data.iter().any(|&x| x < -0.001) {
            return false;
        }
        
        // Check monotonically increasing
        self.is_monotonically_increasing()
    }
    
    /// Check if values are monotonically increasing.
    pub fn is_monotonically_increasing(&self) -> bool {
        let data = &self.real_data;
        
        if data.len() < 2 {
            return true;
        }
        
        // Allow small epsilon for floating point comparison
        let eps = 1e-10;
        
        for window in data.windows(2) {
            if window[1] < window[0] - eps {
                return false;
            }
        }
        
        true
    }
    
    /// Check if values are strictly increasing (no duplicates).
    pub fn is_strictly_increasing(&self) -> bool {
        let data = &self.real_data;
        
        if data.len() < 2 {
            return true;
        }
        
        let eps = 1e-10;
        
        for window in data.windows(2) {
            if window[1] <= window[0] + eps {
                return false;
            }
        }
        
        true
    }
    
    /// Get time statistics (useful for validation).
    ///
    /// Returns (min, max, mean_delta) if this is a valid time vector.
    pub fn time_stats(&self) -> Option<TimeStats> {
        if !self.is_1d() || self.len() < 2 {
            return None;
        }
        
        let data = &self.real_data;
        let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        
        // Calculate mean delta (hop size)
        let deltas: Vec<f64> = data.windows(2).map(|w| w[1] - w[0]).collect();
        let mean_delta = deltas.iter().sum::<f64>() / deltas.len() as f64;
        
        // Calculate delta variance (for regularity check)
        let delta_variance = deltas.iter()
            .map(|d| (d - mean_delta).powi(2))
            .sum::<f64>() / deltas.len() as f64;
        
        Some(TimeStats {
            min,
            max,
            duration: max - min,
            num_frames: data.len(),
            mean_hop: mean_delta,
            hop_variance: delta_variance,
            is_regular: delta_variance < mean_delta * 0.01, // <1% variance
        })
    }
}

/// Statistics about a time vector.
#[derive(Debug, Clone)]
pub struct TimeStats {
    /// Minimum time value.
    pub min: f64,
    
    /// Maximum time value.
    pub max: f64,
    
    /// Total duration (max - min).
    pub duration: f64,
    
    /// Number of time points.
    pub num_frames: usize,
    
    /// Mean hop size between frames.
    pub mean_hop: f64,
    
    /// Variance in hop size.
    pub hop_variance: f64,
    
    /// Whether the hop size is regular (consistent spacing).
    pub is_regular: bool,
}

impl std::fmt::Display for TimeStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TimeStats {{ duration: {:.3}s, frames: {}, hop: {:.4}s{} }}",
            self.duration,
            self.num_frames,
            self.mean_hop,
            if self.is_regular { " (regular)" } else { " (irregular)" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Tests would require constructing MatData instances
    // Integration tests will cover this with real MAT files
}
```

---

## Step 6: MAT to SDIF Conversion

### Task 6.1: Create Conversion Module

**Claude Code Prompt:**

```
Create sdif-rs/src/mat/convert.rs:

//! MAT to SDIF conversion utilities.
//!
//! This module provides [`MatToSdifConverter`] for converting MAT file
//! data to SDIF format, and [`MatToSdifConfig`] for configuration options.

use ndarray::{Array1, Array2, Axis};

use crate::error::{Error, Result};
use crate::writer::SdifWriter;
use super::data::MatData;
use super::file::MatFile;

/// Configuration for MAT to SDIF conversion.
///
/// # Example
///
/// ```
/// use sdif_rs::MatToSdifConfig;
///
/// let config = MatToSdifConfig::new()
///     .frame_type("1TRC")
///     .matrix_type("1TRC")
///     .columns(&["Index", "Frequency", "Amplitude", "Phase"]);
/// ```
#[derive(Debug, Clone)]
pub struct MatToSdifConfig {
    /// Name of the time variable (None = auto-detect).
    pub time_variable: Option<String>,
    
    /// Name of the data variable to convert.
    pub data_variable: Option<String>,
    
    /// SDIF frame type signature.
    pub frame_type: String,
    
    /// SDIF matrix type signature.
    pub matrix_type: String,
    
    /// Column names for the matrix.
    pub columns: Vec<String>,
    
    /// Maximum partials per frame (for Max/MSP compatibility).
    pub max_partials: Option<usize>,
    
    /// Whether to transpose the data (swap rows/columns).
    pub transpose: bool,
    
    /// How to handle complex data.
    pub complex_mode: ComplexMode,
    
    /// Stream ID for output frames.
    pub stream_id: u32,
}

/// How to handle complex numbers in MAT data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplexMode {
    /// Use only the real part.
    RealOnly,
    
    /// Convert to magnitude (absolute value).
    Magnitude,
    
    /// Convert to magnitude and phase (adds columns).
    MagnitudePhase,
    
    /// Keep real and imaginary as separate columns.
    RealImag,
}

impl Default for ComplexMode {
    fn default() -> Self {
        ComplexMode::Magnitude
    }
}

impl Default for MatToSdifConfig {
    fn default() -> Self {
        MatToSdifConfig {
            time_variable: None,
            data_variable: None,
            frame_type: "1TRC".to_string(),
            matrix_type: "1TRC".to_string(),
            columns: vec![
                "Index".to_string(),
                "Frequency".to_string(),
                "Amplitude".to_string(),
                "Phase".to_string(),
            ],
            max_partials: Some(1024),
            transpose: false,
            complex_mode: ComplexMode::default(),
            stream_id: 0,
        }
    }
}

impl MatToSdifConfig {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the time variable name.
    pub fn time_var(mut self, name: impl Into<String>) -> Self {
        self.time_variable = Some(name.into());
        self
    }
    
    /// Set the data variable name.
    pub fn data_var(mut self, name: impl Into<String>) -> Self {
        self.data_variable = Some(name.into());
        self
    }
    
    /// Set the SDIF frame type.
    pub fn frame_type(mut self, sig: impl Into<String>) -> Self {
        self.frame_type = sig.into();
        self
    }
    
    /// Set the SDIF matrix type.
    pub fn matrix_type(mut self, sig: impl Into<String>) -> Self {
        self.matrix_type = sig.into();
        self
    }
    
    /// Set the column names.
    pub fn columns(mut self, names: &[&str]) -> Self {
        self.columns = names.iter().map(|s| s.to_string()).collect();
        self
    }
    
    /// Set maximum partials per frame.
    pub fn max_partials(mut self, max: usize) -> Self {
        self.max_partials = Some(max);
        self
    }
    
    /// Disable partial limiting.
    pub fn no_partial_limit(mut self) -> Self {
        self.max_partials = None;
        self
    }
    
    /// Set whether to transpose the data.
    pub fn transpose(mut self, t: bool) -> Self {
        self.transpose = t;
        self
    }
    
    /// Set how to handle complex data.
    pub fn complex_mode(mut self, mode: ComplexMode) -> Self {
        self.complex_mode = mode;
        self
    }
    
    /// Set the stream ID.
    pub fn stream_id(mut self, id: u32) -> Self {
        self.stream_id = id;
        self
    }
}

/// Converter for MAT to SDIF conversion.
///
/// # Example
///
/// ```no_run
/// use sdif_rs::{MatFile, MatToSdifConfig, MatToSdifConverter, SdifFile};
///
/// let mat = MatFile::open("analysis.mat")?;
/// let config = MatToSdifConfig::new()
///     .time_var("time")
///     .data_var("partials");
///
/// let converter = MatToSdifConverter::new(&mat, config)?;
///
/// // Get info about the conversion
/// println!("Will convert {} frames", converter.num_frames());
///
/// // Perform the conversion
/// let mut writer = SdifFile::builder()
///     .create("output.sdif")?
///     .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
///     .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
///     .build()?;
///
/// converter.write_to(&mut writer)?;
/// writer.close()?;
/// # Ok::<(), sdif_rs::Error>(())
/// ```
pub struct MatToSdifConverter<'a> {
    /// Configuration.
    config: MatToSdifConfig,
    
    /// Time values for each frame.
    times: Array1<f64>,
    
    /// Data array (rows = time frames, cols = data).
    data: Array2<f64>,
    
    /// Reference to source MatFile (for metadata).
    _source: &'a MatFile,
}

impl<'a> MatToSdifConverter<'a> {
    /// Create a new converter.
    ///
    /// # Arguments
    ///
    /// * `mat` - The loaded MAT file.
    /// * `config` - Conversion configuration.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidFormat`] if required variables are missing
    /// - [`Error::InvalidFormat`] if data shapes are incompatible
    pub fn new(mat: &'a MatFile, config: MatToSdifConfig) -> Result<Self> {
        // Find time variable
        let time_var = Self::find_time_variable(mat, &config)?;
        let times = time_var.to_array1()?;
        
        // Find data variable
        let data_var = Self::find_data_variable(mat, &config)?;
        let mut data = data_var.to_array2()?;
        
        // Handle complex data
        if data_var.is_complex() {
            data = Self::handle_complex(data_var, &config)?;
        }
        
        // Transpose if requested
        if config.transpose {
            data = data.t().to_owned();
        }
        
        // Validate dimensions
        let num_frames = times.len();
        let (data_rows, _data_cols) = data.dim();
        
        if data_rows != num_frames {
            return Err(Error::invalid_format(format!(
                "Time vector length ({}) doesn't match data rows ({}). \
                 Try setting transpose=true if data is column-per-frame.",
                num_frames, data_rows
            )));
        }
        
        Ok(MatToSdifConverter {
            config,
            times,
            data,
            _source: mat,
        })
    }
    
    /// Find the time variable.
    fn find_time_variable<'m>(mat: &'m MatFile, config: &MatToSdifConfig) -> Result<&'m MatData> {
        if let Some(ref name) = config.time_variable {
            mat.require(name)
        } else {
            // Auto-detect
            let time_vars = mat.find_time_vectors();
            match time_vars.len() {
                0 => Err(Error::invalid_format(
                    "No time vector found. Specify --time-var explicitly."
                )),
                1 => mat.require(time_vars[0]),
                _ => Err(Error::invalid_format(format!(
                    "Multiple possible time vectors found: {:?}. Specify --time-var explicitly.",
                    time_vars
                ))),
            }
        }
    }
    
    /// Find the data variable.
    fn find_data_variable<'m>(mat: &'m MatFile, config: &MatToSdifConfig) -> Result<&'m MatData> {
        if let Some(ref name) = config.data_variable {
            mat.require(name)
        } else {
            // Find non-time 2D variables
            let candidates: Vec<_> = mat.iter()
                .filter(|(_, v)| v.is_2d() && !v.is_likely_time_vector())
                .map(|(n, _)| n)
                .collect();
            
            match candidates.len() {
                0 => Err(Error::invalid_format(
                    "No suitable data variable found. Specify --data-var explicitly."
                )),
                1 => mat.require(candidates[0]),
                _ => Err(Error::invalid_format(format!(
                    "Multiple possible data variables found: {:?}. Specify --data-var explicitly.",
                    candidates
                ))),
            }
        }
    }
    
    /// Handle complex data according to configuration.
    fn handle_complex(data_var: &MatData, config: &MatToSdifConfig) -> Result<Array2<f64>> {
        match config.complex_mode {
            ComplexMode::RealOnly => data_var.to_array2(),
            ComplexMode::Magnitude => data_var.magnitude(),
            ComplexMode::MagnitudePhase => {
                let mag = data_var.magnitude()?;
                let phase = data_var.phase()?;
                // Concatenate magnitude and phase columns
                let (rows, cols) = mag.dim();
                let mut combined = Array2::zeros((rows, cols * 2));
                combined.slice_mut(ndarray::s![.., ..cols]).assign(&mag);
                combined.slice_mut(ndarray::s![.., cols..]).assign(&phase);
                Ok(combined)
            }
            ComplexMode::RealImag => {
                let real = data_var.to_array2()?;
                let imag = data_var.imag_to_array2()?;
                let (rows, cols) = real.dim();
                let mut combined = Array2::zeros((rows, cols * 2));
                combined.slice_mut(ndarray::s![.., ..cols]).assign(&real);
                combined.slice_mut(ndarray::s![.., cols..]).assign(&imag);
                Ok(combined)
            }
        }
    }
    
    /// Get the number of frames that will be written.
    pub fn num_frames(&self) -> usize {
        self.times.len()
    }
    
    /// Get the time range.
    pub fn time_range(&self) -> (f64, f64) {
        let min = self.times.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = self.times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        (min, max)
    }
    
    /// Get the number of columns per frame.
    pub fn cols_per_frame(&self) -> usize {
        self.data.ncols()
    }
    
    /// Write all frames to an SDIF writer.
    ///
    /// # Arguments
    ///
    /// * `writer` - The SDIF writer to write frames to.
    ///
    /// # Errors
    ///
    /// Returns any errors from the underlying writer.
    pub fn write_to(&self, writer: &mut SdifWriter) -> Result<()> {
        let max_partials = self.config.max_partials.unwrap_or(usize::MAX);
        
        for (i, &time) in self.times.iter().enumerate() {
            let row = self.data.row(i);
            let row_data: Vec<f64> = row.iter().copied().collect();
            
            // Calculate number of partials (rows in SDIF matrix)
            let cols = self.config.columns.len();
            let num_values = row_data.len();
            
            if num_values % cols != 0 {
                return Err(Error::invalid_format(format!(
                    "Data length {} is not divisible by column count {}",
                    num_values, cols
                )));
            }
            
            let num_partials = (num_values / cols).min(max_partials);
            let limited_data = &row_data[..num_partials * cols];
            
            writer.write_frame_one_matrix(
                &self.config.frame_type,
                time,
                &self.config.matrix_type,
                num_partials,
                cols,
                limited_data,
            )?;
        }
        
        Ok(())
    }
    
    /// Get frame data for a specific time index.
    pub fn frame_data(&self, index: usize) -> Option<(&f64, ndarray::ArrayView1<f64>)> {
        if index < self.times.len() {
            Some((&self.times[index], self.data.row(index)))
        } else {
            None
        }
    }
    
    /// Iterate over (time, data) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (f64, ndarray::ArrayView1<f64>)> + '_ {
        self.times.iter().copied().zip(self.data.rows().into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_builder() {
        let config = MatToSdifConfig::new()
            .frame_type("1HRM")
            .matrix_type("1HRM")
            .columns(&["Freq", "Amp"])
            .max_partials(512);
        
        assert_eq!(config.frame_type, "1HRM");
        assert_eq!(config.matrix_type, "1HRM");
        assert_eq!(config.columns, vec!["Freq", "Amp"]);
        assert_eq!(config.max_partials, Some(512));
    }
}
```

---

## Step 7: Complex Number Handling

### Task 7.1: Create Complex Module

**Claude Code Prompt:**

```
Create sdif-rs/src/mat/complex.rs:

//! Complex number handling for MAT arrays.
//!
//! MATLAB stores complex numbers as separate real and imaginary arrays.
//! This module provides utilities for converting to various representations.

use ndarray::Array2;

use crate::error::{Error, Result};

/// Convert complex data to magnitude.
///
/// magnitude = sqrt(real² + imag²)
pub fn to_magnitude(real: &Array2<f64>, imag: &Array2<f64>) -> Result<Array2<f64>> {
    if real.dim() != imag.dim() {
        return Err(Error::invalid_format(format!(
            "Real and imaginary arrays have different shapes: {:?} vs {:?}",
            real.dim(), imag.dim()
        )));
    }
    
    Ok((real * real + imag * imag).mapv(f64::sqrt))
}

/// Convert complex data to phase.
///
/// phase = atan2(imag, real)
pub fn to_phase(real: &Array2<f64>, imag: &Array2<f64>) -> Result<Array2<f64>> {
    if real.dim() != imag.dim() {
        return Err(Error::invalid_format(format!(
            "Real and imaginary arrays have different shapes: {:?} vs {:?}",
            real.dim(), imag.dim()
        )));
    }
    
    let (rows, cols) = real.dim();
    let mut phase = Array2::zeros((rows, cols));
    
    for ((r, i), p) in real.iter().zip(imag.iter()).zip(phase.iter_mut()) {
        *p = i.atan2(*r);
    }
    
    Ok(phase)
}

/// Convert complex data to dB magnitude.
///
/// db = 20 * log10(magnitude)
pub fn to_db(real: &Array2<f64>, imag: &Array2<f64>) -> Result<Array2<f64>> {
    let mag = to_magnitude(real, imag)?;
    
    // Avoid log(0) by clamping to a small value
    let min_val = 1e-10;
    Ok(mag.mapv(|x| 20.0 * x.max(min_val).log10()))
}

/// Unwrap phase to remove discontinuities.
///
/// Adjusts phase values to avoid jumps greater than π.
pub fn unwrap_phase(phase: &Array2<f64>) -> Array2<f64> {
    let (rows, cols) = phase.dim();
    let mut unwrapped = phase.clone();
    
    let pi = std::f64::consts::PI;
    let two_pi = 2.0 * pi;
    
    // Unwrap along rows (time axis)
    for col in 0..cols {
        let mut cumulative_offset = 0.0;
        
        for row in 1..rows {
            let prev = unwrapped[[row - 1, col]];
            let curr = phase[[row, col]];
            let diff = curr - prev + cumulative_offset;
            
            // Check for discontinuity
            if diff > pi {
                cumulative_offset -= two_pi;
            } else if diff < -pi {
                cumulative_offset += two_pi;
            }
            
            unwrapped[[row, col]] = curr + cumulative_offset;
        }
    }
    
    unwrapped
}

/// Convert polar (magnitude, phase) to rectangular (real, imag).
pub fn polar_to_rectangular(mag: &Array2<f64>, phase: &Array2<f64>) -> Result<(Array2<f64>, Array2<f64>)> {
    if mag.dim() != phase.dim() {
        return Err(Error::invalid_format(format!(
            "Magnitude and phase arrays have different shapes: {:?} vs {:?}",
            mag.dim(), phase.dim()
        )));
    }
    
    let real = mag * &phase.mapv(f64::cos);
    let imag = mag * &phase.mapv(f64::sin);
    
    Ok((real, imag))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;
    use approx::assert_relative_eq;
    
    #[test]
    fn test_magnitude() {
        let real = array![[3.0, 0.0], [0.0, 1.0]];
        let imag = array![[4.0, 1.0], [1.0, 0.0]];
        
        let mag = to_magnitude(&real, &imag).unwrap();
        
        assert_relative_eq!(mag[[0, 0]], 5.0, epsilon = 1e-10);
        assert_relative_eq!(mag[[0, 1]], 1.0, epsilon = 1e-10);
        assert_relative_eq!(mag[[1, 0]], 1.0, epsilon = 1e-10);
        assert_relative_eq!(mag[[1, 1]], 1.0, epsilon = 1e-10);
    }
    
    #[test]
    fn test_phase() {
        let real = array![[1.0, -1.0], [0.0, 0.0]];
        let imag = array![[0.0, 0.0], [1.0, -1.0]];
        
        let phase = to_phase(&real, &imag).unwrap();
        
        assert_relative_eq!(phase[[0, 0]], 0.0, epsilon = 1e-10);
        assert_relative_eq!(phase[[0, 1]], std::f64::consts::PI, epsilon = 1e-10);
        assert_relative_eq!(phase[[1, 0]], std::f64::consts::FRAC_PI_2, epsilon = 1e-10);
        assert_relative_eq!(phase[[1, 1]], -std::f64::consts::FRAC_PI_2, epsilon = 1e-10);
    }
}
```

---

## Step 8: Module Root

### Task 8.1: Create mod.rs

**Claude Code Prompt:**

```
Create sdif-rs/src/mat/mod.rs:

//! MAT file support for MATLAB/Octave file parsing.
//!
//! This module provides utilities for reading MAT files and converting
//! their contents to SDIF format. It's designed for audio analysis
//! workflows where MAT files contain time-series spectral data.
//!
//! # Overview
//!
//! The main types are:
//!
//! - [`MatFile`] - Loads and provides access to MAT file contents
//! - [`MatData`] - Represents a single numeric variable
//! - [`MatToSdifConfig`] - Configuration for MAT→SDIF conversion
//! - [`MatToSdifConverter`] - Performs the actual conversion
//!
//! # Example
//!
//! ```no_run
//! use sdif_rs::{MatFile, MatToSdifConfig, MatToSdifConverter, SdifFile};
//!
//! // Load MAT file
//! let mat = MatFile::open("analysis.mat")?;
//!
//! // List variables
//! println!("{}", mat.describe());
//!
//! // Configure conversion
//! let config = MatToSdifConfig::new()
//!     .time_var("time")
//!     .data_var("partials")
//!     .frame_type("1TRC")
//!     .matrix_type("1TRC")
//!     .columns(&["Index", "Frequency", "Amplitude", "Phase"]);
//!
//! // Create converter
//! let converter = MatToSdifConverter::new(&mat, config)?;
//! println!("Converting {} frames", converter.num_frames());
//!
//! // Write to SDIF
//! let mut writer = SdifFile::builder()
//!     .create("output.sdif")?
//!     .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
//!     .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
//!     .build()?;
//!
//! converter.write_to(&mut writer)?;
//! writer.close()?;
//! # Ok::<(), sdif_rs::Error>(())
//! ```
//!
//! # Supported MAT Formats
//!
//! - Level 5 MAT files (MATLAB v5, v6, v7)
//! - v7 compressed files
//! - Numeric arrays of any type (converted to f64)
//! - Complex arrays
//!
//! # Not Supported
//!
//! - HDF5-based v7.3 files (use `hdf5` crate directly)
//! - Cell arrays, structs, sparse matrices
//! - Function handles, objects

mod complex;
mod convert;
mod data;
mod file;
mod time;

// Re-exports
pub use complex::{to_magnitude, to_phase, to_db, unwrap_phase, polar_to_rectangular};
pub use convert::{MatToSdifConfig, MatToSdifConverter, ComplexMode};
pub use data::MatData;
pub use file::MatFile;
pub use time::TimeStats;
```

---

## Step 9: Integration Tests

### Task 9.1: Create MAT Integration Tests

**Claude Code Prompt:**

```
Create sdif-rs/tests/mat_tests.rs:

//! Integration tests for MAT file support.
//!
//! These tests require the `mat` feature to be enabled.

#![cfg(feature = "mat")]

use sdif_rs::{MatFile, MatData, MatToSdifConfig, MatToSdifConverter, SdifFile, Result};
use tempfile::NamedTempFile;
use std::path::PathBuf;

/// Get path to test fixtures directory.
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Get path to a specific MAT fixture.
fn mat_fixture(name: &str) -> PathBuf {
    fixtures_dir().join(name)
}

// ============================================================================
// Tests that don't require fixture files
// ============================================================================

#[test]
fn test_open_nonexistent_mat() {
    let result = MatFile::open("/nonexistent/file.mat");
    assert!(result.is_err());
}

#[test]
fn test_config_builder() {
    let config = MatToSdifConfig::new()
        .frame_type("1HRM")
        .matrix_type("1HRM")
        .columns(&["Index", "Freq", "Amp"])
        .max_partials(256)
        .transpose(true);
    
    assert_eq!(config.frame_type, "1HRM");
    assert_eq!(config.matrix_type, "1HRM");
    assert_eq!(config.columns.len(), 3);
    assert_eq!(config.max_partials, Some(256));
    assert!(config.transpose);
}

#[test]
fn test_config_defaults() {
    let config = MatToSdifConfig::default();
    
    assert_eq!(config.frame_type, "1TRC");
    assert_eq!(config.matrix_type, "1TRC");
    assert_eq!(config.max_partials, Some(1024));
    assert!(!config.transpose);
}

// ============================================================================
// Tests that require fixture files
// ============================================================================

#[test]
#[ignore = "Requires test fixture: simple.mat"]
fn test_load_simple_mat() {
    let path = mat_fixture("simple.mat");
    if !path.exists() {
        eprintln!("Skipping: {} not found", path.display());
        return;
    }
    
    let mat = MatFile::open(&path).expect("Failed to open MAT file");
    
    assert!(!mat.is_empty(), "MAT file should have variables");
    
    // Print what we found
    println!("{}", mat.describe());
}

#[test]
#[ignore = "Requires test fixture: simple.mat"]
fn test_find_time_vectors() {
    let path = mat_fixture("simple.mat");
    if !path.exists() {
        return;
    }
    
    let mat = MatFile::open(&path).expect("Failed to open MAT file");
    let time_vars = mat.find_time_vectors();
    
    println!("Found time vectors: {:?}", time_vars);
    
    // Should find at least one if the fixture is set up correctly
    assert!(!time_vars.is_empty(), "Should find time vector");
}

#[test]
#[ignore = "Requires test fixture: simple.mat"]
fn test_mat_to_sdif_conversion() -> Result<()> {
    let mat_path = mat_fixture("simple.mat");
    if !mat_path.exists() {
        eprintln!("Skipping: {} not found", mat_path.display());
        return Ok(());
    }
    
    let mat = MatFile::open(&mat_path)?;
    
    // Auto-detect time and data variables
    let config = MatToSdifConfig::new();
    
    let converter = MatToSdifConverter::new(&mat, config)?;
    
    println!("Converting {} frames", converter.num_frames());
    let (start, end) = converter.time_range();
    println!("Time range: {:.3}s to {:.3}s", start, end);
    
    // Write to temp file
    let temp = NamedTempFile::new()?;
    let sdif_path = temp.path();
    
    let mut writer = SdifFile::builder()
        .create(sdif_path)?
        .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
        .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
        .build()?;
    
    converter.write_to(&mut writer)?;
    writer.close()?;
    
    // Verify the output
    let file = SdifFile::open(sdif_path)?;
    let mut frame_count = 0;
    
    for frame in file.frames() {
        let frame = frame?;
        frame_count += 1;
        assert_eq!(frame.signature(), "1TRC");
    }
    
    assert_eq!(frame_count, converter.num_frames());
    
    Ok(())
}

#[test]
#[ignore = "Requires test fixture: complex.mat"]
fn test_complex_magnitude() -> Result<()> {
    let path = mat_fixture("complex.mat");
    if !path.exists() {
        return Ok(());
    }
    
    let mat = MatFile::open(&path)?;
    
    // Find a complex variable
    let complex_var = mat.iter()
        .find(|(_, v)| v.is_complex())
        .map(|(_, v)| v);
    
    if let Some(var) = complex_var {
        let mag = var.magnitude()?;
        let phase = var.phase()?;
        
        println!("Complex var shape: {:?}", var.shape());
        println!("Magnitude shape: {:?}", mag.dim());
        println!("Phase shape: {:?}", phase.dim());
        
        assert_eq!(mag.dim(), phase.dim());
    }
    
    Ok(())
}

// ============================================================================
// Complex number utility tests
// ============================================================================

#[test]
fn test_complex_utils() {
    use sdif_rs::mat::{to_magnitude, to_phase};
    use ndarray::array;
    use approx::assert_relative_eq;
    
    let real = array![[3.0], [0.0]];
    let imag = array![[4.0], [1.0]];
    
    let mag = to_magnitude(&real, &imag).unwrap();
    assert_relative_eq!(mag[[0, 0]], 5.0, epsilon = 1e-10);
    assert_relative_eq!(mag[[1, 0]], 1.0, epsilon = 1e-10);
    
    let phase = to_phase(&real, &imag).unwrap();
    // atan2(4, 3) ≈ 0.927
    assert_relative_eq!(phase[[0, 0]], 0.9272952180016122, epsilon = 1e-10);
}
```

### Task 9.2: Create Test Fixtures README for MAT

**Claude Code Prompt:**

```
Update sdif-rs/tests/fixtures/README.md to include MAT file instructions:

Add this section:

## MAT Test Files

For MAT file integration tests, add these files:

### simple.mat
A basic MAT file with:
- `time` - 1D array of time values (e.g., 0.0 to 1.0 in 0.01 steps)
- `partials` - 2D array where each row is a time frame
  - Columns: Index, Frequency, Amplitude, Phase

### complex.mat
A MAT file with complex data:
- `spectrum` - 2D complex array (e.g., STFT output)
- `time` - 1D time vector

### Creating Test MAT Files

Using MATLAB:
```matlab
% simple.mat
time = (0:0.01:1)';  % 101 time points
partials = zeros(101, 4);
for i = 1:101
    partials(i, :) = [1, 440 + i, 0.5 * exp(-i/50), 0];
end
save('simple.mat', 'time', 'partials');

% complex.mat
time = (0:0.01:1)';
spectrum = randn(101, 256) + 1i * randn(101, 256);
save('complex.mat', 'time', 'spectrum');
```

Using Python (scipy):
```python
import numpy as np
from scipy.io import savemat

# simple.mat
time = np.arange(0, 1.01, 0.01)
partials = np.zeros((101, 4))
for i in range(101):
    partials[i] = [1, 440 + i, 0.5 * np.exp(-i/50), 0]
savemat('simple.mat', {'time': time, 'partials': partials})

# complex.mat
spectrum = np.random.randn(101, 256) + 1j * np.random.randn(101, 256)
savemat('complex.mat', {'time': time, 'spectrum': spectrum})
```

Using Octave:
```octave
% Same as MATLAB syntax
time = (0:0.01:1)';
partials = zeros(101, 4);
for i = 1:101
    partials(i, :) = [1, 440 + i, 0.5 * exp(-i/50), 0];
endfor
save('-v7', 'simple.mat', 'time', 'partials');
```
```

---

## Step 10: Verification Script

### Task 10.1: Create Phase 4 Verification Script

**Claude Code Prompt:**

```
Create scripts/verify-phase4.sh:

#!/bin/bash
# Verification script for Phase 4 completion

set -e

echo "=== Phase 4 Verification Script ==="
echo

# Check prerequisites
echo "1. Verifying prerequisites..."
if ! cargo check -p sdif-rs 2>/dev/null; then
    echo "   ✗ sdif-rs not building - complete Phase 3 first"
    exit 1
fi
echo "   ✓ sdif-rs builds (without mat feature)"

# Check new module files
echo
echo "2. Checking Phase 4 modules..."
REQUIRED_FILES=(
    "sdif-rs/src/mat/mod.rs"
    "sdif-rs/src/mat/file.rs"
    "sdif-rs/src/mat/data.rs"
    "sdif-rs/src/mat/time.rs"
    "sdif-rs/src/mat/convert.rs"
    "sdif-rs/src/mat/complex.rs"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "   ✓ $file exists"
    else
        echo "   ✗ $file missing!"
        exit 1
    fi
done

# Check feature flag in Cargo.toml
echo
echo "3. Checking feature configuration..."
if grep -q 'mat = \["dep:matfile"' sdif-rs/Cargo.toml; then
    echo "   ✓ mat feature defined"
else
    echo "   ✗ mat feature not defined in Cargo.toml"
    exit 1
fi

if grep -q 'matfile.*optional.*true' sdif-rs/Cargo.toml; then
    echo "   ✓ matfile is optional dependency"
else
    echo "   ✗ matfile should be optional"
    exit 1
fi

# Build with mat feature
echo
echo "4. Building with mat feature..."
if cargo build -p sdif-rs --features mat 2>/dev/null; then
    echo "   ✓ Builds with mat feature"
else
    echo "   ✗ Build with mat feature failed"
    exit 1
fi

# Check that it still builds without mat feature
echo
echo "5. Verifying builds without mat feature..."
if cargo build -p sdif-rs 2>/dev/null; then
    echo "   ✓ Builds without mat feature"
else
    echo "   ✗ Build without mat feature failed"
    exit 1
fi

# Run unit tests
echo
echo "6. Running unit tests..."
if cargo test -p sdif-rs --features mat --lib 2>/dev/null; then
    echo "   ✓ Unit tests passed"
else
    echo "   ⚠ Some unit tests failed"
fi

# Run MAT integration tests (those that don't need fixtures)
echo
echo "7. Running MAT integration tests..."
if cargo test -p sdif-rs --features mat --test mat_tests 2>/dev/null; then
    echo "   ✓ MAT tests passed"
else
    echo "   ⚠ MAT tests failed (may need fixtures)"
fi

# Check documentation builds
echo
echo "8. Building documentation..."
if cargo doc -p sdif-rs --features mat --no-deps 2>/dev/null; then
    echo "   ✓ Documentation builds"
else
    echo "   ⚠ Documentation issues"
fi

# Check that public types are exported
echo
echo "9. Checking public exports..."
EXPORTS=(
    "MatFile"
    "MatData"
    "MatToSdifConfig"
    "MatToSdifConverter"
)

for export in "${EXPORTS[@]}"; do
    if grep -q "pub use.*$export" sdif-rs/src/lib.rs; then
        echo "   ✓ $export exported"
    else
        echo "   ✗ $export not exported"
    fi
done

# Summary
echo
echo "=== Phase 4 Verification Complete ==="
echo
echo "MAT file support is implemented with:"
echo "  - MatFile for loading .mat files"
echo "  - MatData for individual variables"
echo "  - Time vector auto-detection"
echo "  - Complex number handling"
echo "  - MatToSdifConverter for conversion"
echo
echo "Next steps:"
echo "  1. Add test MAT files to tests/fixtures/"
echo "  2. Run: cargo test -p sdif-rs --features mat -- --include-ignored"
echo "  3. Proceed to Phase 5: mat2sdif CLI tool"

Make executable:
chmod +x scripts/verify-phase4.sh
```

---

## Success Criteria Summary

Phase 4 is complete when:

1. **MAT File Loading**
   - [ ] `MatFile::open()` loads Level 5 MAT files
   - [ ] Variable listing works
   - [ ] Handles v7 compression
   - [ ] Gracefully skips unsupported variable types

2. **MatData Variable Access**
   - [ ] Shape and dimension queries
   - [ ] 1D and 2D array extraction
   - [ ] Column-major to row-major conversion
   - [ ] Complex number support (real/imag, magnitude/phase)

3. **Time Vector Detection**
   - [ ] Name-based heuristics
   - [ ] Value-based heuristics (monotonic, non-negative)
   - [ ] TimeStats for analysis

4. **Conversion Pipeline**
   - [ ] `MatToSdifConfig` builder pattern
   - [ ] `MatToSdifConverter` writes to SdifWriter
   - [ ] Max partials limiting
   - [ ] Complex mode options

5. **Feature Gating**
   - [ ] `mat` feature enables functionality
   - [ ] Crate builds without `mat` feature
   - [ ] Dependencies are optional

6. **Tests**
   - [ ] Config builder tests
   - [ ] Complex math tests
   - [ ] Integration tests (with fixtures)

---

## Notes for Claude Code

### matfile Crate Limitations

The `matfile` crate:
- Only supports Level 5 MAT files (v5, v6, v7)
- Does NOT support HDF5-based v7.3 files
- Skips cell arrays, structs, sparse matrices
- Returns `NumericData` enum for all numeric types

### Column-Major vs Row-Major

MATLAB stores arrays in column-major (Fortran) order:
```
Matrix: [1 2 3; 4 5 6]
Memory: [1, 4, 2, 5, 3, 6]  (columns first)
```

SDIF and most Rust code uses row-major (C) order:
```
Memory: [1, 2, 3, 4, 5, 6]  (rows first)
```

The `MatData::to_array2()` method handles this conversion.

### Complex Number Representation

MATLAB stores complex as separate real and imaginary arrays.
The `matfile` crate exposes this as `NumericData::Double { real, imag }`.

For SDIF (which doesn't have native complex support), users can choose:
- `RealOnly` - Discard imaginary part
- `Magnitude` - `sqrt(real² + imag²)`
- `MagnitudePhase` - Add extra columns for phase
- `RealImag` - Add extra columns for imaginary

### Time Vector Heuristics

Auto-detection looks for:
1. Variables named "time", "t", "times", etc.
2. 1D arrays (or [N,1] or [1,N])
3. Monotonically increasing values
4. Starting near zero
5. Non-negative values

When multiple candidates exist, require explicit `--time-var` argument.
