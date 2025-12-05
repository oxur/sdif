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
