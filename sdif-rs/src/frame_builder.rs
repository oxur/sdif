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
        let (data_type, data_ptr, _element_size) = match &matrix.data {
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
        SdifFWritePadding(handle, calculate_padding(data_bytes));

        Ok(())
    }
}

/// Calculate padding needed to reach 8-byte alignment.
fn calculate_padding(bytes_written: usize) -> u32 {
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
            let mut vec = Vec::with_capacity(rows * cols);
            for row in data.rows() {
                vec.extend(row.iter().copied());
            }
            vec
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
            let mut vec = Vec::with_capacity(rows * cols);
            for row in data.rows() {
                vec.extend(row.iter().copied());
            }
            vec
        };

        self.add_matrix_f32(signature, rows, cols, &data_vec)
    }
}

#[cfg(test)]
mod tests {
    // Integration tests cover the main functionality
}
