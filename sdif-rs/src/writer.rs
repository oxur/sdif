//! SDIF file writer for adding frames to an open file.
//!
//! `SdifWriter` is obtained from `SdifFileBuilder::build()` and provides
//! methods for writing frames to the file.

use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::ptr::NonNull;

use sdif_sys::{
    SdifFClose, SdifFWriteFrameAndOneMatrix, SdifFileT,
    SdifDataTypeET_eFloat4, SdifDataTypeET_eFloat8,
};

use crate::error::{Error, Result};
use crate::frame_builder::FrameBuilder;
use crate::signature::string_to_signature;

/// Active writer for an SDIF file.
///
/// Created by [`SdifFileBuilder::build()`](crate::SdifFileBuilder::build).
/// Provides methods for writing frames to the file.
///
/// # Thread Safety
///
/// Like `SdifFile`, `SdifWriter` is `!Send + !Sync` because the
/// underlying C library is not thread-safe.
///
/// # Example
///
/// ```no_run
/// use sdif_rs::SdifFile;
///
/// let mut writer = SdifFile::builder()
///     .create("output.sdif")?
///     .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
///     .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
///     .build()?;
///
/// // Write a simple frame with one matrix
/// let data = vec![
///     1.0, 440.0, 0.5, 0.0,  // Partial 1: Index, Freq, Amp, Phase
///     2.0, 880.0, 0.3, 1.57, // Partial 2
/// ];
/// writer.write_frame_one_matrix("1TRC", 0.0, "1TRC", 2, 4, &data)?;
///
/// writer.close()?;
/// # Ok::<(), sdif_rs::Error>(())
/// ```
pub struct SdifWriter {
    /// Pointer to the C file handle.
    handle: NonNull<SdifFileT>,

    /// Path to the file (for error messages).
    path: PathBuf,

    /// Whether the file has been closed.
    closed: bool,

    /// Track the last written time for validation.
    last_time: Option<f64>,

    /// Count of frames written.
    frame_count: usize,

    /// Marker to make SdifWriter !Send and !Sync.
    _not_send_sync: PhantomData<*const ()>,
}

impl SdifWriter {
    /// Create a new writer (called internally by SdifFileBuilder).
    pub(crate) fn new(handle: NonNull<SdifFileT>, path: PathBuf) -> Self {
        SdifWriter {
            handle,
            path,
            closed: false,
            last_time: None,
            frame_count: 0,
            _not_send_sync: PhantomData,
        }
    }

    /// Get the file path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the number of frames written so far.
    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    /// Get the last written timestamp.
    pub fn last_time(&self) -> Option<f64> {
        self.last_time
    }

    /// Write a frame containing a single matrix.
    ///
    /// This is a convenience method for the common case of one matrix per frame.
    /// For frames with multiple matrices, use [`new_frame()`](Self::new_frame).
    ///
    /// # Arguments
    ///
    /// * `frame_sig` - Frame type signature (e.g., "1TRC")
    /// * `time` - Timestamp in seconds
    /// * `matrix_sig` - Matrix type signature (e.g., "1TRC")
    /// * `rows` - Number of rows in the matrix
    /// * `cols` - Number of columns in the matrix
    /// * `data` - Matrix data in row-major order (f64)
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidSignature`] if signatures are invalid
    /// - [`Error::InvalidState`] if the file is closed
    /// - [`Error::Io`] if writing fails
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
    /// // 2 partials, 4 columns each
    /// let data = vec![
    ///     1.0, 440.0, 0.5, 0.0,
    ///     2.0, 880.0, 0.3, 1.57,
    /// ];
    /// writer.write_frame_one_matrix("1TRC", 0.0, "1TRC", 2, 4, &data)?;
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn write_frame_one_matrix(
        &mut self,
        frame_sig: &str,
        time: f64,
        matrix_sig: &str,
        rows: usize,
        cols: usize,
        data: &[f64],
    ) -> Result<()> {
        self.check_not_closed()?;
        self.validate_time(time)?;

        // Validate data size
        let expected_len = rows * cols;
        if data.len() != expected_len {
            return Err(Error::InvalidDimensions { rows, cols });
        }

        // Convert signatures
        let frame_sig_u32 = string_to_signature(frame_sig)?;
        let matrix_sig_u32 = string_to_signature(matrix_sig)?;

        unsafe {
            self.write_frame_and_matrix_raw(
                frame_sig_u32,
                time,
                0, // stream_id
                matrix_sig_u32,
                rows as u32,
                cols as u32,
                data,
            )?;
        }

        self.last_time = Some(time);
        self.frame_count += 1;

        Ok(())
    }

    /// Write a frame with one matrix containing f32 data.
    ///
    /// Similar to [`write_frame_one_matrix`](Self::write_frame_one_matrix)
    /// but writes 32-bit floats instead of 64-bit.
    pub fn write_frame_one_matrix_f32(
        &mut self,
        frame_sig: &str,
        time: f64,
        matrix_sig: &str,
        rows: usize,
        cols: usize,
        data: &[f32],
    ) -> Result<()> {
        self.check_not_closed()?;
        self.validate_time(time)?;

        let expected_len = rows * cols;
        if data.len() != expected_len {
            return Err(Error::InvalidDimensions { rows, cols });
        }

        let frame_sig_u32 = string_to_signature(frame_sig)?;
        let matrix_sig_u32 = string_to_signature(matrix_sig)?;

        unsafe {
            self.write_frame_and_matrix_raw_f32(
                frame_sig_u32,
                time,
                0,
                matrix_sig_u32,
                rows as u32,
                cols as u32,
                data,
            )?;
        }

        self.last_time = Some(time);
        self.frame_count += 1;

        Ok(())
    }

    /// Start building a frame with multiple matrices.
    ///
    /// Returns a [`FrameBuilder`] that allows adding multiple matrices
    /// to the frame. The frame is written when `FrameBuilder::finish()`
    /// is called.
    ///
    /// # Arguments
    ///
    /// * `signature` - Frame type signature (e.g., "1TRC")
    /// * `time` - Timestamp in seconds
    /// * `stream_id` - Stream ID (usually 0)
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
    /// let data1 = vec![1.0, 440.0, 0.5, 0.0];
    /// let data2 = vec![2.0, 880.0, 0.3, 1.57];
    ///
    /// writer.new_frame("1TRC", 0.0, 0)?
    ///     .add_matrix("1TRC", 1, 4, &data1)?
    ///     .add_matrix("1TRC", 1, 4, &data2)?
    ///     .finish()?;
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn new_frame(
        &mut self,
        signature: &str,
        time: f64,
        stream_id: u32,
    ) -> Result<FrameBuilder<'_>> {
        self.check_not_closed()?;
        self.validate_time(time)?;

        let sig = string_to_signature(signature)?;

        Ok(FrameBuilder::new(self, sig, time, stream_id))
    }

    /// Close the file and finalize writing.
    ///
    /// This must be called to ensure all data is flushed and the file
    /// is properly closed. After calling `close()`, no more frames can
    /// be written.
    ///
    /// # Note
    ///
    /// The file will also be closed when the `SdifWriter` is dropped,
    /// but calling `close()` explicitly allows you to handle any errors.
    pub fn close(mut self) -> Result<()> {
        self.do_close()
    }

    /// Internal close implementation.
    fn do_close(&mut self) -> Result<()> {
        if self.closed {
            return Ok(());
        }

        self.closed = true;

        unsafe {
            SdifFClose(self.handle.as_ptr());
        }

        Ok(())
    }

    /// Check that the file hasn't been closed.
    fn check_not_closed(&self) -> Result<()> {
        if self.closed {
            Err(Error::invalid_state("Writer has been closed"))
        } else {
            Ok(())
        }
    }

    /// Validate that time is non-decreasing.
    fn validate_time(&self, time: f64) -> Result<()> {
        if let Some(last) = self.last_time {
            if time < last {
                return Err(Error::invalid_format(format!(
                    "Time must be non-decreasing: {} < {}",
                    time, last
                )));
            }
        }
        Ok(())
    }

    /// Get the raw file handle (for FrameBuilder).
    pub(crate) fn handle(&self) -> *mut SdifFileT {
        self.handle.as_ptr()
    }

    /// Record that a frame was written (called by FrameBuilder).
    pub(crate) fn record_frame_written(&mut self, time: f64) {
        self.last_time = Some(time);
        self.frame_count += 1;
    }

    /// Write a frame with one matrix using raw signatures (f64 data).
    unsafe fn write_frame_and_matrix_raw(
        &self,
        frame_sig: u32,
        time: f64,
        stream_id: u32,
        matrix_sig: u32,
        rows: u32,
        cols: u32,
        data: &[f64],
    ) -> Result<()> {
        let bytes_written = SdifFWriteFrameAndOneMatrix(
            self.handle.as_ptr(),
            frame_sig,
            stream_id,
            time,
            matrix_sig,
            SdifDataTypeET_eFloat8,
            rows,
            cols,
            data.as_ptr() as *mut libc::c_void,
        );

        if bytes_written == 0 {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write frame",
            )))
        } else {
            Ok(())
        }
    }

    /// Write a frame with one matrix using raw signatures (f32 data).
    unsafe fn write_frame_and_matrix_raw_f32(
        &self,
        frame_sig: u32,
        time: f64,
        stream_id: u32,
        matrix_sig: u32,
        rows: u32,
        cols: u32,
        data: &[f32],
    ) -> Result<()> {
        let bytes_written = SdifFWriteFrameAndOneMatrix(
            self.handle.as_ptr(),
            frame_sig,
            stream_id,
            time,
            matrix_sig,
            SdifDataTypeET_eFloat4,
            rows,
            cols,
            data.as_ptr() as *mut libc::c_void,
        );

        if bytes_written == 0 {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to write frame",
            )))
        } else {
            Ok(())
        }
    }
}

impl Drop for SdifWriter {
    fn drop(&mut self) {
        if !self.closed {
            // Best-effort close, ignore errors
            let _ = self.do_close();
        }
    }
}

// PhantomData<*const ()> makes SdifWriter !Send and !Sync automatically

// ============================================================================
// ndarray Integration
// ============================================================================

#[cfg(feature = "ndarray")]
use ndarray::Array2;

#[cfg(feature = "ndarray")]
impl SdifWriter {
    /// Write a frame with one matrix from an ndarray Array2<f64>.
    ///
    /// The array is automatically converted to row-major order if needed.
    ///
    /// # Arguments
    ///
    /// * `frame_sig` - Frame type signature
    /// * `time` - Timestamp in seconds
    /// * `matrix_sig` - Matrix type signature
    /// * `data` - 2D array of f64 values
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sdif_rs::SdifFile;
    /// use ndarray::array;
    ///
    /// let mut writer = SdifFile::builder()
    ///     .create("output.sdif")?
    ///     .add_matrix_type("1TRC", &["Index", "Frequency", "Amplitude", "Phase"])?
    ///     .add_frame_type("1TRC", &["1TRC SinusoidalTracks"])?
    ///     .build()?;
    ///
    /// let data = array![
    ///     [1.0, 440.0, 0.5, 0.0],
    ///     [2.0, 880.0, 0.3, 1.57],
    /// ];
    /// writer.write_frame_one_matrix_array("1TRC", 0.0, "1TRC", &data)?;
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn write_frame_one_matrix_array(
        &mut self,
        frame_sig: &str,
        time: f64,
        matrix_sig: &str,
        data: &Array2<f64>,
    ) -> Result<()> {
        let (rows, cols) = data.dim();

        // Convert to row-major (C order) if needed
        let data_vec: Vec<f64> = if data.is_standard_layout() {
            data.iter().copied().collect()
        } else {
            // Need to copy in row-major order
            let mut vec = Vec::with_capacity(rows * cols);
            for row in data.rows() {
                vec.extend(row.iter().copied());
            }
            vec
        };

        self.write_frame_one_matrix(frame_sig, time, matrix_sig, rows, cols, &data_vec)
    }

    /// Write a frame with one matrix from an ndarray Array2<f32>.
    pub fn write_frame_one_matrix_array_f32(
        &mut self,
        frame_sig: &str,
        time: f64,
        matrix_sig: &str,
        data: &Array2<f32>,
    ) -> Result<()> {
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

        self.write_frame_one_matrix_f32(frame_sig, time, matrix_sig, rows, cols, &data_vec)
    }
}

#[cfg(test)]
mod tests {
    // Most tests require actual file I/O - see integration tests
}
