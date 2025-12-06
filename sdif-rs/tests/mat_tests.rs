//! Integration tests for MAT file support.
//!
//! These tests require the `mat` feature to be enabled.

#![cfg(feature = "mat")]

use sdif_rs::{ComplexMode, MatFile, MatToSdifConfig, MatToSdifConverter, Result, SdifFile};
use std::path::PathBuf;
use tempfile::NamedTempFile;

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

#[test]
fn test_complex_mode_default() {
    let mode = ComplexMode::default();
    assert_eq!(mode, ComplexMode::Magnitude);
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
    let complex_var = mat.iter().find(|(_, v)| v.is_complex()).map(|(_, v)| v);

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
    use approx::assert_relative_eq;
    use ndarray::array;
    use sdif_rs::mat::{to_magnitude, to_phase};

    let real = array![[3.0], [0.0]];
    let imag = array![[4.0], [1.0]];

    let mag = to_magnitude(&real, &imag).unwrap();
    assert_relative_eq!(mag[[0, 0]], 5.0, epsilon = 1e-10);
    assert_relative_eq!(mag[[1, 0]], 1.0, epsilon = 1e-10);

    let phase = to_phase(&real, &imag).unwrap();
    // atan2(4, 3) ≈ 0.927
    assert_relative_eq!(phase[[0, 0]], 0.9272952180016122, epsilon = 1e-10);
}
