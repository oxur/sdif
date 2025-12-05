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
