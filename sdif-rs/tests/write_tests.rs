//! Integration tests for SDIF writing functionality.

use sdif_rs::{SdifFile, Result, Error};
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
#[cfg_attr(sdif_stub_bindings, ignore = "Requires actual SDIF library")]
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
