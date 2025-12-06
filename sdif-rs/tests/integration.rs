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
