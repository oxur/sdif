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
pub use complex::{polar_to_rectangular, to_db, to_magnitude, to_phase, unwrap_phase};
pub use convert::{ComplexMode, MatToSdifConfig, MatToSdifConverter};
pub use data::MatData;
pub use file::MatFile;
pub use time::TimeStats;
