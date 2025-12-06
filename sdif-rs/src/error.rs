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

    /// The file has already been closed.
    #[error("File has been closed")]
    FileClosed,

    /// Frame has no matrices.
    #[error("Frame must contain at least one matrix")]
    EmptyFrame,

    /// Time values must be non-decreasing.
    #[error("Time must be non-decreasing: {current} < {previous}")]
    TimeNotIncreasing {
        /// Current time value.
        current: f64,
        /// Previous time value.
        previous: f64,
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

    /// Create a TimeNotIncreasing error.
    pub const fn time_not_increasing(current: f64, previous: f64) -> Self {
        Self::TimeNotIncreasing { current, previous }
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
