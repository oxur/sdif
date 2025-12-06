//! SDIF frame representation and iteration.
//!
//! A frame is a time-stamped container for one or more matrices.
//! Frames are the primary unit of data organization in SDIF files.

use std::marker::PhantomData;

use sdif_sys::{
    SdifFCurrFrameSignature, SdifFCurrNbMatrix, SdifFCurrTime,
    SdifFGetSignature, SdifFReadFrameHeader, SdifFSkipFrameData,
    SdifFileT,
};

use crate::error::{Error, Result};
use crate::file::SdifFile;
use crate::matrix::MatrixIterator;
use crate::signature::{signature_to_string, Signature};

/// A single frame from an SDIF file.
///
/// A frame represents a snapshot of data at a specific point in time.
/// It contains one or more matrices, all sharing the same timestamp.
///
/// `Frame` borrows from its parent [`SdifFile`], ensuring the file
/// remains open while the frame is in use.
///
/// # Example
///
/// ```no_run
/// use sdif_rs::SdifFile;
///
/// let file = SdifFile::open("input.sdif")?;
/// for frame in file.frames() {
///     let frame = frame?;
///     println!("Frame '{}' at {:.3}s with {} matrices",
///         frame.signature(),
///         frame.time(),
///         frame.num_matrices()
///     );
/// }
/// # Ok::<(), sdif_rs::Error>(())
/// ```
pub struct Frame<'a> {
    /// Reference to the parent file.
    file: &'a SdifFile,

    /// Frame timestamp in seconds.
    time: f64,

    /// Frame type signature.
    signature: Signature,

    /// Stream ID for this frame.
    stream_id: u32,

    /// Number of matrices in this frame.
    num_matrices: u32,

    /// Current matrix index during iteration.
    current_matrix: u32,

    /// Whether we've finished reading this frame's data.
    finished: bool,

    /// Lifetime marker.
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Frame<'a> {
    /// Create a new Frame from the current file state.
    ///
    /// This should only be called after SdifFReadFrameHeader succeeds.
    pub(crate) fn from_current(file: &'a SdifFile) -> Self {
        let handle = file.handle();

        let time = unsafe { SdifFCurrTime(handle) };
        let signature = unsafe { SdifFCurrFrameSignature(handle) };
        let stream_id = unsafe { SdifFGetSignature(handle) }; // Stream ID is stored here
        let num_matrices = unsafe { SdifFCurrNbMatrix(handle) };

        Frame {
            file,
            time,
            signature,
            stream_id,
            num_matrices,
            current_matrix: 0,
            finished: false,
            _phantom: PhantomData,
        }
    }

    /// Get the frame timestamp in seconds.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sdif_rs::SdifFile;
    /// # let file = SdifFile::open("input.sdif")?;
    /// # let frame = file.frames().next().unwrap()?;
    /// if frame.time() >= 1.0 {
    ///     println!("Frame is at or after 1 second");
    /// }
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn time(&self) -> f64 {
        self.time
    }

    /// Get the frame type signature as a string (e.g., "1TRC").
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sdif_rs::SdifFile;
    /// # let file = SdifFile::open("input.sdif")?;
    /// # let frame = file.frames().next().unwrap()?;
    /// if frame.signature() == "1TRC" {
    ///     println!("This is a sinusoidal tracks frame");
    /// }
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn signature(&self) -> String {
        signature_to_string(self.signature)
    }

    /// Get the frame type signature as a raw u32.
    pub fn signature_raw(&self) -> Signature {
        self.signature
    }

    /// Get the stream ID for this frame.
    ///
    /// Stream IDs allow multiple parallel streams in one SDIF file.
    /// Most files use stream ID 0.
    pub fn stream_id(&self) -> u32 {
        self.stream_id
    }

    /// Get the number of matrices in this frame.
    ///
    /// Most frames contain a single matrix, but some frame types
    /// (like 1TRC) can contain multiple matrices per frame.
    pub fn num_matrices(&self) -> usize {
        self.num_matrices as usize
    }

    /// Create an iterator over the matrices in this frame.
    ///
    /// Matrices are read sequentially. Each matrix can only be
    /// read once per frame iteration.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sdif_rs::SdifFile;
    /// # let file = SdifFile::open("input.sdif")?;
    /// for frame in file.frames() {
    ///     let frame = frame?;
    ///     for matrix in frame.matrices() {
    ///         let matrix = matrix?;
    ///         println!("  Matrix '{}': {}x{}",
    ///             matrix.signature(),
    ///             matrix.rows(),
    ///             matrix.cols()
    ///         );
    ///     }
    /// }
    /// # Ok::<(), sdif_rs::Error>(())
    /// ```
    pub fn matrices(&mut self) -> MatrixIterator<'_, 'a> {
        MatrixIterator::new(self)
    }

    /// Get the file handle for matrix reading.
    pub(crate) fn handle(&self) -> *mut SdifFileT {
        self.file.handle()
    }

    /// Get the current matrix index.
    pub(crate) fn current_matrix_index(&self) -> u32 {
        self.current_matrix
    }

    /// Increment the matrix counter.
    pub(crate) fn advance_matrix(&mut self) {
        self.current_matrix += 1;
    }

    /// Check if there are more matrices to read.
    pub(crate) fn has_more_matrices(&self) -> bool {
        self.current_matrix < self.num_matrices
    }

    /// Mark this frame as finished (all matrices read or skipped).
    pub(crate) fn mark_finished(&mut self) {
        self.finished = true;
    }

    /// Skip remaining matrices in this frame.
    ///
    /// Called when the frame is dropped without reading all matrices.
    fn skip_remaining(&mut self) {
        if !self.finished && self.current_matrix < self.num_matrices {
            // Skip remaining frame data
            unsafe {
                SdifFSkipFrameData(self.file.handle());
            }
        }
        self.finished = true;
    }
}

impl Drop for Frame<'_> {
    fn drop(&mut self) {
        self.skip_remaining();
    }
}

/// Iterator over frames in an SDIF file.
///
/// Created by [`SdifFile::frames()`].
pub struct FrameIterator<'a> {
    file: &'a SdifFile,
    finished: bool,
}

impl<'a> FrameIterator<'a> {
    pub(crate) fn new(file: &'a SdifFile) -> Self {
        FrameIterator {
            file,
            finished: false,
        }
    }
}

impl<'a> Iterator for FrameIterator<'a> {
    type Item = Result<Frame<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let handle = self.file.handle();

        // Try to read the next frame header
        let bytes_read = unsafe { SdifFReadFrameHeader(handle) };

        if bytes_read == 0 {
            // End of file or error
            self.finished = true;
            return None;
        }

        if bytes_read < 0 {
            // Read error
            self.finished = true;
            return Some(Err(Error::read_error("Failed to read frame header")));
        }

        // Successfully read a frame header
        Some(Ok(Frame::from_current(self.file)))
    }
}

impl Drop for FrameIterator<'_> {
    fn drop(&mut self) {
        self.file.end_iteration();
    }
}

#[cfg(test)]
mod tests {
    // Tests require test fixtures - see integration tests
}
