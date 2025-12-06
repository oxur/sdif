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
//!         let mut frame = frame?;
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
//!         let mut frame = frame?;
//!         for matrix in frame.matrices() {
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

#![deny(missing_docs)]

// Modules
mod data_type;
mod error;
mod file;
mod frame;
pub mod init;
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
